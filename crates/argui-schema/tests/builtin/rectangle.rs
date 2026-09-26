use argui_core::{Affine2D, Color, Point, Rect, Size};
use argui_paint::{Border, ClipChain, CornerRadii, Fill, Filter};
use argui_schema::{NativeElementInput, NativeSlotValue, SchemaValue, builtin};
use argui_ui::{
    CursorIcon, Element, ExpandedLengthPercentageAuto, FocusPolicy, HitRegion, HitShape, Overflow,
    Position, PropertyBinding, Sides, UiTree, length,
};

#[test]
fn rectangle_paints_brush_border_and_clips_rounded_children() {
    let registry = builtin::registry().unwrap();
    let background = Fill::Solid(Color::srgba(0.2, 0.4, 0.6, 1.0));
    let rectangle = registry
        .construct(
            builtin::RECTANGLE,
            &NativeElementInput::new()
                .property(builtin::BACKGROUND, SchemaValue::Brush(background.clone()))
                .property(
                    builtin::BORDER,
                    SchemaValue::Border(Border::all(2.0, Color::BLACK)),
                )
                .property(builtin::RADII, SchemaValue::Radii(CornerRadii::all(8.0)))
                .property(builtin::CLIP, SchemaValue::Bool(true))
                .property(
                    builtin::BACKDROP_FILTER,
                    SchemaValue::String("blur(8px)".into()),
                )
                .property(builtin::POSITION, SchemaValue::String("absolute".into()))
                .property(
                    builtin::INSET,
                    SchemaValue::PositionInsets(argui_ui::PositionInsets {
                        left: Some(12.0),
                        top: Some(4.0),
                        ..argui_ui::PositionInsets::default()
                    }),
                )
                .slot(NativeSlotValue::new(
                    builtin::CHILDREN,
                    [Element::text("child")],
                )),
        )
        .unwrap();

    assert_eq!(rectangle.paint.quad.background, Some(background));
    assert_eq!(rectangle.paint.quad.border.unwrap().widths.left, 2.0);
    assert_eq!(rectangle.paint.quad.radii.top_left, 8.0);
    assert_eq!(rectangle.style.overflow.x, Overflow::Hidden);
    assert_eq!(rectangle.style.overflow.y, Overflow::Hidden);
    assert_eq!(rectangle.children.len(), 1);
    assert_eq!(
        rectangle.layer.as_ref().unwrap().backdrop_filters,
        vec![Filter::Blur(8.0)]
    );
    assert_eq!(rectangle.style.position, Position::Absolute);
    assert_eq!(
        rectangle.style.inset.left.expand(),
        ExpandedLengthPercentageAuto::Length(12.0)
    );
    assert_eq!(
        rectangle.style.inset.top.expand(),
        ExpandedLengthPercentageAuto::Length(4.0)
    );
}

#[test]
fn rectangle_focus_border_tracks_a_focused_descendant_natively() {
    let focused = Element::container([])
        .keyed("field")
        .interaction(argui_ui::Interaction::default().focus_policy(FocusPolicy::TabStop));
    let rectangle = builtin::registry()
        .unwrap()
        .construct(
            builtin::RECTANGLE,
            &NativeElementInput::new()
                .property(
                    builtin::BORDER,
                    SchemaValue::Border(Border::all(1.0, Color::BLACK)),
                )
                .property(
                    builtin::FOCUS_BORDER_COLOR,
                    SchemaValue::Color(Color::WHITE),
                )
                .slot(NativeSlotValue::new(builtin::CHILDREN, [focused])),
        )
        .unwrap();
    let mut tree = UiTree::new(rectangle);
    let root = tree.node_id_at(0).unwrap();
    let field = tree.node_id_at(1).unwrap();
    let border_color = |tree: &UiTree| {
        tree.resolved_quad(root, tree.element_for(root).unwrap())
            .border
            .unwrap()
            .color
    };
    assert_eq!(border_color(&tree), Color::BLACK);
    let region = HitRegion {
        node: field,
        bounds: Rect::new(Point::default(), Size::new(100.0, 40.0)),
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
        shape: HitShape::Bounds,
        slop: Sides::default(),
        enabled: true,
        focus_policy: FocusPolicy::TabStop,
        cursor: CursorIcon::Default,
        gestures: Default::default(),
        window_drag: None,
    };
    tree.sync_focus(&[region], Some(argui_ui::FocusRequest::Focus(field.into())));
    assert_eq!(border_color(&tree), Color::WHITE);
}

#[test]
fn rectangle_rejects_invalid_pressed_scales() {
    let registry = builtin::registry().unwrap();
    for scale in [0.0, -0.1, 1.1, f32::NAN] {
        let result = registry.construct(
            builtin::RECTANGLE,
            &NativeElementInput::new().property(builtin::PRESSED_SCALE, SchemaValue::Float(scale)),
        );
        assert!(result.is_err(), "accepted {scale}");
    }
}

