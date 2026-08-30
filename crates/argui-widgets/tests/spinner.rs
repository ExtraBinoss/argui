use argui_animation::{Duration, Frame, Time};
use argui_core::{ColorScheme, Transform2D};
use argui_paint::VectorId;
use argui_runtime::{Context, Render, WindowEnvironment};
use argui_widgets::Spinner;

#[test]
fn spinner_ticks_only_when_motion_is_allowed() {
    let mut spinner = Spinner::new(VectorId::fresh(), 18.0);
    let initial = Render::render(&mut spinner, &mut Context::default());
    assert_eq!(initial.transform, Transform2D::IDENTITY);
    assert!(spinner.wants_animation_frame());
    let mut cx = Context::default();
    Render::animation_frame(
        &mut spinner,
        Frame {
            now: Time::from_nanos(16_000_000),
            elapsed: Duration::from_millis(16),
        },
        &mut cx,
    );
    assert_ne!(
        Render::render(&mut spinner, &mut Context::default()).transform,
        Transform2D::IDENTITY
    );

    let entity = argui_runtime::Entity::new(Spinner::new(VectorId::fresh(), 18.0));
    let _ = entity.render_in(WindowEnvironment {
        color_scheme: ColorScheme::Dark,
        reduced_motion: true,
        ..WindowEnvironment::default()
    });
    entity.read(|spinner| assert!(!spinner.wants_animation_frame()));
}
