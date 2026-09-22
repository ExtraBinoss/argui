use std::time::Duration;

use argui_core::{Point, ScrollDelta};
use argui_ui::{Element, InertialScroll, ScrollPhysics, UiTree};
use winit::event::TouchPhase;

#[path = "../../src/app/inertia.rs"]
mod implementation;

use implementation::{ScrollInertia, ScrollSample, touch_scroll_delta};

fn sample(delta: Point, phase: TouchPhase, now: Duration) -> ScrollSample {
    ScrollSample {
        delta: ScrollDelta::Pixels(delta),
        phase,
        now,
        target: None,
        physics: ScrollPhysics::Hybrid,
        point: Point::new(12.0, 24.0),
        dispatch_wheel: false,
        reduced_motion: false,
    }
}

fn seed_release(inertia: &mut ScrollInertia) {
    inertia.observe(sample(
        Point::default(),
        TouchPhase::Started,
        Duration::ZERO,
    ));
    inertia.observe(sample(
        Point::new(0.0, 8.0),
        TouchPhase::Moved,
        Duration::from_millis(16),
    ));
    inertia.observe(sample(
        Point::default(),
        TouchPhase::Ended,
        Duration::from_millis(32),
    ));
}

#[test]
fn released_scroll_decays_deterministically() {
    let mut inertia = ScrollInertia::default();
    seed_release(&mut inertia);
    assert_eq!(inertia.target(), None);

    let first = inertia.advance(Duration::from_millis(48)).unwrap().0;
    let second = inertia.advance(Duration::from_millis(64)).unwrap().0;

    assert!(first.y > 0.0);
    assert!(second.y > 0.0 && second.y < first.y);
}

#[test]
fn touch_scroll_uses_the_layout_delta_convention_so_content_follows_the_finger() {
    assert_eq!(
        touch_scroll_delta(Point::new(-8.0, 3.0), true),
        Point::new(-8.0, 3.0)
    );
    assert_eq!(
        touch_scroll_delta(Point::new(-8.0, 3.0), false),
        Point::new(8.0, -3.0)
    );
}

#[test]
fn new_contact_discards_previous_release_velocity() {
    let mut inertia = ScrollInertia::default();
    seed_release(&mut inertia);
    inertia.observe(sample(
        Point::default(),
        TouchPhase::Started,
        Duration::from_millis(40),
    ));

    assert!(inertia.advance(Duration::from_secs(1)).is_none());
}

#[test]
fn reduced_motion_and_cancelled_touch_stop_momentum() {
    let mut inertia = ScrollInertia::default();
    seed_release(&mut inertia);
    let mut reduced = sample(
        Point::default(),
        TouchPhase::Moved,
        Duration::from_millis(40),
    );
    reduced.reduced_motion = true;
    inertia.observe(reduced);
    assert!(!inertia.needs_frame());
    assert!(inertia.advance(Duration::from_secs(1)).is_none());

    seed_release(&mut inertia);
    inertia.observe(sample(
        Point::default(),
        TouchPhase::Cancelled,
        Duration::from_millis(40),
    ));
    assert!(!inertia.needs_frame());
    assert!(inertia.advance(Duration::from_secs(1)).is_none());
}

/// Non-inertial physics and line-based deltas must discard a previous release.
#[test]
fn direct_native_and_line_inputs_cancel_pending_momentum() {
    for physics in [ScrollPhysics::Direct, ScrollPhysics::Native] {
        let mut inertia = ScrollInertia::default();
        seed_release(&mut inertia);
        let mut input = sample(
            Point::new(0.0, 8.0),
            TouchPhase::Moved,
            Duration::from_millis(40),
        );
        input.physics = physics;
        inertia.observe(input);
        assert!(!inertia.needs_frame());
        assert!(inertia.advance(Duration::from_secs(1)).is_none());
    }

    let mut inertia = ScrollInertia::default();
    seed_release(&mut inertia);
    let mut input = sample(
        Point::new(0.0, 8.0),
        TouchPhase::Moved,
        Duration::from_millis(40),
    );
    input.delta = ScrollDelta::Lines(Point::new(0.0, 1.0));
    inertia.observe(input);
    assert!(!inertia.needs_frame());
}

