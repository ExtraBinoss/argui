#![cfg(feature = "implicit-animation")]

use argui_animation::{Duration, Time, Tween, curves};
use argui_core::{Color, Rect, Transform2D};
use argui_paint::{Border, CornerRadii, Fill, LayerStyle};
use argui_ui::{Element, Sides, TreeUpdate, UiTree, length};
use argui_widgets::{AnimatedContainer, AnimatedOpacity};

/// Returns the resolved group opacity for the root test element.
fn group_opacity(tree: &UiTree, element: &Element) -> f32 {
    let node = tree.node_ids()[0];
    let layer = element.layer.as_deref().expect("animated group layer");
    tree.resolved_layer(node, element, layer).opacity
}

#[test]
fn animated_opacity_snaps_on_mount_and_retargets_as_a_composite_change() {
    let visible = AnimatedOpacity::new("fade", 1.0, Element::text("Hello"))
        .duration(Duration::from_millis(200))
        .curve(curves::LINEAR)
        .build();
    let mut tree = UiTree::new(visible.clone());
    assert_eq!(group_opacity(&tree, &visible), 1.0);
    assert!(!tree.wants_animation_frame());

    let hidden = AnimatedOpacity::new("fade", 0.0, Element::text("Hello"))
        .duration(Duration::from_millis(200))
        .curve(curves::LINEAR)
        .build();
    assert_eq!(tree.update(hidden.clone()), TreeUpdate::Paint);
    assert_eq!(group_opacity(&tree, &hidden), 1.0);
    assert!(tree.wants_animation_frame());
    tree.advance_animations(Time::from_nanos(1));
    assert_eq!(
        tree.advance_animations(Time::from_nanos(100_000_001)),
        TreeUpdate::Composite
    );
    assert!((group_opacity(&tree, &hidden) - 0.5).abs() < 0.01);

    let visible_again = AnimatedOpacity::new("fade", 1.0, Element::text("Hello"))
        .duration(Duration::from_millis(200))
        .curve(curves::LINEAR)
        .build();
    tree.update(visible_again.clone());
    assert!((group_opacity(&tree, &visible_again) - 0.5).abs() < 0.01);
}

#[test]
fn animated_opacity_clamps_targets_and_reduced_motion_finishes_immediately() {
    let start = AnimatedOpacity::new("fade", -5.0, Element::text("Hello")).build();
    let mut tree = UiTree::new(start.clone());
    assert_eq!(group_opacity(&tree, &start), 0.0);
    tree.set_reduced_motion(true);

    let target = AnimatedOpacity::new("fade", 5.0, Element::text("Hello")).build();
    tree.update(target.clone());
    assert_eq!(group_opacity(&tree, &target), 1.0);
    assert!(!tree.wants_animation_frame());
}

#[test]
fn animated_container_interpolates_compatible_layout_and_paint_values() {
    let start = AnimatedContainer::new("card", [])
        .width(length(100.0))
        .height(length(40.0))
        .background(Color::BLACK)
        .radius(CornerRadii::all(4.0))
        .duration(Duration::from_millis(200))
        .curve(curves::LINEAR)
        .build();
    let mut tree = UiTree::new(start);
    let target = AnimatedContainer::new("card", [])
        .width(length(200.0))
        .height(length(80.0))
        .background(Color::WHITE)
        .radius(CornerRadii::all(20.0))
        .duration(Duration::from_millis(200))
        .curve(curves::LINEAR)
        .build();
    assert_eq!(tree.update(target.clone()), TreeUpdate::Layout);
    tree.advance_animations(Time::from_nanos(1));
    tree.advance_animations(Time::from_nanos(100_000_001));

    let node = tree.node_ids()[0];
    let layout = tree.resolved_layout_style(node, &target);
    let quad = tree.resolved_quad(node, &target);
    assert!((layout.size.width.value() - 150.0).abs() < 0.01);
    assert!((layout.size.height.value() - 60.0).abs() < 0.01);
    assert_eq!(quad.radii, CornerRadii::all(12.0));
    assert_eq!(
        quad.background,
        Some(argui_paint::Fill::Solid(Color::BLACK.mix(
            Color::WHITE,
            0.5,
            argui_core::ColorInterpolation::Oklab,
        )))
    );
}

#[test]
fn animated_container_configures_existing_elements_and_switches_units_discretely() {
    let row = Element::row([Element::text("A"), Element::text("B")]);
    let start = AnimatedContainer::from_element(row.clone().keyed("row"))
        .width(length(100.0))
        .configure(|element| element.gap(8.0))
        .build();
    let mut tree = UiTree::new(start);
    let target = AnimatedContainer::from_element(row.keyed("row"))
        .width(argui_ui::percent(0.5))
        .configure(|element| element.gap(16.0))
        .build();
    tree.update(target.clone());
    let node = tree.node_ids()[0];
    assert_eq!(
        tree.resolved_layout_style(node, &target).size.width,
        argui_ui::percent(0.5)
    );
    assert!(
        tree.wants_animation_frame(),
        "the compatible gap still animates"
    );
}

#[test]
fn implicit_animation_builders_expose_the_complete_surface_policy() {
    let opacity = AnimatedOpacity::new("fade", 0.4, Element::text("Hello"))
        .delay(Duration::from_millis(20))
        .duration(Duration::from_millis(180))
        .curve(|progress: f32| progress * progress)
        .build();
    assert_eq!(opacity.key.as_deref(), Some("fade"));
    assert_eq!(opacity.layer.as_deref().unwrap().opacity, 0.4);

    let layer = LayerStyle::new(Rect::default()).opacity(0.8);
    let container = AnimatedContainer::new("surface", [Element::text("Body")])
        .min_width(length(40.0))
        .min_height(length(30.0))
        .max_width(length(400.0))
        .max_height(length(300.0))
        .padding(Sides::length(12.0))
        .fill(Fill::Solid(Color::BLACK))
        .border(Border::all(2.0, Color::WHITE))
        .transform(Transform2D::IDENTITY.translate(4.0, 6.0))
        .opacity(0.8)
        .layer(layer)
        .delay(Duration::from_millis(15))
        .tween(Tween::new(Duration::from_millis(240)).easing(curves::EASE_OUT))
        .build();
    assert_eq!(container.key.as_deref(), Some("surface"));
    assert_eq!(container.style.min_size.width, length(40.0));
    assert_eq!(container.style.min_size.height, length(30.0));
    assert_eq!(container.style.max_size.width, length(400.0));
    assert_eq!(container.style.max_size.height, length(300.0));
    assert_eq!(container.style.padding, Sides::length(12.0));
    assert_eq!(container.transform.translation.x, 4.0);
    assert_eq!(container.layer.as_deref().unwrap().opacity, 0.8);
}
