use std::time::Duration;
use web_time::Instant;

use argui_core::{
    Affine2D, CaretAffinity, Key, KeyInput, KeyState, Modifiers, Point, Rect, Size, TextPosition,
};
use argui_paint::ClipChain;
use argui_text::TextStyle;
use argui_ui::{
    CaretStyle, CursorIcon, Element, EventHandlerId, EventListener, EventOwnerId, EventType,
    FocusPolicy, HitRegion, HitShape, Interaction, TextEdit, TextEditorSpec, TextInputFilter,
    TextSelection, TextSelectionRequest, UiTree,
};

fn report(label: &str, mut samples: Vec<Duration>) {
    samples.sort();
    eprintln!(
        "{label}: p50={:?} p95={:?} max={:?} / {} keys",
        samples[samples.len() / 2],
        samples[samples.len() * 95 / 100],
        samples[samples.len() - 1],
        samples.len()
    );
}

fn field(value: &str, controlled: bool) -> Element {
    let mut element = Element::text_editor(TextEditorSpec {
        value: value.to_owned(),
        placeholder: String::new(),
        multiline: true,
        read_only: false,
        filter: TextInputFilter::Any,
        text: TextStyle::default(),
        placeholder_text: TextStyle::default(),
        selection: argui_ui::Color::WHITE,
        caret: CaretStyle::default(),
    })
    .keyed("input")
    .interaction(
        Interaction::default()
            .focus_policy(FocusPolicy::TabStop)
            .cursor(CursorIcon::Text),
    );
    if controlled {
        element = element.on(EventListener::new(
            EventType::TextEdit,
            EventHandlerId::new(EventOwnerId(1), 0),
        ));
    }
    element
}

fn editor(value: &str, controlled: bool) -> (UiTree, argui_ui::NodeId) {
    let mut tree = UiTree::new(field(value, controlled));
    let node = tree.node_ids()[0];
    tree.sync_focus(
        &[HitRegion {
            node,
            bounds: Rect::new(Point::default(), Size::new(500.0, 30.0)),
            transform: Affine2D::IDENTITY,
            clips: ClipChain::default(),
            shape: HitShape::Bounds,
            slop: argui_ui::Sides::length(0.0),
            enabled: true,
            focus_policy: FocusPolicy::TabStop,
            cursor: CursorIcon::Text,
            gestures: argui_ui::GestureSet::EMPTY,
            window_drag: None,
        }],
        Some(argui_ui::FocusRequest::Focus(node.into())),
    );
    (tree, node)
}

fn typed_x() -> KeyInput {
    KeyInput {
        key: Key::Character("x".into()),
        text: Some("x".into()),
        modifiers: Modifiers::default(),
        state: KeyState::Pressed,
        repeat: false,
    }
}

fn navigation_key(key: Key) -> KeyInput {
    KeyInput {
        key,
        text: None,
        modifiers: Modifiers::default(),
        state: KeyState::Pressed,
        repeat: false,
    }
}

#[test]
fn pointer_caret_clamps_to_unicode_grapheme_starts() {
    for grapheme in ["e\u{301}", "👩‍👩‍👧‍👧", "🇺🇸"] {
        let value = format!("A{grapheme}B");
        let (mut tree, node) = editor(&value, false);
        for offset in grapheme.char_indices().skip(1).map(|(index, _)| index) {
            tree.place_text_cursor(node, 1 + offset, false);
            assert_eq!(tree.text_input_cursor(node), Some(1));
        }
        tree.place_text_cursor(node, 1 + grapheme.len(), false);
        assert_eq!(tree.text_input_cursor(node), Some(1 + grapheme.len()));
    }
}

#[test]
fn million_character_uncontrolled_edit_survives_parent_update() {
    let seed = "a".repeat(1_000_000);
    let (mut tree, node) = editor(&seed, false);
    tree.edit_text_input(&typed_x());
    tree.replace(field(&seed, false).width(argui_ui::length(450.0)));
    assert_eq!(tree.text_input_value(node).unwrap().len(), seed.len() + 1);
    assert!(tree.text_input_value(node).unwrap().ends_with('x'));
}

#[test]
fn million_character_controlled_edits_accept_partial_and_final_echo() {
    let seed = "a".repeat(1_000_000);
    let (mut tree, node) = editor(&seed, true);
    for _ in 0..100 {
        tree.edit_text_input(&typed_x());
    }
    tree.replace(field(&(seed.clone() + &"x".repeat(50)), true));
    assert_eq!(tree.text_input_value(node).unwrap().len(), seed.len() + 100);
    tree.replace(field(&(seed.clone() + &"x".repeat(100)), true));
    assert_eq!(tree.text_input_value(node).unwrap().len(), seed.len() + 100);
    tree.edit_text_input(&typed_x());
    assert_eq!(tree.text_input_value(node).unwrap().len(), seed.len() + 101);
}

