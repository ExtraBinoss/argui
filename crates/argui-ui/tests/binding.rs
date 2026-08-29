use argui_animation::{Composition, Duration, Motion, MotionBinding, Time, Tween};
use argui_core::{Color, Point, Rect, Transform2D};
use argui_paint::{
    Border, CornerRadii, EffectArgument, EffectId, EffectInstance, EffectValue, Fill, Filter,
    GradientStop, LayerMask, LayerStyle, LinearGradient, RadialGradient, Shadow,
};
use argui_ui::{Element, Length, TreeUpdate, UiTree, property};

#[test]
fn one_typed_bind_entry_resolves_each_property_family() {
    let transform = Motion::new(Transform2D::IDENTITY.translate(4.0, 2.0));
    let width = Motion::new(120.0_f32);
    let opacity = Motion::new(0.4_f32);
    let scroll = Motion::new(Point::new(0.0, 30.0));
    let element = Element::container([])
        .width(Length::Px(10.0))
        .paint_opacity(1.0)
        .bind(property::Transform, transform)
        .bind(property::WidthPx, width)
        .bind(property::Opacity, opacity)
        .bind(property::Scroll, scroll);
    let tree = UiTree::new(element.clone());
    let node = tree.node_ids()[0];

    assert_eq!(tree.resolved_transform(node, &element).translation.x, 4.0);
    assert_eq!(tree.resolved_quad(node, &element).opacity, 0.4);
    assert_eq!(
        tree.resolved_layout_style(&element).width,
        Length::Px(120.0)
    );
    assert_eq!(tree.scroll_offset(node), Point::new(0.0, 30.0));
}

#[test]
fn additive_bindings_compose_in_priority_order() {
    let replace = Motion::new(10.0_f32);
    let add = Motion::new(3.0_f32);
    let element = Element::container([])
        .paint_opacity(1.0)
        .bind(
            property::Opacity,
            MotionBinding::new(add)
                .composition(Composition::Add)
                .priority(10),
        )
        .bind(property::Opacity, replace);
    let tree = UiTree::new(element.clone());
    assert_eq!(
        tree.resolved_quad(tree.node_ids()[0], &element).opacity,
        13.0
    );
}

#[test]
fn active_layout_motion_reports_layout_without_replacing_the_tree() {
    let width = Motion::new(100.0_f32);
    let element = Element::container([]).bind(property::WidthPx, width.clone());
    let mut tree = UiTree::new(element);
    width.animate_to(200.0, Tween::new(Duration::from_millis(100)));

    assert_eq!(
        tree.advance_animations(Time::from_nanos(1)),
        TreeUpdate::None
    );
    assert_eq!(
        tree.advance_animations(Time::from_nanos(50_000_001)),
        TreeUpdate::Layout
    );
    assert_eq!(tree.revision(), 0);
    assert_eq!(tree.update_stats().visited, 0);
}

#[test]
fn f32_effect_binding_updates_custom_uniform_value() {
    const EFFECT: EffectId = EffectId::new("tests.animated");
    let phase = Motion::new(0.75_f32);
    let layer = LayerStyle::new(Rect::default()).filter(Filter::Effect(EffectInstance::new(
        EFFECT,
        [("phase", EffectValue::F32(0.0))],
    )));
    let element = Element::container([])
        .layer(layer.clone())
        .bind(property::effect_f32(EFFECT, "phase"), phase);
    let tree = UiTree::new(element.clone());
    let resolved = tree.resolved_layer(&element, &layer);
    let Filter::Effect(effect) = &resolved.filters[0] else {
        panic!("expected custom effect");
    };
    assert_eq!(
        effect.parameters,
        vec![EffectArgument::new("phase", EffectValue::F32(0.75))]
    );
}

