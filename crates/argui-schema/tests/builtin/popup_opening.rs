use argui_animation::Time;
use argui_core::{Point, Size, Transform2D};
use argui_layout::LayoutEngine;
use argui_schema::{NativeElementInput, NativeSlotValue, SchemaValue, builtin};
use argui_text::TextEngine;
use argui_ui::{Element, FocusTarget, Interaction, Sides, TreeUpdate, UiTree, length, percent};

fn popup(y: f32) -> Element {
    builtin::registry()
        .unwrap()
        .construct(
            builtin::POPUP_WINDOW,
            &NativeElementInput::new()
                .property(builtin::ID, SchemaValue::String("popup".into()))
                .property(builtin::ANCHOR, SchemaValue::String("trigger".into()))
                .property(builtin::OPENING_MS, SchemaValue::Float(150.0))
                .property(builtin::OPENING_SCALE, SchemaValue::Float(0.98))
                .property(builtin::OPENING_TRANSLATE_Y, SchemaValue::Float(y))
                .property(builtin::WIDTH, SchemaValue::Dimension(length(180.0)))
                .slot(NativeSlotValue::new(
                    builtin::CHILDREN,
                    [Element::container([Element::container([])
                        .keyed("option")
                        .width(percent(1.0))
                        .height(length(90.0))
                        .background(argui_ui::Color::WHITE)
                        .interaction(Interaction::default())])
                    .width(percent(1.0))
                    .padding(Sides {
                        left: length(18.0),
                        right: length(18.0),
                        top: length(18.0),
                        bottom: length(18.0),
                    })],
                )),
        )
        .unwrap()
}

#[test]
fn popup_opening_is_finite_compositor_motion_and_retains_phase_on_updates() {
    let mut tree = UiTree::new(popup(-4.0));
    assert!(tree.wants_animation_frame());
    tree.advance_animations(Time::from_nanos(1));
    let node = tree.node_id_at(0).unwrap();
    assert_eq!(
        tree.resolved_transform(node, tree.root()).translation.y,
        -4.0
    );
    assert_eq!(
        tree.advance_animations(Time::from_nanos(50_000_001)),
        TreeUpdate::Composite
    );
    let moving = tree.resolved_transform(node, tree.root());
    assert!(moving.scale.x > 0.98 && moving.scale.x < 1.0);
    tree.update(popup(-4.0));
    assert_eq!(tree.resolved_transform(node, tree.root()), moving);
    tree.advance_animations(Time::from_nanos(150_000_001));
    assert_eq!(
        tree.resolved_transform(node, tree.root()),
        Transform2D::IDENTITY
    );
    assert!(!tree.wants_animation_frame());
    tree.update(popup(-4.0));
    assert_eq!(
        tree.resolved_transform(node, tree.root()),
        Transform2D::IDENTITY
    );
    assert!(!tree.wants_animation_frame());
}

#[test]
fn reduced_motion_opens_without_transform_or_frame_requests() {
    let mut tree = UiTree::new(popup(-4.0));
    tree.set_reduced_motion(true);
    let node = tree.node_id_at(0).unwrap();
    assert_eq!(
        tree.resolved_transform(node, tree.root()),
        Transform2D::IDENTITY
    );
    tree.update(popup(-4.0));
    assert!(!tree.wants_animation_frame());
}

#[test]
fn closing_and_remounting_popup_starts_a_new_entrance() {
    let mut tree = UiTree::new(Element::container([popup(-4.0)]));
    tree.advance_animations(Time::from_nanos(1));
    tree.advance_animations(Time::from_nanos(150_000_001));
    assert!(!tree.wants_animation_frame());
    tree.update(Element::container([]));
    tree.update(Element::container([popup(-4.0)]));
    assert!(tree.wants_animation_frame());
    let node = tree
        .resolve_node(&FocusTarget::Key("popup".into()))
        .unwrap();
    let element = tree.element_for(node).unwrap();
    assert_eq!(tree.resolved_transform(node, element).scale.x, 0.98);
}

#[test]
fn default_popups_stay_static_and_opening_values_validate_boundaries() {
    let registry = builtin::registry().unwrap();
    let element = registry
        .construct(builtin::POPUP_WINDOW, &NativeElementInput::new())
        .unwrap();
    assert!(element.bindings.is_empty());
    for duration in [0.0, -1.0, 60_001.0, f32::NAN, f32::INFINITY] {
        assert!(
            registry
                .construct(
                    builtin::POPUP_WINDOW,
                    &NativeElementInput::new()
                        .property(builtin::OPENING_MS, SchemaValue::Float(duration))
                )
                .is_err()
        );
    }
    for scale in [0.0, -0.98, f32::NAN, f32::INFINITY] {
        assert!(
            registry
                .construct(
                    builtin::POPUP_WINDOW,
                    &NativeElementInput::new()
                        .property(builtin::OPENING_MS, SchemaValue::Float(150.0))
                        .property(builtin::OPENING_SCALE, SchemaValue::Float(scale))
                )
                .is_err()
        );
    }
    for id in [builtin::OPENING_SCALE, builtin::OPENING_TRANSLATE_Y] {
        assert!(
            registry
                .construct(
                    builtin::POPUP_WINDOW,
                    &NativeElementInput::new().property(id, SchemaValue::Float(0.98))
                )
                .is_err()
        );
    }
    for duration in [1.0, 60_000.0] {
        assert!(
            registry
                .construct(
                    builtin::POPUP_WINDOW,
                    &NativeElementInput::new()
                        .property(builtin::OPENING_MS, SchemaValue::Float(duration))
                )
                .is_ok()
        );
    }
    assert!(
        registry
            .construct(
                builtin::POPUP_WINDOW,
                &NativeElementInput::new()
                    .property(builtin::OPENING_MS, SchemaValue::Float(150.0))
                    .property(builtin::OPENING_TRANSLATE_Y, SchemaValue::Float(f32::NAN))
            )
            .is_err()
    );
}

