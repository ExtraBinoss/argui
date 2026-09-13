use argui_animation::{Duration, Frame, Time};
use argui_core::{Color, ColorScheme, Point, PointerId};
use argui_inspect::InspectorHandle;
use argui_platform::{WindowConfig, WindowKey, WindowSpec};
use argui_runtime::{
    AppCommand, AppEvent, AppModel, AppUpdate, Context, LayoutSnapshot, Render, SingleWindowModel,
    ThemeRequest, ViewUpdate, WindowEnvironment,
};
use argui_ui::{
    ClickEvent, ClipboardRequest, Element, EventType, FocusRequest, FocusTarget, ScrollRequest,
    ScrollTarget, SelectionCommand, TextSelection, TextSelectionRequest, UiEventKind, UiTree,
};

#[cfg(feature = "tasks")]
#[test]
fn foreign_window_events_do_not_close_hide_or_drain_a_retained_view() {
    struct Panel;
    impl Render for Panel {
        fn render(&mut self, _: &mut Context<Self>) -> Element {
            Element::text("alive")
        }
        fn tasks_ready(&mut self, cx: &mut Context<Self>) {
            cx.request_animation_frame();
        }
    }
    let main = WindowKey::main();
    let foreign = WindowKey::new("foreign");
    let model = argui_runtime::Entity::new(Panel);
    let mut app = SingleWindowModel::from_entity(model.clone()).unwrap();
    let before = app.view(&main, Default::default()).unwrap();
    for event in [
        argui_platform::PlatformEvent::VisibilityChanged(false),
        argui_platform::PlatformEvent::Closed,
    ] {
        assert!(
            app.update(&AppEvent::Window {
                window: foreign.clone(),
                event
            })
            .windows
            .is_empty()
        );
        assert_eq!(app.view(&main, Default::default()).unwrap(), before);
    }
    assert!(app.tasks_ready(&foreign).windows.is_empty());
    assert!(!app.wants_animation_frame(&main));
    app.tasks_ready(&main);
    assert!(app.wants_animation_frame(&main));
    app.tasks_ready(&main);
    assert!(app.wants_animation_frame(&main));
    let revision = model.revision();
    model.update(|_, cx| cx.command(AppCommand::Quit));
    assert_eq!(model.revision(), revision);
    assert_eq!(app.tasks_ready(&main).commands, [AppCommand::Quit]);
}

#[test]
fn clearing_focus_replaces_a_pending_target_and_is_consumed_only_by_its_window() {
    struct Panel;
    impl Render for Panel {
        fn render(&mut self, _: &mut Context<Self>) -> Element {
            Element::text("panel")
        }
        fn layout_changed(&mut self, _: &LayoutSnapshot, cx: &mut Context<Self>) {
            cx.request_focus("previous");
            cx.clear_focus();
        }
    }
    let key = WindowKey::main();
    let mut app = SingleWindowModel::new(Panel);
    app.view(&key, WindowEnvironment::default()).unwrap();
    let update = app.layout_changed(&key, &LayoutSnapshot::default());
    assert_eq!(update.windows.len(), 1);
    assert_eq!(update.windows[0].update, ViewUpdate::Paint);
    assert_eq!(app.take_focus_request(&WindowKey::new("other")), None);
    assert_eq!(app.take_focus_request(&key), Some(FocusRequest::Clear));
    assert_eq!(app.take_focus_request(&key), None);
}

#[test]
fn app_updates_coalesce_each_window_to_its_strongest_invalidation() {
    let main = WindowKey::main();
    let auxiliary = WindowKey::new("auxiliary");
    let update = AppUpdate::none()
        .window(main.clone(), ViewUpdate::Paint)
        .window(auxiliary, ViewUpdate::Paint)
        .window(main.clone(), ViewUpdate::Rebuild)
        .window(main.clone(), ViewUpdate::None);
    assert_eq!(update.windows.len(), 2);
    assert_eq!(update.windows[0].window, main);
    assert_eq!(update.windows[0].update, ViewUpdate::Rebuild);
}

#[test]
fn app_updates_keep_commands_and_explicit_tray_changes() {
    let spec = WindowSpec::new(WindowKey::new("settings"), WindowConfig::default());
    let update = AppUpdate::none()
        .command(AppCommand::OpenWindow(spec.clone()))
        .tray_changed();
    assert_eq!(update.commands, vec![AppCommand::OpenWindow(spec)]);
    assert!(update.tray_changed);
}

#[test]
fn invalidate_mutates_existing_window_without_losing_the_strongest_update() {
    let main = WindowKey::main();
    let mut update = AppUpdate::none();
    update.invalidate(main.clone(), ViewUpdate::Paint);
    update.invalidate(main.clone(), ViewUpdate::None);
    update.invalidate(main.clone(), ViewUpdate::Rebuild);

    assert_eq!(update.windows.len(), 1);
    assert_eq!(update.windows[0].window, main);
    assert_eq!(update.windows[0].update, ViewUpdate::Rebuild);
}

struct MinimalModel;

