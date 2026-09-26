use argui_animation::{
    Direction, Duration, Easing, FrameSchedule, Iterations, Keyframe, Keyframes, Motion,
    MotionTrack, StepPosition, Steps, Time, Timeline, Timing, Tween,
};

fn ms(value: u64) -> Time {
    Time::from_nanos(value * 1_000_000)
}

fn deadline(schedule: FrameSchedule) -> Time {
    match schedule {
        FrameSchedule::At(time) => time,
        other => panic!("expected a one-shot deadline, got {other:?}"),
    }
}

fn step_timeline(timing: Timing) -> Timeline<f32> {
    Timeline::new(
        Keyframes::new([
            Keyframe::new(0.0, 0.0)
                .easing(Easing::Steps(Steps::new(4, StepPosition::JumpEnd).unwrap())),
            Keyframe::new(1.0, 100.0),
        ])
        .unwrap(),
        timing,
    )
    .unwrap()
}

#[test]
fn step_tween_sleeps_through_plateaus_and_wakes_at_each_jump() {
    let motion = Motion::new(0.0_f32);
    motion.animate_to(
        100.0,
        Tween::new(Duration::from_millis(100))
            .easing(Easing::Steps(Steps::new(4, StepPosition::JumpEnd).unwrap())),
    );
    assert_eq!(
        MotionTrack::frame_schedule(&motion),
        FrameSchedule::Continuous
    );
    assert!(!MotionTrack::advance(&motion, ms(0)));
    for (index, lower, upper) in [(1, 25, 26), (2, 50, 51), (3, 75, 76), (4, 100, 101)] {
        let wake = deadline(MotionTrack::frame_schedule(&motion));
        assert!(wake >= ms(lower) && wake < ms(upper), "{wake:?}");
        assert!(MotionTrack::advance(&motion, wake));
        assert_eq!(motion.value(), index as f32 * 25.0);
    }
    assert_eq!(MotionTrack::frame_schedule(&motion), FrameSchedule::None);
}

#[test]
fn delay_rate_reverse_seek_and_pause_reanchor_step_deadlines() {
    let mut timeline =
        step_timeline(Timing::new(Duration::from_millis(100)).delay(Duration::from_millis(20)));
    timeline.play(ms(0));
    assert_eq!(deadline(timeline.frame_schedule(ms(0))), ms(20));
    assert_eq!(timeline.sample(ms(20)).value, Some(0.0));
    let first = deadline(timeline.frame_schedule(ms(20)));
    assert!(first >= ms(45) && first < ms(46));

    timeline.set_playback_rate(2.0, ms(20)).unwrap();
    let faster = deadline(timeline.frame_schedule(ms(20)));
    assert!(faster >= ms(32) && faster < ms(33));
    timeline.seek(Duration::from_millis(70), ms(40));
    assert_eq!(timeline.sample(ms(40)).value, Some(50.0));
    let next = deadline(timeline.frame_schedule(ms(40)));
    assert!(next >= ms(52) && next < ms(53));

    timeline.reverse(ms(40));
    let backward = deadline(timeline.frame_schedule(ms(40)));
    assert!(backward > ms(40) && backward < ms(41));
    assert_eq!(timeline.sample(backward).value, Some(25.0));
    timeline.pause(backward);
    assert_eq!(timeline.frame_schedule(ms(100)), FrameSchedule::None);
    timeline.resume(ms(100));
    let resumed = deadline(timeline.frame_schedule(ms(100)));
    assert!(resumed > ms(100) && resumed < ms(113));
}