#[test]
fn custom_effect_bindings_preserve_parameter_types() {
    const EFFECT: EffectId = EffectId::new("tests.typed");
    let layer = LayerStyle::new(Rect::default()).filter(Filter::Effect(EffectInstance::new(
        EFFECT,
        [
            ("logical", EffectValue::LogicalPixels(0.0)),
            ("vec2", EffectValue::Vec2([0.0; 2])),
            ("vec3", EffectValue::Vec3([0.0; 3])),
            ("vec4", EffectValue::Vec4([0.0; 4])),
            ("mat3", EffectValue::Mat3([0.0; 9])),
            ("mat4", EffectValue::Mat4([0.0; 16])),
            ("tint", EffectValue::Color(Color::TRANSPARENT)),
        ],
    )));
    let element = Element::container([])
        .layer(layer.clone())
        .bind(
            property::effect_logical_pixels(EFFECT, "logical"),
            Motion::new(2.0),
        )
        .bind(
            property::effect_vec2(EFFECT, "vec2"),
            Motion::new([2.0, 3.0]),
        )
        .bind(
            property::effect_vec3(EFFECT, "vec3"),
            Motion::new([3.0, 4.0, 5.0]),
        )
        .bind(
            property::effect_vec4(EFFECT, "vec4"),
            Motion::new([4.0, 5.0, 6.0, 7.0]),
        )
        .bind(property::effect_mat3(EFFECT, "mat3"), Motion::new([3.0; 9]))
        .bind(
            property::effect_mat4(EFFECT, "mat4"),
            Motion::new([4.0; 16]),
        )
        .bind(
            property::effect_color(EFFECT, "tint"),
            Motion::new(Color::rgba(0.1, 0.2, 0.3, 0.4)),
        );
    let tree = UiTree::new(element.clone());
    let resolved = tree.resolved_layer(&element, &layer);
    let Filter::Effect(effect) = &resolved.filters[0] else {
        panic!("expected custom effect");
    };

    assert_eq!(
        effect
            .parameters
            .iter()
            .map(|value| &value.value)
            .collect::<Vec<_>>(),
        vec![
            &EffectValue::LogicalPixels(2.0),
            &EffectValue::Vec2([2.0, 3.0]),
            &EffectValue::Vec3([3.0, 4.0, 5.0]),
            &EffectValue::Vec4([4.0, 5.0, 6.0, 7.0]),
            &EffectValue::Mat3([3.0; 9]),
            &EffectValue::Mat4([4.0; 16]),
            &EffectValue::Color(Color::rgba(0.1, 0.2, 0.3, 0.4)),
        ]
    );
}

#[test]
fn every_layout_property_resolves_to_its_exact_typed_slot() {
    let element = Element::container([])
        .bind(property::WidthPercent, Motion::new(1.0))
        .bind(property::HeightPx, Motion::new(2.0))
        .bind(property::MinWidthPercent, Motion::new(3.0))
        .bind(property::MinHeightPx, Motion::new(4.0))
        .bind(property::MaxWidthPercent, Motion::new(5.0))
        .bind(property::MaxHeightPx, Motion::new(6.0))
        .bind(property::PaddingLeft, Motion::new(7.0))
        .bind(property::PaddingRight, Motion::new(8.0))
        .bind(property::PaddingTop, Motion::new(9.0))
        .bind(property::PaddingBottom, Motion::new(10.0))
        .bind(property::Gap, Motion::new(11.0))
        .bind(property::Grow, Motion::new(12.0))
        .bind(property::Shrink, Motion::new(13.0))
        .bind(property::InsetLeftPx, Motion::new(14.0))
        .bind(property::InsetRightPx, Motion::new(15.0))
        .bind(property::InsetTopPx, Motion::new(16.0))
        .bind(property::InsetBottomPx, Motion::new(17.0));
    let style = UiTree::new(element.clone()).resolved_layout_style(&element);

    assert_eq!(style.width, Length::Percent(1.0));
    assert_eq!(style.height, Length::Px(2.0));
    assert_eq!(style.min_width, Length::Percent(3.0));
    assert_eq!(style.min_height, Length::Px(4.0));
    assert_eq!(style.max_width, Length::Percent(5.0));
    assert_eq!(style.max_height, Length::Px(6.0));
    assert_eq!(
        [
            style.padding.left,
            style.padding.right,
            style.padding.top,
            style.padding.bottom,
            style.gap,
            style.grow,
            style.shrink,
        ],
        [7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0]
    );
    assert_eq!(style.inset.left, Length::Px(14.0));
    assert_eq!(style.inset.right, Length::Px(15.0));
    assert_eq!(style.inset.top, Length::Px(16.0));
    assert_eq!(style.inset.bottom, Length::Px(17.0));
}

