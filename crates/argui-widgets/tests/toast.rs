use argui_widgets::{Toast, ToastInsertError, ToastPause, ToastState};
use std::time::Duration;
fn sec(value: u64) -> Duration {
    Duration::from_secs(value)
}
fn toast(id: &str) -> Toast {
    Toast::new(id, id, Some(sec(10)))
}

#[test]
fn queue_capacity_duplicate_ids_and_visible_lifetimes_are_explicit() {
    let mut state = ToastState::new(1, 2, sec(0));
    state.insert(toast("a"), sec(0)).unwrap();
    state.insert(toast("b"), sec(0)).unwrap();
    assert_eq!(
        state.insert(toast("a"), sec(0)),
        Err(ToastInsertError::DuplicateId)
    );
    assert_eq!(
        state.insert(toast("c"), sec(0)),
        Err(ToastInsertError::Capacity)
    );
    assert_eq!(state.next_deadline(), Some(sec(10)));
    assert!(!state.advance(sec(5)));
    assert!(state.advance(sec(10)));
    assert_eq!(state.visible().next().unwrap().id, "b");
    assert_eq!(state.next_deadline(), Some(sec(20)));
    assert!(state.advance(sec(20)));
    assert!(state.is_empty());
    assert_eq!(state.next_deadline(), None);
}

#[test]
fn independent_pause_reasons_and_updates_do_not_lose_remaining_time() {
    let mut state = ToastState::new(2, 3, sec(0));
    state.insert(toast("a"), sec(0)).unwrap();
    state
        .insert(Toast::new("b", "Persistent", None), sec(0))
        .unwrap();
    assert!(state.pause("a", ToastPause::Hover, true, sec(3)));
    assert_eq!(state.next_deadline(), None);
    assert!(state.pause("a", ToastPause::Focus, true, sec(4)));
    assert!(state.pause("a", ToastPause::Hover, false, sec(5)));
    assert!(!state.advance(sec(30)));
    assert!(state.pause("a", ToastPause::Focus, false, sec(30)));
    assert_eq!(state.next_deadline(), Some(sec(37)));
    assert!(!state.pause("a", ToastPause::Focus, false, sec(31)));
    assert!(!state.advance(sec(20)));
    assert_eq!(state.next_deadline(), Some(sec(37)));
    assert!(state.update(Toast::new("a", "Updated", Some(sec(2))), sec(32)));
    assert_eq!(state.next_deadline(), Some(sec(34)));
    assert!(state.close("a", sec(33)));
    assert_eq!(state.len(), 1);
    assert_eq!(state.next_deadline(), None);
    assert!(!state.close("missing", sec(33)));
    assert!(!state.pause("missing", ToastPause::Hover, true, sec(33)));
    assert!(!state.update(toast("missing"), sec(33)));
}

#[test]
fn expiry_removes_multiple_visible_entries_before_promoting_the_queue() {
    let mut state = ToastState::new(2, 3, sec(0));
    for id in ["a", "b", "c"] {
        state.insert(toast(id), sec(0)).unwrap();
    }
    assert!(state.advance(sec(20)));
    assert_eq!(
        state
            .visible()
            .map(|toast| toast.id.as_str())
            .collect::<Vec<_>>(),
        ["c"]
    );
    assert_eq!(state.next_deadline(), Some(sec(30)));
}

