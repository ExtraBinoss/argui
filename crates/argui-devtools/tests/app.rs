#[path = "app/rendering.rs"]
mod rendering;

use std::{cell::RefCell, rc::Rc, sync::Arc};

use argui_animation::{Duration, Frame, Time};
use argui_core::{Color, ColorScheme, Point, Rect, Size};
use argui_devtools::{DevtoolsApp, DockMode};
use argui_inspect::InspectorHandle;
use argui_paint::{ImageAsset, ImageId, VectorAsset, VectorId};
use argui_platform::{PlatformEvent, TrayConfig, WindowKey};
use argui_runtime::{
    AnyEntity, AppCommand, AppEvent, AppModel, AppUpdate, Context, LayoutBounds, LayoutSnapshot,
    Render, ThemeRequest, ViewUpdate, WindowEnvironment,
};
use argui_ui::{
    ClipboardRequest, Element, FocusRequest, ScrollRequest, TextSelection, TextSelectionRequest,
    UiEvent, UiEventKind, UiTree,
};

#[derive(Default)]
struct Calls {
    views: Vec<(WindowKey, WindowEnvironment)>,
    updates: Vec<AppEvent>,
    frames: Vec<(WindowKey, Frame)>,
    layouts: Vec<(WindowKey, LayoutSnapshot)>,
}

struct Route;

impl Render for Route {
    fn render(&mut self, _: &mut Context<Self>) -> Element {
        Element::container([])
    }
}

struct Model {
    calls: Rc<RefCell<Calls>>,
    route: Option<AnyEntity>,
    inspector: InspectorHandle,
}

impl Model {
    fn new(routed: bool) -> (Self, Rc<RefCell<Calls>>, InspectorHandle) {
        let calls = Rc::new(RefCell::new(Calls::default()));
        let inspector = InspectorHandle::default();
        let route = routed.then(|| argui_runtime::Entity::new(Route).erase());
        (
            Self {
                calls: calls.clone(),
                route,
                inspector: inspector.clone(),
            },
            calls,
            inspector,
        )
    }
}

impl AppModel for Model {
    fn view(&self, window: &WindowKey, environment: WindowEnvironment) -> Option<Element> {
        self.calls
            .borrow_mut()
            .views
            .push((window.clone(), environment));
        Some(Element::text(format!("application:{}", window.as_str())).keyed("app-content"))
    }

    fn event_router(&self, _: &WindowKey) -> Option<AnyEntity> {
        self.route.clone()
    }

    fn update(&mut self, event: &AppEvent) -> AppUpdate {
        self.calls.borrow_mut().updates.push(event.clone());
        AppUpdate::none().window(WindowKey::new("model-output"), ViewUpdate::Paint)
    }

    fn animation_frame(&mut self, window: &WindowKey, frame: Frame) -> AppUpdate {
        self.calls.borrow_mut().frames.push((window.clone(), frame));
        AppUpdate::none().window(window.clone(), ViewUpdate::Paint)
    }

    fn wants_animation_frame(&self, _: &WindowKey) -> bool {
        true
    }

    fn layout_changed(&mut self, window: &WindowKey, layout: &LayoutSnapshot) -> AppUpdate {
        self.calls
            .borrow_mut()
            .layouts
            .push((window.clone(), layout.clone()));
        AppUpdate::none().window(window.clone(), ViewUpdate::Paint)
    }

    fn image_assets(&self) -> Vec<ImageAsset> {
        vec![ImageAsset::rgba8(ImageId(7), 1, 1, vec![0; 4]).expect("valid test image")]
    }

    fn vector_assets(&self) -> Vec<VectorAsset> {
        vec![VectorAsset {
            id: VectorId(99),
            size: Size::new(12.0, 8.0),
            svg: Arc::from(b"<svg/>".to_vec()),
            tintable: false,
        }]
    }

    fn tray(&self) -> Option<TrayConfig> {
        Some(TrayConfig {
            title: Some("model tray".into()),
            ..TrayConfig::default()
        })
    }

    fn inspector(&self, _: &WindowKey) -> Option<InspectorHandle> {
        Some(self.inspector.clone())
    }

    fn take_clipboard_request(&mut self, _: &WindowKey) -> Option<ClipboardRequest> {
        Some(ClipboardRequest::Write("model clipboard".into()))
    }

    fn take_scroll_request(&mut self, _: &WindowKey) -> Option<ScrollRequest> {
        Some(ScrollRequest::offset("model-scroll", Point::new(2.0, 3.0)))
    }