#[test]
fn bindings_ignore_absent_or_incompatible_paint_targets() {
    const EFFECT: EffectId = EffectId::new("tests.exact-target");
    let layer = LayerStyle::new(Rect::default()).filter(Filter::Effect(EffectInstance::new(
        EFFECT,
        [("phase", EffectValue::F32(1.0))],
    )));
    let element = Element::container([])
        .layer(layer.clone())
        .bind(property::BackgroundColor, Motion::new(Color::WHITE))
        .bind(property::BorderColor, Motion::new(Color::WHITE))
        .bind(property::BorderWidths, Motion::new([2.0; 4]))
        .bind(
            property::LinearGradientStart,
            Motion::new(Point::new(1.0, 1.0)),
        )
        .bind(property::gradient_stop_offset(8), Motion::new(0.5))
        .bind(property::LayerMaskRadii, Motion::new([2.0; 4]))
        .bind(property::shadow_blur(8), Motion::new(2.0))
        .bind(
            property::effect_vec2(EFFECT, "phase"),
            Motion::new([2.0; 2]),
        )
        .bind(property::effect_f32(EFFECT, "missing"), Motion::new(2.0));
    let tree = UiTree::new(element.clone());

    assert_eq!(
        tree.resolved_quad(tree.node_ids()[0], &element),
        element.paint.quad
    );
    assert_eq!(tree.resolved_layer(&element, &layer), layer);
}

#[test]
fn shared_motion_is_advanced_once_with_the_strongest_invalidation() {
    let shared = Motion::new(1.0_f32);
    let mut tree = UiTree::new(
        Element::column([
            Element::container([])
                .bind(property::Opacity, shared.clone())
                .bind(property::WidthPx, shared.clone()),
            Element::container([]).bind(property::HeightPx, shared.clone()),
        ])
        .bind(property::Opacity, shared.clone()),
    );
    assert_eq!(tree.animation_count(), 1);
    assert_eq!(tree.layout_animation_indices(), &[1, 2]);

    shared.animate_to(2.0, Tween::new(Duration::from_millis(100)));
    tree.advance_animations(Time::from_nanos(1));
    assert_eq!(
        tree.advance_animations(Time::from_nanos(50_000_001)),
        TreeUpdate::Layout
    );
    assert!((shared.value() - 1.5).abs() < 0.001);
}

#[test]
fn reduced_motion_finishes_only_active_tracks() {
    let active = Motion::new(0.0_f32);
    let idle = Motion::new(4.0_f32);
    let mut tree = UiTree::new(
        Element::container([])
            .bind(property::Opacity, active.clone())
            .bind(property::WidthPx, idle.clone()),
    );
    active.animate_to(1.0, Tween::new(Duration::from_secs(1)));

    assert_eq!(tree.set_reduced_motion(true), TreeUpdate::Paint);
    assert_eq!(active.value(), 1.0);
    assert_eq!(idle.value(), 4.0);
    assert!(!tree.wants_animation_frame());
    assert_eq!(tree.set_reduced_motion(false), TreeUpdate::None);
}

#[test]
fn replacing_bindings_uses_their_exact_invalidation_class() {
    let mut layout =
        UiTree::new(Element::container([]).bind(property::WidthPx, Motion::new(10.0_f32)));
    assert_eq!(
        layout.update(Element::container([]).bind(property::WidthPx, Motion::new(20.0_f32))),
        TreeUpdate::Layout
    );

    let mut scroll =
        UiTree::new(Element::container([]).bind(property::Scroll, Motion::new(Point::default())));
    assert_eq!(
        scroll.update(
            Element::container([]).bind(property::Scroll, Motion::new(Point::new(0.0, 20.0)))
        ),
        TreeUpdate::Scroll
    );
}