#[test]
fn host_announces_errors_and_pauses_for_pointer_or_action_focus() {
    use argui_core::{Color, ColorScheme, Point, PointerEvent, PointerPhase};
    use argui_ui::{
        ActionId, ActionInvocation, ActionState, Element, Role, UiEvent, UiEventKind, UiTree,
    };
    use argui_widgets::{ToastHost, ToastVariant, shadcn};
    let mut state = ToastState::new(1, 2, sec(0));
    let mut error = toast("error");
    error.variant = ToastVariant::Error;
    error.actions.push((
        ActionInvocation::new(ActionId("retry")),
        ActionState::new("Retry"),
    ));
    state.insert(error, sec(0)).unwrap();
    state.insert(toast("queued"), sec(0)).unwrap();
    let host = ToastHost {
        key: "host",
        state: &state,
        close_label: "Close",
    };
    let themes = shadcn(Color::WHITE);
    let tree = UiTree::new(host.build(
        themes.resolve(ColorScheme::Light),
        &argui_widgets::WidgetAssets::tabler(Color::WHITE),
    ));
    assert!(
        tree.semantic_tree(&[], 1.0)
            .nodes
            .iter()
            .any(|node| node.semantics.role == Role::Alert)
    );
    let id = UiTree::new(Element::container([])).node_ids()[0];
    let event = |target: &str, kind| UiEvent::new(id, Some(target.into()), kind);
    for (phase, expected) in [
        (PointerPhase::Entered, Some(true)),
        (PointerPhase::Left, Some(false)),
        (PointerPhase::Moved, None),
    ] {
        let response = host.pause_action(&event(
            "host::toast::error",
            UiEventKind::Pointer(PointerEvent::mouse(phase, Point::default())),
        ));
        assert_eq!(
            response.map(|(_, reason, paused)| (reason, paused)),
            expected.map(|paused| (ToastPause::Hover, paused))
        );
    }
    assert_eq!(
        host.pause_action(&event("host::action::error::0", UiEventKind::Focused)),
        Some(("error".into(), ToastPause::Focus, true))
    );
    assert!(
        host.pause_action(&event("unknown", UiEventKind::Focused))
            .is_none()
    );
    assert!(
        host.close_action(&event(
            "host::close::queued",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility())
        ))
        .is_none()
    );
}

#[test]
fn notifications_float_at_the_viewport_corner_and_keep_every_variant_readable() {
    use argui_core::{Color, ColorScheme, Size};
    use argui_ui::{Element, UiTree};
    use argui_widgets::{ToastHost, ToastVariant, WidgetAssets, shadcn};
    let themes = shadcn(Color::from_srgb8(30, 100, 220));
    let icons = WidgetAssets::tabler(Color::WHITE);
    for variant in [
        ToastVariant::Information,
        ToastVariant::Success,
        ToastVariant::Warning,
        ToastVariant::Error,
    ] {
        let mut state = ToastState::new(1, 1, sec(0));
        let mut notification = toast("notice");
        notification.variant = variant;
        notification.description = "Your changes have been saved.".into();
        state.insert(notification, sec(0)).unwrap();
        let host = ToastHost {
            key: "host",
            state: &state,
            close_label: "Close notification",
        };
        for scheme in [ColorScheme::Light, ColorScheme::Dark] {
            let theme = themes.resolve(scheme);
            let built = host.build(theme, &icons);
            assert!(
                built.children[0]
                    .layer
                    .as_ref()
                    .is_some_and(|layer| !layer.shadows.is_empty())
            );
            assert_eq!(
                built.portal.as_ref().unwrap().layer,
                argui_ui::WindowLayer::Popover
            );
            let mut tree = UiTree::new(Element::column([Element::text("Page"), built]));
            let mut text = argui_text::TextEngine::new();
            for width in [320.0, 1000.0] {
                let output = argui_layout::LayoutEngine::new()
                    .compute(&mut tree, &mut text, Size::new(width, 720.0))
                    .unwrap();
                let host_id = tree
                    .node_ids()
                    .iter()
                    .find(|id| tree.key(**id) == Some("host"))
                    .unwrap();
                let bounds = output
                    .nodes
                    .iter()
                    .find(|node| node.node == *host_id)
                    .unwrap()
                    .bounds;
                assert!((bounds.origin.x + bounds.size.width - (width - 24.0)).abs() <= 1.0);
                assert!((bounds.origin.y + bounds.size.height - 696.0).abs() <= 1.0);
                assert!(bounds.origin.x >= 24.0);
            }
        }
    }
}