/// Momentum starts after its continuation grace and preserves pointer metadata.
#[test]
fn unreleased_scroll_waits_for_grace_and_preserves_dispatch_policy() {
    let mut inertia = ScrollInertia::default();
    let mut input = sample(
        Point::new(0.0, 16.0),
        TouchPhase::Started,
        Duration::from_millis(10),
    );
    input.dispatch_wheel = true;
    inertia.observe(input);
    assert!(inertia.needs_frame());
    assert!(inertia.advance(Duration::from_millis(20)).is_none());
    let (delta, point, dispatch_wheel) = inertia.advance(Duration::from_millis(40)).unwrap();
    assert!(delta.y > 0.0);
    assert_eq!(point, Point::new(12.0, 24.0));
    assert!(dispatch_wheel);
}

/// Invalid custom settings are normalized without producing non-finite motion.
#[test]
fn invalid_custom_physics_falls_back_to_finite_motion() {
    let mut inertia = ScrollInertia::default();
    let invalid = InertialScroll {
        velocity_limit: f32::NAN,
        stop_velocity: f32::INFINITY,
        decay: f32::NEG_INFINITY,
        sample_weight: f32::NAN,
        ..InertialScroll::default()
    };
    let mut input = sample(
        Point::new(100.0, -100.0),
        TouchPhase::Started,
        Duration::ZERO,
    );
    input.physics = ScrollPhysics::Inertial(invalid);
    inertia.observe(input);
    let mut release = sample(
        Point::default(),
        TouchPhase::Ended,
        Duration::from_millis(16),
    );
    release.physics = ScrollPhysics::Inertial(invalid);
    inertia.observe(release);
    let delta = inertia.advance(Duration::from_millis(32)).unwrap().0;
    assert!(delta.x.is_finite() && delta.y.is_finite());
    assert!(delta.x > 0.0 && delta.y < 0.0);
}

/// Zero decay and immediate stopping exercise the two terminal momentum paths.
#[test]
fn zero_decay_and_stop_threshold_are_respected() {
    let mut inertia = ScrollInertia::default();
    let physics = ScrollPhysics::Inertial(InertialScroll {
        decay: 0.0,
        stop_velocity: 0.0,
        sample_weight: 1.0,
        ..InertialScroll::default()
    });
    let mut input = sample(Point::new(12.0, 0.0), TouchPhase::Started, Duration::ZERO);
    input.physics = physics;
    inertia.observe(input);
    let mut release = sample(
        Point::default(),
        TouchPhase::Ended,
        Duration::from_millis(16),
    );
    release.physics = physics;
    inertia.observe(release);
    let first = inertia.advance(Duration::from_millis(32)).unwrap().0;
    let second = inertia.advance(Duration::from_millis(48)).unwrap().0;
    assert!(first.x > 0.0 && second.x > 0.0);
    assert!(inertia.needs_frame());

    let mut inertia = ScrollInertia::default();
    let physics = ScrollPhysics::Inertial(InertialScroll {
        stop_velocity: 10_000.0,
        ..InertialScroll::default()
    });
    input.physics = physics;
    inertia.observe(input);
    release.physics = physics;
    inertia.observe(release);
    let _ = inertia.advance(Duration::from_millis(32));
    assert!(!inertia.needs_frame());
}

/// Out-of-range finite coefficients are clamped before filtering velocity.
#[test]
fn finite_custom_physics_is_clamped() {
    let mut inertia = ScrollInertia::default();
    let physics = ScrollPhysics::Inertial(InertialScroll {
        velocity_limit: -1.0,
        stop_velocity: -1.0,
        decay: -1.0,
        sample_weight: 2.0,
        ..InertialScroll::default()
    });
    let mut input = sample(Point::new(8.0, 8.0), TouchPhase::Started, Duration::ZERO);
    input.physics = physics;
    inertia.observe(input);
    assert!(inertia.needs_frame());
    let mut release = sample(
        Point::default(),
        TouchPhase::Ended,
        Duration::from_millis(16),
    );
    release.physics = physics;
    inertia.observe(release);
    assert_eq!(
        inertia.advance(Duration::from_millis(32)).unwrap().0,
        Point::default()
    );
}

/// Retargeting a continuous gesture discards velocity from the old scroll region.
#[test]
fn changing_scroll_target_resets_velocity_without_losing_the_new_target() {
    let tree = UiTree::new(Element::column([
        Element::text("first"),
        Element::text("second"),
    ]));
    let first = tree.node_ids()[1];
    let second = tree.node_ids()[2];
    let mut inertia = ScrollInertia::default();
    let mut input = sample(Point::new(0.0, 16.0), TouchPhase::Started, Duration::ZERO);
    input.target = Some(first);
    inertia.observe(input);
    assert_eq!(inertia.target(), Some(first));

    input.phase = TouchPhase::Moved;
    input.now = Duration::from_millis(16);
    input.delta = ScrollDelta::Pixels(Point::default());
    input.target = Some(second);
    inertia.observe(input);
    assert_eq!(inertia.target(), Some(second));
    input.phase = TouchPhase::Ended;
    input.now = Duration::from_millis(32);
    inertia.observe(input);
    assert!(inertia.advance(Duration::from_millis(48)).is_none());
    assert!(!inertia.needs_frame());
}

