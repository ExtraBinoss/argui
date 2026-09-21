use argui_animation::Time;
use argui_core::Color;
use argui_schema::{
    PropertyAnimation, PropertyMotionKey, PropertyMotionStore, StateTransitionPolicy,
};
use argui_ui::{Dimension, ExpandedDimension, RetainedIdentity};

#[path = "property_motion/engine.rs"]
mod engine;

fn key(property: u64) -> PropertyMotionKey {
    PropertyMotionKey::new(RetainedIdentity::new(7, 11), property, 13)
}

#[test]
fn repeated_numeric_animation_advances_only_while_mounted() {
    let mut store = PropertyMotionStore::new();
    let spec = PropertyAnimation::new(Some(0.0), Some(360.0), 1000.0, "infinite").unwrap();
    store.begin_render();
    assert_eq!(
        store
            .sample_number(key(1), 0.0, spec.clone(), false)
            .unwrap(),
        0.0
    );
    store.end_render();
    assert!(store.needs_frame());
    assert!(!store.advance(Time::from_nanos(1)));
    assert!(store.advance(Time::from_nanos(500_000_001)));
    store.begin_render();
    let halfway = store.sample_number(key(1), 0.0, spec, false).unwrap();
    store.end_render();
    assert!((halfway - 180.0).abs() < 0.001);
    store.begin_render();
    store.end_render();
    assert!(!store.needs_frame());
    assert_eq!(store.len(), 0);
}

#[test]
fn implicit_transition_retargets_only_on_declarative_value_change() {
    let mut store = PropertyMotionStore::new();
    let spec = PropertyAnimation::new(None, None, 100.0, "once").unwrap();
    store.begin_render();
    assert_eq!(
        store
            .sample_number(key(1), 10.0, spec.clone(), false)
            .unwrap(),
        10.0
    );
    store.end_render();
    assert!(!store.needs_frame());
    store.begin_render();
    assert_eq!(
        store
            .sample_number(key(1), 20.0, spec.clone(), false)
            .unwrap(),
        10.0
    );
    store.end_render();
    assert!(store.needs_frame());
    store.advance(Time::from_nanos(1));
    store.advance(Time::from_nanos(100_000_001));
    store.begin_render();
    assert_eq!(
        store.sample_number(key(1), 20.0, spec, false).unwrap(),
        20.0
    );
    store.end_render();
    assert!(!store.needs_frame());
}

#[test]
fn colors_and_dimensions_use_the_same_retained_store() {
    let mut store = PropertyMotionStore::new();
    let red = Color::from_srgba8(255, 0, 0, 255);
    let blue = Color::from_srgba8(0, 0, 255, 255);
    store.begin_render();
    assert_eq!(
        store
            .sample_color(
                key(2),
                blue,
                PropertyAnimation::new(Some(red), Some(blue), 100.0, "once").unwrap(),
                false
            )
            .unwrap(),
        red
    );
    let width = store
        .sample_dimension(
            key(3),
            Dimension::length(40.0),
            PropertyAnimation::new(
                Some(Dimension::length(20.0)),
                Some(Dimension::length(40.0)),
                100.0,
                "once",
            )
            .unwrap(),
            false,
        )
        .unwrap();
    assert_eq!(width.expand(), ExpandedDimension::Length(20.0));
    store.end_render();
    assert_eq!(store.len(), 2);
    assert!(
        store
            .sample_dimension(
                key(4),
                Dimension::percent(0.5),
                PropertyAnimation::new(
                    Some(Dimension::length(20.0)),
                    Some(Dimension::percent(0.5)),
                    100.0,
                    "once"
                )
                .unwrap(),
                false
            )
            .is_err()
    );
    assert!(
        store
            .sample_dimension(
                key(5),
                Dimension::auto(),
                PropertyAnimation::new(None, None, 100.0, "once").unwrap(),
                false
            )
            .is_err()
    );
}

