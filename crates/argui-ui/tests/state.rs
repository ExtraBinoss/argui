use argui_animation::{Duration, Motion, Time, Transition, Tween};
use argui_core::{Affine2D, Color, Point, Rect, Size, Transform2D};
use argui_paint::{
    ClipChain, CornerRadii, EffectId, EffectInstance, EffectValue, Fill, Filter, GradientStop,
    LayerMask, LayerStyle, LinearGradient, RadialGradient, Shadow,
};
use argui_ui::{
    CursorIcon, Element, GestureSet, HitRegion, Interaction, PropertyKey, StateName, StateScopeId,
    StateSelector, StyleCondition, StylePatch, StyleTransition, TransitionDirection,
    TransitionRule, TreeUpdate, UiTree, VisualState, length, property,
};

fn region(node: argui_ui::NodeId) -> HitRegion {
    HitRegion {
        node,
        bounds: Rect::new(Point::default(), Size::new(100.0, 40.0)),
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
        shape: argui_ui::HitShape::Bounds,
        slop: argui_ui::HitTestStyle::default().slop,
        enabled: true,
        focusable: true,
        cursor: CursorIcon::Pointer,
        gestures: GestureSet::NONE,
        window_drag: None,
    }
}

fn tween() -> StyleTransition {
    StyleTransition::new(Transition::tween(Tween::new(Duration::from_millis(100))))
}

fn interactive() -> Element {
    Element::container([])
        .keyed("surface")
        .background(black())
        .interaction(Interaction::default().focusable(true))
        .when(
            VisualState::Hovered,
            StylePatch::new().set(property::BackgroundColor, Color::WHITE),
        )
        .transition(tween())
}

#[test]
fn compound_conditions_compose_all_any_and_not() {
    let selected = StateName::new("selected");
    let blocked = StateName::new("blocked");
    let condition = StyleCondition::all([
        StyleCondition::state(selected),
        !StyleCondition::state(blocked),
        StyleCondition::any([
            StyleCondition::state(selected),
            StyleCondition::state(VisualState::Hovered),
        ]),
    ]);
    let element = Element::container([])
        .active_state(selected, true)
        .when(condition, StylePatch::new().set(property::Opacity, 0.35));
    let mut tree = UiTree::new(element.clone());
    let node = tree.node_ids()[0];
    assert!((tree.resolved_quad(node, &element).opacity - 0.35).abs() < 0.001);

    let blocked_element = element.active_state(blocked, true);
    tree.update(blocked_element.clone());
    assert!((tree.resolved_quad(node, &blocked_element).opacity - 1.0).abs() < 0.001);
}

#[test]
fn inherited_state_promotes_the_retained_update_to_layout() {
    let scope = StateScopeId::new("layout-owner");
    let expanded = StateName::new("expanded");
    let child = Element::container([]).when(
        StateSelector::scope(scope, expanded),
        StylePatch::new().set(property::WidthPx, 240.0),
    );
    let root = Element::container([child.clone()])
        .state_scope(scope)
        .active_state(expanded, false);
    let mut tree = UiTree::new(root);

    let update = tree.update(
        Element::container([child])
            .state_scope(scope)
            .active_state(expanded, true),
    );

    assert_eq!(update, TreeUpdate::Layout);
    assert!(tree.layout_dirty());
}

#[test]
fn first_mount_snaps_and_hover_runs_one_retained_tween() {
    let element = interactive();
    let mut tree = UiTree::new(element.clone());
    let node = tree.node_ids()[0];
    assert!(!tree.wants_animation_frame());

    let update = tree.pointer_moved(Point::new(10.0, 10.0), &[region(node)]);
    assert!(update.paint_changed);
    assert!(tree.wants_animation_frame());
    assert_eq!(
        solid(tree.resolved_quad(node, &element).background),
        black()
    );

    assert_eq!(
        tree.advance_animations(Time::from_nanos(1)),
        TreeUpdate::None
    );
    assert_eq!(
        tree.advance_animations(Time::from_nanos(50_000_001)),
        TreeUpdate::Paint
    );
    let color = solid(tree.resolved_quad(node, &element).background);
    assert_eq!(
        color,
        Color::BLACK.mix(Color::WHITE, 0.5, argui_core::ColorInterpolation::Oklab)
    );
}

