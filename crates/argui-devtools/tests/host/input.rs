use super::*;
use argui_core::{Key, KeyInput, KeyState, Modifiers};

fn typing(key: &str, text: &str, modifiers: Modifiers) -> UiEvent {
    event(
        key,
        UiEventKind::KeyInput(KeyInput {
            key: Key::Character(text.into()),
            state: KeyState::Pressed,
            text: Some(text.into()),
            modifiers,
            repeat: false,
        }),
    )
}

#[test]
fn hovering_tree_rows_highlights_without_selecting_or_rebuilding() {
    let mut host = populated_host();
    let inspector = host.inspector();
    inspector.select(Some(InspectNodeId(1)));
    let entered = event(
        "__devtools-node-2",
        pointer(PointerPhase::Entered, Point::default()),
    );
    assert_eq!(host.update(&entered), ViewUpdate::Paint);
    assert_eq!(inspector.highlighted(), Some(InspectNodeId(2)));
    assert_eq!(inspector.selected(), Some(InspectNodeId(1)));
    assert_eq!(host.update(&entered), ViewUpdate::None);
    assert_eq!(
        host.update(&event(
            "__devtools-properties",
            pointer(PointerPhase::Entered, Point::default())
        )),
        ViewUpdate::Paint
    );
    assert_eq!(inspector.highlighted(), None);
    assert_eq!(inspector.selected(), Some(InspectNodeId(1)));
    host.update(&entered);
    assert_eq!(
        host.update(&event(
            "__devtools-node-2",
            pointer(PointerPhase::Left, Point::default())
        )),
        ViewUpdate::Paint
    );
    assert_eq!(inspector.highlighted(), None);
    assert_eq!(
        host.update(&event(
            "__devtools-node-invalid",
            pointer(PointerPhase::Entered, Point::default())
        )),
        ViewUpdate::None
    );
}

#[test]
fn typing_filters_without_losing_unicode_or_stealing_editor_input() {
    let host = Entity::new(populated_host()).mount().unwrap();
    let typed = typing("__devtools-node-1", "é", Modifiers::default());
    assert_eq!(
        change_tools(&host, |tools| tools.update(&typed)),
        ViewUpdate::Rebuild
    );
    assert!(typed.default_prevented());
    assert!(typed.propagation_stopped());
    host.update(|tools, _| {
        assert_eq!(
            tools.take_focus_request(),
            Some(argui_ui::FocusRequest::Focus("__devtools-search".into()))
        );
        assert_eq!(
            tools.take_text_selection_request(),
            Some(argui_ui::TextSelectionRequest::new(
                "__devtools-search",
                argui_ui::TextSelection::Caret(argui_core::TextPosition::new(
                    2,
                    argui_core::CaretAffinity::After
                ))
            ))
        );
    })
    .unwrap();
    let filtered = host.render(Default::default()).unwrap();
    let search = find_element(&filtered, "__devtools-search").unwrap();
    assert!(
        matches!(&search.kind, argui_ui::ElementKind::TextEditor { value, .. } if value == "é")
    );
    for key in [
        "__devtools-search",
        "__devtools-value-2-width-0",
        "__devtools-color-background::field",
        "app-content",
    ] {
        let input = typing(key, "2", Modifiers::default());
        assert_eq!(
            change_tools(&host, |tools| tools.update(&input)),
            ViewUpdate::None
        );
        assert!(!input.default_prevented());
    }
    let shortcut = typing(
        "__devtools-profiling",
        "f",
        Modifiers {
            control: true,
            ..Modifiers::default()
        },
    );
    assert_eq!(
        change_tools(&host, |tools| tools.update(&shortcut)),
        ViewUpdate::Rebuild
    );
    host.update(|tools, _| {
        assert_eq!(
            tools.take_text_selection_request().unwrap().selection,
            argui_ui::TextSelection::All
        );
    })
    .unwrap();
}

#[test]
fn filter_ignores_shortcuts_releases_control_text_and_dock_editor_typing() {
    let host = Entity::new(populated_host()).mount().unwrap();
    for (target, text, state, control, alt) in [
        ("__devtools-node-1", "x", KeyState::Released, false, false),
        ("__devtools-node-1", "x", KeyState::Pressed, false, true),
        ("__devtools-node-1", "x", KeyState::Pressed, true, false),
        ("__devtools-node-1", " ", KeyState::Pressed, false, false),
        ("__devtools-node-1", "x\n", KeyState::Pressed, false, false),
        ("__devtools-dock", "x", KeyState::Pressed, false, false),
    ] {
        let input = event(
            target,
            UiEventKind::KeyInput(KeyInput {
                key: Key::Character(text.into()),
                state,
                text: Some(text.into()),
                repeat: false,
                modifiers: Modifiers {
                    control,
                    alt,
                    ..Default::default()
                },
            }),
        );
        change_tools(&host, |tools| tools.update(&input));
        assert!(!input.default_prevented(), "{target}: {text:?}");
        assert!(!input.propagation_stopped());
        host.update(|tools, _| {
            assert!(tools.take_focus_request().is_none());
            assert!(tools.take_text_selection_request().is_none());
        })
        .unwrap();
    }
    let view = host.render(Default::default()).unwrap();
    let search = find_element(&view, "__devtools-search").unwrap();
    assert!(
        matches!(&search.kind, argui_ui::ElementKind::TextEditor { value, .. } if value.is_empty())
    );
    change_tools(&host, |tools| {
        tools.update(&event(
            "__devtools-profiling",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        ))
    });
    let input = typing("__devtools-panel", "x", Modifiers::default());
    change_tools(&host, |tools| tools.update(&input));
    assert!(!input.default_prevented());
    assert!(contains_key(
        &host.render(Default::default()).unwrap(),
        "__devtools-profiling-panel"
    ));
}

#[test]
fn property_splitter_changes_layout_and_keeps_both_panes_visible() {
    let host = Entity::new(populated_host()).mount().unwrap();
    host.read(|tools| tools.inspector())
        .select(Some(InspectNodeId(2)));
    let before = host.render(Default::default()).unwrap();
    assert!(contains_key(&before, "__devtools-properties-splitter"));
    let input = event(
        "__devtools-properties-splitter",
        UiEventKind::KeyInput(KeyInput {
            key: Key::ArrowLeft,
            state: KeyState::Pressed,
            modifiers: Modifiers::default(),
            repeat: false,
            text: None,
        }),
    );
    assert_eq!(
        change_tools(&host, |tools| tools.update(&input)),
        ViewUpdate::Rebuild
    );
    let after = host.render(Default::default()).unwrap();
    let old = find_element(&before, "__devtools-properties-splitter").unwrap();
    let new = find_element(&after, "__devtools-properties-splitter").unwrap();
    assert_ne!(
        old.semantics.as_ref().unwrap().value,
        new.semantics.as_ref().unwrap().value
    );
    assert!(contains_key(&after, "__devtools-tree"));
    assert!(contains_key(&after, "__devtools-reset"));
}

pub(super) fn find_element<'a>(element: &'a Element, key: &str) -> Option<&'a Element> {
    if element.key.as_deref() == Some(key) {
        Some(element)
    } else {
        element
            .children
            .iter()
            .find_map(|child| find_element(child, key))
    }
}
