use argui_core::{Affine2D, Point, Rect, Size};
use argui_paint::ClipChain;
use argui_schema::{NativeElementInput, NativeSlotValue, SchemaValue, builtin};
use argui_ui::{CursorIcon, FocusPolicy, HitRegion, HitShape, Sides, UiTree};

#[test]
fn rectangle_press_translation_returns_to_rest_after_repeated_clicks() {
    let registry = builtin::registry().unwrap();
    let rectangle = registry
        .construct(
            builtin::RECTANGLE,
            &NativeElementInput::new()
                .property(builtin::PRESSED_TRANSLATE_Y, SchemaValue::Float(1.0))
                .property(builtin::TRANSITION_MS, SchemaValue::Float(150.0))
                .property(
                    builtin::TRANSITION_TIMING_FUNCTION,
                    SchemaValue::String("cubic-bezier(0.4, 0, 0.2, 1)".into()),
                ),
        )
        .unwrap();
    let scope = registry
        .construct(
            builtin::FOCUS_SCOPE,
            &NativeElementInput::new().slot(NativeSlotValue::new(builtin::CHILDREN, [rectangle])),
        )
        .unwrap();
    let mut tree = UiTree::new(scope);
    let scope_id = tree.node_id_at(0).unwrap();
    let rectangle_id = tree.node_id_at(1).unwrap();
    let region = HitRegion {
        node: scope_id,
        bounds: Rect::new(Point::default(), Size::new(100.0, 40.0)),
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
        shape: HitShape::Bounds,
        slop: Sides::default(),
        enabled: true,
        focus_policy: FocusPolicy::TabStop,
        cursor: CursorIcon::Pointer,
        gestures: Default::default(),
        window_drag: None,
    };
    let transform = |tree: &UiTree| {
        tree.resolved_transform(rectangle_id, tree.element_for(rectangle_id).unwrap())
            .translation
            .y
    };
    tree.pointer_moved(Point::new(10.0, 10.0), std::slice::from_ref(&region));
    tree.primary_pressed(std::slice::from_ref(&region));
    tree.advance_animations(argui_animation::Time::from_nanos(1));
    tree.advance_animations(argui_animation::Time::from_nanos(75_000_001));
    assert!(transform(&tree) > 0.5 && transform(&tree) < 1.0);
    tree.primary_released();
    tree.primary_pressed(&[region]);
    tree.primary_released();
    tree.advance_animations(argui_animation::Time::from_nanos(75_000_002));
    tree.advance_animations(argui_animation::Time::from_nanos(225_000_002));
    assert!(transform(&tree).abs() < 0.001);
    assert!(!tree.wants_animation_frame());
}

#[test]
fn rectangle_rejects_invalid_press_translation_and_timing() {
    let registry = builtin::registry().unwrap();
    assert!(
        registry
            .construct(
                builtin::RECTANGLE,
                &NativeElementInput::new()
                    .property(builtin::TRANSITION_MS, SchemaValue::Float(150.0))
                    .property(
                        builtin::TRANSITION_TIMING_FUNCTION,
                        SchemaValue::String("linear".into()),
                    ),
            )
            .is_ok()
    );
    assert!(
        registry
            .construct(
                builtin::RECTANGLE,
                &NativeElementInput::new().property(
                    builtin::TRANSITION_TIMING_FUNCTION,
                    SchemaValue::String("linear".into()),
                ),
            )
            .is_err()
    );
    assert!(
        registry
            .construct(
                builtin::RECTANGLE,
                &NativeElementInput::new()
                    .property(builtin::PRESSED_TRANSLATE_Y, SchemaValue::Float(f32::NAN)),
            )
            .is_err()
    );
    for timing in [
        "ease",
        "cubic-bezier(2, 0, 0, 1)",
        "cubic-bezier(0.4, 0, 1)",
    ] {
        assert!(
            registry
                .construct(
                    builtin::RECTANGLE,
                    &NativeElementInput::new()
                        .property(builtin::TRANSITION_MS, SchemaValue::Float(150.0))
                        .property(
                            builtin::TRANSITION_TIMING_FUNCTION,
                            SchemaValue::String(timing.into())
                        ),
                )
                .is_err(),
            "accepted {timing}"
        );
    }
}
