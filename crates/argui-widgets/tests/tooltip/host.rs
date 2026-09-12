use super::*;
use argui_runtime::{Context, Entity, Render, tasks::TaskRuntime};
use argui_ui::{EventType, FocusPolicy, Interaction};
use argui_widgets::{Button, Popover, TooltipHost};
use std::sync::mpsc;

struct App {
    open: bool,
    visible: bool,
    enabled: bool,
    clicks: usize,
}

impl Default for App {
    fn default() -> Self {
        Self {
            open: false,
            visible: true,
            enabled: true,
            clicks: 0,
        }
    }
}

impl Render for App {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = shadcn(Color::BLACK);
        let theme = themes.resolve(cx.environment().color_scheme);
        Element::column([
            Element::container([]).viewport_portal(
                argui_ui::WindowLayer::Popover,
                argui_ui::ViewportPlacement::centered(),
            ),
            Button::new("run", "Run", theme.button())
                .enabled(self.enabled)
                .tooltip("Run the current draft")
                .build()
                .semantic_hidden(!self.visible),
            Button::new("plain", "Plain", theme.button()).build(),
            Button::new("silent", "No tooltip", theme.button())
                .without_tooltip()
                .build(),
            Button::new("loading", "Loading", theme.button())
                .loading(Element::text("…"))
                .build(),
            Popover::new(
                "menu",
                "Settings",
                self.open,
                Button::new("menu", "Settings", theme.button()).build(),
                Element::text("Panel contents"),
            )
            .build(theme),
            Tooltip::new(
                "manual",
                "Managed elsewhere",
                false,
                Button::new("manual", "Manual", theme.button()).build(),
            )
            .build(theme),
        ])
        .on(cx.listener(EventType::Click, |app, event, cx| {
            match event.target_key() {
                Some("run") => app.clicks += 1,
                Some("menu") => app.open = !app.open,
                _ => return,
            }
            cx.notify();
        }))
    }
}

fn dispatch<A: Render>(host: &Entity<TooltipHost<A>>, key: &str, kind: UiEventKind) {
    let mut tree = UiTree::new(host.render());
    let node = tree
        .node_ids()
        .iter()
        .copied()
        .find(|id| tree.key(*id) == Some(key))
        .unwrap();
    for event in tree.event_deliveries(node, kind) {
        if event.should_dispatch() {
            host.dispatch_event(&event);
        }
    }
}

fn hovered<A: Render>(host: &Entity<TooltipHost<A>>, key: &str, phase: PointerPhase) {
    dispatch(
        host,
        key,
        UiEventKind::Pointer(PointerEvent::mouse(phase, Point::default())),
    );
}

fn tooltips(root: &Element) -> Vec<&Element> {
    let mut result = Vec::new();
    if root
        .semantics
        .as_ref()
        .is_some_and(|semantics| semantics.role == Role::Tooltip)
    {
        result.push(root);
    }
    for child in &root.children {
        result.extend(tooltips(child));
    }
    result
}

#[test]
fn default_custom_and_opted_out_button_help_preserves_actions_and_reopens_after_click() {
    let app = Entity::new(App::default());
    let host = Entity::new(TooltipHost::from_entity(app.clone()).delay(Duration::ZERO));
    hovered(&host, "plain", PointerPhase::Entered);
    assert_eq!(
        tooltips(&host.render())[0]
            .semantics
            .as_ref()
            .unwrap()
            .label
            .as_deref(),
        Some("Plain")
    );
    hovered(&host, "plain", PointerPhase::Left);
    hovered(&host, "run", PointerPhase::Entered);
    assert_eq!(
        tooltips(&host.render())[0]
            .semantics
            .as_ref()
            .unwrap()
            .label
            .as_deref(),
        Some("Run the current draft")
    );
    hovered(&host, "run", PointerPhase::Pressed);
    dispatch(&host, "run", UiEventKind::Focused);
    dispatch(
        &host,
        "run",
        UiEventKind::Click(ClickEvent::accessibility()),
    );
    assert_eq!(app.read(|app| app.clicks), 1);
    assert!(tooltips(&host.render()).is_empty());
    hovered(&host, "run", PointerPhase::Left);
    hovered(&host, "run", PointerPhase::Entered);
    assert_eq!(tooltips(&host.render()).len(), 1);
    hovered(&host, "run", PointerPhase::Left);
    dispatch(&host, "run", UiEventKind::Blurred);
    for key in ["silent", "loading", "manual"] {
        dispatch(&host, key, UiEventKind::Focused);
        assert!(tooltips(&host.render()).is_empty(), "{key}");
    }
}

#[test]
fn opening_a_popover_suppresses_help_until_it_closes_including_programmatic_opening() {
    let app = Entity::new(App::default());
    let host = Entity::new(TooltipHost::from_entity(app.clone()).delay(Duration::ZERO));
    hovered(&host, "menu", PointerPhase::Entered);
    assert_eq!(tooltips(&host.render()).len(), 1);
    dispatch(
        &host,
        "menu",
        UiEventKind::Click(ClickEvent::accessibility()),
    );
    assert!(app.read(|app| app.open));
    assert!(tooltips(&host.render()).is_empty());
    hovered(&host, "run", PointerPhase::Entered);
    assert!(tooltips(&host.render()).is_empty());
    app.update(|app, cx| {
        app.open = false;
        cx.notify();
    });
    let _ = host.render();
    hovered(&host, "run", PointerPhase::Entered);
    assert_eq!(tooltips(&host.render()).len(), 1);
    app.update(|app, cx| {
        app.open = true;
        cx.notify();
    });
    assert!(tooltips(&host.render()).is_empty());
}

