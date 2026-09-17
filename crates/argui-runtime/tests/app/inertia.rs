use std::time::Duration;

use argui_core::{Point, ScrollDelta};
use argui_ui::ScrollPhysics;
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
