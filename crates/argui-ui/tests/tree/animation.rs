use argui_animation::{Duration, Motion, Time, Tween};
use argui_core::Transform2D;
use argui_ui::{Element, TreeUpdate, UiTree, property};

/// A completed transform repaints even while an unrelated compositor track continues.
#[test]
fn settled_transform_repaints_while_other_animation_keeps_running() {
    let transform = Motion::new(Transform2D::IDENTITY);
    let opacity = Motion::new(1.0_f32);
    let mut tree = UiTree::new(Element::row([
        Element::text("Crisp at rest").bind(property::Transform, transform.clone()),
        Element::container([]).bind(property::LayerOpacity, opacity.clone()),
    ]));
    transform.animate_to(
        Transform2D::IDENTITY.scale(1.5, 1.5).translate(0.25, 0.0),
        Tween::new(Duration::from_millis(100)),
    );
    opacity.animate_to(0.5, Tween::new(Duration::from_millis(200)));
    assert_eq!(
        tree.advance_animations(Time::from_nanos(1)),
        TreeUpdate::None
    );
    assert_eq!(
        tree.advance_animations(Time::from_nanos(50_000_001)),
        TreeUpdate::Composite
    );
    assert_eq!(
        tree.advance_animations(Time::from_nanos(100_000_001)),
        TreeUpdate::Paint
    );
    assert!(!transform.is_active());
    assert!(tree.wants_animation_frame());
    assert_eq!(
        tree.advance_animations(Time::from_nanos(150_000_001)),
        TreeUpdate::Composite
    );
    assert_eq!(
        tree.advance_animations(Time::from_nanos(200_000_001)),
        TreeUpdate::Composite
    );
    assert!(!tree.wants_animation_frame());
    assert_eq!(
        tree.advance_animations(Time::from_nanos(250_000_001)),
        TreeUpdate::None
    );
}

/// Reduced motion refreshes settled transform pixels without forcing opacity repaint.
#[test]
fn reduced_motion_finishes_transform_with_paint_and_opacity_with_composition() {
    let transform = Motion::new(Transform2D::IDENTITY);
    let mut tree = UiTree::new(Element::text("Text").bind(property::Transform, transform.clone()));
    transform.animate_to(
        Transform2D::IDENTITY.scale(2.0, 2.0),
        Tween::new(Duration::from_secs(1)),
    );
    assert_eq!(tree.set_reduced_motion(true), TreeUpdate::Paint);
    assert!(!tree.wants_animation_frame());
    transform.animate_to(
        Transform2D::IDENTITY.scale(1.5, 1.5),
        Tween::new(Duration::from_secs(1)),
    );
    assert_eq!(
        tree.advance_animations(Time::from_nanos(1)),
        TreeUpdate::Paint
    );
    assert_eq!(
        tree.advance_animations(Time::from_nanos(2)),
        TreeUpdate::None
    );

    let opacity = Motion::new(1.0_f32);
    let mut tree = UiTree::new(Element::text("Text").bind(property::LayerOpacity, opacity.clone()));
    opacity.animate_to(0.5, Tween::new(Duration::from_secs(1)));
    assert_eq!(tree.set_reduced_motion(true), TreeUpdate::Composite);
}
