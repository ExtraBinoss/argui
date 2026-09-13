use argui_animation::{Duration, Time, Transition, Tween};
use argui_core::{Affine2D, Point, Rect, Size};
use argui_paint::{ClipChain, Fill, GradientStop, LayerStyle, LinearGradient, Shadow};
use argui_ui::{
    Color, CursorIcon, Element, ElementKind, GestureSet, HitRegion, HitShape, Interaction,
    StylePatch, StyleTransition, TreeUpdate, UiTree, VisualState, length, property,
};

fn transition() -> StyleTransition {
    StyleTransition::new(Transition::tween(Tween::new(Duration::from_millis(100))))
}

fn region(node: argui_ui::NodeId) -> HitRegion {
    HitRegion {
        node,
        bounds: Rect::new(Point::default(), Size::new(100.0, 40.0)),
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
        shape: HitShape::Bounds,
        slop: argui_ui::Sides::default(),
        enabled: true,
        focus_policy: argui_ui::FocusPolicy::None,
        cursor: CursorIcon::Auto,
        gestures: GestureSet::EMPTY,
        window_drag: None,
    }
}

#[test]
fn layout_transitions_report_indices_and_disappear_after_removal() {
    let element = Element::container([])
        .width(length(100.0))
        .interaction(Interaction::default())
        .when(
            VisualState::Hovered,
            StylePatch::new().set(property::WidthPx, 200.0),
        )
        .transition(transition());
    let mut tree = UiTree::new(element);
    let node = tree.node_id_at(0).expect("transition node");

    assert_eq!(tree.layout_animation_indices(), vec![0]);
    assert_eq!(tree.visual_revision(node), 0);
    let update = tree.pointer_moved(Point::new(10.0, 10.0), &[region(node)]);
    assert!(update.layout_changed);
    assert_eq!(tree.layout_animation_indices(), vec![0]);
    assert!(tree.visual_revision(node) > 0);
    assert!(tree.wants_animation_frame());

    assert_eq!(
        tree.advance_animations(Time::from_nanos(1)),
        TreeUpdate::None
    );
    assert_eq!(
        tree.advance_animations(Time::from_nanos(50_000_001)),
        TreeUpdate::Layout
    );
    assert!(tree.visual_revision(node) > 1);

    assert_eq!(tree.update(Element::container([])), TreeUpdate::Layout);
    assert!(tree.layout_animation_indices().is_empty());
    assert!(!tree.wants_animation_frame());
    assert_eq!(tree.visual_revision(node), 0);
}

#[test]
fn base_layout_changes_keep_scalar_transitions_and_discrete_fields() {
    let element = Element::container([])
        .width(length(100.0))
        .transition(transition());
    let mut tree = UiTree::new(element.clone());
    let node = tree.node_ids()[0];
    let mut changed = element.width(length(200.0));
    changed.style.flex_direction = argui_ui::FlexDirection::Column;
    tree.update(changed.clone());
    tree.advance_animations(Time::from_nanos(1));
    tree.advance_animations(Time::from_nanos(50_000_001));
    let resolved = tree.resolved_layout_style(node, tree.root());
    assert_eq!(resolved.size.width, length(150.0));
    assert_eq!(resolved.flex_direction, argui_ui::FlexDirection::Column);
    assert_eq!(changed.style.size.width, length(200.0));
    tree.advance_animations(Time::from_nanos(100_000_001));
    assert_eq!(tree.resolved_layout_style(node, tree.root()), changed.style);
    assert!(!tree.wants_animation_frame());
}

#[test]
fn whole_layout_override_returns_to_the_latest_base_after_hover() {
    let mut patch = argui_ui::LayoutStyle::default();
    patch.size.width = length(240.0);
    patch.flex_direction = argui_ui::FlexDirection::Column;
    let element = Element::container([])
        .width(length(100.0))
        .interaction(Interaction::default())
        .when(
            VisualState::Hovered,
            StylePatch::new().layout(patch.clone()),
        )
        .transition(transition());
    let mut tree = UiTree::new(element.clone());
    let node = tree.node_ids()[0];
    tree.pointer_moved(Point::new(10.0, 10.0), &[region(node)]);
    assert_eq!(tree.resolved_layout_style(node, tree.root()), patch);
    let changed = element.width(length(180.0));
    tree.update(changed.clone());
    assert_eq!(tree.resolved_layout_style(node, tree.root()), patch);
    tree.pointer_left();
    assert_eq!(tree.resolved_layout_style(node, tree.root()), changed.style);
    assert!(!tree.wants_animation_frame());
}

