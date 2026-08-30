use argui_core::{
    Affine2D, CaretAffinity, ImeInput, Key, KeyInput, KeyState, Modifiers, Point, Rect, Size,
    TextPosition,
};
use argui_paint::{ClipBehavior, ClipChain, ClipRegion, PaintStyle, QuadStyle};
use argui_text::TextStyle;
use argui_ui::{
    ClipboardRequest, CursorIcon, HitRegion, TextArea, TextInput, TextInputStyle, UiEventKind,
    UiTree,
};

fn tree(value: &str) -> (UiTree, HitRegion) {
    let input = TextInput::new(
        "field",
        value,
        "placeholder",
        TextInputStyle::new(PaintStyle::default(), TextStyle::default()),
    )
    .build();
    let tree = UiTree::new(input);
    let node = tree.node_id_at(0).unwrap();
    let bounds = Rect::new(Point::default(), Size::new(300.0, 40.0));
    (
        tree,
        HitRegion {
            node,
            bounds,
            transform: Affine2D::IDENTITY,
            clips: ClipChain::from_regions([ClipRegion::new(bounds, Affine2D::IDENTITY)]),
            focusable: true,
            cursor: CursorIcon::Text,
            gestures: argui_ui::GestureSet::NONE,
            window_drag: None,
        },
    )
}

fn key(key: Key, text: Option<&str>, modifiers: Modifiers) -> KeyInput {
    KeyInput {
        key,
        state: KeyState::Pressed,
        modifiers,
        repeat: false,
        text: text.map(ToOwned::to_owned),
    }
}

fn focus(tree: &mut UiTree, region: &HitRegion) {
    tree.pointer_moved(Point::new(10.0, 10.0), std::slice::from_ref(region));
    tree.primary_pressed(std::slice::from_ref(region));
}

#[test]
fn editing_respects_graphemes_selection_and_clipboard_requests() {
    let (mut tree, region) = tree("A👋🏽");
    focus(&mut tree, &region);
    let node = region.node;

    let deleted = tree.edit_text_input(&key(Key::Backspace, None, Modifiers::default()));
    assert_eq!(tree.text_input_value(node), Some("A"));
    assert!(matches!(
        &deleted.events[0].kind,
        UiEventKind::TextChanged(value) if value == "A"
    ));

    let command = Modifiers {
        control: true,
        ..Modifiers::default()
    };
    tree.edit_text_input(&key(Key::Character("a".into()), Some("a"), command));
    let copied = tree.edit_text_input(&key(Key::Character("c".into()), Some("c"), command));
    assert_eq!(copied.clipboard, Some(ClipboardRequest::Write("A".into())));
    let cut = tree.edit_text_input(&key(Key::Character("x".into()), Some("x"), command));
    assert!(cut.layout_changed);
    assert_eq!(tree.text_input_value(node), Some(""));

    let paste = tree.edit_text_input(&key(Key::Character("v".into()), Some("v"), command));
    assert_eq!(paste.clipboard, Some(ClipboardRequest::Read));
    tree.paste_text("é");
    assert_eq!(tree.text_input_value(node), Some("é"));
}

#[test]
fn ime_preedit_is_visible_but_only_commit_changes_the_value() {
    let (mut tree, region) = tree("");
    focus(&mut tree, &region);
    let node = region.node;

    let preedit = tree.ime_input(ImeInput::Preedit {
        text: "é".into(),
        cursor: Some((0, 3)),
    });
    assert!(preedit.layout_changed);
    assert_eq!(tree.text_input_display(node).as_deref(), Some("é"));
    assert_eq!(tree.text_input_value(node), Some(""));

    let commit = tree.ime_input(ImeInput::Commit("é".into()));
    assert!(matches!(
        &commit.events[0].kind,
        UiEventKind::TextChanged(value) if value == "é"
    ));
    assert_eq!(tree.text_input_value(node), Some("é"));
}

#[test]
fn text_input_style_remains_composed_from_existing_primitives() {
    let style = TextInputStyle::new(PaintStyle::new(QuadStyle::default()), TextStyle::default());
    let input = TextInput::new("field", "", "hint", style).build();
    let interaction = input.interaction.as_ref().unwrap();
    assert!(interaction.focusable);
    assert_eq!(interaction.cursor, CursorIcon::Text);
    assert!(input.children.is_empty());
}