#[test]
fn rectangle_rejects_invalid_radii() {
    let registry = builtin::registry().unwrap();
    let error = registry
        .construct(
            builtin::RECTANGLE,
            &NativeElementInput::new()
                .property(builtin::RADII, SchemaValue::Radii(CornerRadii::all(-1.0))),
        )
        .unwrap_err();
    assert!(error.to_string().contains("finite and nonnegative"));
}

#[test]
fn rectangle_rejects_invalid_backdrop_filter() {
    let registry = builtin::registry().unwrap();
    let error = registry
        .construct(
            builtin::RECTANGLE,
            &NativeElementInput::new().property(
                builtin::BACKDROP_FILTER,
                SchemaValue::String("blur(-2px)".into()),
            ),
        )
        .unwrap_err();
    assert!(error.to_string().contains("invalid backdrop_filter"));
}

#[test]
fn rectangle_authored_targets_animate_natively_and_stop_at_rest() {
    let registry = builtin::registry().unwrap();
    let rect = |color: Color| {
        registry
            .construct(
                builtin::RECTANGLE,
                &NativeElementInput::new()
                    .property(builtin::ID, SchemaValue::String("motion".into()))
                    .property(builtin::TRANSITION_MS, SchemaValue::Float(200.0))
                    .property(builtin::BACKGROUND, SchemaValue::Brush(Fill::Solid(color)))
                    .property(builtin::WIDTH, SchemaValue::Dimension(length(80.0))),
            )
            .unwrap()
    };
    let mut tree = UiTree::new(rect(Color::BLACK));
    tree.update(rect(Color::WHITE));
    assert!(tree.wants_animation_frame());
    tree.advance_animations(argui_animation::Time::from_nanos(1));
    tree.advance_animations(argui_animation::Time::from_nanos(250_000_001));
    assert!(!tree.wants_animation_frame());
    assert!(
        registry
            .construct(
                builtin::RECTANGLE,
                &NativeElementInput::new()
                    .property(builtin::TRANSITION_MS, SchemaValue::Float(0.0))
            )
            .is_err()
    );
}

#[test]
fn rectangle_rotation_loop_advances_on_native_frames_without_rematerializing() {
    let registry = builtin::registry().unwrap();
    let rectangle = registry
        .construct(
            builtin::RECTANGLE,
            &NativeElementInput::new()
                .property(builtin::ROTATION_LOOP_MS, SchemaValue::Float(1000.0)),
        )
        .unwrap();
    let PropertyBinding::Transform(binding) = &rectangle.bindings[0] else {
        panic!("rotation loop must bind a native transform");
    };
    let motion = binding.motion.clone();
    let mut tree = UiTree::new(rectangle.clone());
    assert!(tree.wants_animation_frame());
    for frame in 0..=60 {
        tree.advance_animations(argui_animation::Time::from_nanos(frame * 16_666_667));
        assert!(tree.root().ptr_eq(&rectangle));
        if frame == 30 {
            assert!((motion.value().rotation - std::f32::consts::PI).abs() < 0.01);
        }
    }
    assert!(motion.completed_iterations() >= 1);
    assert!(tree.wants_animation_frame());
    tree.set_reduced_motion(true);
    assert!(!tree.wants_animation_frame());
}

#[test]
fn rectangle_alternating_loop_slows_before_reversing_without_slipping() {
    let rectangle = builtin::registry()
        .unwrap()
        .construct(
            builtin::RECTANGLE,
            &NativeElementInput::new()
                .property(builtin::LOOP_MS, SchemaValue::Float(1000.0))
                .property(builtin::LOOP_TRANSLATE_X, SchemaValue::Float(100.0)),
        )
        .unwrap();
    let PropertyBinding::Transform(binding) = &rectangle.bindings[0] else {
        panic!("translation loop must bind a transform");
    };
    let motion = binding.motion.clone();
    let mut tree = UiTree::new(rectangle);
    tree.advance_animations(argui_animation::Time::from_nanos(1));
    tree.advance_animations(argui_animation::Time::from_nanos(950_000_001));
    let before = motion.value().translation.x;
    tree.advance_animations(argui_animation::Time::from_nanos(1_000_000_001));
    let turn = motion.value().translation.x;
    tree.advance_animations(argui_animation::Time::from_nanos(1_050_000_001));
    let after = motion.value().translation.x;
    assert!((turn - 100.0).abs() < 0.01);
    assert!(turn - before < 1.5, "the outbound leg should ease to rest");
    assert!(turn - after < 1.5, "the inbound leg should ease from rest");
    assert!((before - after).abs() < 0.01);
}

