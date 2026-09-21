use argui_animation::Time;
use argui_core::Color;
use argui_schema::{
    PropertyAnimation, PropertyMotionKey, PropertyMotionStore, StateTransitionPolicy,
};
use argui_ui::{Dimension, RetainedIdentity};

#[test]
fn animation_timing_and_repetition_reject_invalid_inputs() {
    for duration in [0.0, -1.0, f64::NAN, f64::INFINITY, 1.0e20] {
        assert!(PropertyAnimation::new(None::<f64>, None, duration, "once").is_err());
    }
    assert!(PropertyAnimation::new(None::<f64>, None, 100.0, "infinite").is_err());
    assert!(PropertyAnimation::new(Some(0.0), None, 100.0, "infinite").is_err());
    assert!(PropertyAnimation::new(Some(0.0), Some(1.0), 100.0, "twice").is_err());
    assert!(PropertyAnimation::keyframes(vec![(0.0, 0.0), (1.0, 1.0)], 100.0, "twice").is_err());
    assert!(PropertyAnimation::spring(None::<f64>, None, 0.0, 1.0).is_err());
    assert!(PropertyAnimation::spring(None::<f64>, None, 100.0, -1.0).is_err());
}

#[test]
fn keyframes_reject_nonfinite_out_of_range_and_conflicting_endpoints() {
    let base = PropertyAnimation::new(None::<f64>, None, 100.0, "once").unwrap();
    for frames in [
        vec![(0.0, 0.0)],
        vec![(0.0, 0.0), (f32::NAN, 0.5), (1.0, 1.0)],
        vec![(0.0, 0.0), (1.1, 0.5), (1.0, 1.0)],
        vec![(-0.1, 0.0), (1.0, 1.0)],
        vec![(0.0, 0.0), (0.8, 0.5), (0.7, 1.0), (1.0, 2.0)],
    ] {
        assert!(base.clone().with_keyframes(frames).is_err());
    }
    assert!(
        PropertyAnimation::new(Some(0.0), None, 100.0, "once")
            .unwrap()
            .with_keyframes(vec![(0.0, 0.0), (1.0, 1.0)])
            .is_err()
    );
    assert!(
        PropertyAnimation::new(None, Some(1.0), 100.0, "once")
            .unwrap()
            .with_keyframes(vec![(0.0, 0.0), (1.0, 1.0)])
            .is_err()
    );
}

#[test]
fn named_easing_accepts_all_supported_curves_but_not_springs() {
    let base = PropertyAnimation::new(None::<f64>, None, 100.0, "once").unwrap();
    for name in ["linear", "ease", "ease-in", "ease-out", "ease-in-out"] {
        assert!(base.clone().with_easing_name(name).is_ok());
    }
    assert!(
        PropertyAnimation::spring(None::<f64>, None, 120.0, 20.0)
            .unwrap()
            .with_easing_name("ease")
            .is_err()
    );
}

#[test]
fn state_transitions_reject_keyframes_and_infinite_timelines() {
    assert!(
        PropertyAnimation::keyframes(vec![(0.0, 0.0), (1.0, 1.0)], 100.0, "once")
            .unwrap()
            .with_state_transition(StateTransitionPolicy::Enter, true, 0.0)
            .is_err()
    );
    assert!(
        PropertyAnimation::new(Some(0.0), Some(1.0), 100.0, "infinite")
            .unwrap()
            .with_state_transition(StateTransitionPolicy::Leave, false, 0.0)
            .is_err()
    );
}

#[test]
fn retained_slot_can_change_color_and_dimension_types_without_stale_motion() {
    let key = PropertyMotionKey::new(RetainedIdentity::new(1, 2), 3, 4);
    let mut store = PropertyMotionStore::new();
    let color = Color::BLACK;
    store.begin_render();
    store
        .sample_color(
            key.clone(),
            color,
            PropertyAnimation::new(None, None, 100.0, "once").unwrap(),
            false,
        )
        .unwrap();
    store.end_render();
    store.begin_render();
    let width = store
        .sample_dimension(
            key.clone(),
            Dimension::length(15.0),
            PropertyAnimation::new(None, None, 100.0, "once").unwrap(),
            false,
        )
        .unwrap();
    store.end_render();
    assert_eq!(width, Dimension::length(15.0));
    assert!(!store.needs_frame());
    assert_eq!(store.len(), 1);
    assert!(!store.advance(Time::from_nanos(1)));
}

