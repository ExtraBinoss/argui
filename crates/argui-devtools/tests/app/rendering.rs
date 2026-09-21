use super::*;

struct NoView;

impl AppModel for NoView {
    /// Returns no application content for any window.
    ///
    /// * `window` — queried window, intentionally unsupported.
    /// * `environment` — platform environment, unused without a view.
    fn view(&self, _window: &WindowKey, _environment: WindowEnvironment) -> Option<Element> {
        None
    }

    /// Ignores events because this model has no visible application content.
    ///
    /// * `event` — event that has no model-owned effect.
    fn update(&mut self, _event: &AppEvent) -> AppUpdate {
        AppUpdate::none()
    }
}

/// Missing app views remain absent while the detached tools surface stays available.
#[test]
fn missing_application_views_do_not_create_phantom_host_content() {
    let app = DevtoolsApp::new(NoView).open(true);
    let target = WindowKey::main();
    let other = WindowKey::new("unmanaged");
    let detached = WindowKey::new("__argui-devtools");
    assert!(app.view(&target, WindowEnvironment::default()).is_none());
    assert!(app.view(&other, WindowEnvironment::default()).is_none());
    assert!(app.view(&detached, WindowEnvironment::default()).is_some());
}

fn send(app: &mut DevtoolsApp<Model>, window: &WindowKey, key: &str, kind: UiEventKind) {
    app.update(&AppEvent::Ui {
        window: window.clone(),
        event: UiEvent::new(
            UiTree::new(Element::container([])).node_ids()[0],
            Some(key.into()),
            kind,
        ),
    });
}

#[test]
fn detached_theme_edits_receive_geometry_and_remain_scoped_to_the_target_window() {
    let (model, calls, _) = Model::new(false);
    let target = WindowKey::main();
    let other = WindowKey::new("other");
    let detached = WindowKey::new("__argui-devtools");
    let mut app = DevtoolsApp::new(model).open(true);
    let environment = WindowEnvironment {
        color_scheme: ColorScheme::Dark,
        ..Default::default()
    };
    app.view(&target, environment.clone()).unwrap();
    app.set_dock_mode(DockMode::Detached);
    app.update(&AppEvent::WindowReady {
        window: detached.clone(),
    });
    for key in ["__devtools-theme", "__devtools-theme-color-background"] {
        app.update(&AppEvent::Ui {
            window: detached.clone(),
            event: event(key),
        });
    }
    send(
        &mut app,
        &detached,
        "__devtools-color-theme-background::field::0",
        UiEventKind::TextChanged("#FF0000FF".into()),
    );
    let node = UiTree::new(Element::container([])).node_ids()[0];
    app.layout_changed(
        &detached,
        &LayoutSnapshot {
            viewport: Rect::new(Point::default(), Size::new(900.0, 650.0)),
            nodes: vec![LayoutBounds {
                node,
                key: Some("__devtools-color-theme-background::pad".into()),
                retained_identity: None,
                bounds: Rect::new(Point::new(20.0, 40.0), Size::new(200.0, 160.0)),
            }],
        },
    );
    send(
        &mut app,
        &detached,
        "__devtools-color-theme-background::pad",
        UiEventKind::Gesture(argui_ui::GestureEvent {
            target: node,
            pointer: argui_core::PointerId::MOUSE,
            phase: argui_ui::GesturePhase::Started,
            delivery: argui_ui::GestureDelivery::Immediate,
            kind: argui_ui::GestureKind::Pan {
                position: Point::new(120.0, 120.0),
                delta: Point::default(),
                total: Point::default(),
                velocity: Point::default(),
            },
        }),
    );
    // The tools window has a different system scheme; it must not replace the app's base.
    app.view(&detached, WindowEnvironment::default()).unwrap();
    app.view(&target, environment.clone()).unwrap();
    let effective = calls.borrow().views.last().unwrap().1.clone();
    assert_eq!(effective.color_scheme, ColorScheme::Dark);
    let Some(argui_theme::ThemeValue::Color(color)) =
        effective.theme_overrides.unwrap().get("background")
    else {
        panic!("edited color")
    };
    assert_eq!(color.to_srgba8(), [128, 64, 64, 255]);
    app.view(&other, environment.clone()).unwrap();
    assert_eq!(calls.borrow().views.last().unwrap().1, environment);
    app.update(&AppEvent::Ui {
        window: detached.clone(),
        event: event("__devtools-theme-reset"),
    });
    app.view(&target, environment.clone()).unwrap();
    assert_eq!(calls.borrow().views.last().unwrap().1, environment);
    app.update(&AppEvent::Ui {
        window: detached.clone(),
        event: event("__devtools-elements"),
    });
    let mut engine = argui_layout::LayoutEngine::new();
    let mut text = argui_showcase::text_engine();
    for query in ["", "no matches"] {
        send(
            &mut app,
            &detached,
            "__devtools-search",
            UiEventKind::TextChanged(query.into()),
        );
        let mut tree = UiTree::new(app.view(&detached, Default::default()).unwrap());
        let layout = engine
            .compute(&mut tree, &mut text, Size::new(900.0, 650.0))
            .unwrap();
        assert_eq!(
            layout.nodes[0].bounds.size,
            Size::new(900.0, 650.0),
            "detached panel fills the window independent of filter results"
        );
    }
}
