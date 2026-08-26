use argui_animation::{Duration, Scheduler, Time};

#[test]
fn idle_scheduler_requests_no_frame() {
    let mut scheduler = Scheduler::default();
    assert!(!scheduler.needs_frame());
    assert_eq!(scheduler.frame(Time::from_nanos(50)), None);
    assert!(scheduler.active().is_empty());
}

#[test]
fn every_active_animation_shares_one_exact_frame_time() {
    let mut scheduler = Scheduler::default();
    let first = scheduler.start();
    let second = scheduler.start();

    assert!(scheduler.needs_frame());
    assert!(scheduler.contains(first));
    assert!(scheduler.contains(second));
    assert_eq!(scheduler.active(), &[first, second]);
    assert_eq!(
        scheduler.frame(Time::from_nanos(100)),
        Some(argui_animation::Frame {
            now: Time::from_nanos(100),
            elapsed: Duration::ZERO,
        })
    );
    assert_eq!(
        scheduler.frame(Time::from_nanos(116)),
        Some(argui_animation::Frame {
            now: Time::from_nanos(116),
            elapsed: Duration::from_nanos(16),
        })
    );
}

#[test]
fn stopping_work_resets_idle_timing() {
    let mut scheduler = Scheduler::default();
    let first = scheduler.start();
    let second = scheduler.start();

    let _ = scheduler.frame(Time::from_nanos(50));
    assert!(scheduler.stop(first));
    assert!(!scheduler.stop(first));
    assert!(scheduler.needs_frame());
    assert!(scheduler.stop(second));
    assert!(!scheduler.needs_frame());

    let replacement = scheduler.start();
    assert_ne!(replacement, first);
    assert_eq!(
        scheduler.frame(Time::from_nanos(500)).unwrap().elapsed,
        Duration::ZERO
    );
}
