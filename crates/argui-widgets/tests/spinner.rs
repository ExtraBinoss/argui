use argui_animation::Time;
use argui_core::{ColorScheme, Transform2D};
use argui_paint::VectorId;
use argui_runtime::{Context, Render, WindowEnvironment};
use argui_ui::{TreeUpdate, UiTree};
use argui_widgets::Spinner;

#[test]
fn spinner_uses_retained_compositor_motion_only_when_allowed() {
    let mut spinner = Spinner::new(VectorId::fresh(), 18.0);
    let initial = Render::render(&mut spinner, &mut Context::default());
    assert_eq!(initial.transform, Transform2D::IDENTITY);
    assert!(!spinner.wants_animation_frame());
    let mut tree = UiTree::new(initial.clone());
    assert!(tree.wants_animation_frame());
    tree.advance_animations(Time::from_nanos(1));
    assert_eq!(
        tree.advance_animations(Time::from_nanos(250_000_001)),
        TreeUpdate::Composite
    );
    assert_ne!(
        tree.resolved_transform(tree.node_ids()[0], &initial),
        Transform2D::IDENTITY
    );

    let entity = argui_runtime::Entity::new(Spinner::new(VectorId::fresh(), 18.0));
    let reduced = entity.render_in(WindowEnvironment {
        color_scheme: ColorScheme::Dark,
        reduced_motion: true,
        ..WindowEnvironment::default()
    });
    assert!(!UiTree::new(reduced).wants_animation_frame());
}