#[test]
fn alternate_iterations_and_mixed_segments_schedule_boundaries() {
    let mut timeline = step_timeline(
        Timing::new(Duration::from_millis(100))
            .iterations(Iterations::Finite(2.0))
            .direction(Direction::Alternate),
    );
    timeline.play(ms(0));
    assert_eq!(timeline.sample(ms(75)).value, Some(75.0));
    let end = deadline(timeline.frame_schedule(ms(75)));
    assert!(end >= ms(100) && end < ms(101));
    assert_eq!(timeline.sample(end).value, Some(75.0));
    let reverse_jump = deadline(timeline.frame_schedule(end));
    assert!(reverse_jump >= ms(125) && reverse_jump < ms(126));
    assert_eq!(timeline.sample(reverse_jump).value, Some(50.0));

    let mut mixed = Timeline::new(
        Keyframes::new([
            Keyframe::new(0.0, 0.0).hold(),
            Keyframe::new(0.5, 50.0),
            Keyframe::new(1.0, 100.0),
        ])
        .unwrap(),
        Timing::new(Duration::from_millis(100)),
    )
    .unwrap();
    mixed.play(ms(0));
    let held_wake = deadline(mixed.frame_schedule(ms(0)));
    assert!(held_wake >= ms(50) && held_wake < ms(51));
    assert_eq!(mixed.sample(ms(50)).value, Some(50.0));
    assert_eq!(mixed.frame_schedule(ms(50)), FrameSchedule::Continuous);
}

#[test]
fn retargeted_step_timeline_restarts_its_deadline_from_presented_value() {
    let mut timeline = step_timeline(Timing::new(Duration::from_millis(100)));
    timeline.play(ms(0));
    assert_eq!(timeline.sample(ms(30)).value, Some(25.0));
    timeline
        .retarget(
            200.0,
            Duration::from_millis(100),
            Easing::Steps(Steps::new(4, StepPosition::JumpEnd).unwrap()),
            ms(30),
        )
        .unwrap();
    assert_eq!(timeline.sample(ms(30)).value, Some(25.0));
    let first = deadline(timeline.frame_schedule(ms(30)));
    assert!(first >= ms(55) && first < ms(56));
    assert_eq!(timeline.sample(first).value, Some(68.75));
}

#[test]
fn every_step_position_uses_one_shot_wakes_between_jumps() {
    for position in [
        StepPosition::JumpStart,
        StepPosition::JumpEnd,
        StepPosition::JumpNone,
        StepPosition::JumpBoth,
    ] {
        let mut timeline = Timeline::new(
            Keyframes::new([
                Keyframe::new(0.0, 0.0).easing(Easing::Steps(Steps::new(4, position).unwrap())),
                Keyframe::new(1.0, 100.0),
            ])
            .unwrap(),
            Timing::new(Duration::from_millis(100)),
        )
        .unwrap();
        timeline.play(Time::ZERO);
        let initial = timeline.sample(Time::ZERO).value.unwrap();
        let wake = deadline(timeline.frame_schedule(Time::ZERO));
        assert!(wake >= ms(25) && wake < ms(26), "{position:?}: {wake:?}");
        let next = timeline.sample(wake).value.unwrap();
        assert!(next > initial, "{position:?}: {initial} -> {next}");
    }
}

#[test]
fn paused_motion_has_no_deadline_and_resumes_from_its_held_value() {
    let motion = Motion::new(0.0_f32);
    motion.animate_to(
        100.0,
        Tween::new(Duration::from_millis(100))
            .easing(Easing::Steps(Steps::new(4, StepPosition::JumpEnd).unwrap())),
    );
    assert!(!MotionTrack::advance(&motion, ms(0)));
    motion.pause();
    assert_eq!(MotionTrack::frame_schedule(&motion), FrameSchedule::None);
    motion.resume();
    assert_eq!(
        MotionTrack::frame_schedule(&motion),
        FrameSchedule::Continuous
    );
    assert!(!MotionTrack::advance(&motion, ms(100)));
    let wake = deadline(MotionTrack::frame_schedule(&motion));
    assert!(wake >= ms(125) && wake < ms(126));
    assert!(MotionTrack::advance(&motion, wake));
    assert_eq!(motion.value(), 25.0);
}