#[test]
fn paint_bindings_resolve_borders_radii_and_gradient_components() {
    let linear = LinearGradient::new(
        Point::new(0.0, 0.0),
        Point::new(1.0, 0.0),
        [
            GradientStop::new(0.0, Color::rgb(0.0, 0.0, 0.0)),
            GradientStop::new(1.0, Color::WHITE),
        ],
    )
    .unwrap();
    let element = Element::container([])
        .fill(Fill::Linear(linear))
        .border(Border::all(1.0, Color::WHITE))
        .radius(CornerRadii::all(2.0))
        .bind(
            property::LinearGradientStart,
            Motion::new(Point::new(0.2, 0.3)),
        )
        .bind(
            property::LinearGradientEnd,
            Motion::new(Point::new(0.8, 0.7)),
        )
        .bind(property::gradient_stop_offset(1), Motion::new(0.9_f32))
        .bind(
            property::gradient_stop_color(0),
            Motion::new(Color::rgb(0.2, 0.4, 0.6)),
        )
        .bind(property::BorderWidths, Motion::new([3.0, 4.0, 5.0, 6.0]))
        .bind(
            property::BorderColor,
            Motion::new(Color::rgb(0.8, 0.1, 0.2)),
        )
        .bind(property::CornerRadii, Motion::new([7.0, 8.0, 9.0, 10.0]));
    let tree = UiTree::new(element.clone());
    let resolved = tree.resolved_quad(tree.node_ids()[0], &element);
    let Fill::Linear(gradient) = resolved.background.unwrap() else {
        panic!("expected a linear gradient");
    };

    assert_eq!(gradient.start, Point::new(0.2, 0.3));
    assert_eq!(gradient.end, Point::new(0.8, 0.7));
    assert_eq!(gradient.stops.as_slice()[1].offset, 0.9);
    assert_eq!(
        gradient.stops.as_slice()[0].color,
        Color::rgb(0.2, 0.4, 0.6)
    );
    let border = resolved.border.unwrap();
    assert_eq!(border.widths.as_array(), [3.0, 4.0, 5.0, 6.0]);
    assert_eq!(border.color, Color::rgb(0.8, 0.1, 0.2));
    assert_eq!(resolved.radii.as_array(), [7.0, 8.0, 9.0, 10.0]);
}

#[test]
fn radial_and_layer_bindings_resolve_masks_and_indexed_shadows() {
    let radial = RadialGradient::new(
        Point::new(0.5, 0.5),
        Point::new(0.5, 0.5),
        [
            GradientStop::new(0.0, Color::WHITE),
            GradientStop::new(1.0, Color::rgb(0.0, 0.0, 0.0)),
        ],
    )
    .unwrap();
    let layer = LayerStyle::new(Rect::default())
        .opacity(0.9)
        .mask(LayerMask::Rounded(CornerRadii::all(2.0)))
        .shadow(Shadow::drop([1.0, 2.0], 3.0, Color::rgb(0.0, 0.0, 0.0)));
    let element = Element::container([])
        .fill(Fill::Radial(radial))
        .layer(layer.clone())
        .bind(
            property::RadialGradientCenter,
            Motion::new(Point::new(0.4, 0.6)),
        )
        .bind(
            property::RadialGradientRadius,
            Motion::new(Point::new(0.7, 0.8)),
        )
        .bind(property::LayerOpacity, Motion::new(0.5_f32))
        .bind(property::LayerMaskRadii, Motion::new([4.0, 5.0, 6.0, 7.0]))
        .bind(property::shadow_offset(0), Motion::new([8.0, 9.0]))
        .bind(property::shadow_blur(0), Motion::new(10.0_f32))
        .bind(property::shadow_spread(0), Motion::new(11.0_f32))
        .bind(
            property::shadow_color(0),
            Motion::new(Color::rgb(0.3, 0.4, 0.5)),
        );
    let tree = UiTree::new(element.clone());
    let quad = tree.resolved_quad(tree.node_ids()[0], &element);
    let Fill::Radial(gradient) = quad.background.unwrap() else {
        panic!("expected a radial gradient");
    };
    let resolved = tree.resolved_layer(&element, &layer);

    assert_eq!(gradient.center, Point::new(0.4, 0.6));
    assert_eq!(gradient.radius, Point::new(0.7, 0.8));
    assert_eq!(resolved.opacity, 0.5);
    assert_eq!(
        resolved.mask,
        LayerMask::Rounded(CornerRadii {
            top_left: 4.0,
            top_right: 5.0,
            bottom_right: 6.0,
            bottom_left: 7.0,
        })
    );
    assert_eq!(resolved.shadows[0].offset, [8.0, 9.0]);
    assert_eq!(resolved.shadows[0].blur, 10.0);
    assert_eq!(resolved.shadows[0].spread, 11.0);
    assert_eq!(resolved.shadows[0].color, Color::rgb(0.3, 0.4, 0.5));
}