#[test]
fn controlled_burst_beyond_previous_limit_accepts_late_partial_echo() {
    let seed = "a".repeat(1_000_000);
    let (mut tree, node) = editor(&seed, true);
    for _ in 0..300 {
        tree.edit_text_input(&typed_x());
    }
    tree.replace(field(&(seed.clone() + &"x".repeat(150)), true));
    assert_eq!(tree.text_input_value(node).unwrap().len(), seed.len() + 300);
    tree.replace(field(&(seed.clone() + &"x".repeat(300)), true));
    assert_eq!(tree.text_input_value(node).unwrap().len(), seed.len() + 300);
    tree.edit_text_input(&typed_x());
    assert_eq!(tree.text_input_value(node).unwrap().len(), seed.len() + 301);
}

#[test]
fn controlled_middle_burst_accepts_late_partial_echo() {
    let seed = "م".repeat(500_000);
    let middle = seed.len() / 2;
    let (mut tree, node) = editor(&seed, true);
    tree.move_text_cursor(node, middle, false);
    for _ in 0..300 {
        tree.edit_text_input(&typed_x());
    }
    let partial = format!("{}{}{}", &seed[..middle], "x".repeat(150), &seed[middle..]);
    tree.replace(field(&partial, true));
    assert_eq!(tree.text_input_value(node).unwrap().len(), seed.len() + 300);
    let final_value = format!("{}{}{}", &seed[..middle], "x".repeat(300), &seed[middle..]);
    tree.replace(field(&final_value, true));
    assert_eq!(tree.text_input_value(node).unwrap(), final_value);
}

#[test]
fn controlled_same_length_edits_accept_late_partial_echo() {
    let seed = "a".repeat(1_000_000);
    let (mut tree, node) = editor(&seed, true);
    let mut partial = String::new();
    for key in 0..300 {
        let offset = if key % 2 == 0 {
            250_000 + key / 2
        } else {
            750_000 + key / 2
        };
        tree.select_text(TextSelectionRequest::new(
            node,
            TextSelection::Range {
                anchor: TextPosition::new(offset, CaretAffinity::Before),
                cursor: TextPosition::new(offset + 1, CaretAffinity::Before),
            },
        ));
        tree.edit_text_input(&typed_x());
        if key == 149 {
            partial = tree.text_input_value(node).unwrap().to_owned();
        }
    }
    tree.replace(field(&partial, true));
    assert_eq!(tree.text_input_value(node).unwrap().len(), seed.len());
    assert_eq!(&tree.text_input_value(node).unwrap()[250_000..250_001], "x");
}

#[test]
#[ignore = "manual CPU benchmark; run with --ignored --nocapture"]
fn million_character_controlled_middle_burst_partial_echo_cost() {
    let seed = "a".repeat(1_000_000);
    let middle = seed.len() / 2;
    let (mut tree, node) = editor(&seed, true);
    tree.move_text_cursor(node, middle, false);
    for _ in 0..2_000 {
        tree.edit_text_input(&typed_x());
    }
    let partial = format!(
        "{}{}{}",
        &seed[..middle],
        "x".repeat(1_000),
        &seed[middle..]
    );

    let start = Instant::now();
    let mut candidate = seed.clone();
    for key in 0..=1_000 {
        TextEdit::new(middle + key..middle + key, "x")
            .apply_to(&mut candidate)
            .unwrap();
        if candidate.len() == partial.len() && candidate == partial {
            break;
        }
    }
    let replay = start.elapsed();

    let start = Instant::now();
    tree.replace(field(&partial, true));
    let echo = start.elapsed();
    assert_eq!(
        tree.text_input_value(node).unwrap().len(),
        seed.len() + 2_000
    );
    eprintln!("1M middle controlled 2000-key partial ack: replay={replay:?} indexed echo={echo:?}");
}

