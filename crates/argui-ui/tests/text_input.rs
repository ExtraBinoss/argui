#[path = "text_input/filter.rs"]
mod filter;
use argui_core::{
    Affine2D, CaretAffinity, ImeInput, Key, KeyInput, KeyState, Modifiers, Point, PointerButton,
    PointerEvent, PointerId, PointerKind, PointerPhase, Rect, Size, TextPosition,
};
use argui_paint::{ClipChain, ClipRegion};
use argui_text::TextStyle;
use argui_ui::{
    CaretStyle, ClipboardRequest, CursorIcon, Element, ElementKind, EventHandlerId, EventListener,
    EventOwnerId, EventType, FocusPolicy, HitRegion, Interaction, TextEditorSpec, TextInputFilter,
    TextPrivacy, TextSelection, TextSelectionRequest, UiEventKind, UiTree,
};

fn tree(value: &str) -> (UiTree, HitRegion) {
    filtered_tree(value, TextInputFilter::Any)
}

fn filtered_tree(value: &str, filter: TextInputFilter) -> (UiTree, HitRegion) {
    with_region(UiTree::new(listens(field(value, filter))))
}

fn editor_with_privacy(
    value: &str,
    filter: TextInputFilter,
    privacy: TextPrivacy,
) -> (UiTree, HitRegion) {
    let element = field(value, filter).text_privacy(privacy);
    let (mut tree, region) = with_region(UiTree::new(listens(element)));
    focus(&mut tree, &region);
    (tree, region)
}

fn field(value: &str, filter: TextInputFilter) -> Element {
    text_editor("field", value, "placeholder", false, false, filter)
}

fn text_editor(
    key: &str,
    value: &str,
    placeholder: &str,
    multiline: bool,
    read_only: bool,
    filter: TextInputFilter,
) -> Element {
    Element::text_editor(TextEditorSpec {
        value: value.to_owned(),
        placeholder: placeholder.to_owned(),
        multiline,
        read_only,
        filter,
        text: TextStyle::default(),
        placeholder_text: TextStyle::default(),
        selection: argui_ui::Color::WHITE,
        caret: CaretStyle::default(),
    })
    .keyed(key)
    .interaction(
        Interaction::default()
            .focus_policy(FocusPolicy::TabStop)
            .cursor(CursorIcon::Text),
    )
}

fn multiline_field(value: &str, placeholder: &str) -> Element {
    text_editor(
        "notes",
        value,
        placeholder,
        true,
        false,
        TextInputFilter::Any,
    )
}

fn with_region(tree: UiTree) -> (UiTree, HitRegion) {
    let node = tree.node_id_at(0).unwrap();
    let bounds = Rect::new(Point::default(), Size::new(300.0, 40.0));
    (
        tree,
        HitRegion {
            node,
            bounds,
            transform: Affine2D::IDENTITY,
            clips: ClipChain::from_regions([ClipRegion::new(bounds, Affine2D::IDENTITY)]),
            shape: argui_ui::HitShape::Bounds,
            slop: argui_ui::HitTestStyle::default().slop,
            enabled: true,
            focus_policy: argui_ui::FocusPolicy::TabStop,
            cursor: CursorIcon::Text,
            gestures: argui_ui::GestureSet::EMPTY,
            window_drag: None,
        },
    )
}

/// A native editor with no PointerDown listener still has a focusable touch target.
#[test]
fn touch_press_focuses_editor_without_pointer_listener() {
    let input = text_editor("field", "", "Search", false, false, TextInputFilter::Any);
    let (mut tree, region) = with_region(UiTree::new(input));
    let pointer = PointerId::new(7);
    let event = PointerEvent {
        id: pointer,
        kind: PointerKind::Touch,
        phase: PointerPhase::Pressed,
        position: Point::new(100.0, 20.0),
        button: Some(PointerButton::Primary),
        buttons: 1,
        pressure: Some(1.0),
        primary: true,
        modifiers: Modifiers::default(),
        timestamp: std::time::Duration::ZERO,
    };
    let update = tree.pointer_event(event, std::slice::from_ref(&region));
    assert!(update.events.is_empty());
    tree.focus_pointer_default(pointer, std::slice::from_ref(&region));
    assert_eq!(tree.focused_node(), Some(region.node));
}

