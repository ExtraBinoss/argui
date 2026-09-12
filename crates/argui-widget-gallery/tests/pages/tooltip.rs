use super::*;
use argui::{
    core::{Key, KeyInput, KeyState, Modifiers, Point, PointerEvent, PointerPhase},
    runtime::tasks::TaskRuntime,
};
use std::{sync::mpsc, time::Duration};

#[test]
fn tooltip_focus_escape_and_click_keep_the_buttons_usable() {
    let app = Entity::new(WidgetGallery::default());
    click(&app, "nav::tooltip");
    for (key, status) in [
        ("tooltip-save", "Draft saved locally."),
        ("tooltip-preview", "Page preview is ready."),
        ("tooltip-history", "History: three saved versions."),
    ] {
        dispatch(&app, key, UiEventKind::Focused);
        let root = app.render();
        let panel = keyed(&root, &format!("{key}::content")).unwrap();
        assert_eq!(
            panel.semantics.as_ref().unwrap().role,
            argui::ui::Role::Tooltip
        );
        assert!(panel.focus_scope.is_none());
        assert!(
            !keyed(&root, key)
                .unwrap()
                .semantic_bindings
                .described_by
                .is_empty()
        );
        dispatch(
            &app,
            "gallery-search",
            UiEventKind::KeyInput(KeyInput {
                key: Key::Escape,
                state: KeyState::Pressed,
                modifiers: Modifiers::default(),
                repeat: false,
                text: None,
            }),
        );
        assert!(keyed(&app.render(), &format!("{key}::content")).is_none());
        click(&app, key);
        assert!(contains_text(&app.render(), status));
        dispatch(&app, key, UiEventKind::Blurred);
    }
}

#[test]
fn tooltip_timers_cancel_fast_passes_and_close_after_leaving() {
    let app = Entity::new(WidgetGallery::default());
    let (sender, receiver) = mpsc::channel();
    let runtime = TaskRuntime::new(move || {
        let _ = sender.send(());
    });
    app.set_task_runtime(runtime.clone());
    click(&app, "nav::tooltip");
    let pointer = |key, phase| {
        dispatch(
            &app,
            key,
            UiEventKind::Pointer(PointerEvent::mouse(phase, Point::default())),
        )
    };
    let drain = || {
        while runtime.pending() > 0 {
            runtime.drain();
            if runtime.pending() > 0 {
                receiver.recv_timeout(Duration::from_secs(3)).unwrap();
            }
        }
    };
    pointer("tooltip-save", PointerPhase::Entered);
    pointer("tooltip-save", PointerPhase::Left);
    drain();
    assert!(keyed(&app.render(), "tooltip-save::content").is_none());
    pointer("tooltip-preview", PointerPhase::Entered);
    assert!(keyed(&app.render(), "tooltip-preview::content").is_none());
    drain();
    assert!(keyed(&app.render(), "tooltip-preview::content").is_some());
    pointer("tooltip-preview", PointerPhase::Left);
    pointer("tooltip-preview::content", PointerPhase::Entered);
    drain();
    assert!(keyed(&app.render(), "tooltip-preview::content").is_some());
    pointer("tooltip-preview::content", PointerPhase::Left);
    drain();
    assert!(keyed(&app.render(), "tooltip-preview::content").is_none());
}

#[test]
fn tooltip_can_still_show_keyboard_help_without_a_task_runtime() {
    let app = Entity::new(WidgetGallery::default());
    click(&app, "nav::tooltip");
    dispatch(
        &app,
        "tooltip-save",
        UiEventKind::Pointer(PointerEvent::mouse(PointerPhase::Entered, Point::default())),
    );
    assert!(keyed(&app.render(), "tooltip-save::content").is_none());
    dispatch(&app, "tooltip-save", UiEventKind::Focused);
    assert!(keyed(&app.render(), "tooltip-save::content").is_some());
}

#[test]
fn tooltip_panels_contain_every_wrapped_line_after_portal_placement() {
    let app = Entity::new(WidgetGallery::default());
    click(&app, "nav::tooltip");
    dispatch(&app, "tooltip-history", UiEventKind::Focused);
    let mut ui = UiTree::new(app.render());
    let font = include_bytes!("../../../argui-web-demo/assets/fonts/NotoSans-Regular.ttf");
    let mut text =
        TextEngine::from_embedded_fonts([font.as_slice()], "Noto Sans", "Noto Sans", "Noto Sans");
    let output = LayoutEngine::new()
        .compute(&mut ui, &mut text, Size::new(1220.0, 900.0))
        .unwrap();
    let panel = output
        .nodes
        .iter()
        .find(|node| ui.key(node.node) == Some("tooltip-history::content"))
        .unwrap();
    let block = output
        .text
        .blocks()
        .iter()
        .find(|block| block.content.as_str() == "See the previous versions of this project.")
        .unwrap();
    let measured = text.measure(
        block.content.as_str(),
        &block.style,
        Some(block.bounds.size.width),
    );
    assert!(
        block.bounds.origin.y + measured.height <= panel.bounds.origin.y + panel.bounds.size.height,
        "text {:?}, measured {measured:?}, panel {:?}",
        block.bounds,
        panel.bounds
    );
}

#[test]
fn leaving_overlay_pages_resets_hover_focus_and_open_panels() {
    let app = Entity::new(WidgetGallery::default());
    click(&app, "nav::tooltip");
    dispatch(&app, "tooltip-save", UiEventKind::Focused);
    click(&app, "tooltip-save");
    click(&app, "nav::popover");
    click(&app, "popover-project");
    click(&app, "nav::tooltip");
    assert!(keyed(&app.render(), "tooltip-save::content").is_none());
    dispatch(&app, "tooltip-save", UiEventKind::Focused);
    assert!(keyed(&app.render(), "tooltip-save::content").is_some());
    click(&app, "nav::popover");
    assert!(keyed(&app.render(), "popover-project::content").is_none());
}