#[test]
fn reduced_motion_stops_an_existing_timeline() {
    let mut store = PropertyMotionStore::new();
    let spec = PropertyAnimation::new(Some(0.0), Some(360.0), 1000.0, "infinite").unwrap();
    store.begin_render();
    store
        .sample_number(key(1), 0.0, spec.clone(), false)
        .unwrap();
    store.end_render();
    assert!(store.needs_frame());
    store.begin_render();
    assert_eq!(store.sample_number(key(1), 0.0, spec, true).unwrap(), 0.0);
    store.end_render();
    assert!(!store.needs_frame());
}

#[test]
fn hot_reload_reinitializes_a_slot_when_its_interpolable_type_changes() {
    let mut store = PropertyMotionStore::new();
    let number = PropertyAnimation::new(None, None, 100.0, "once").unwrap();
    store.begin_render();
    assert_eq!(
        store.sample_number(key(1), 12.0, number, false).unwrap(),
        12.0
    );
    store.end_render();
    let color = Color::from_srgba8(12, 34, 56, 255);
    let color_spec = PropertyAnimation::new(None, None, 100.0, "once").unwrap();
    store.begin_render();
    assert_eq!(
        store
            .sample_color(key(1), color, color_spec, false)
            .unwrap(),
        color
    );
    store.end_render();
    assert_eq!(store.len(), 1);
    assert!(!store.needs_frame());
}

#[test]
fn spring_retargets_without_replacing_the_retained_motion() {
    let mut store = PropertyMotionStore::new();
    let spring = PropertyAnimation::spring(None, None, 220.0, 24.0).unwrap();
    store.begin_render();
    assert_eq!(
        store
            .sample_number(key(1), 0.0, spring.clone(), false)
            .unwrap(),
        0.0
    );
    store.end_render();
    store.begin_render();
    assert_eq!(
        store
            .sample_number(key(1), 100.0, spring.clone(), false)
            .unwrap(),
        0.0
    );
    store.end_render();
    assert!(store.needs_frame());
    store.advance(Time::from_nanos(1));
    store.advance(Time::from_nanos(50_000_001));
    store.begin_render();
    let halfway = store.sample_number(key(1), 200.0, spring, false).unwrap();
    store.end_render();
    assert!(halfway > 0.0 && halfway < 200.0);
    assert_eq!(store.len(), 1);
}

#[test]
fn spring_interpolates_percentage_dimensions_without_losing_units() {
    let mut store = PropertyMotionStore::new();
    let spring = PropertyAnimation::spring(None, None, 220.0, 24.0).unwrap();
    store.begin_render();
    store
        .sample_dimension(key(2), Dimension::percent(0.1), spring.clone(), false)
        .unwrap();
    store.end_render();
    store.begin_render();
    store
        .sample_dimension(key(2), Dimension::percent(0.8), spring.clone(), false)
        .unwrap();
    store.end_render();
    store.advance(Time::from_nanos(1));
    store.advance(Time::from_nanos(50_000_001));
    store.begin_render();
    let sampled = store
        .sample_dimension(key(2), Dimension::percent(0.8), spring, false)
        .unwrap();
    store.end_render();
    let ExpandedDimension::Percent(progress) = sampled.expand() else {
        panic!("percentage unit was lost")
    };
    assert!(progress > 0.1 && progress < 0.8);
}

#[test]
fn dynamic_animation_error_is_visible_for_one_render_pass() {
    let mut store = PropertyMotionStore::new();
    store.begin_render();
    store.report_error("invalid animated dimension");
    assert_eq!(store.last_error(), Some("invalid animated dimension"));
    store.end_render();
    store.begin_render();
    assert_eq!(store.last_error(), None);
}