    fn take_focus_request(&mut self, _: &WindowKey) -> Option<FocusRequest> {
        Some(FocusRequest::Focus("model-focus".into()))
    }

    fn take_text_selection_request(&mut self, _: &WindowKey) -> Option<TextSelectionRequest> {
        Some(TextSelectionRequest::new("model-text", TextSelection::All))
    }

    fn take_theme_request(&mut self, _: &WindowKey) -> Option<ThemeRequest> {
        Some(ThemeRequest {
            color_scheme: Some(ColorScheme::Dark),
            primary: Some(Color::srgb(0.8, 0.2, 0.4)),
        })
    }
}

fn event(key: &str) -> UiEvent {
    UiEvent::new(
        UiTree::new(Element::container([])).node_id_at(0).unwrap(),
        Some(key.into()),
        UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
    )
}

fn contains_key(element: &Element, key: &str) -> bool {
    element.key.as_deref() == Some(key)
        || element
            .children
            .iter()
            .any(|child| contains_key(child, key))
}

fn contains_text(element: &Element, needle: &str) -> bool {
    matches!(&element.kind, argui_ui::ElementKind::Text { content, .. } if content.as_str().contains(needle))
        || element
            .children
            .iter()
            .any(|child| contains_text(child, needle))
}

#[test]
fn target_open_and_window_selection_are_explicit() {
    let (model, calls, model_inspector) = Model::new(false);
    let target = WindowKey::new("canvas");
    let detached = WindowKey::new("__argui-devtools");
    model_inspector.publish_tree(argui_inspect::TreeSnapshot {
        revision: 19,
        ..argui_inspect::TreeSnapshot::default()
    });
    let app = DevtoolsApp::new(model).target(target.clone()).open(true);

    assert_eq!(app.dock_mode(), DockMode::Bottom);
    assert_eq!(app.tools_window(), &target);
    assert!(app.inspector().recording());
    assert_ne!(app.inspector().tree(), model_inspector.tree());

    let environment = WindowEnvironment {
        color_scheme: ColorScheme::Dark,
        reduced_motion: true,
        ..WindowEnvironment::default()
    };
    let target_view = app.view(&target, environment.clone()).expect("target view");
    assert!(contains_key(&target_view, "__devtools-app-root"));
    assert!(contains_text(&target_view, "application:canvas"));
    let other = WindowKey::new("other");
    assert!(contains_text(
        &app.view(&other, environment.clone()).expect("other view"),
        "application:other"
    ));
    assert!(contains_key(
        &app.view(&detached, environment).expect("detached view"),
        "__devtools-dock"
    ));
    assert_eq!(calls.borrow().views.len(), 2);
    assert!(app.event_router(&detached).is_none());
    assert!(app.event_router(&other).is_none());
    assert!(app.captures_ui_events());
}

#[test]
#[should_panic]
fn target_rejects_the_reserved_detached_window_key() {
    let (model, _, _) = Model::new(false);
    let _ = DevtoolsApp::new(model).target(WindowKey::new("__argui-devtools"));
}

#[test]
fn detached_window_lifecycle_opens_focuses_and_closes() {
    let (model, calls, _) = Model::new(false);
    let target = WindowKey::main();
    let detached = WindowKey::new("__argui-devtools");
    let mut app = DevtoolsApp::new(model).open(true);

    let open = app.set_dock_mode(DockMode::Detached);
    assert!(open.windows.is_empty());
    assert_eq!(open.commands.len(), 1);
    let spec = match &open.commands[0] {
        AppCommand::OpenWindow(spec) => spec,
        other => panic!("expected a detached window command, got {other:?}"),
    };
    assert_eq!(spec.key, detached);
    assert_eq!(spec.window.title, "Argui DevTools");
    assert_eq!(spec.window.width, 900.0);
    assert_eq!(spec.window.height, 650.0);
    assert_eq!(
        spec.window.close_behavior,
        argui_platform::CloseBehavior::NotifyApp
    );
    assert_eq!(app.dock_mode(), DockMode::Bottom);
    assert_eq!(app.tools_window(), &target);

    let repeated = app.set_dock_mode(DockMode::Detached);
    assert_eq!(repeated.windows, Vec::new());
    assert_eq!(
        repeated.commands,
        vec![AppCommand::FocusWindow(detached.clone())]
    );

    let ready = app.update(&AppEvent::WindowReady {
        window: detached.clone(),
    });
    assert_eq!(app.dock_mode(), DockMode::Detached);
    assert_eq!(app.tools_window(), &detached);
    assert_eq!(
        ready.commands,
        vec![AppCommand::FocusWindow(detached.clone())]
    );
    assert_eq!(
        ready
            .windows
            .iter()
            .map(|window| (&window.window, window.update))
            .collect::<Vec<_>>(),
        vec![
            (&target, ViewUpdate::Rebuild),
            (&detached, ViewUpdate::Rebuild)
        ]
    );
    assert!(calls.borrow().updates.is_empty());

    let back = app.set_dock_mode(DockMode::Right);
    assert_eq!(app.dock_mode(), DockMode::Right);
    assert_eq!(back.commands, vec![AppCommand::CloseWindow(detached)]);
    assert_eq!(back.windows[0].window, target);
    assert_eq!(back.windows[0].update, ViewUpdate::Rebuild);
}