#[test]
fn rectangle_rotation_loop_rejects_invalid_duration() {
    let registry = builtin::registry().unwrap();
    for duration in [0.0, -1.0, f32::NAN, f32::INFINITY, 60_001.0] {
        let error = registry
            .construct(
                builtin::RECTANGLE,
                &NativeElementInput::new()
                    .property(builtin::ROTATION_LOOP_MS, SchemaValue::Float(duration)),
            )
            .unwrap_err();
        assert!(error.to_string().contains("rotationLoopMs"));
    }
}

#[test]
fn rectangle_rotation_loop_preserves_an_authored_initial_rotation() {
    let rectangle = builtin::registry()
        .unwrap()
        .construct(
            builtin::RECTANGLE,
            &NativeElementInput::new()
                .property(builtin::ROTATION, SchemaValue::Float(45.0))
                .property(builtin::ROTATION_LOOP_MS, SchemaValue::Float(1000.0)),
        )
        .unwrap();
    let PropertyBinding::Transform(binding) = &rectangle.bindings[0] else {
        panic!("rotation loop must bind a transform");
    };
    let motion = binding.motion.clone();
    let mut tree = UiTree::new(rectangle);
    tree.advance_animations(argui_animation::Time::from_nanos(1));
    tree.advance_animations(argui_animation::Time::from_nanos(500_000_001));
    let expected = 45_f32.to_radians() + std::f32::consts::PI;
    assert!(
        (motion.value().rotation - expected).abs() < 0.01,
        "rotation {:?} should be {expected}",
        motion.value().rotation
    );
}

#[test]
fn rectangle_native_hold_has_a_real_keyframe_plateau_and_validates_target() {
    let registry = builtin::registry().unwrap();
    let rectangle = registry
        .construct(
            builtin::RECTANGLE,
            &NativeElementInput::new()
                .property(builtin::LOOP_MS, SchemaValue::Float(1000.0))
                .property(builtin::LOOP_TRANSLATE_X, SchemaValue::Float(100.0))
                .property(builtin::LOOP_HOLD, SchemaValue::Bool(true)),
        )
        .unwrap();
    let PropertyBinding::Transform(binding) = &rectangle.bindings[0] else {
        panic!("hold loop must bind a transform");
    };
    let motion = binding.motion.clone();
    let mut tree = UiTree::new(rectangle);
    tree.advance_animations(argui_animation::Time::from_nanos(1));
    tree.advance_animations(argui_animation::Time::from_nanos(450_000_001));
    let plateau_start = motion.value();
    tree.advance_animations(argui_animation::Time::from_nanos(550_000_001));
    assert_eq!(motion.value(), plateau_start);
    tree.advance_animations(argui_animation::Time::from_nanos(900_000_001));
    assert!(motion.value().translation.x < plateau_start.translation.x);
    let error = registry
        .construct(
            builtin::RECTANGLE,
            &NativeElementInput::new()
                .property(builtin::LOOP_MS, SchemaValue::Float(1000.0))
                .property(builtin::LOOP_HOLD, SchemaValue::Bool(true)),
        )
        .unwrap_err();
    assert!(error.to_string().contains("loop_hold requires"));
}

#[test]
fn rectangle_opacity_loop_uses_compositor_invalidation() {
    let rectangle = builtin::registry()
        .unwrap()
        .construct(
            builtin::RECTANGLE,
            &NativeElementInput::new()
                .property(builtin::LOOP_MS, SchemaValue::Float(1000.0))
                .property(builtin::LOOP_OPACITY, SchemaValue::Float(0.25)),
        )
        .unwrap();
    assert!(matches!(
        rectangle.bindings[0],
        PropertyBinding::LayerOpacity(_)
    ));
    let mut tree = UiTree::new(rectangle);
    tree.advance_animations(argui_animation::Time::from_nanos(1));
    assert_eq!(
        tree.advance_animations(argui_animation::Time::from_nanos(250_000_001)),
        argui_ui::TreeUpdate::Composite
    );
}

#[test]
fn rectangle_width_and_radius_loops_advance_native_properties() {
    let registry = builtin::registry().unwrap();
    let rectangle = registry
        .construct(
            builtin::RECTANGLE,
            &NativeElementInput::new()
                .property(builtin::WIDTH, SchemaValue::Dimension(length(74.0)))
                .property(builtin::RADII, SchemaValue::Radii(CornerRadii::all(7.0)))
                .property(builtin::LOOP_MS, SchemaValue::Float(1000.0))
                .property(builtin::LOOP_WIDTH, SchemaValue::Float(174.0))
                .property(builtin::LOOP_RADIUS, SchemaValue::Float(34.0)),
        )
        .unwrap();
    assert!(
        rectangle
            .bindings
            .iter()
            .any(|binding| matches!(binding, PropertyBinding::Layout(..)))
    );
    assert!(
        rectangle
            .bindings
            .iter()
            .any(|binding| matches!(binding, PropertyBinding::CornerRadii(..)))
    );
    let mut tree = UiTree::new(rectangle);
    tree.advance_animations(argui_animation::Time::from_nanos(1));
    assert_eq!(
        tree.advance_animations(argui_animation::Time::from_nanos(500_000_001)),
        argui_ui::TreeUpdate::Layout
    );
    assert!(tree.wants_animation_frame());
}