#[test]
fn stable_authored_changes_retarget_and_explicit_bindings_win() {
    let mut tree = UiTree::new(interactive());
    let node = tree.node_ids()[0];
    let replacement = interactive().background(Color::WHITE);
    assert_eq!(tree.update(replacement.clone()), TreeUpdate::Paint);
    assert!(tree.wants_animation_frame());
    tree.advance_animations(Time::from_nanos(1));
    tree.advance_animations(Time::from_nanos(50_000_001));
    assert_eq!(
        solid(tree.resolved_quad(node, &replacement).background),
        Color::BLACK.mix(Color::WHITE, 0.5, argui_core::ColorInterpolation::Oklab)
    );

    let bound = replacement.bind(
        property::BackgroundColor,
        Motion::new(Color::srgb(0.2, 0.8, 0.3)),
    );
    tree.update(bound.clone());
    assert_eq!(
        solid(tree.resolved_quad(node, &bound).background),
        Color::srgb(0.2, 0.8, 0.3)
    );
}

#[test]
fn composed_and_inherited_states_keep_each_property() {
    let scope = StateScopeId::new("test-control");
    let child = Element::container([])
        .keyed("child")
        .background(black())
        .when(
            StateSelector::scope(scope, VisualState::Hovered),
            StylePatch::new()
                .set(property::BackgroundColor, Color::WHITE)
                .set(property::Opacity, 0.6),
        );
    let root = Element::container([child])
        .keyed("parent")
        .state_scope(scope)
        .interaction(Interaction::default().focusable(true));
    let mut tree = UiTree::new(root);
    let parent = tree.node_ids()[0];
    let child = tree.root().children[0].clone();
    tree.pointer_moved(Point::new(10.0, 10.0), &[region(parent)]);
    let resolved = tree.resolved_quad(tree.node_ids()[1], &child);
    assert_eq!(solid(resolved.background), Color::WHITE);
    assert_eq!(resolved.opacity, 0.6);
}

#[test]
fn descendant_interaction_and_named_state_compose_in_declaration_order() {
    let scope = StateScopeId::new("ordered-control");
    let selected = StateName::new("selected");
    let child = Element::container([])
        .background(black())
        .when(
            StateSelector::scope(scope, selected),
            StylePatch::new().set(property::BackgroundColor, Color::WHITE),
        )
        .when(
            StateSelector::scope(scope, VisualState::Hovered),
            StylePatch::new()
                .set(property::BackgroundColor, Color::srgb(1.0, 0.0, 0.0))
                .set(property::Opacity, 0.7),
        );
    let root = Element::container([child])
        .state_scope(scope)
        .active_state(selected, true);
    let mut tree = UiTree::new(root);
    let child_node = tree.node_ids()[1];
    let child = tree.root().children[0].clone();
    tree.pointer_moved(Point::new(10.0, 10.0), &[region(child_node)]);
    let resolved = tree.resolved_quad(child_node, &child);
    assert_eq!(solid(resolved.background), Color::srgb(1.0, 0.0, 0.0));
    assert_eq!(resolved.opacity, 0.7);
}

#[test]
fn a_nested_scope_with_the_same_identity_shadows_its_ancestor() {
    let scope = StateScopeId::new("nested-control");
    let outer = StateName::new("outer");
    let leaf = Element::container([]).when(
        StateSelector::scope(scope, outer),
        StylePatch::new().set(property::Opacity, 0.2),
    );
    let inner = Element::container([leaf]).state_scope(scope);
    let root = Element::container([inner])
        .state_scope(scope)
        .active_state(outer, true);
    let tree = UiTree::new(root);
    let leaf = tree.root().children[0].children[0].clone();
    assert_eq!(tree.resolved_quad(tree.node_ids()[2], &leaf).opacity, 1.0);
}