#[test]
fn numeric_motion_rejects_nonfinite_targets_endpoints_and_keyframes() {
    let key = PropertyMotionKey::new(RetainedIdentity::new(9, 2), 3, 4);
    let mut store = PropertyMotionStore::new();
    let ordinary = PropertyAnimation::new(None, None, 100.0, "once").unwrap();
    assert!(
        store
            .sample_number(key.clone(), f64::NAN, ordinary, false)
            .is_err()
    );
    let endpoint = PropertyAnimation::new(Some(f64::INFINITY), Some(1.0), 100.0, "once").unwrap();
    assert!(
        store
            .sample_number(key.clone(), 1.0, endpoint, false)
            .is_err()
    );
    let frames = PropertyAnimation::keyframes(
        vec![(0.0, 0.0), (0.5, f64::NEG_INFINITY), (1.0, 1.0)],
        100.0,
        "once",
    )
    .unwrap();
    assert!(store.sample_number(key, 1.0, frames, false).is_err());
    assert!(store.is_empty());
}

#[test]
fn dimensions_reject_nonfinite_values_and_unit_changes_in_keyframes() {
    let key = PropertyMotionKey::new(RetainedIdentity::new(9, 3), 3, 4);
    let mut store = PropertyMotionStore::new();
    assert!(
        store
            .sample_dimension(
                key.clone(),
                Dimension::length(f32::INFINITY),
                PropertyAnimation::new(None, None, 100.0, "once").unwrap(),
                false,
            )
            .is_err()
    );
    let frames = PropertyAnimation::keyframes(
        vec![
            (0.0, Dimension::length(0.0)),
            (1.0, Dimension::percent(1.0)),
        ],
        100.0,
        "once",
    )
    .unwrap();
    assert!(
        store
            .sample_dimension(key, Dimension::length(1.0), frames, false)
            .is_err()
    );
    assert!(store.is_empty());
}

#[test]
fn active_state_spring_starts_from_canonical_base() {
    let key = PropertyMotionKey::new(RetainedIdentity::new(9, 4), 3, 4);
    let mut store = PropertyMotionStore::new();
    let spec = PropertyAnimation::spring(None, None, 220.0, 24.0)
        .unwrap()
        .with_state_transition(StateTransitionPolicy::Enter, true, 1.0)
        .unwrap();
    store.begin_render();
    assert_eq!(
        store
            .sample_number(key.clone(), 0.0, spec.clone(), false)
            .unwrap(),
        1.0
    );
    store.end_render();
    assert!(store.needs_frame());
    store.advance(Time::from_nanos(1));
    store.advance(Time::from_nanos(50_000_001));
    store.begin_render();
    let sampled = store.sample_number(key, 0.0, spec, false).unwrap();
    store.end_render();
    assert!((0.0..1.0).contains(&sampled));
}