#[test]
fn failed_detach_restores_the_previous_dock_and_reports_the_error() {
    for failure in [
        AppEvent::WindowFailed {
            window: WindowKey::new("__argui-devtools"),
            error: "renderer unavailable".into(),
        },
        AppEvent::Window {
            window: WindowKey::new("__argui-devtools"),
            event: PlatformEvent::WindowCreationFailed("creation denied".into()),
        },
    ] {
        let (model, _, _) = Model::new(false);
        let mut app = DevtoolsApp::new(model).open(true);
        assert!(app.set_dock_mode(DockMode::Detached).commands.len() == 1);
        let update = app.update(&failure);
        assert_eq!(app.dock_mode(), DockMode::Bottom);
        assert_eq!(app.tools_window(), &WindowKey::main());
        assert_eq!(
            update.commands,
            vec![AppCommand::CloseWindow(WindowKey::new("__argui-devtools",))]
        );
        let view = app
            .view(&WindowKey::main(), WindowEnvironment::default())
            .unwrap();
        assert!(contains_text(
            &view,
            "Could not open developer tools window"
        ));
    }
}

#[test]
fn application_and_tool_events_use_the_correct_router() {
    let (model, calls, _) = Model::new(false);
    let mut app = DevtoolsApp::new(model).open(true);
    let before = calls.borrow().updates.len();
    let update = app.update(&AppEvent::Ui {
        window: WindowKey::main(),
        event: event("application-key"),
    });
    assert_eq!(calls.borrow().updates.len(), before + 1);
    assert_eq!(update.windows[0].window, WindowKey::new("model-output"));

    let before = calls.borrow().updates.len();
    let tool_update = app.update(&AppEvent::Ui {
        window: WindowKey::main(),
        event: event("__devtools-elements"),
    });
    assert_eq!(calls.borrow().updates.len(), before);
    assert_eq!(tool_update.windows[0].window, WindowKey::main());
    assert_eq!(tool_update.windows[0].update, ViewUpdate::Rebuild);

    let (routed_model, routed_calls, _) = Model::new(true);
    let mut routed = DevtoolsApp::new(routed_model).open(true);
    assert!(routed.event_router(&WindowKey::main()).is_some());
    let routed_update = routed.update(&AppEvent::Ui {
        window: WindowKey::main(),
        event: event("application-key"),
    });
    assert!(routed_update.windows.is_empty());
    assert!(routed_calls.borrow().updates.is_empty());

    let other = WindowKey::new("other");
    let _ = routed.update(&AppEvent::Ui {
        window: other,
        event: event("application-key"),
    });
    assert_eq!(routed_calls.borrow().updates.len(), 1);
}