#[test]
fn focus_overrides_hover_without_waiting_for_pointer_exit() {
    let focused = Color::srgb(0.0, 0.4, 1.0);
    let element = Element::container([])
        .border(argui_paint::Border::all(1.0, black()))
        .interaction(Interaction::default().focusable(true))
        .when(
            VisualState::Hovered,
            StylePatch::new().set(property::BorderColor, Color::WHITE),
        )
        .when(
            VisualState::Focused,
            StylePatch::new().set(property::BorderColor, focused),
        );
    let mut tree = UiTree::new(element.clone());
    let node = tree.node_ids()[0];
    let regions = [region(node)];

    tree.pointer_moved(Point::new(10.0, 10.0), &regions);
    assert_eq!(
        tree.resolved_quad(node, &element).border.unwrap().color,
        Color::WHITE
    );
    tree.primary_pressed(&regions);
    assert!(tree.visual_states(node).contains(VisualState::Hovered));
    assert!(tree.visual_states(node).contains(VisualState::Focused));
    assert_eq!(
        tree.resolved_quad(node, &element).border.unwrap().color,
        focused
    );
}

#[test]
fn layout_states_invalidate_layout_and_reduced_motion_snaps() {
    let element = Element::container([])
        .width(length(100.0))
        .interaction(Interaction::default())
        .when(
            VisualState::Hovered,
            StylePatch::new().set(property::WidthPx, 200.0),
        )
        .transition(tween());
    let mut tree = UiTree::new(element.clone());
    let node = tree.node_ids()[0];
    tree.set_reduced_motion(true);
    let update = tree.pointer_moved(Point::new(10.0, 10.0), &[region(node)]);
    assert!(update.layout_changed);
    assert_eq!(
        tree.resolved_layout_style(node, &element).size.width,
        length(200.0)
    );
    assert!(!tree.wants_animation_frame());
}

#[test]
fn scroll_state_animates_from_the_retained_offset() {
    let element = Element::container([])
        .interaction(Interaction::default())
        .when(
            VisualState::Hovered,
            StylePatch::new().set(property::Scroll, Point::new(0.0, 100.0)),
        )
        .transition(tween());
    let mut tree = UiTree::new(element);
    let node = tree.node_ids()[0];
    assert!(tree.set_scroll_offset(node, Point::new(0.0, 20.0)));
    tree.pointer_moved(Point::new(10.0, 10.0), &[region(node)]);
    tree.advance_animations(Time::from_nanos(1));
    tree.advance_animations(Time::from_nanos(50_000_001));
    assert!((tree.scroll_offset(node).y - 60.0).abs() < 0.01);
}

#[test]
fn gradient_components_are_retained_and_animated() {
    let gradient = LinearGradient::new(
        Point::new(0.0, 0.0),
        Point::new(1.0, 0.0),
        argui_paint::ColorInterpolation::Oklab,
        [
            GradientStop::new(0.0, black()),
            GradientStop::new(1.0, Color::WHITE),
        ],
    )
    .expect("valid gradient");
    let element = Element::container([])
        .paint_style(argui_paint::PaintStyle::new(argui_paint::QuadStyle {
            background: Some(Fill::Linear(gradient)),
            ..argui_paint::QuadStyle::default()
        }))
        .interaction(Interaction::default())
        .when(
            VisualState::Hovered,
            StylePatch::new()
                .set(property::LinearGradientStart, Point::new(1.0, 1.0))
                .set(property::gradient_stop_offset(0), 0.2)
                .set(property::gradient_stop_color(1), Color::srgb(1.0, 0.0, 0.0)),
        )
        .transition(tween());
    let mut tree = UiTree::new(element.clone());
    let node = tree.node_ids()[0];
    tree.pointer_moved(Point::new(10.0, 10.0), &[region(node)]);
    tree.advance_animations(Time::from_nanos(1));
    tree.advance_animations(Time::from_nanos(50_000_001));

    let Fill::Linear(gradient) = tree
        .resolved_quad(node, &element)
        .background
        .expect("gradient remains present")
    else {
        panic!("expected linear gradient");
    };
    assert!((gradient.start.x - 0.5).abs() < 0.01);
    assert!((gradient.start.y - 0.5).abs() < 0.01);
    assert!((gradient.stops.as_slice()[0].offset - 0.1).abs() < 0.01);
    assert_eq!(
        gradient.stops.as_slice()[1].color,
        Color::WHITE.mix(
            Color::srgb(1.0, 0.0, 0.0),
            0.5,
            argui_core::ColorInterpolation::Oklab,
        )
    );
}