#[test]
fn multi_stop_keyframes_follow_offsets_and_named_easing() {
    let mut store = PropertyMotionStore::new();
    let spec = PropertyAnimation::new(None, None, 100.0, "once")
        .unwrap()
        .with_keyframes(vec![(0.0, 0.0), (0.5, 100.0), (1.0, 0.0)])
        .unwrap();
    store.begin_render();
    assert_eq!(
        store
            .sample_number(key(7), 0.0, spec.clone(), false)
            .unwrap(),
        0.0
    );
    store.end_render();
    store.advance(Time::from_nanos(1));
    store.advance(Time::from_nanos(50_000_001));
    store.begin_render();
    assert!((store.sample_number(key(7), 0.0, spec, false).unwrap() - 100.0).abs() < 0.01);
    store.end_render();
    let eased = PropertyAnimation::new(None::<f64>, None, 100.0, "once")
        .unwrap()
        .with_easing_name("ease-in")
        .unwrap();
    assert!(eased.with_easing_name("unknown").is_err());
}

#[test]
fn keyframe_validation_rejects_missing_endpoints_and_mixed_drivers() {
    let spec = PropertyAnimation::new(None::<f64>, None, 100.0, "once").unwrap();
    assert!(
        spec.clone()
            .with_keyframes(vec![(0.0, 0.0), (0.5, 1.0)])
            .is_err()
    );
    assert!(
        spec.clone()
            .with_keyframes(vec![(0.0, 0.0), (0.5, 1.0), (0.5, 2.0), (1.0, 0.0)])
            .is_err()
    );
    assert!(
        PropertyAnimation::spring(None::<f64>, None, 220.0, 24.0)
            .unwrap()
            .with_keyframes(vec![(0.0, 0.0), (1.0, 1.0)])
            .is_err()
    );
    assert!(PropertyAnimation::keyframes(vec![(0.0, 0.0), (1.0, 1.0)], 100.0, "infinite").is_ok());
}

#[test]
fn state_enter_animates_only_the_rising_edge_and_does_not_restart_idle() {
    let mut store = PropertyMotionStore::new();
    let spec = |active| {
        PropertyAnimation::new(None, None, 100.0, "once")
            .unwrap()
            .with_state_transition(StateTransitionPolicy::Enter, active, 1.0)
            .unwrap()
    };
    store.begin_render();
    assert_eq!(
        store
            .sample_number(key(8), 1.0, spec(false), false)
            .unwrap(),
        1.0
    );
    store.end_render();
    assert!(!store.needs_frame());
    store.begin_render();
    assert_eq!(
        store.sample_number(key(8), 0.0, spec(true), false).unwrap(),
        1.0
    );
    store.end_render();
    assert!(store.needs_frame());
    store.advance(Time::from_nanos(1));
    store.advance(Time::from_nanos(100_000_001));
    store.begin_render();
    assert_eq!(
        store.sample_number(key(8), 0.0, spec(true), false).unwrap(),
        0.0
    );
    store.end_render();
    assert!(!store.needs_frame());
    store.begin_render();
    assert_eq!(
        store
            .sample_number(key(8), 1.0, spec(false), false)
            .unwrap(),
        1.0
    );
    store.end_render();
    assert!(!store.needs_frame());
}

#[test]
fn state_leave_animates_falling_edge_and_in_out_handles_both() {
    for (policy, rise, fall) in [
        (StateTransitionPolicy::Leave, false, true),
        (StateTransitionPolicy::InOut, true, true),
    ] {
        let mut store = PropertyMotionStore::new();
        let spec = |active| {
            PropertyAnimation::new(None, None, 100.0, "once")
                .unwrap()
                .with_state_transition(policy, active, 1.0)
                .unwrap()
        };
        store.begin_render();
        store
            .sample_number(key(9), 1.0, spec(false), false)
            .unwrap();
        store.end_render();
        store.begin_render();
        store.sample_number(key(9), 0.0, spec(true), false).unwrap();
        store.end_render();
        assert_eq!(store.needs_frame(), rise);
        store.advance(Time::from_nanos(1));
        store.advance(Time::from_nanos(100_000_001));
        store.begin_render();
        store.sample_number(key(9), 0.0, spec(true), false).unwrap();
        store.end_render();
        store.begin_render();
        assert_eq!(
            store
                .sample_number(key(9), 1.0, spec(false), false)
                .unwrap(),
            0.0
        );
        store.end_render();
        assert_eq!(store.needs_frame(), fall);
    }
}

