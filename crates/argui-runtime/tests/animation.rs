use super::*;

/// Creates the real model-free runtime without an OS surface or GPU.
fn application() -> Application {
    Application::new(
        Default::default(),
        Default::default(),
        Default::default(),
        None,
        None,
        None,
        |_| {},
    )
}

#[test]
fn native_animation_ticks_are_bounded_from_the_previous_frame() {
    let mut animation = RuntimeAnimations::new(None);
    animation.native_paced = true;
    assert!(animation.requests_immediate_frame());
    animation.sync(true);
    assert!(!animation.requests_immediate_frame());
    let frame = animation.frame().unwrap();
    assert_eq!(
        animation.next_frame_at(),
        Some(frame.now + NATIVE_FRAME_INTERVAL)
    );
    animation.sync(false);
    assert!(animation.requests_immediate_frame());
    assert!(animation.next_frame_at().is_none());
}

#[test]
fn idle_native_paced_windows_and_pending_paint_do_not_poll() {
    let mut app = application();
    app.animations.native_paced = true;
    app.invalidate(ViewUpdate::Paint);
    assert!(app.next_animation_deadline().is_none());
    app.animations.sync(true);
    assert!(app.next_animation_deadline().is_some());
    app.set_window_visible(false);
    assert!(app.next_animation_deadline().is_none());
}

#[test]
fn one_shot_deadlines_are_preserved_with_or_without_native_pacing() {
    let mut animation = RuntimeAnimations::new(None);
    animation.wake_at = Some(Time::ZERO);
    assert_eq!(animation.next_frame_at(), Some(Time::ZERO));
    animation.native_paced = true;
    animation.sync(true);
    animation.frame();
    assert_eq!(animation.next_frame_at(), Some(Time::ZERO));
    assert!(animation.take_due_wake());
    assert!(!animation.take_due_wake());
    assert!(animation.next_frame_at().is_some());
}

#[test]
fn ordinary_vsync_animation_retains_native_frame_pacing() {
    let mut animation = RuntimeAnimations::new(None);
    animation.sync(true);
    animation.frame();
    assert!(animation.requests_immediate_frame());
    assert!(animation.next_frame_at().is_none());
}

#[test]
fn a_native_animation_has_a_deadline_before_any_redraw_arrives() {
    let mut animation = RuntimeAnimations::new(None);
    animation.set_native_pacing(true);
    animation.sync(true);
    assert!(animation.next_frame_at().is_some());
    animation.sync(false);
    assert!(animation.next_frame_at().is_none());
}