#[test]
fn radial_gradient_state_updates_each_compatible_component() {
    let gradient = RadialGradient::new(
        Point::new(0.5, 0.5),
        Point::new(0.5, 0.5),
        argui_paint::ColorInterpolation::Oklab,
        [
            GradientStop::new(0.0, black()),
            GradientStop::new(1.0, Color::WHITE),
        ],
    )
    .expect("valid gradient");
    let element = Element::container([])
        .paint_style(argui_paint::PaintStyle::new(argui_paint::QuadStyle {
            background: Some(Fill::Radial(gradient)),
            ..argui_paint::QuadStyle::default()
        }))
        .interaction(Interaction::default().enabled(false))
        .when(
            VisualState::Disabled,
            StylePatch::new()
                .set(property::RadialGradientCenter, Point::new(0.25, 0.75))
                .set(property::RadialGradientRadius, Point::new(0.8, 0.4))
                .set(property::gradient_stop_offset(0), 0.1)
                .set(property::gradient_stop_color(1), Color::srgb(0.0, 1.0, 0.0)),
        );
    let tree = UiTree::new(element.clone());
    let Fill::Radial(gradient) = tree
        .resolved_quad(tree.node_ids()[0], &element)
        .background
        .expect("gradient remains present")
    else {
        panic!("expected radial gradient");
    };
    assert_eq!(gradient.center, Point::new(0.25, 0.75));
    assert_eq!(gradient.radius, Point::new(0.8, 0.4));
    assert_eq!(gradient.stops.as_slice()[0].offset, 0.1);
    assert_eq!(
        gradient.stops.as_slice()[1].color,
        Color::srgb(0.0, 1.0, 0.0)
    );
}

#[test]
fn layer_and_custom_effect_properties_animate_from_the_authored_values() {
    let effect_id = EffectId::new("tests.state");
    let effect = EffectInstance::new(
        effect_id,
        [
            ("f32", EffectValue::F32(0.0)),
            ("logical", EffectValue::LogicalPixels(2.0)),
            ("vec2", EffectValue::Vec2([0.0; 2])),
            ("vec3", EffectValue::Vec3([0.0; 3])),
            ("vec4", EffectValue::Vec4([0.0; 4])),
            ("mat3", EffectValue::Mat3([0.0; 9])),
            ("mat4", EffectValue::Mat4([0.0; 16])),
            ("color", EffectValue::Color(black())),
            ("integer", EffectValue::I32(1)),
            ("unsigned", EffectValue::U32(2)),
            ("toggle", EffectValue::Bool(true)),
        ],
    );
    let layer = LayerStyle::new(Rect::new(Point::default(), Size::new(20.0, 20.0)))
        .opacity(1.0)
        .mask(LayerMask::Rounded(CornerRadii::all(2.0)))
        .shadow(Shadow::drop([0.0, 0.0], 2.0, black()))
        .filter(Filter::Effect(effect));
    let state = StylePatch::new()
        .set(property::LayerOpacity, 0.5)
        .set(property::LayerMaskRadii, [10.0; 4])
        .set(property::shadow_offset(0), [4.0, 6.0])
        .set(property::shadow_blur(0), 10.0)
        .set(property::shadow_spread(0), 4.0)
        .set(property::shadow_color(0), Color::WHITE)
        .set(property::effect_f32(effect_id, "f32"), 1.0)
        .set(property::effect_logical_pixels(effect_id, "logical"), 6.0)
        .set(property::effect_vec2(effect_id, "vec2"), [2.0; 2])
        .set(property::effect_vec3(effect_id, "vec3"), [3.0; 3])
        .set(property::effect_vec4(effect_id, "vec4"), [4.0; 4])
        .set(property::effect_mat3(effect_id, "mat3"), [3.0; 9])
        .set(property::effect_mat4(effect_id, "mat4"), [4.0; 16])
        .set(property::effect_color(effect_id, "color"), Color::WHITE);
    let element = Element::container([])
        .interaction(Interaction::default())
        .layer(layer.clone())
        .when(VisualState::Hovered, state)
        .transition(tween());
    let mut tree = UiTree::new(element.clone());
    let node = tree.node_ids()[0];
    tree.pointer_moved(Point::new(10.0, 10.0), &[region(node)]);
    tree.advance_animations(Time::from_nanos(1));
    tree.advance_animations(Time::from_nanos(50_000_001));

    let resolved = tree.resolved_layer(node, &element, &layer);
    assert!((resolved.opacity - 0.75).abs() < 0.01);
    assert_eq!(resolved.mask, LayerMask::Rounded(CornerRadii::all(6.0)));
    assert!((resolved.shadows[0].offset[0] - 2.0).abs() < 0.01);
    assert!((resolved.shadows[0].blur - 6.0).abs() < 0.01);
    assert!((resolved.shadows[0].spread - 2.0).abs() < 0.01);
    let midpoint = Color::BLACK.mix(Color::WHITE, 0.5, argui_core::ColorInterpolation::Oklab);
    assert_eq!(resolved.shadows[0].color, midpoint);
    let Filter::Effect(effect) = &resolved.filters[0] else {
        panic!("expected custom effect");
    };
    assert_eq!(effect.parameters[0].value, EffectValue::F32(0.5));
    assert_eq!(effect.parameters[1].value, EffectValue::LogicalPixels(4.0));
    assert_eq!(effect.parameters[2].value, EffectValue::Vec2([1.0; 2]));
    assert_eq!(effect.parameters[3].value, EffectValue::Vec3([1.5; 3]));
    assert_eq!(effect.parameters[4].value, EffectValue::Vec4([2.0; 4]));
    assert_eq!(effect.parameters[5].value, EffectValue::Mat3([1.5; 9]));
    assert_eq!(effect.parameters[6].value, EffectValue::Mat4([2.0; 16]));
    let EffectValue::Color(color) = effect.parameters[7].value else {
        panic!("expected color parameter");
    };
    assert_eq!(color, midpoint);
}