#[test]
fn color_and_dimension_keyframes_advance_through_typed_retained_slots() {
    let mut store = PropertyMotionStore::new();
    let color_key = PropertyMotionKey::new(RetainedIdentity::new(10, 1), 2, 3);
    let width_key = PropertyMotionKey::new(RetainedIdentity::new(10, 1), 4, 5);
    let red = Color::from_srgba8(255, 0, 0, 255);
    let blue = Color::from_srgba8(0, 0, 255, 255);
    let color_spec =
        PropertyAnimation::keyframes(vec![(0.0, red), (0.5, blue), (1.0, red)], 100.0, "once")
            .unwrap();
    let width_spec = PropertyAnimation::keyframes(
        vec![
            (0.0, Dimension::length(10.0)),
            (0.5, Dimension::length(30.0)),
            (1.0, Dimension::length(10.0)),
        ],
        100.0,
        "once",
    )
    .unwrap();
    store.begin_render();
    assert_eq!(
        store
            .sample_color(color_key.clone(), red, color_spec.clone(), false)
            .unwrap(),
        red
    );
    assert_eq!(
        store
            .sample_dimension(
                width_key.clone(),
                Dimension::length(10.0),
                width_spec.clone(),
                false
            )
            .unwrap(),
        Dimension::length(10.0)
    );
    store.end_render();
    assert_eq!(store.len(), 2);
    store.advance(Time::from_nanos(1));
    assert!(store.advance(Time::from_nanos(50_000_001)));
    store.begin_render();
    let sampled_color = store
        .sample_color(color_key, red, color_spec, false)
        .unwrap();
    for (actual, expected) in sampled_color
        .to_linear_rgba()
        .into_iter()
        .zip(blue.to_linear_rgba())
    {
        assert!((actual - expected).abs() < 1e-6);
    }
    assert_eq!(
        store
            .sample_dimension(width_key, Dimension::length(10.0), width_spec, false)
            .unwrap(),
        Dimension::length(30.0)
    );
    store.end_render();
}

#[test]
fn color_and_dimension_state_transitions_start_on_enter() {
    let mut store = PropertyMotionStore::new();
    let color_key = PropertyMotionKey::new(RetainedIdentity::new(10, 2), 2, 3);
    let width_key = PropertyMotionKey::new(RetainedIdentity::new(10, 2), 4, 5);
    let red = Color::from_srgba8(255, 0, 0, 255);
    let blue = Color::from_srgba8(0, 0, 255, 255);
    let color_spec = PropertyAnimation::new(None, None, 100.0, "once")
        .unwrap()
        .with_state_transition(StateTransitionPolicy::Enter, true, red)
        .unwrap();
    let width_spec = PropertyAnimation::new(None, None, 100.0, "once")
        .unwrap()
        .with_state_transition(StateTransitionPolicy::Enter, true, Dimension::length(10.0))
        .unwrap();
    store.begin_render();
    assert_eq!(
        store
            .sample_color(color_key, blue, color_spec, false)
            .unwrap(),
        red
    );
    assert_eq!(
        store
            .sample_dimension(width_key, Dimension::length(30.0), width_spec, false)
            .unwrap(),
        Dimension::length(10.0)
    );
    store.end_render();
    assert!(store.needs_frame());
}

#[test]
fn explicit_spring_endpoints_support_color_and_dimensions() {
    let mut store = PropertyMotionStore::new();
    let color_key = PropertyMotionKey::new(RetainedIdentity::new(10, 3), 2, 3);
    let width_key = PropertyMotionKey::new(RetainedIdentity::new(10, 3), 4, 5);
    let red = Color::from_srgba8(255, 0, 0, 255);
    let blue = Color::from_srgba8(0, 0, 255, 255);
    let color_spec = PropertyAnimation::spring(Some(red), Some(blue), 200.0, 20.0).unwrap();
    let width_spec = PropertyAnimation::spring(
        Some(Dimension::percent(0.1)),
        Some(Dimension::percent(0.8)),
        200.0,
        20.0,
    )
    .unwrap();
    store.begin_render();
    assert_eq!(
        store
            .sample_color(color_key.clone(), blue, color_spec.clone(), false)
            .unwrap(),
        red
    );
    assert_eq!(
        store
            .sample_dimension(
                width_key.clone(),
                Dimension::percent(0.8),
                width_spec.clone(),
                false
            )
            .unwrap(),
        Dimension::percent(0.1)
    );
    store.end_render();
    store.advance(Time::from_nanos(1));
    assert!(store.advance(Time::from_nanos(50_000_001)));
    store.begin_render();
    let sampled_color = store
        .sample_color(color_key, blue, color_spec, false)
        .unwrap();
    let sampled_width = store
        .sample_dimension(width_key, Dimension::percent(0.8), width_spec, false)
        .unwrap();
    store.end_render();
    assert_ne!(sampled_color, red);
    assert_ne!(sampled_width, Dimension::percent(0.1));
}