#[test]
fn rectangle_loop_pause_and_resume_keep_the_current_phase() {
    let registry = builtin::registry().unwrap();
    let rectangle = |playing| {
        registry
            .construct(
                builtin::RECTANGLE,
                &NativeElementInput::new()
                    .property(builtin::ID, SchemaValue::String("phase".into()))
                    .property(builtin::LOOP_MS, SchemaValue::Float(1000.0))
                    .property(builtin::LOOP_OPACITY, SchemaValue::Float(0.2))
                    .property(builtin::LOOP_PLAYING, SchemaValue::Bool(playing)),
            )
            .unwrap()
    };
    let initial = rectangle(true);
    let PropertyBinding::LayerOpacity(binding) = &initial.bindings[0] else {
        panic!("expected opacity loop")
    };
    let motion = binding.motion.clone();
    let mut tree = UiTree::new(initial);
    tree.advance_animations(argui_animation::Time::from_nanos(1));
    tree.advance_animations(argui_animation::Time::from_nanos(400_000_001));
    let phase = motion.value();
    tree.update(rectangle(false));
    assert!(!tree.wants_animation_frame());
    assert_eq!(motion.value(), phase);
    tree.advance_animations(argui_animation::Time::from_nanos(1_400_000_001));
    assert_eq!(motion.value(), phase);
    tree.update(rectangle(true));
    assert!(tree.wants_animation_frame());
    tree.advance_animations(argui_animation::Time::from_nanos(1_400_000_001));
    assert_eq!(motion.value(), phase);
    tree.advance_animations(argui_animation::Time::from_nanos(1_500_000_001));
    assert_ne!(motion.value(), phase);
}

#[test]
fn rectangle_loop_retargets_when_timing_changes() {
    let registry = builtin::registry().unwrap();
    let rectangle = |milliseconds| {
        registry
            .construct(
                builtin::RECTANGLE,
                &NativeElementInput::new()
                    .property(builtin::LOOP_MS, SchemaValue::Float(milliseconds))
                    .property(builtin::LOOP_OPACITY, SchemaValue::Float(0.2)),
            )
            .unwrap()
    };
    let initial = rectangle(1000.0);
    let PropertyBinding::LayerOpacity(binding) = &initial.bindings[0] else {
        panic!("expected opacity loop")
    };
    let original = binding.motion.clone();
    let mut tree = UiTree::new(initial);
    tree.update(rectangle(1200.0));
    let PropertyBinding::LayerOpacity(binding) = &tree.root().bindings[0] else {
        panic!("expected opacity loop")
    };
    assert_ne!(binding.motion.identity(), original.identity());
}

#[test]
fn rectangle_layout_loop_rejects_missing_or_invalid_base() {
    let registry = builtin::registry().unwrap();
    for input in [
        NativeElementInput::new()
            .property(builtin::LOOP_MS, SchemaValue::Float(1000.0))
            .property(builtin::LOOP_WIDTH, SchemaValue::Float(120.0)),
        NativeElementInput::new()
            .property(builtin::RADII, SchemaValue::Radii(CornerRadii::all(7.0)))
            .property(builtin::LOOP_MS, SchemaValue::Float(1000.0))
            .property(builtin::LOOP_RADIUS, SchemaValue::Float(-1.0)),
    ] {
        assert!(registry.construct(builtin::RECTANGLE, &input).is_err());
    }
}

#[test]
fn rectangle_spring_retargets_natively_and_rejects_mixed_drivers() {
    let registry = builtin::registry().unwrap();
    let rect = |rotation| {
        registry
            .construct(
                builtin::RECTANGLE,
                &NativeElementInput::new()
                    .property(builtin::ID, SchemaValue::String("spring".into()))
                    .property(builtin::TRANSITION_SPRING, SchemaValue::Bool(true))
                    .property(builtin::ROTATION, SchemaValue::Float(rotation)),
            )
            .unwrap()
    };
    let mut tree = UiTree::new(rect(0.0));
    tree.update(rect(130.0));
    assert!(tree.wants_animation_frame());
    let error = registry
        .construct(
            builtin::RECTANGLE,
            &NativeElementInput::new()
                .property(builtin::TRANSITION_SPRING, SchemaValue::Bool(true))
                .property(builtin::TRANSITION_MS, SchemaValue::Float(200.0)),
        )
        .unwrap_err();
    assert!(error.to_string().contains("cannot combine"));
}