#[test]
fn transition_rules_are_directional_specific_and_last_rule_wins() {
    let transition = StyleTransition::new(Transition::tween(Tween::new(Duration::from_millis(20))))
        .rule(
            TransitionRule::new(Transition::tween(Tween::new(Duration::from_millis(300))))
                .direction(TransitionDirection::Enter(VisualState::Hovered.into())),
        )
        .rule(
            TransitionRule::new(Transition::tween(Tween::new(Duration::from_millis(200))))
                .property(PropertyKey::Opacity)
                .direction(TransitionDirection::Enter(VisualState::Hovered.into())),
        )
        .rule(
            TransitionRule::new(Transition::tween(Tween::new(Duration::from_millis(100))))
                .property(PropertyKey::Opacity)
                .direction(TransitionDirection::Enter(VisualState::Hovered.into())),
        );
    let element = Element::container([])
        .interaction(Interaction::default())
        .when(
            VisualState::Hovered,
            StylePatch::new().set(property::Opacity, 0.0),
        )
        .transition(transition);
    let mut tree = UiTree::new(element.clone());
    let node = tree.node_ids()[0];
    tree.pointer_moved(Point::new(10.0, 10.0), &[region(node)]);
    tree.advance_animations(Time::from_nanos(1));
    tree.advance_animations(Time::from_nanos(50_000_001));
    assert!((tree.resolved_quad(node, &element).opacity - 0.5).abs() < 0.01);
}

#[test]
fn repeated_properties_replace_and_disabled_state_composes_on_mount() {
    let element = Element::container([])
        .interaction(Interaction::default().enabled(false))
        .when(
            VisualState::Pressed,
            StylePatch::new().set(property::Opacity, 0.1),
        )
        .when(
            VisualState::Disabled,
            StylePatch::new()
                .set(property::Opacity, 0.8)
                .set(property::Opacity, 0.3)
                .set(
                    property::Transform,
                    Transform2D::IDENTITY.translate(8.0, 4.0),
                ),
        );
    let tree = UiTree::new(element.clone());
    let node = tree.node_ids()[0];
    assert_eq!(tree.resolved_quad(node, &element).opacity, 0.3);
    assert_eq!(
        tree.resolved_transform(node, &element).translation,
        Point::new(8.0, 4.0)
    );
}

fn solid(fill: Option<Fill>) -> Color {
    match fill {
        Some(Fill::Solid(color)) => color,
        _ => panic!("expected a solid fill"),
    }
}

fn black() -> Color {
    Color::srgb(0.0, 0.0, 0.0)
}
