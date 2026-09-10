use argui_core::{Color, ColorScheme, Key, KeyInput, KeyState, Size};
use argui_layout::LayoutEngine;
use argui_text::TextEngine;
use argui_ui::{
    ClickEvent, Element, EventHandlerId, EventListener, EventOwnerId, EventType, FocusRequest,
    Role, UiEvent, UiEventKind, UiTree,
};
use argui_widgets::{Button, Collapsible, shadcn};

#[test]
fn long_trigger_labels_wrap_inside_the_button_with_room_for_the_indicator() {
    let palette = shadcn(Color::WHITE);
    let theme = palette.resolve(ColorScheme::Dark);
    for width in [180.0, 480.0] {
        for custom in [false, true] {
            let label = "Three components in this shared project";
            let mut disclosure = Collapsible::new("files", label, false, Element::text("Content"))
                .indicator(
                    Element::container([])
                        .width(argui_ui::length(16.0))
                        .height(argui_ui::length(16.0)),
                );
            if custom {
                disclosure = disclosure.trigger(Element::text(label));
            }
            let mut tree = UiTree::new(disclosure.build(theme));
            let output = LayoutEngine::new()
                .compute(&mut tree, &mut TextEngine::new(), Size::new(width, 400.0))
                .unwrap();
            let trigger = output
                .nodes
                .iter()
                .find(|node| tree.key(node.node) == Some("files::trigger"))
                .unwrap()
                .bounds;
            for node in &output.nodes {
                if matches!(
                    &tree.element_at(node.index).unwrap().kind,
                    argui_ui::ElementKind::Text { .. }
                ) {
                    assert!(
                        node.bounds.origin.x + node.bounds.size.width
                            <= trigger.origin.x + trigger.size.width
                    );
                    assert!(
                        node.bounds.origin.y + node.bounds.size.height
                            <= trigger.origin.y + trigger.size.height
                    );
                }
            }
            if width == 480.0 {
                assert!(
                    trigger.size.height < 48.0,
                    "short labels must use the available row width"
                );
            }
        }
    }
}

#[test]
fn closed_content_leaves_neither_focus_targets_nor_accessible_references() {
    let palette = shadcn(Color::WHITE);
    let theme = palette.resolve(ColorScheme::Dark);
    let build = |open| {
        Collapsible::new(
            "details",
            "Details",
            open,
            Button::new("inside", "Inner action", theme.button()).build(),
        )
        .trigger(Element::text("Custom trigger"))
        .indicator(Element::text("chevron"))
        .build(theme)
    };
    let mut tree = UiTree::new(build(false));
    for open in [false, true, false] {
        tree.update(build(open));
        assert_eq!(
            tree.node_ids()
                .iter()
                .any(|node| tree.key(*node) == Some("inside")),
            open
        );
        assert!(tree.semantic_diagnostics().is_empty());
        let semantic = tree.semantic_tree(&[], 1.0);
        let trigger = semantic
            .nodes
            .iter()
            .find(|node| node.semantics.label.as_deref() == Some("Details"))
            .unwrap();
        assert_eq!(trigger.semantics.state.expanded, Some(open));
        assert_eq!(
            trigger.semantics.relations.controls.len(),
            usize::from(open)
        );
        assert_eq!(
            semantic
                .nodes
                .iter()
                .filter(|node| node.semantics.focus_policy.is_tab_stop())
                .count(),
            if open { 2 } else { 1 }
        );
        if open {
            assert!(
                semantic
                    .nodes
                    .iter()
                    .any(|node| node.semantics.role == Role::Group
                        && node.semantics.relations.labelled_by == [trigger.id])
            );
        }
    }
}

#[test]
fn keyboard_activation_uses_enter_press_and_space_release_without_repeating() {
    let palette = shadcn(Color::WHITE);
    let theme = palette.resolve(ColorScheme::Light);
    let disclosure = Collapsible::new("details", "Details", false, Element::text("Content"));
    let root = disclosure.clone().build(theme).on(EventListener::new(
        EventType::Click,
        EventHandlerId::new(EventOwnerId(1), 0),
    ));
    let mut tree = UiTree::new(root);
    let output = LayoutEngine::new()
        .compute(&mut tree, &mut TextEngine::new(), Size::new(300.0, 300.0))
        .unwrap();
    let trigger = tree
        .node_ids()
        .iter()
        .copied()
        .find(|node| tree.key(*node) == Some(disclosure.trigger_key().as_str()))
        .unwrap();
    tree.sync_focus(
        &output.hit_regions,
        Some(FocusRequest::Focus(trigger.into())),
    );
    for (key, state, repeat, expected) in [
        (Key::Enter, KeyState::Pressed, false, Some(true)),
        (Key::Enter, KeyState::Pressed, true, None),
        (Key::Enter, KeyState::Released, false, None),
        (Key::Character(" ".into()), KeyState::Pressed, false, None),
        (
            Key::Character(" ".into()),
            KeyState::Released,
            false,
            Some(true),
        ),
    ] {
        let update = tree.key_input(
            &KeyInput {
                key,
                state,
                repeat,
                modifiers: Default::default(),
                text: None,
            },
            &output.hit_regions,
        );
        assert_eq!(
            update
                .events
                .iter()
                .find_map(|event| disclosure.action(event)),
            expected
        );
    }
}

#[test]
fn only_enabled_trigger_activation_requests_a_controlled_state_change() {
    let palette = shadcn(Color::WHITE);
    let theme = palette.resolve(ColorScheme::Light);
    let node = UiTree::new(Element::container([])).node_ids()[0];
    for open in [false, true] {
        for enabled in [false, true] {
            let disclosure = Collapsible::new("details", "Details", open, Element::text("Content"))
                .enabled(enabled);
            for (key, kind, activates) in [
                (
                    "details::trigger",
                    UiEventKind::Click(ClickEvent::accessibility()),
                    true,
                ),
                (
                    "other::trigger",
                    UiEventKind::Click(ClickEvent::accessibility()),
                    false,
                ),
                (
                    "details::content",
                    UiEventKind::Click(ClickEvent::accessibility()),
                    false,
                ),
                ("details::trigger", UiEventKind::Focused, false),
            ] {
                assert_eq!(
                    disclosure.action(&UiEvent::new(node, Some(key.into()), kind)),
                    (enabled && activates).then_some(!open)
                );
            }
            let root = disclosure.build(theme);
            let trigger = &root.children[0];
            assert_eq!(trigger.semantics.as_ref().unwrap().state.disabled, !enabled);
            assert_eq!(
                trigger
                    .interaction
                    .as_ref()
                    .unwrap()
                    .focus_policy
                    .is_tab_stop(),
                enabled
            );
        }
    }
}
