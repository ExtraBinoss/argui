use std::time::Duration;
use web_time::Instant;

use argui_core::{Key, KeyInput, KeyState, Modifiers, Size};
use argui_layout::LayoutEngine;
use argui_text::{TextEngine, TextStyle, TextWrap};
use argui_ui::{
    CaretStyle, CursorIcon, Element, FocusPolicy, Interaction, TextEditorSpec, TextInputFilter,
    UiTree, length,
};

const NOTO_SANS: &[u8] = include_bytes!("../../../../assets/fonts/NotoSans-Regular.ttf");

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

fn editor(value: &str) -> UiTree {
    let style = TextStyle {
        wrap: TextWrap::None,
        ..TextStyle::default()
    };
    UiTree::new(
        Element::text_editor(TextEditorSpec {
            value: value.to_owned(),
            placeholder: String::new(),
            multiline: true,
            read_only: false,
            filter: TextInputFilter::Any,
            text: style.clone(),
            placeholder_text: style,
            selection: argui_ui::Color::WHITE,
            caret: CaretStyle::default(),
        })
        .width(length(480.0))
        .height(length(240.0))
        .interaction(
            Interaction::default()
                .focus_policy(FocusPolicy::TabStop)
                .cursor(CursorIcon::Text),
        ),
    )
}

#[test]
#[ignore = "manual CPU benchmark; run with --ignored --nocapture"]
fn million_character_native_input_to_prepared_text() {
    for (label, seed, middle) in [
        ("1M ASCII append native frame", "a".repeat(1_000_000), false),
        (
            "1M Unicode scalars middle native frame",
            "مرحبا".repeat(200_000),
            true,
        ),
        (
            "1M multiline middle native frame",
            "line\n".repeat(200_000),
            true,
        ),
    ] {
        let mut ui = editor(&seed);
        let node = ui.node_ids()[0];
        let mut layout_engine = LayoutEngine::new();
        let mut text_engine =
            TextEngine::from_embedded_fonts([NOTO_SANS], "Noto Sans", "Noto Sans", "Noto Sans");
        let viewport = Size::new(600.0, 300.0);
        let mut layout = layout_engine
            .compute(&mut ui, &mut text_engine, viewport)
            .unwrap();
        ui.sync_focus(
            &layout.hit_regions,
            Some(argui_ui::FocusRequest::Focus(node.into())),
        );
        if middle {
            ui.move_text_cursor(node, seed.len() / 2, false);
            layout_engine.update_text_inputs(&mut ui, &mut text_engine, &mut layout);
        }
        let mut samples = Vec::new();
        for _ in 0..100 {
            let key = KeyInput {
                key: Key::Character("x".into()),
                text: Some("x".into()),
                modifiers: Modifiers::default(),
                state: KeyState::Pressed,
                repeat: false,
            };
            let start = Instant::now();
            let update = ui.edit_text_input(&key);
            if update.layout_changed {
                layout = layout_engine
                    .compute(&mut ui, &mut text_engine, viewport)
                    .unwrap();
            } else {
                layout_engine.update_text_inputs(&mut ui, &mut text_engine, &mut layout);
            }
            let prepared = text_engine.prepare(&layout.text, 1.0);
            assert!(prepared.glyphs.len() < 1_000);
            samples.push(start.elapsed());
        }
        report(label, samples);
    }
}

#[test]
#[ignore = "manual CPU benchmark; run with --ignored --nocapture"]
fn million_character_backspace_to_prepared_text() {
    let seed = "line\n".repeat(200_000);
    let mut ui = editor(&seed);
    let node = ui.node_ids()[0];
    let mut layout_engine = LayoutEngine::new();
    let mut text_engine =
        TextEngine::from_embedded_fonts([NOTO_SANS], "Noto Sans", "Noto Sans", "Noto Sans");
    let viewport = Size::new(600.0, 300.0);
    let mut layout = layout_engine
        .compute(&mut ui, &mut text_engine, viewport)
        .unwrap();
    ui.sync_focus(
        &layout.hit_regions,
        Some(argui_ui::FocusRequest::Focus(node.into())),
    );
    ui.move_text_cursor(node, seed.len() / 2, false);
    layout_engine.update_text_inputs(&mut ui, &mut text_engine, &mut layout);
    let mut samples = Vec::new();
    let mut edits = Vec::new();
    let mut layouts = Vec::new();
    let mut prepares = Vec::new();
    let mut newline_samples = Vec::new();
    let mut ordinary_samples = Vec::new();
    for _ in 0..100 {
        let cursor = ui.text_input_cursor(node).unwrap();
        let deletes_newline = ui
            .text_input_value(node)
            .unwrap()
            .as_bytes()
            .get(cursor - 1)
            == Some(&b'\n');
        let key = KeyInput {
            key: Key::Backspace,
            text: None,
            modifiers: Modifiers::default(),
            state: KeyState::Pressed,
            repeat: false,
        };
        let start = Instant::now();
        let update = ui.edit_text_input(&key);
        let after_edit = Instant::now();
        if update.layout_changed {
            layout = layout_engine
                .compute(&mut ui, &mut text_engine, viewport)
                .unwrap();
        } else {
            layout_engine.update_text_inputs(&mut ui, &mut text_engine, &mut layout);
        }
        let after_layout = Instant::now();
        let prepared = text_engine.prepare(&layout.text, 1.0);
        assert!(prepared.glyphs.len() < 1_000);
        let after_prepare = Instant::now();
        edits.push(after_edit - start);
        layouts.push(after_layout - after_edit);
        prepares.push(after_prepare - after_layout);
        samples.push(after_prepare - start);
        if deletes_newline {
            newline_samples.push(after_prepare - start);
        } else {
            ordinary_samples.push(after_prepare - start);
        }
    }
    report("1M multiline middle backspace native frame", samples);
    report("  edit", edits);
    report("  layout", layouts);
    report("  prepare", prepares);
    report("  newline", newline_samples);
    report("  ordinary", ordinary_samples);
}
