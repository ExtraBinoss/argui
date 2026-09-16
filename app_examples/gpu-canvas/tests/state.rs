use argui_example_gpu_canvas::state::LabState;

#[test]
fn revisions_track_scene_mutations_and_pause_stops_animation_work() {
    let mut state = LabState::default();
    let initial = state.revision();
    assert!(state.advance(0.016));
    assert_eq!(state.revision(), initial + 1);

    state.pan_by(12.0, -4.0);
    state.zoom_by(2.0);
    assert_eq!(state.pan_offset(), [12.0, -4.0]);
    assert_eq!(state.zoom_factor(), 2.0);

    assert!(state.toggle_paused());
    let paused_revision = state.revision();
    assert!(!state.advance(1.0));
    assert_eq!(state.revision(), paused_revision);
    assert!(!state.toggle_paused());
    assert!(state.advance(0.016));
}

#[test]
fn zoom_reset_and_error_controls_are_bounded_and_explicit() {
    let mut state = LabState::default();
    state.zoom_by(100.0);
    assert_eq!(state.zoom_factor(), 8.0);
    state.zoom_by(0.0001);
    assert_eq!(state.zoom_factor(), 0.2);
    assert!(state.toggle_error());
    assert!(state.force_error());
    state.pan_by(9.0, 8.0);
    state.reset_view();
    assert_eq!(state.pan_offset(), [0.0, 0.0]);
    assert_eq!(state.zoom_factor(), 1.0);
    assert!(state.force_error());
}