#[test]
fn shared_description_keeps_running_transition_and_responds_to_hover_exit() {
    let child = Element::container([])
        .keyed("animated")
        .interaction(Interaction::default())
        .when(
            VisualState::Hovered,
            StylePatch::new().set(property::Opacity, 0.0),
        )
        .transition(transition());
    let mut tree = UiTree::new(Element::column([child.clone(), Element::text("before")]));
    let node = tree.node_ids()[1];
    tree.pointer_moved(Point::new(10.0, 10.0), &[region(node)]);
    tree.advance_animations(Time::from_nanos(1));
    tree.advance_animations(Time::from_nanos(50_000_001));
    let opacity = |tree: &UiTree| tree.resolved_quad(node, &tree.root().children[0]).opacity;
    assert!((opacity(&tree) - 0.5).abs() < 0.001);
    tree.update(Element::column([child.clone(), Element::text("after")]));
    assert!(tree.root().children[0].ptr_eq(&child));
    assert!((opacity(&tree) - 0.5).abs() < 0.001);
    tree.advance_animations(Time::from_nanos(75_000_001));
    assert!((opacity(&tree) - 0.25).abs() < 0.001);
    tree.pointer_left();
    tree.set_reduced_motion(true);
    assert_eq!(opacity(&tree), 1.0);
    assert!(!tree.wants_animation_frame());
}

#[test]
fn authored_changes_retarget_then_reduced_motion_finishes_the_new_target() {
    let element = |opacity| {
        Element::container([])
            .interaction(Interaction::default())
            .when(
                VisualState::Hovered,
                StylePatch::new().set(property::Opacity, opacity),
            )
            .transition(transition())
    };
    let mut tree = UiTree::new(element(0.0));
    let node = tree.node_id_at(0).expect("transition node");
    let hit = region(node);

    tree.pointer_moved(Point::new(10.0, 10.0), std::slice::from_ref(&hit));
    tree.advance_animations(Time::from_nanos(1));
    tree.advance_animations(Time::from_nanos(50_000_001));
    let halfway = tree
        .resolved_quad(node, tree.element_at(0).unwrap())
        .opacity;
    assert!((halfway - 0.5).abs() < 0.01);
    let revision = tree.visual_revision(node);

    assert_eq!(tree.update(element(1.0)), TreeUpdate::Paint);
    assert!(tree.visual_revision(node) > revision);
    assert!(tree.wants_animation_frame());
    assert_eq!(tree.set_reduced_motion(true), TreeUpdate::Paint);
    assert_eq!(
        tree.resolved_quad(node, tree.element_at(0).unwrap())
            .opacity,
        1.0
    );
    assert!(!tree.wants_animation_frame());
    assert_eq!(tree.set_reduced_motion(false), TreeUpdate::None);
}

#[test]
fn text_and_vector_color_targets_are_resolved_after_state_changes() {
    let text = Element::text("label")
        .text_style(argui_text::TextStyle {
            color: Color::BLACK,
            ..argui_text::TextStyle::default()
        })
        .interaction(Interaction::default())
        .when(
            VisualState::Hovered,
            StylePatch::new().set(property::TextColor, Color::WHITE),
        )
        .transition(transition());
    let mut text_tree = UiTree::new(text.clone());
    let text_node = text_tree.node_id_at(0).unwrap();
    assert_eq!(
        text_tree.resolved_text_color(text_node, Color::BLACK),
        Color::BLACK
    );
    text_tree.pointer_moved(Point::new(10.0, 10.0), &[region(text_node)]);
    text_tree.set_reduced_motion(true);
    assert_eq!(
        text_tree.resolved_text_color(text_node, Color::BLACK),
        Color::WHITE
    );
    assert_eq!(
        text_tree
            .resolved_text_style(
                text_node,
                match &text.kind {
                    ElementKind::Text { style, .. } => style,
                    _ => unreachable!("text element changed kind"),
                },
            )
            .color,
        Color::WHITE
    );

    let vector = Element::vector(argui_ui::VectorId(4))
        .vector_color(Color::BLACK)
        .interaction(Interaction::default())
        .when(
            VisualState::Hovered,
            StylePatch::new().set(property::VectorColor, Color::WHITE),
        )
        .transition(transition());
    let mut vector_tree = UiTree::new(vector);
    let vector_node = vector_tree.node_id_at(0).unwrap();
    vector_tree.pointer_moved(Point::new(10.0, 10.0), &[region(vector_node)]);
    vector_tree.set_reduced_motion(true);
    assert_eq!(
        vector_tree.resolved_vector_color(vector_node, Color::BLACK),
        Color::WHITE
    );
}