#[test]
fn help_stays_in_the_triggers_semantic_scope_across_nested_entities() {
    struct Outer(Entity<App>);
    impl Render for Outer {
        fn render(&mut self, cx: &mut Context<Self>) -> Element {
            Element::row([Element::text("Surrounding content"), cx.entity(&self.0)])
        }
    }
    let host = Entity::new(TooltipHost::new(Outer(Entity::new(App::default()))));
    dispatch(&host, "run", UiEventKind::Focused);
    let mut ui = UiTree::new(host.render());
    argui_layout::LayoutEngine::new()
        .compute(
            &mut ui,
            &mut argui_text::TextEngine::new(),
            argui_core::Size::new(800.0, 600.0),
        )
        .unwrap();
    assert!(
        ui.semantic_diagnostics().is_empty(),
        "{:?}",
        ui.semantic_diagnostics()
    );
    assert_eq!(tooltips(&host.render()).len(), 1);
}

#[test]
fn timers_cancel_fast_passes_and_stale_help_when_a_trigger_is_hidden_or_disabled() {
    let app = Entity::new(App::default());
    let host = Entity::new(TooltipHost::from_entity(app.clone()).delay(ms(10)));
    let (sender, receiver) = mpsc::channel();
    let runtime = TaskRuntime::new(move || {
        let _ = sender.send(());
    });
    host.set_task_runtime(runtime.clone());
    let drain = || {
        while runtime.pending() > 0 {
            runtime.drain();
            if runtime.pending() > 0 {
                receiver.recv_timeout(Duration::from_secs(3)).unwrap();
            }
        }
    };
    hovered(&host, "run", PointerPhase::Entered);
    assert!(tooltips(&host.render()).is_empty());
    hovered(&host, "run", PointerPhase::Left);
    drain();
    assert!(tooltips(&host.render()).is_empty());
    hovered(&host, "run", PointerPhase::Entered);
    drain();
    assert_eq!(tooltips(&host.render()).len(), 1);
    hovered(&host, "run", PointerPhase::Left);
    hovered(&host, "run::content", PointerPhase::Entered);
    drain();
    assert_eq!(tooltips(&host.render()).len(), 1);
    hovered(&host, "run::content", PointerPhase::Left);
    drain();
    assert!(tooltips(&host.render()).is_empty());
    hovered(&host, "run", PointerPhase::Entered);
    app.update(|app, cx| {
        app.visible = false;
        cx.notify();
    });
    assert!(tooltips(&host.render()).is_empty());
    drain();
    app.update(|app, cx| {
        app.visible = true;
        app.enabled = false;
        cx.notify();
    });
    dispatch(&host, "run", UiEventKind::Focused);
    drain();
    assert!(tooltips(&host.render()).is_empty());
}

#[test]
fn raw_elements_can_request_custom_effect_help_without_a_task_service() {
    struct Leaf;
    impl Render for Leaf {
        fn render(&mut self, _: &mut Context<Self>) -> Element {
            Element::text("Help")
                .keyed("leaf")
                .tooltip("Extra details")
                .interaction(Interaction::default().focus_policy(FocusPolicy::TabStop))
        }
    }
    let layer = LayerStyle::new(Default::default()).backdrop(Filter::Blur(24.0));
    let paint = PaintStyle::new(QuadStyle::solid(Color::BLACK.with_alpha(0.8)));
    let host = Entity::new(
        TooltipHost::new(Leaf)
            .paint(paint.clone())
            .layer(layer.clone()),
    );
    let mut touch = PointerEvent::mouse(PointerPhase::Entered, Point::default());
    touch.kind = PointerKind::Touch;
    dispatch(&host, "leaf", UiEventKind::Pointer(touch));
    assert!(tooltips(&host.render()).is_empty());
    hovered(&host, "leaf", PointerPhase::Entered);
    let root = host.render();
    assert_eq!(tooltips(&root)[0].paint, paint);
    assert_eq!(tooltips(&root)[0].layer.as_ref(), Some(&layer));
    assert_eq!(root.children[0].semantic_bindings.described_by.len(), 1);
    let mut ui = UiTree::new(root);
    argui_layout::LayoutEngine::new()
        .compute(
            &mut ui,
            &mut argui_text::TextEngine::new(),
            argui_core::Size::new(400.0, 240.0),
        )
        .unwrap();
    assert!(
        ui.semantic_diagnostics().is_empty(),
        "{:?}",
        ui.semantic_diagnostics()
    );
    dispatch(
        &host,
        "leaf",
        UiEventKind::KeyInput(KeyInput {
            key: Key::Escape,
            state: KeyState::Pressed,
            modifiers: Modifiers::default(),
            repeat: false,
            text: None,
        }),
    );
    assert!(tooltips(&host.render()).is_empty());
    assert!(host.read(Render::image_assets).is_empty());
    assert!(host.read(Render::vector_assets).is_empty());
    assert!(host.read(Render::inspector).is_none());
}