#[test]
fn numeric_filters_reject_invalid_keyboard_ime_and_paste_edits() {
    let (mut decimal, region) = filtered_tree("", TextInputFilter::Decimal);
    focus(&mut decimal, &region);
    let letter = decimal.edit_text_input(&key(
        Key::Character("x".into()),
        Some("x"),
        Modifiers::default(),
    ));
    assert!(letter.events.is_empty());
    assert_eq!(decimal.text_input_value(region.node), Some(""));
    assert!(decimal.paste_text(None, "12.5").layout_changed);
    assert!(!decimal.paste_text(None, "px").layout_changed);
    assert!(
        !decimal
            .ime_input(ImeInput::Commit("a".into()))
            .layout_changed
    );
    assert_eq!(decimal.text_input_value(region.node), Some("12.5"));

    let (mut expression, region) = filtered_tree("", TextInputFilter::Arithmetic);
    focus(&mut expression, &region);
    assert!(expression.paste_text(None, "(50 + 10) / 2").layout_changed);
    assert!(!expression.paste_text(None, "px").layout_changed);
    assert_eq!(
        expression.text_input_value(region.node),
        Some("(50 + 10) / 2")
    );
}

fn read_only_tree(value: &str) -> (UiTree, HitRegion) {
    let input = text_editor(
        "field",
        value,
        "placeholder",
        false,
        true,
        TextInputFilter::Any,
    );
    with_region(UiTree::new(listens(input)))
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

fn listens(element: Element) -> Element {
    EventType::ALL
        .into_iter()
        .enumerate()
        .fold(element, |element, (slot, event)| {
            element.on(EventListener::new(
                event,
                EventHandlerId::new(EventOwnerId(1), slot as u32),
            ))
        })
}

#[test]
fn editing_respects_graphemes_selection_and_clipboard_requests() {
    let (mut tree, region) = tree("A👋🏽");
    focus(&mut tree, &region);
    let node = region.node;

    let deleted = tree.edit_text_input(&key(Key::Backspace, None, Modifiers::default()));
    assert_eq!(tree.text_input_value(node), Some("A"));
    assert!(matches!(deleted.events[0].kind, UiEventKind::TextEdited(_)));
    assert!(matches!(
        &deleted.events[1].kind,
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
    assert_eq!(
        paste.clipboard,
        Some(ClipboardRequest::Read { target: None })
    );
    tree.paste_text(None, "é");
    assert_eq!(tree.text_input_value(node), Some("é"));
}

#[test]
fn incremental_listener_receives_a_delta_without_a_full_value_event() {
    let input = |value: &str| {
        field(value, TextInputFilter::Any).on(EventListener::new(
            EventType::TextEdit,
            EventHandlerId::new(EventOwnerId(1), 0),
        ))
    };
    let (mut tree, region) = with_region(UiTree::new(input("hello world")));
    focus(&mut tree, &region);

    let update = tree.edit_text_input(&key(
        Key::Character("!".into()),
        Some("!"),
        Modifiers::default(),
    ));

    assert_eq!(update.events.len(), 1);
    assert!(matches!(
        &update.events[0].kind,
        UiEventKind::TextEdited(edit)
            if edit.range == (11..11) && edit.replacement == "!"
    ));
    let node = region.node;
    tree.update(input("hello world!"));
    let next = tree.replace_text_input(node, "hello world!!");
    assert!(
        next.events
            .iter()
            .any(|event| matches!(event.kind, UiEventKind::TextEdited(_)))
    );
    assert!(
        !next
            .events
            .iter()
            .any(|event| matches!(event.kind, UiEventKind::TextChanged(_)))
    );
    tree.update(input("hello world!"));
    assert_eq!(tree.text_input_value(node), Some("hello world!!"));
    tree.update(input("external reset"));
    assert_eq!(tree.text_input_value(node), Some("external reset"));
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
    assert!(matches!(commit.events[0].kind, UiEventKind::TextEdited(_)));
    assert!(matches!(
        &commit.events[1].kind,
        UiEventKind::TextChanged(value) if value == "é"
    ));
    assert_eq!(tree.text_input_value(node), Some("é"));
}

#[test]
fn read_only_inputs_allow_navigation_selection_and_copy_without_mutation() {
    let (mut tree, region) = read_only_tree("readonly");
    focus(&mut tree, &region);
    let node = region.node;
    tree.edit_text_input(&key(Key::Home, None, Modifiers::default()));
    tree.edit_text_input(&key(
        Key::ArrowRight,
        None,
        Modifiers {
            shift: true,
            ..Modifiers::default()
        },
    ));
    assert_eq!(tree.text_input_selection(node), Some((0, 1)));
    let command = Modifiers {
        control: true,
        ..Modifiers::default()
    };
    assert_eq!(
        tree.edit_text_input(&key(Key::Character("c".into()), Some("c"), command))
            .clipboard,
        Some(ClipboardRequest::Write("r".into()))
    );
    assert_eq!(
        tree.edit_text_input(&key(Key::Character("x".into()), Some("x"), command))
            .clipboard,
        None
    );
    assert_eq!(
        tree.edit_text_input(&key(Key::Character("v".into()), Some("v"), command))
            .clipboard,
        None
    );
    tree.edit_text_input(&key(Key::Backspace, None, Modifiers::default()));
    tree.edit_text_input(&key(Key::Delete, None, Modifiers::default()));
    tree.edit_text_input(&key(
        Key::Character("z".into()),
        Some("z"),
        Modifiers::default(),
    ));
    assert_eq!(tree.text_input_value(node), Some("readonly"));
    assert!(!tree.paste_text(None, "mutate").layout_changed);
    assert!(
        !tree
            .ime_input(ImeInput::Commit("mutate".into()))
            .layout_changed
    );
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
    tree.edit_text_input(&key(Key::ArrowLeft, None, Modifiers::default()));
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
    assert!(!tree.paste_text(None, "x").layout_changed);
    assert!(!tree.ime_input(ImeInput::Enabled).layout_changed);
    assert!(!tree.edit_text_input(&released).layout_changed);

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
    assert!(!tree.paste_text(None, "").layout_changed);
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
fn controlled_value_replaces_internal_state_only_when_it_differs() {
    let (mut tree, region) = tree("seed");
    focus(&mut tree, &region);
    let node = region.node;
    tree.paste_text(None, " value");

    let replacement = text_editor(
        "field",
        "ignored initial value",
        "new placeholder",
        false,
        false,
        TextInputFilter::Any,
    );
    tree.update(replacement);

    assert_eq!(tree.text_input_value(node), Some("ignored initial value"));

    tree.place_text_cursor(node, 7, false);
    let same = text_editor(
        "field",
        "ignored initial value",
        "same value",
        false,
        false,
        TextInputFilter::Any,
    );
    tree.update(same);
    assert_eq!(tree.text_input_cursor(node), Some(7));
}

#[test]
fn text_area_inserts_lines_and_command_enter_submits() {
    let area = multiline_field("first", "notes");
    let mut tree = UiTree::new(listens(area));
    let node = tree.node_ids()[0];
    let bounds = Rect::new(Point::default(), Size::new(300.0, 140.0));
    let region = HitRegion {
        node,
        bounds,
        transform: Affine2D::IDENTITY,
        clips: ClipChain::from_regions([ClipRegion::new(bounds, Affine2D::IDENTITY)]),
        shape: argui_ui::HitShape::Bounds,
        slop: argui_ui::HitTestStyle::default().slop,
        enabled: true,
        focus_policy: argui_ui::FocusPolicy::TabStop,
        cursor: CursorIcon::Text,
        gestures: argui_ui::GestureSet::EMPTY,
        window_drag: None,
    };
    focus(&mut tree, &region);
    let newline = tree.edit_text_input(&key(Key::Enter, None, Modifiers::default()));
    assert!(matches!(newline.events[0].kind, UiEventKind::TextEdited(_)));
    assert!(matches!(
        &newline.events[1].kind,
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

    tree.update(multiline_field("first\nsecond", "notes"));
    tree.place_text_cursor(node, 9, false);
    tree.edit_text_input(&key(Key::Home, None, Modifiers::default()));
    assert_eq!(tree.text_input_cursor(node), Some(6));
    tree.edit_text_input(&key(Key::End, None, Modifiers::default()));
    assert_eq!(tree.text_input_cursor(node), Some(12));
    tree.edit_text_input(&key(Key::End, None, command));
    assert_eq!(tree.text_input_cursor(node), Some(12));
    tree.edit_text_input(&key(Key::Home, None, command));
    assert_eq!(tree.text_input_cursor(node), Some(0));
    tree.paste_text(None, "pasted\nlines");
    assert_eq!(
        tree.text_input_value(node),
        Some("pasted\nlinesfirst\nsecond")
    );

    let mut readonly = tree.root().clone();
    if let ElementKind::TextEditor { read_only, .. } = &mut readonly.kind {
        *read_only = true;
    }
    tree.update(readonly);
    assert!(
        tree.edit_text_input(&key(Key::Enter, None, Modifiers::default()))
            .events
            .is_empty()
    );
}

#[path = "text_input/navigation.rs"]
mod navigation;
#[path = "text_input/selection.rs"]
mod selection;