#[test]
fn popup_fade_finishes_at_its_authored_opacity() {
    let popup = builtin::registry()
        .unwrap()
        .construct(
            builtin::POPUP_WINDOW,
            &NativeElementInput::new()
                .property(builtin::OPENING_MS, SchemaValue::Float(150.0))
                .property(builtin::OPACITY, SchemaValue::Float(0.4)),
        )
        .unwrap();
    let mut tree = UiTree::new(popup);
    let node = tree.node_id_at(0).unwrap();
    let opacity = |tree: &UiTree| {
        tree.resolved_layer(node, tree.root(), tree.root().layer.as_ref().unwrap())
            .opacity
    };
    assert_eq!(opacity(&tree), 0.0);
    tree.advance_animations(Time::from_nanos(1));
    tree.advance_animations(Time::from_nanos(50_000_001));
    assert!(opacity(&tree) > 0.0 && opacity(&tree) < 0.4);
    tree.advance_animations(Time::from_nanos(150_000_001));
    assert_eq!(opacity(&tree), 0.4);
    assert!(!tree.wants_animation_frame());
}

#[test]
fn compact_popup_pivots_at_the_anchor_and_keeps_native_bounds_static() {
    check_geometry(0.0, 100.0, false, false);
    check_geometry(0.0, 100.0, false, true);
}

#[test]
fn classic_popup_moves_from_trigger_on_either_side_and_stays_clickable() {
    check_geometry(-4.0, 0.0, false, false);
    check_geometry(-4.0, 330.0, true, false);
    check_geometry(-4.0, 330.0, true, true);
}

fn check_geometry(y: f32, trigger_y: f32, above: bool, native: bool) {
    let trigger = Element::container([])
        .keyed("trigger")
        .width(length(180.0))
        .height(length(30.0));
    let root = Element::column([
        Element::container([]).height(length(trigger_y)),
        trigger,
        popup(y),
    ]);
    let mut tree = UiTree::new(root);
    tree.advance_animations(Time::from_nanos(1));
    let mut engine = LayoutEngine::new();
    let mut text = TextEngine::new();
    let mut output = engine
        .compute(&mut tree, &mut text, Size::new(400.0, 430.0))
        .unwrap();
    let popup_id = tree
        .resolve_node(&FocusTarget::Key("popup".into()))
        .unwrap();
    let option_id = tree
        .resolve_node(&FocusTarget::Key("option".into()))
        .unwrap();
    let bounds = output
        .nodes
        .iter()
        .find(|node| node.node == popup_id)
        .unwrap()
        .bounds;
    if native {
        assert!(tree.set_native_portal(popup_id, Some(bounds)));
        engine.repaint(&tree, &mut output);
        assert_eq!(output.native_surfaces.len(), 1);
        assert_eq!(output.native_surfaces[0].bounds, bounds);
    }
    let hit = output
        .hit_regions
        .iter()
        .find(|hit| hit.node == option_id)
        .unwrap();
    let pivot = Point::new(90.0, trigger_y + 15.0);
    let animated = hit.transform.transform_point(pivot);
    assert!((animated.x - pivot.x).abs() < 0.001);
    assert!((animated.y - pivot.y - if above { -y } else { y }).abs() < 0.001);
    assert!(
        hit.contains(
            hit.transform
                .transform_point(Point::new(bounds.origin.x + 90.0, bounds.origin.y + 45.0))
        )
    );
    tree.advance_animations(Time::from_nanos(50_000_001));
    let composited = engine.composite(&tree, &mut output);
    if !composited {
        // A fixed native surface can crop the transformed transparent margin.
        // Newly exposed pixels require the runtime's normal repaint path.
        assert!(native);
        engine.repaint(&tree, &mut output);
    }
    let moving_hit = output
        .hit_regions
        .iter()
        .find(|hit| hit.node == option_id)
        .unwrap();
    assert!(moving_hit.transform.matrix[0] > 0.98 && moving_hit.transform.matrix[0] < 1.0);
    assert!(
        moving_hit.contains(
            moving_hit
                .transform
                .transform_point(Point::new(bounds.origin.x + 90.0, bounds.origin.y + 45.0))
        )
    );
    if native {
        assert_eq!(output.native_surfaces[0].bounds, bounds);
    }
    assert_eq!(
        output
            .nodes
            .iter()
            .find(|node| node.node == popup_id)
            .unwrap()
            .bounds,
        bounds
    );
    let composed = output.hit_regions.clone();
    engine.repaint(&tree, &mut output);
    assert_eq!(output.hit_regions, composed);
}