#[test]
fn state_targets_replace_incompatible_paint_and_ignore_invalid_components() {
    let gradient = LinearGradient::new(
        Point::new(0.0, 0.0),
        Point::new(1.0, 0.0),
        argui_paint::ColorInterpolation::Oklab,
        [
            GradientStop::new(0.0, Color::BLACK),
            GradientStop::new(1.0, Color::WHITE),
        ],
    )
    .unwrap();
    let gradient_element = Element::container([])
        .fill(Fill::Linear(gradient.clone()))
        .interaction(Interaction::default())
        .when(
            VisualState::Hovered,
            StylePatch::new().set(property::BackgroundColor, Color::WHITE),
        );
    let mut gradient_tree = UiTree::new(gradient_element.clone());
    let gradient_node = gradient_tree.node_id_at(0).unwrap();
    gradient_tree.pointer_moved(Point::new(10.0, 10.0), &[region(gradient_node)]);
    assert_eq!(
        gradient_tree
            .resolved_quad(gradient_node, &gradient_element)
            .background,
        Some(Fill::Solid(Color::WHITE))
    );

    let invalid_index = Element::container([])
        .fill(Fill::Linear(gradient.clone()))
        .interaction(Interaction::default())
        .when(
            VisualState::Hovered,
            StylePatch::new().set(property::gradient_stop_offset(9), 0.5),
        );
    let mut invalid_tree = UiTree::new(invalid_index.clone());
    let invalid_node = invalid_tree.node_id_at(0).unwrap();
    invalid_tree.pointer_moved(Point::new(10.0, 10.0), &[region(invalid_node)]);
    let Fill::Linear(invalid_gradient) = invalid_tree
        .resolved_quad(invalid_node, &invalid_index)
        .background
        .unwrap()
    else {
        panic!("expected linear gradient");
    };
    assert_eq!(invalid_gradient.stops.as_slice()[0].offset, 0.0);

    let rejected = Element::container([])
        .fill(Fill::Linear(gradient))
        .interaction(Interaction::default())
        .when(
            VisualState::Hovered,
            StylePatch::new().set(property::gradient_stop_offset(0), f32::NAN),
        );
    let mut rejected_tree = UiTree::new(rejected.clone());
    let rejected_node = rejected_tree.node_id_at(0).unwrap();
    rejected_tree.pointer_moved(Point::new(10.0, 10.0), &[region(rejected_node)]);
    let Fill::Linear(rejected_gradient) = rejected_tree
        .resolved_quad(rejected_node, &rejected)
        .background
        .unwrap()
    else {
        panic!("expected linear gradient");
    };
    assert_eq!(rejected_gradient.stops.as_slice()[0].offset, 0.0);
}

#[test]
fn border_and_layer_targets_are_safe_when_the_authored_components_are_missing() {
    let border_element = Element::container([])
        .interaction(Interaction::default())
        .when(
            VisualState::Hovered,
            StylePatch::new().set(property::BorderColor, Color::WHITE),
        );
    let mut border_tree = UiTree::new(border_element.clone());
    let border_node = border_tree.node_id_at(0).unwrap();
    border_tree.pointer_moved(Point::new(10.0, 10.0), &[region(border_node)]);
    let border = border_tree
        .resolved_quad(border_node, &border_element)
        .border
        .unwrap();
    assert_eq!(border.color, Color::WHITE);
    assert_eq!(border.widths.left, 0.0);

    let layer = LayerStyle::new(Rect::new(Point::default(), Size::new(20.0, 20.0)))
        .shadow(Shadow::drop([1.0, 2.0], 3.0, Color::BLACK));
    let layer_element = Element::container([])
        .interaction(Interaction::default())
        .layer(layer.clone())
        .when(
            VisualState::Hovered,
            StylePatch::new()
                .set(property::shadow_offset(9), [4.0, 5.0])
                .set(property::shadow_blur(9), 8.0)
                .set(property::shadow_spread(9), 6.0)
                .set(property::shadow_color(9), Color::WHITE),
        );
    let mut layer_tree = UiTree::new(layer_element.clone());
    let layer_node = layer_tree.node_id_at(0).unwrap();
    layer_tree.pointer_moved(Point::new(10.0, 10.0), &[region(layer_node)]);
    let resolved = layer_tree.resolved_layer(layer_node, &layer_element, &layer);
    assert_eq!(resolved.shadows, layer.shadows);
}