#[test]
fn navigation_delete_submit_and_escape_cover_editor_edges() {
    let (mut tree, region) = tree("one two");
    focus(&mut tree, &region);
    let node = region.node;
    let shift = Modifiers {
        shift: true,
        ..Modifiers::default()
    };
    tree.edit_text_input(&key(Key::Home, None, Modifiers::default()));
    tree.edit_text_input(&key(Key::ArrowRight, None, shift));
    assert_eq!(tree.text_input_selection(node), Some((0, 1)));
    tree.edit_text_input(&key(Key::Escape, None, Modifiers::default()));
    assert_eq!(tree.text_input_selection(node), None);

    tree.edit_text_input(&key(Key::End, None, Modifiers::default()));
    assert!(
        !tree
            .edit_text_input(&key(Key::Delete, None, Modifiers::default()))
            .layout_changed
    );
    tree.edit_text_input(&key(Key::Home, None, Modifiers::default()));
    assert!(
        !tree
            .edit_text_input(&key(Key::Backspace, None, Modifiers::default()))
            .layout_changed
    );
    let inserted = tree.edit_text_input(&key(
        Key::Character("é".into()),
        Some("é"),
        Modifiers::default(),
    ));
    assert!(inserted.layout_changed);
    let submitted = tree.edit_text_input(&key(Key::Enter, None, Modifiers::default()));
    assert!(matches!(
        &submitted.events[0].kind,
        UiEventKind::Submitted(value) if value.starts_with('é')
    ));

    let alt = Modifiers {
        alt: true,
        ..Modifiers::default()
    };
    let before = tree.text_input_value(node).unwrap().to_owned();
    tree.edit_text_input(&key(Key::Character("x".into()), Some("x"), alt));
    assert_eq!(tree.text_input_value(node), Some(before.as_str()));
}

#[test]
fn word_motion_pointer_drag_and_state_retention_are_explicit() {
    let (mut tree, region) = tree("one two three");
    focus(&mut tree, &region);
    let node = region.node;
    let command = Modifiers {
        control: true,
        ..Modifiers::default()
    };
    tree.edit_text_input(&key(Key::ArrowLeft, None, command));
    tree.edit_text_input(&key(Key::ArrowRight, None, command));

    tree.place_text_cursor(node, 0, false);
    assert!(tree.text_cursor_dragging());
    tree.drag_text_cursor(node, 3);
    assert_eq!(tree.text_input_selection(node), Some((0, 3)));
    assert!(tree.release_text_cursor());
    assert!(!tree.release_text_cursor());
    assert!(!tree.drag_text_cursor(node, 6).layout_changed);
    tree.move_text_cursor(node, 7, false);
    assert_eq!(tree.text_input_selection(node), None);

    tree.update(argui_ui::Element::container([]));
    assert_eq!(tree.text_input_value(node), None);
    assert!(!tree.text_cursor_dragging());
}

#[test]
fn unfocused_released_and_empty_ime_inputs_do_no_work() {
    let (mut tree, region) = tree("");
    let released = KeyInput {
        state: KeyState::Released,
        ..key(Key::Character("a".into()), Some("a"), Modifiers::default())
    };
    assert!(!tree.edit_text_input(&released).layout_changed);
    assert!(!tree.paste_text("x").layout_changed);
    assert!(!tree.ime_input(ImeInput::Enabled).layout_changed);

    focus(&mut tree, &region);
    let command = Modifiers {
        control: true,
        ..Modifiers::default()
    };
    assert_eq!(
        tree.edit_text_input(&key(Key::Character("c".into()), Some("c"), command))
            .clipboard,
        None
    );
    assert!(
        !tree
            .edit_text_input(&key(Key::Character("x".into()), Some("x"), command))
            .layout_changed
    );
    assert_eq!(
        tree.edit_text_input(&key(Key::Character("z".into()), Some("z"), command))
            .clipboard,
        None
    );
    assert!(!tree.paste_text("").layout_changed);
    assert!(
        !tree
            .edit_text_input(&key(Key::ArrowDown, None, Modifiers::default()))
            .layout_changed
    );
    assert!(!tree.ime_input(ImeInput::Enabled).layout_changed);
    assert!(
        !tree
            .ime_input(ImeInput::Preedit {
                text: String::new(),
                cursor: None,
            })
            .layout_changed
    );
    assert!(
        tree.ime_input(ImeInput::Commit(String::new()))
            .events
            .is_empty()
    );
    tree.ime_input(ImeInput::Preedit {
        text: "abc".into(),
        cursor: Some((0, 2)),
    });
    assert!(tree.ime_input(ImeInput::Disabled).layout_changed);
}

#[test]
fn reverse_selection_real_delete_and_default_ime_cursor_cover_boundaries() {
    let (mut tree, region) = tree("abc");
    focus(&mut tree, &region);
    let node = region.node;

    tree.place_text_cursor(node, 3, false);
    tree.drag_text_cursor(node, 1);
    assert_eq!(tree.text_input_selection(node), Some((1, 3)));
    tree.edit_text_input(&key(Key::Delete, None, Modifiers::default()));
    assert_eq!(tree.text_input_value(node), Some("a"));

    tree.edit_text_input(&key(Key::Home, None, Modifiers::default()));
    tree.edit_text_input(&key(Key::Delete, None, Modifiers::default()));
    assert_eq!(tree.text_input_value(node), Some(""));

    tree.ime_input(ImeInput::Preedit {
        text: "é".into(),
        cursor: None,
    });
    assert_eq!(tree.text_input_display(node).as_deref(), Some("é"));
}