/// A zero-weight sample cannot introduce velocity, even when input continues.
#[test]
fn zero_sample_weight_ignores_motion_until_a_new_config_accepts_it() {
    let mut inertia = ScrollInertia::default();
    let mut input = sample(Point::new(0.0, 20.0), TouchPhase::Started, Duration::ZERO);
    input.physics = ScrollPhysics::Inertial(InertialScroll {
        sample_weight: 0.0,
        ..InertialScroll::default()
    });
    inertia.observe(input);
    assert!(inertia.needs_frame());
    assert!(inertia.advance(Duration::from_millis(10)).is_none());

    input.phase = TouchPhase::Moved;
    input.now = Duration::from_millis(16);
    input.physics = ScrollPhysics::Inertial(InertialScroll {
        sample_weight: 1.0,
        ..InertialScroll::default()
    });
    inertia.observe(input);
    input.phase = TouchPhase::Ended;
    input.now = Duration::from_millis(32);
    inertia.observe(input);
    assert!(inertia.advance(Duration::from_millis(48)).unwrap().0.y > 0.0);
}

/// Time moving backwards must not produce a negative or non-finite scroll delta.
#[test]
fn out_of_order_timestamps_use_bounded_positive_frame_time() {
    let mut inertia = ScrollInertia::default();
    let mut input = sample(
        Point::new(4.0, -8.0),
        TouchPhase::Started,
        Duration::from_millis(100),
    );
    inertia.observe(input);
    input.phase = TouchPhase::Moved;
    input.now = Duration::from_millis(90);
    inertia.observe(input);
    input.phase = TouchPhase::Ended;
    input.now = Duration::from_millis(80);
    inertia.observe(input);
    let (delta, point, dispatch_wheel) = inertia.advance(Duration::from_millis(70)).unwrap();
    assert!(delta.x.is_finite() && delta.x > 0.0);
    assert!(delta.y.is_finite() && delta.y < 0.0);
    assert_eq!(point, Point::new(12.0, 24.0));
    assert!(!dispatch_wheel);
}

/// An end phase without displacement cannot leave a frame loop running.
#[test]
fn release_without_movement_stops_before_dispatching_momentum() {
    let mut inertia = ScrollInertia::default();
    inertia.observe(sample(
        Point::default(),
        TouchPhase::Ended,
        Duration::from_millis(16),
    ));
    assert!(inertia.needs_frame());
    assert!(inertia.advance(Duration::from_millis(32)).is_none());
    assert!(!inertia.needs_frame());
}

/// A finite zero sample weight cannot create velocity even after release.
#[test]
fn fully_filtered_gesture_never_dispatches_momentum() {
    let mut inertia = ScrollInertia::default();
    let physics = ScrollPhysics::Inertial(InertialScroll {
        sample_weight: 0.0,
        ..InertialScroll::default()
    });
    let mut input = sample(Point::new(50.0, -50.0), TouchPhase::Started, Duration::ZERO);
    input.physics = physics;
    inertia.observe(input);
    input.phase = TouchPhase::Moved;
    input.now = Duration::from_millis(16);
    inertia.observe(input);
    input.phase = TouchPhase::Ended;
    input.now = Duration::from_millis(32);
    inertia.observe(input);
    assert!(inertia.advance(Duration::from_millis(48)).is_none());
    assert!(!inertia.needs_frame());
}

/// Cancelling a gesture clears both its target and any pending animation frame.
#[test]
fn explicit_cancel_clears_target_and_pending_momentum() {
    let tree = UiTree::new(Element::text("scroll target"));
    let mut inertia = ScrollInertia::default();
    let mut input = sample(Point::new(0.0, 4.0), TouchPhase::Started, Duration::ZERO);
    input.target = Some(tree.node_ids()[0]);
    inertia.observe(input);
    assert_eq!(inertia.target(), input.target);
    assert!(inertia.needs_frame());

    inertia.cancel();
    assert_eq!(inertia.target(), None);
    assert!(!inertia.needs_frame());
    assert!(inertia.advance(Duration::from_secs(1)).is_none());
}
