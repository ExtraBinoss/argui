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