#[test]
fn state_transition_respects_reduced_motion_and_validates_canonical_base() {
    let mut store = PropertyMotionStore::new();
    let spec = PropertyAnimation::new(None, None, 100.0, "once")
        .unwrap()
        .with_state_transition(StateTransitionPolicy::InOut, true, 1.0)
        .unwrap();
    store.begin_render();
    assert_eq!(
        store
            .sample_number(key(10), 0.0, spec.clone(), true)
            .unwrap(),
        0.0
    );
    store.end_render();
    assert!(!store.needs_frame());
    assert!(
        store
            .sample_number(
                key(11),
                0.0,
                PropertyAnimation::new(None, None, 100.0, "once")
                    .unwrap()
                    .with_state_transition(StateTransitionPolicy::Enter, true, f64::NAN)
                    .unwrap(),
                false
            )
            .is_err()
    );
    assert!(
        PropertyAnimation::new(Some(0.0), Some(1.0), 100.0, "once")
            .unwrap()
            .with_state_transition(StateTransitionPolicy::Enter, true, 0.0)
            .is_err()
    );
}

#[test]
fn non_selected_state_edge_cancels_an_in_flight_transition() {
    for (policy, active_before_cancel, canceled_target) in [
        (StateTransitionPolicy::Enter, false, 1.0),
        (StateTransitionPolicy::Leave, true, 0.0),
    ] {
        let mut store = PropertyMotionStore::new();
        let spec = |active| {
            PropertyAnimation::new(None, None, 100.0, "once")
                .unwrap()
                .with_state_transition(policy, active, 1.0)
                .unwrap()
        };
        store.begin_render();
        store
            .sample_number(key(12), 1.0, spec(false), false)
            .unwrap();
        store.end_render();
        store.begin_render();
        store
            .sample_number(key(12), 0.0, spec(true), false)
            .unwrap();
        store.end_render();
        if policy == StateTransitionPolicy::Leave {
            store.begin_render();
            store
                .sample_number(key(12), 1.0, spec(false), false)
                .unwrap();
            store.end_render();
        }
        assert!(store.needs_frame());
        store.begin_render();
        assert_eq!(
            store
                .sample_number(key(12), canceled_target, spec(active_before_cancel), false)
                .unwrap(),
            canceled_target
        );
        store.end_render();
        assert!(!store.needs_frame());
    }
}

#[test]
fn advancing_one_frame_updates_every_active_property_slot() {
    let mut store = PropertyMotionStore::new();
    let spec = PropertyAnimation::new(None, None, 100.0, "once").unwrap();
    assert!(store.is_empty());
    store.begin_render();
    store
        .sample_number(key(21), 0.0, spec.clone(), false)
        .unwrap();
    store
        .sample_number(key(22), 0.0, spec.clone(), false)
        .unwrap();
    store.end_render();
    store.begin_render();
    store
        .sample_number(key(21), 10.0, spec.clone(), false)
        .unwrap();
    store
        .sample_number(key(22), 20.0, spec.clone(), false)
        .unwrap();
    store.end_render();
    assert_eq!(store.len(), 2);
    store.advance(Time::from_nanos(1));
    assert!(store.advance(Time::from_nanos(100_000_001)));
    store.begin_render();
    assert_eq!(
        store
            .sample_number(key(21), 10.0, spec.clone(), false)
            .unwrap(),
        10.0
    );
    assert_eq!(
        store.sample_number(key(22), 20.0, spec, false).unwrap(),
        20.0
    );
    store.end_render();
    assert!(!store.needs_frame());
}