#[test]
#[ignore = "manual CPU benchmark; run with --ignored --nocapture"]
fn million_character_controlled_noncontiguous_partial_echo_cost() {
    let seed = "a".repeat(1_000_000);
    let keys = std::env::var("ARGUI_ACK_KEYS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(2_000);
    let (mut tree, node) = editor(&seed, true);
    let mut partial = String::new();
    for key in 0..keys {
        tree.move_text_cursor(node, if key % 2 == 0 { 250_000 } else { 750_000 }, false);
        tree.edit_text_input(&typed_x());
        if key == keys / 2 - 1 {
            partial = tree.text_input_value(node).unwrap().to_owned();
        }
    }
    let start = Instant::now();
    tree.replace(field(&partial, true));
    let echo = start.elapsed();
    assert_eq!(
        tree.text_input_value(node).unwrap().len(),
        seed.len() + keys
    );
    eprintln!("1M noncontiguous controlled {keys}-key partial ack: piece echo={echo:?}");
}

#[test]
#[ignore = "manual CPU benchmark; run with --ignored --nocapture"]
fn million_character_controlled_same_length_partial_echo_cost() {
    let seed = "a".repeat(1_000_000);
    let (mut tree, node) = editor(&seed, true);
    let mut partial = String::new();
    for key in 0..10_000 {
        let offset = if key % 2 == 0 {
            250_000 + key / 2
        } else {
            750_000 + key / 2
        };
        tree.select_text(TextSelectionRequest::new(
            node,
            TextSelection::Range {
                anchor: TextPosition::new(offset, CaretAffinity::Before),
                cursor: TextPosition::new(offset + 1, CaretAffinity::Before),
            },
        ));
        tree.edit_text_input(&typed_x());
        if key == 4_999 {
            partial = tree.text_input_value(node).unwrap().to_owned();
        }
    }
    let start = Instant::now();
    tree.replace(field(&partial, true));
    let echo = start.elapsed();
    assert_eq!(tree.text_input_value(node).unwrap().len(), seed.len());
    eprintln!("1M same-length controlled 10000-key partial ack: echo={echo:?}");
}

#[test]
#[ignore = "manual CPU benchmark; run with --ignored --nocapture"]
fn million_character_native_edit_burst() {
    for (label, seed, middle, controlled) in [
        (
            "ASCII append autonomous",
            "a".repeat(1_000_000),
            false,
            false,
        ),
        (
            "ASCII append edit listener",
            "a".repeat(1_000_000),
            false,
            true,
        ),
        (
            "Unicode middle autonomous",
            "مرحبا".repeat(100_000),
            true,
            false,
        ),
        (
            "Unicode middle edit listener",
            "مرحبا".repeat(100_000),
            true,
            true,
        ),
    ] {
        let (mut tree, node) = editor(&seed, controlled);
        if middle {
            tree.move_text_cursor(node, 500_000, false);
        }
        let mut samples = Vec::new();
        for _ in 0..100 {
            let key = typed_x();
            let start = Instant::now();
            tree.edit_text_input(&key);
            samples.push(start.elapsed());
        }
        assert_eq!(tree.text_input_value(node).unwrap().len(), 1_000_100);
        report(label, samples);
    }
}

#[test]
#[ignore = "manual CPU benchmark; run with --ignored --nocapture"]
fn million_unicode_grapheme_navigation_and_backspace() {
    let seed = "م".repeat(1_000_000);
    let middle = seed.len() / 2;
    for (label, operation) in [
        ("1M Unicode left arrow", 0),
        ("1M Unicode backspace", 1),
        ("1M Unicode mouse caret placement", 2),
    ] {
        let (mut tree, node) = editor(&seed, false);
        tree.move_text_cursor(node, middle, false);
        let mut samples = Vec::new();
        for index in 0..100 {
            let start = Instant::now();
            match operation {
                0 => {
                    tree.edit_text_input(&navigation_key(Key::ArrowLeft));
                }
                1 => {
                    tree.edit_text_input(&navigation_key(Key::Backspace));
                }
                _ => {
                    tree.place_text_cursor(node, middle + index * 2, false);
                }
            }
            samples.push(start.elapsed());
        }
        report(label, samples);
    }
}

#[test]
#[ignore = "manual CPU benchmark; run with --ignored --nocapture"]
fn million_character_flat_string_middle_insert_cost() {
    for (label, mut value) in [
        ("1M ASCII String middle insert", "a".repeat(1_000_000)),
        (
            "1M Unicode scalars String middle insert",
            "مرحبا".repeat(200_000),
        ),
    ] {
        let mut samples = Vec::new();
        let middle = value.len() / 2;
        for index in 0..100 {
            let start = Instant::now();
            value.insert(middle + index, 'x');
            samples.push(start.elapsed());
        }
        report(label, samples);
    }
}