impl AppModel for MinimalModel {
    fn view(&self, _window: &WindowKey, _environment: WindowEnvironment) -> Option<Element> {
        Some(Element::container([]))
    }

    fn update(&mut self, _event: &AppEvent) -> AppUpdate {
        AppUpdate::none()
    }
}

#[test]
fn app_model_defaults_are_noop_and_return_empty_resources() {
    let mut model = MinimalModel;
    let main = WindowKey::main();
    let frame = Frame {
        now: Time::ZERO,
        elapsed: Duration::from_millis(16),
    };

    assert!(model.view(&main, WindowEnvironment::default()).is_some());
    assert_eq!(model.animation_frame(&main, frame), AppUpdate::none());
    assert!(!model.wants_animation_frame(&main));
    assert_eq!(
        model.layout_changed(&main, &LayoutSnapshot::default()),
        AppUpdate::none()
    );
    assert_eq!(model.tray(), None);
    assert!(model.image_assets().is_empty());
    assert!(model.vector_assets().is_empty());
    assert!(model.inspector(&main).is_none());
    assert_eq!(model.take_clipboard_request(&main), None);
    assert_eq!(model.take_scroll_request(&main), None);
    assert_eq!(model.take_focus_request(&main), None);
    assert_eq!(model.take_text_selection_request(&main), None);
    assert_eq!(model.take_theme_request(&main), None);
}

struct EffectsSurface;

#[test]
fn named_window_adapter_ignores_foreign_frames_layout_and_effect_requests() {
    use std::{cell::Cell, rc::Rc};
    struct Probe(Rc<Cell<usize>>);
    impl Render for Probe {
        fn render(&mut self, _: &mut Context<Self>) -> Element {
            Element::text("auxiliary")
        }
        fn animation_frame(&mut self, _: Frame, cx: &mut Context<Self>) {
            self.0.set(self.0.get() + 1);
            cx.request_paint();
        }
        fn layout_changed(&mut self, _: &LayoutSnapshot, cx: &mut Context<Self>) {
            self.0.set(self.0.get() + 10);
            cx.write_clipboard(ClipboardRequest::Write("mailbox".into()));
        }
    }
    let calls = Rc::new(Cell::new(0));
    let key = WindowKey::new("mailbox");
    let foreign = WindowKey::main();
    let mut app = SingleWindowModel::new(Probe(calls.clone())).window_key(key.clone());
    assert!(app.view(&foreign, Default::default()).is_none());
    assert!(app.event_router(&foreign).is_none());
    assert!(app.view(&key, Default::default()).is_some());
    assert!(app.event_router(&key).is_some());
    let frame = Frame {
        now: Time::ZERO,
        elapsed: Duration::from_millis(16),
    };
    assert!(app.animation_frame(&foreign, frame).windows.is_empty());
    assert!(
        app.layout_changed(&foreign, &LayoutSnapshot::default())
            .windows
            .is_empty()
    );
    assert_eq!(calls.get(), 0);
    app.animation_frame(&key, frame);
    app.layout_changed(&key, &LayoutSnapshot::default());
    assert_eq!(calls.get(), 11);
    assert!(app.take_focus_request(&foreign).is_none());
    assert!(app.take_clipboard_request(&foreign).is_none());
    assert_eq!(
        app.take_clipboard_request(&key),
        Some(ClipboardRequest::Write("mailbox".into()))
    );
    assert!(app.take_ui_commands(&foreign).is_empty());
}

#[test]
fn window_adapters_retain_independent_mounts_of_one_shared_entity() {
    struct Shared;
    impl Render for Shared {
        fn render(&mut self, cx: &mut Context<Self>) -> Element {
            Element::text(format!("{:?}", cx.environment().color_scheme))
        }
    }
    let model = argui_runtime::Entity::new(Shared);
    let first = SingleWindowModel::from_entity(model.clone()).unwrap();
    let second = SingleWindowModel::from_entity(model.clone()).unwrap();
    let window = WindowKey::main();
    let dark = WindowEnvironment {
        color_scheme: ColorScheme::Dark,
        ..Default::default()
    };
    let first_view = first.view(&window, dark.clone()).unwrap();
    let second_view = second.view(&window, WindowEnvironment::default()).unwrap();
    assert_ne!(first_view, second_view);
    assert_eq!(first.view(&window, dark.clone()).unwrap(), first_view);
    assert_eq!(model.resources().resource_count(), 2);
    drop(first);
    assert_eq!(model.resources().resource_count(), 1);
    assert_eq!(
        second.view(&window, WindowEnvironment::default()).unwrap(),
        second_view
    );
    drop(second);
    assert_eq!(model.resources().resource_count(), 0);
    model.resources().close();
    assert!(matches!(
        SingleWindowModel::from_entity(model),
        Err(argui_runtime::ScopeClosed)
    ));
}

