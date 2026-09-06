use super::{click, filter, find};
use argui::{
    paint::{EffectValue, Filter},
    runtime::Entity,
    ui::{SemanticAction, SemanticValue, UiEventKind, UiTree},
};
use argui_widget_gallery::WidgetGallery;

fn set(app: &Entity<WidgetGallery>, key: &str, value: f64) {
    let mut tree = UiTree::new(app.render());
    let node = tree
        .node_ids()
        .iter()
        .copied()
        .find(|id| tree.key(*id) == Some(key))
        .unwrap();
    for event in tree.event_deliveries(
        node,
        UiEventKind::SemanticAction {
            action: SemanticAction::SetValue,
            value: Some(SemanticValue::Number {
                value,
                minimum: None,
                maximum: None,
                step: None,
            }),
        },
    ) {
        if event.should_dispatch() {
            app.dispatch_event(&event);
        }
    }
}

fn parameter(app: &Entity<WidgetGallery>, name: &str) -> EffectValue {
    let Filter::Effect(effect) = filter(app).unwrap() else {
        panic!()
    };
    effect
        .parameters
        .into_iter()
        .find(|p| p.name == name)
        .unwrap()
        .value
}

#[test]
fn sliders_update_shader_parameters_clamp_and_reset_without_moving_pane() {
    let app = Entity::new(WidgetGallery::default());
    click(&app, "nav::effects");
    let initial = filter(&app);
    for (key, name, min, max, pixels) in [
        ("glass-strength", "refraction", 0.0, 64.0, true),
        ("glass-blur", "blur", 0.0, 16.0, true),
        ("glass-ior", "ior", 1.0, 2.5, false),
        ("glass-edge", "edge-width", 0.0, 64.0, true),
        ("glass-fresnel", "fresnel", 0.0, 1.0, false),
        ("glass-highlight", "highlight", 0.0, 1.0, false),
        ("glass-chroma", "chromatic-aberration", 0.0, 8.0, true),
        ("glass-saturation", "saturation", 0.0, 4.0, false),
    ] {
        for (requested, expected) in [(-100.0, min), (100.0, max)] {
            set(&app, key, requested);
            assert_eq!(
                parameter(&app, name),
                if pixels {
                    EffectValue::LogicalPixels(expected)
                } else {
                    EffectValue::F32(expected)
                },
                "{key}"
            );
        }
    }
    set(&app, "glass-tint-amount", 0.73);
    click(&app, "glass-tint-rose");
    let EffectValue::Vec4(tint) = parameter(&app, "tint") else {
        panic!()
    };
    assert_eq!(&tint[..3], &[1.0, 0.15, 0.3]);
    assert!((tint[3] - 0.73).abs() < 0.001);
    set(&app, "glass-tint-amount", -1.0);
    assert_eq!(
        parameter(&app, "tint"),
        EffectValue::Vec4([1.0, 0.15, 0.3, 0.0])
    );
    set(&app, "glass-tint-amount", 2.0);
    assert_eq!(
        parameter(&app, "tint"),
        EffectValue::Vec4([1.0, 0.15, 0.3, 1.0])
    );
    let before = find(&app.render(), "liquid-glass-pane")
        .unwrap()
        .style
        .inset;
    click(&app, "glass-reset");
    assert_eq!(filter(&app), initial);
    assert_eq!(
        find(&app.render(), "liquid-glass-pane")
            .unwrap()
            .style
            .inset,
        before
    );
}

#[test]
fn noise_controls_are_inactive_until_enabled_and_keep_settings_when_toggled() {
    let app = Entity::new(WidgetGallery::default());
    click(&app, "nav::effects");
    let initial = filter(&app);
    set(&app, "glass-frequency", 0.5);
    set(&app, "glass-turbulence", 0.5);
    assert_eq!(filter(&app), initial);
    click(&app, "glass-noise");
    set(&app, "glass-frequency", 0.5);
    set(&app, "glass-turbulence", 0.5);
    assert_eq!(
        parameter(&app, "wavelength"),
        EffectValue::LogicalPixels(2.0)
    );
    assert_eq!(parameter(&app, "turbulence"), EffectValue::F32(0.5));
    click(&app, "glass-noise");
    assert_eq!(parameter(&app, "turbulence"), EffectValue::F32(0.0));
    click(&app, "glass-noise");
    assert_eq!(parameter(&app, "turbulence"), EffectValue::F32(0.5));
}

#[test]
fn blur_drag_uses_track_geometry_cancels_and_supports_keyboard() {
    use argui::{
        core::{Key, KeyInput, KeyState, Modifiers, Point, PointerId, Rect, Size},
        runtime::{LayoutBounds, LayoutSnapshot, Render},
        ui::{GestureDelivery, GestureEvent, GestureKind, GesturePhase},
    };
    let app = Entity::new(WidgetGallery::default());
    click(&app, "nav::effects");
    let initial = parameter(&app, "blur");
    let mut tree = UiTree::new(app.render());
    let node = tree
        .node_ids()
        .iter()
        .copied()
        .find(|id| tree.key(*id) == Some("glass-blur"))
        .unwrap();
    app.update(|gallery, cx| {
        gallery.layout_changed(
            &LayoutSnapshot {
                viewport: Rect::new(Point::default(), Size::new(800.0, 600.0)),
                nodes: vec![LayoutBounds {
                    node,
                    key: Some("glass-blur::track".into()),
                    bounds: Rect::new(Point::new(100.0, 100.0), Size::new(200.0, 6.0)),
                }],
            },
            cx,
        )
    });
    for (phase, x, expected) in [
        (
            GesturePhase::Started,
            200.0,
            EffectValue::LogicalPixels(8.0),
        ),
        (
            GesturePhase::Changed,
            500.0,
            EffectValue::LogicalPixels(16.0),
        ),
        (GesturePhase::Cancelled, 500.0, initial),
    ] {
        let kind = UiEventKind::Gesture(GestureEvent {
            target: node,
            pointer: PointerId::MOUSE,
            phase,
            delivery: GestureDelivery::Immediate,
            kind: GestureKind::Pan {
                position: Point::new(x, 103.0),
                delta: Point::default(),
                total: Point::default(),
                velocity: Point::default(),
            },
        });
        for event in tree.event_deliveries(node, kind) {
            if event.should_dispatch() {
                app.dispatch_event(&event);
            }
        }
        assert_eq!(parameter(&app, "blur"), expected);
    }
    for state in [KeyState::Pressed, KeyState::Released] {
        for event in tree.event_deliveries(
            node,
            UiEventKind::KeyInput(KeyInput {
                key: Key::End,
                state,
                modifiers: Modifiers::default(),
                repeat: false,
                text: None,
            }),
        ) {
            if event.should_dispatch() {
                app.dispatch_event(&event);
            }
        }
    }
    assert_eq!(parameter(&app, "blur"), EffectValue::LogicalPixels(16.0));
}
