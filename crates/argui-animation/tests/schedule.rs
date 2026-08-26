use argui_animation::{CueId, Duration, Schedule, ScheduleBuilder};

#[test]
fn sequence_parallel_and_stagger_have_exact_offsets() {
    let sequence = Schedule::sequence([Duration::from_millis(10), Duration::from_millis(20)]);
    assert_eq!(sequence.cue(sequence_id(0)).unwrap().start, Duration::ZERO);
    assert_eq!(
        sequence.cue(sequence_id(1)).unwrap().start,
        Duration::from_millis(10)
    );
    assert_eq!(sequence.duration(), Duration::from_millis(30));

    let parallel = Schedule::parallel([Duration::from_millis(10), Duration::from_millis(25)]);
    assert_eq!(parallel.cue(sequence_id(1)).unwrap().start, Duration::ZERO);
    assert_eq!(parallel.duration(), Duration::from_millis(25));

    let stagger = Schedule::stagger(3, Duration::from_millis(10), Duration::from_millis(4));
    assert_eq!(
        stagger.cue(sequence_id(2)).unwrap().start,
        Duration::from_millis(8)
    );
    assert_eq!(stagger.duration(), Duration::from_millis(18));
}

#[test]
fn builder_supports_parallel_groups_and_dependencies() {
    let mut builder = ScheduleBuilder::new();
    let first = builder.then(Duration::from_millis(10));
    let parallel = builder.with(Duration::from_millis(20));
    let dependent = builder.after(first, Duration::from_millis(5), Duration::from_millis(3));
    let schedule = builder.build();

    assert_eq!(schedule.cue(first).unwrap().start, Duration::ZERO);
    assert_eq!(schedule.cue(parallel).unwrap().start, Duration::ZERO);
    assert_eq!(
        schedule.cue(dependent).unwrap().start,
        Duration::from_millis(15)
    );
    assert_eq!(schedule.duration(), Duration::from_millis(20));
    assert_eq!(schedule.len(), 3);
    assert!(!schedule.is_empty());
    assert!(schedule.cue(sequence_id(99)).is_none());
}

#[test]
fn cue_progress_handles_before_active_after_and_instantaneous() {
    let schedule = Schedule::stagger(2, Duration::from_millis(10), Duration::from_millis(5));
    let cue = schedule.cue(sequence_id(1)).unwrap();
    assert_eq!(cue.progress(Duration::from_millis(3)), 0.0);
    assert_eq!(cue.progress(Duration::from_millis(10)), 0.5);
    assert_eq!(cue.progress(Duration::from_millis(20)), 1.0);

    let instant = Schedule::sequence([Duration::ZERO]);
    let cue = instant.cue(sequence_id(0)).unwrap();
    assert_eq!(cue.progress(Duration::ZERO), 1.0);
    assert!(Schedule::default().is_empty());
}

fn sequence_id(index: usize) -> argui_animation::CueId {
    CueId::from_index(index)
}