impl Render for EffectsSurface {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        Element::container([]).keyed("effects").on(cx.listener(
            EventType::Click,
            |_surface, _event, cx| {
                cx.command(AppCommand::Quit);
                cx.write_clipboard(ClipboardRequest::Write("copied".into()));
                cx.scroll(ScrollRequest::offset("effects", Point::new(2.0, 4.0)));
                cx.request_focus("effects");
                cx.select_text("effects", TextSelection::All);
                assert!(cx.capture_pointer(PointerId::MOUSE));
                assert!(cx.release_pointer(PointerId::MOUSE));
                cx.selection_command(SelectionCommand::Copy);
                cx.edit_text("effects", "atomic edit");
                cx.invoke_action(argui_ui::ActionInvocation::new(argui_ui::ActionId::UNDO));
                cx.set_theme(ThemeRequest {
                    color_scheme: Some(ColorScheme::Dark),
                    primary: Some(Color::srgb(0.8, 0.2, 0.4)),
                });
                cx.request_animation_frame();
            },
        ))
    }

    fn animation_frame(&mut self, _frame: Frame, cx: &mut Context<Self>) {
        cx.notify();
    }

    fn wants_animation_frame(&self) -> bool {
        true
    }

    fn layout_changed(&mut self, _layout: &LayoutSnapshot, cx: &mut Context<Self>) {
        cx.request_paint();
    }

    fn inspector(&self) -> Option<InspectorHandle> {
        Some(InspectorHandle::default())
    }
}

#[test]
fn single_window_adapter_exposes_every_component_effect_without_a_native_window() {
    let main = WindowKey::main();
    let other = WindowKey::new("other");
    let mut model = SingleWindowModel::new(EffectsSurface);
    assert!(model.view(&other, WindowEnvironment::default()).is_none());
    let root = model.view(&main, WindowEnvironment::default()).unwrap();
    assert!(model.event_router(&main).is_some());
    assert!(model.event_router(&other).is_none());
    let mut tree = UiTree::new(root);
    let target = tree.node_ids()[0];
    let deliveries = tree.event_deliveries(target, UiEventKind::Click(ClickEvent::accessibility()));
    let update = deliveries
        .iter()
        .filter(|event| event.should_dispatch())
        .fold(AppUpdate::none(), |_, event| {
            model.update(&AppEvent::Ui {
                window: main.clone(),
                event: event.clone(),
            })
        });
    assert_eq!(update.commands, [AppCommand::Quit]);
    assert_eq!(update.windows[0].update, ViewUpdate::Rebuild);
    assert_eq!(
        model.take_clipboard_request(&main),
        Some(ClipboardRequest::Write("copied".into()))
    );
    assert!(matches!(
        model.take_scroll_request(&main).unwrap().target,
        ScrollTarget::Offset {
            container: FocusTarget::Key(key),
            ..
        } if key == "effects"
    ));
    assert_eq!(
        model.take_focus_request(&main),
        Some(FocusRequest::Focus(FocusTarget::Key("effects".into())))
    );
    assert_eq!(
        model.take_text_selection_request(&main),
        Some(TextSelectionRequest::new("effects", TextSelection::All))
    );
    assert_eq!(
        model.take_theme_request(&main).unwrap().color_scheme,
        Some(ColorScheme::Dark)
    );
    assert!(model.take_clipboard_request(&other).is_none());
    assert!(model.take_scroll_request(&other).is_none());
    assert!(model.take_focus_request(&other).is_none());
    assert!(model.take_text_selection_request(&other).is_none());
    assert!(model.take_ui_commands(&other).is_empty());
    let commands = model.take_ui_commands(&main);
    assert_eq!(commands.len(), 3);
    assert!(matches!(
        commands[0],
        argui_ui::UiCommand::Selection {
            command: SelectionCommand::Copy,
            ..
        }
    ));
    assert!(
        matches!(&commands[1], argui_ui::UiCommand::ReplaceText { value, .. } if value == "atomic edit")
    );
    assert!(
        matches!(commands[2], argui_ui::UiCommand::Action(invocation) if invocation.id == argui_ui::ActionId::UNDO)
    );
    assert!(model.take_ui_commands(&main).is_empty());
    assert!(model.take_theme_request(&other).is_none());
    assert!(model.wants_animation_frame(&main));
    assert!(!model.wants_animation_frame(&other));
    assert!(model.inspector(&main).is_some());
    assert!(model.inspector(&other).is_none());
}

#[test]
fn single_window_adapter_routes_frame_layout_and_ignores_unrelated_events() {
    let main = WindowKey::main();
    let other = WindowKey::new("other");
    let mut model = SingleWindowModel::new(EffectsSurface);
    assert_eq!(
        model.update(&AppEvent::Ui {
            window: other,
            event: argui_ui::UiEvent::new(
                UiTree::new(Element::container([])).node_ids()[0],
                None,
                UiEventKind::Click(ClickEvent::accessibility()),
            ),
        }),
        AppUpdate::none()
    );
    assert_eq!(
        model
            .animation_frame(
                &main,
                Frame {
                    now: Time::ZERO,
                    elapsed: Duration::from_millis(16),
                },
            )
            .windows[0]
            .update,
        ViewUpdate::Rebuild
    );
    assert_eq!(
        model
            .layout_changed(&main, &LayoutSnapshot::default())
            .windows[0]
            .update,
        ViewUpdate::Paint
    );
}