#[test]
fn modifier_and_selection_paths_keep_editing_deterministic() {
    let (mut tree, region) = tree("one two");
    focus(&mut tree, &region);
    let node = region.node;
    let alt = Modifiers {
        alt: true,
        ..Modifiers::default()
    };
    let shift = Modifiers {
        shift: true,
        ..Modifiers::default()
    };
    let command = Modifiers {
        control: true,
        ..Modifiers::default()
    };

    tree.edit_text_input(&key(Key::Home, None, Modifiers::default()));
    tree.edit_text_input(&key(Key::ArrowRight, None, alt));
    tree.edit_text_input(&key(Key::ArrowLeft, None, alt));
    tree.edit_text_input(&key(Key::ArrowRight, None, shift));
    tree.edit_text_input(&key(Key::ArrowRight, None, shift));
    assert_eq!(tree.text_input_selection(node), Some((0, 2)));
    tree.edit_text_input(&key(Key::Backspace, None, Modifiers::default()));
    assert_eq!(tree.text_input_value(node), Some("e two"));

    assert!(
        !tree
            .edit_text_input(&key(Key::Home, None, command))
            .layout_changed
    );
    assert!(
        !tree
            .edit_text_input(&key(
                Key::Character(String::new()),
                Some(""),
                Modifiers::default()
            ))
            .layout_changed
    );
}

#[test]
fn controlled_value_replaces_internal_state_only_when_it_differs() {
    let (mut tree, region) = tree("seed");
    focus(&mut tree, &region);
    let node = region.node;
    tree.paste_text(" value");

    let replacement = TextInput::new(
        "field",
        "ignored initial value",
        "new placeholder",
        TextInputStyle::new(PaintStyle::default(), TextStyle::default()),
    )
    .build();
    tree.update(replacement);

    assert_eq!(tree.text_input_value(node), Some("ignored initial value"));

    tree.place_text_cursor(node, 7, false);
    let same = TextInput::new(
        "field",
        "ignored initial value",
        "same value",
        TextInputStyle::new(PaintStyle::default(), TextStyle::default()),
    )
    .build();
    tree.update(same);
    assert_eq!(tree.text_input_cursor(node), Some(7));
}

#[test]
fn text_area_inserts_lines_and_command_enter_submits() {
    let area = TextArea::new(
        "notes",
        "first",
        "notes",
        TextInputStyle::new(PaintStyle::default(), TextStyle::default()),
    )
    .build();
    assert_eq!(area.paint.clip, ClipBehavior::Bounds);
    assert!(area.scroll.is_some());
    let mut tree = UiTree::new(area);
    let node = tree.node_ids()[0];
    let bounds = Rect::new(Point::default(), Size::new(300.0, 140.0));
    let region = HitRegion {
        node,
        bounds,
        transform: Affine2D::IDENTITY,
        clips: ClipChain::from_regions([ClipRegion::new(bounds, Affine2D::IDENTITY)]),
        focusable: true,
        cursor: CursorIcon::Text,
        gestures: argui_ui::GestureSet::NONE,
        window_drag: None,
    };
    focus(&mut tree, &region);
    let newline = tree.edit_text_input(&key(Key::Enter, None, Modifiers::default()));
    assert!(matches!(
        &newline.events[0].kind,
        UiEventKind::TextChanged(value) if value == "first\n"
    ));
    let command = Modifiers {
        control: true,
        ..Modifiers::default()
    };
    let submitted = tree.edit_text_input(&key(Key::Enter, None, command));
    assert!(matches!(
        &submitted.events[0].kind,
        UiEventKind::Submitted(value) if value == "first\n"
    ));

    tree.update(
        TextArea::new(
            "notes",
            "first\nsecond",
            "notes",
            TextInputStyle::new(PaintStyle::default(), TextStyle::default()),
        )
        .build(),
    );
    tree.place_text_cursor(node, 9, false);
    tree.edit_text_input(&key(Key::Home, None, Modifiers::default()));
    assert_eq!(tree.text_input_cursor(node), Some(6));
    tree.edit_text_input(&key(Key::End, None, Modifiers::default()));
    assert_eq!(tree.text_input_cursor(node), Some(12));
    tree.edit_text_input(&key(Key::Home, None, command));
    assert_eq!(tree.text_input_cursor(node), Some(0));
    tree.paste_text("pasted\nlines");
    assert_eq!(
        tree.text_input_value(node),
        Some("pasted\nlinesfirst\nsecond")
    );
}

#[test]
fn visual_positions_preserve_affinity_across_pointer_and_keyboard_updates() {
    let (mut tree, region) = tree("abc العربية xyz");
    focus(&mut tree, &region);
    let node = region.node;
    let after = TextPosition::new(4, CaretAffinity::After);
    let before = TextPosition::new(4, CaretAffinity::Before);

    tree.place_text_position(node, after, false);
    assert_eq!(tree.text_input_position(node), Some(after));
    tree.drag_text_position(node, before);
    assert_eq!(tree.text_input_position(node), Some(before));
    tree.move_text_position(node, after, true);
    assert_eq!(tree.text_input_position(node), Some(after));

    tree.update(argui_ui::Element::container([]));
    assert!(
        !tree
            .place_text_position(node, before, false)
            .text_input_changed
    );
    assert!(!tree.drag_text_position(node, before).text_input_changed);
    assert!(
        !tree
            .move_text_position(node, before, false)
            .text_input_changed
    );
}