#[test]
fn close_requests_stop_recording_and_merge_application_updates() {
    let (model, calls, _) = Model::new(false);
    let mut app = DevtoolsApp::new(model).open(true);
    app.inspector()
        .set_hovered(Some(argui_inspect::InspectNodeId(8)));
    let update = app.update(&AppEvent::Window {
        window: WindowKey::main(),
        event: PlatformEvent::CloseRequested,
    });
    assert_eq!(calls.borrow().updates.len(), 1);
    assert!(!app.inspector().recording());
    assert_eq!(app.inspector().highlighted(), None);
    assert!(update.windows.iter().any(|window| {
        window.window == WindowKey::new("model-output") && window.update == ViewUpdate::Paint
    }));
    assert!(update.windows.iter().any(|window| {
        window.window == WindowKey::main() && window.update == ViewUpdate::Rebuild
    }));
    assert!(update.commands.is_empty());

    let (model, _, _) = Model::new(false);
    let mut detached = DevtoolsApp::new(model).open(true);
    detached.set_dock_mode(DockMode::Detached);
    detached.update(&AppEvent::WindowReady {
        window: WindowKey::new("__argui-devtools"),
    });
    let update = detached.update(&AppEvent::Window {
        window: WindowKey::new("__argui-devtools"),
        event: PlatformEvent::CloseRequested,
    });
    assert_eq!(detached.dock_mode(), DockMode::Bottom);
    assert_eq!(
        update.commands,
        vec![AppCommand::CloseWindow(WindowKey::new("__argui-devtools",))]
    );
    assert!(!detached.inspector().recording());
}

#[test]
fn frame_layout_resource_and_pending_request_delegation_is_preserved() {
    let (model, calls, model_inspector) = Model::new(false);
    model_inspector.publish_tree(argui_inspect::TreeSnapshot {
        revision: 19,
        ..argui_inspect::TreeSnapshot::default()
    });
    let mut app = DevtoolsApp::new(model);
    let frame = Frame {
        now: Time::from_nanos(10),
        elapsed: Duration::from_millis(16),
    };
    assert!(app.wants_animation_frame(&WindowKey::main()));
    let frame_update = app.animation_frame(&WindowKey::main(), frame);
    assert_eq!(calls.borrow().frames.len(), 1);
    assert_eq!(frame_update.windows[0].window, WindowKey::main());
    let detached_update = app.animation_frame(&WindowKey::new("__argui-devtools"), frame);
    assert!(detached_update.windows.is_empty());
    assert_eq!(calls.borrow().frames.len(), 1);

    let root = UiTree::new(Element::container([]));
    let app_bounds = Rect::new(Point::new(4.0, 5.0), Size::new(640.0, 320.0));
    let layout = LayoutSnapshot {
        viewport: Rect::new(Point::default(), Size::new(800.0, 700.0)),
        nodes: vec![LayoutBounds {
            node: root.node_id_at(0).unwrap(),
            key: Some("__devtools-app-root".into()),
            retained_identity: None,
            bounds: app_bounds,
        }],
    };
    let layout_update = app.layout_changed(&WindowKey::main(), &layout);
    assert_eq!(calls.borrow().layouts[0].1.viewport, app_bounds);
    assert_eq!(layout_update.windows[0].window, WindowKey::main());
    let other = WindowKey::new("other");
    app.layout_changed(&other, &layout);
    assert_eq!(calls.borrow().layouts[1].1, layout);

    assert_eq!(app.image_assets().len(), 1);
    assert_eq!(app.image_assets()[0].id, ImageId(7));
    assert_eq!(app.vector_assets().len(), 9);
    assert_eq!(app.tray().unwrap().title.as_deref(), Some("model tray"));
    assert_eq!(
        AppModel::inspector(&app, &WindowKey::main())
            .unwrap()
            .tree()
            .revision,
        0
    );
    assert_eq!(
        AppModel::inspector(&app, &other).unwrap().tree().revision,
        19
    );
    assert!(AppModel::inspector(&app, &WindowKey::new("__argui-devtools")).is_none());

    assert_eq!(
        app.take_clipboard_request(&other),
        Some(ClipboardRequest::Write("model clipboard".into()))
    );
    assert!(matches!(
        app.take_scroll_request(&other).unwrap().target,
        argui_ui::ScrollTarget::Offset { .. }
    ));
    assert_eq!(
        app.take_focus_request(&other),
        Some(FocusRequest::Focus("model-focus".into()))
    );
    assert_eq!(
        app.take_text_selection_request(&other),
        Some(TextSelectionRequest::new("model-text", TextSelection::All))
    );
    assert_eq!(
        app.take_theme_request(&other).unwrap().color_scheme,
        Some(ColorScheme::Dark)
    );
}

#[test]
fn repeated_dock_requests_are_idempotent_and_resize_layout_is_reflected() {
    let (model, _, _) = Model::new(false);
    let mut app = DevtoolsApp::new(model).open(true);
    assert_eq!(
        app.set_dock_mode(DockMode::Bottom),
        AppUpdate::none().window(WindowKey::main(), ViewUpdate::Rebuild,)
    );
    assert_eq!(
        app.set_dock_mode(DockMode::Right).windows[0].update,
        ViewUpdate::Rebuild
    );
    let right = app
        .view(&WindowKey::main(), WindowEnvironment::default())
        .unwrap();
    assert!(contains_key(&right, "__devtools-surface"));

    let resize = LayoutSnapshot {
        viewport: Rect::new(Point::default(), Size::new(380.0, 180.0)),
        nodes: vec![LayoutBounds {
            node: UiTree::new(Element::container([])).node_id_at(0).unwrap(),
            key: Some("__devtools-frames".into()),
            retained_identity: None,
            bounds: Rect::new(Point::default(), Size::new(380.0, 110.0)),
        }],
    };
    let update = app.layout_changed(&WindowKey::main(), &resize);
    assert_eq!(update.windows[0].update, ViewUpdate::Rebuild);
    assert!(app.wants_animation_frame(&WindowKey::main()));
}

#[test]
fn stale_window_events_are_forwarded_and_pending_detach_can_close() {
    let (model, calls, _) = Model::new(false);
    let stale = WindowKey::new("stale-window");
    let detached = WindowKey::new("__argui-devtools");
    let mut app = DevtoolsApp::new(model);
    app.set_dock_mode(DockMode::Detached);
    for event in [
        AppEvent::WindowReady {
            window: stale.clone(),
        },
        AppEvent::WindowFailed {
            window: stale.clone(),
            error: "stale failure".into(),
        },
        AppEvent::Window {
            window: stale.clone(),
            event: PlatformEvent::WindowCreationFailed("stale creation".into()),
        },
        AppEvent::Window {
            window: stale,
            event: PlatformEvent::CloseRequested,
        },
    ] {
        app.update(&event);
    }
    assert_eq!(calls.borrow().updates.len(), 4);

    app.update(&AppEvent::WindowReady {
        window: detached.clone(),
    });
    assert_eq!(
        app.set_dock_mode(DockMode::Detached).commands,
        vec![AppCommand::FocusWindow(detached.clone())]
    );
    assert!(!app.wants_animation_frame(&detached));
    let detached_layout = LayoutSnapshot {
        viewport: Rect::new(Point::default(), Size::new(900.0, 650.0)),
        nodes: vec![LayoutBounds {
            node: UiTree::new(Element::container([])).node_id_at(0).unwrap(),
            key: Some("__devtools-tree".into()),
            retained_identity: None,
            bounds: Rect::new(Point::default(), Size::new(900.0, 400.0)),
        }],
    };
    assert_eq!(
        app.layout_changed(&detached, &detached_layout).windows[0].update,
        ViewUpdate::Rebuild
    );
    assert!(
        app.layout_changed(&detached, &detached_layout)
            .windows
            .is_empty()
    );
    app.update(&AppEvent::WindowReady { window: detached });
    assert_eq!(calls.borrow().updates.len(), 5);
    let detached_event = app.update(&AppEvent::Ui {
        window: WindowKey::new("__argui-devtools"),
        event: event("__devtools-elements"),
    });
    assert_eq!(app.dock_mode(), DockMode::Bottom);
    assert_eq!(
        detached_event.commands,
        vec![AppCommand::CloseWindow(WindowKey::new("__argui-devtools"))]
    );

    let (model, _, _) = Model::new(false);
    let mut hidden = DevtoolsApp::new(model);
    hidden.set_dock_mode(DockMode::Detached);
    let update = hidden.update(&AppEvent::Ui {
        window: WindowKey::main(),
        event: event("__devtools-elements"),
    });
    assert_eq!(hidden.dock_mode(), DockMode::Bottom);
    assert!(!hidden.inspector().recording());
    assert_eq!(
        update.commands,
        vec![AppCommand::CloseWindow(WindowKey::new("__argui-devtools"))]
    );
}

#[test]
fn native_dock_selection_requests_a_detached_window() {
    let (model, _, _) = Model::new(false);
    let mut app = DevtoolsApp::new(model).open(true);
    app.update(&AppEvent::Ui {
        window: WindowKey::main(),
        event: event("__devtools-dock"),
    });
    let update = app.update(&AppEvent::Ui {
        window: WindowKey::main(),
        event: event("__devtools-dock::option::2"),
    });
    assert_eq!(app.dock_mode(), DockMode::Bottom);
    assert!(matches!(
        update.commands.as_slice(),
        [AppCommand::OpenWindow(spec)]
            if spec.key == WindowKey::new("__argui-devtools")
    ));
}
