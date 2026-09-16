use argui_example_gpu_canvas::state::LabState;

#[test]
fn revisions_track_scene_mutations_and_pause_stops_animation_work() {
    let mut state = LabState::default();
    let initial = state.revision();
    assert!(state.advance(0.016));
    assert_eq!(state.revision(), initial + 1);

    state.pan_by(12.0, -4.0);
    state.zoom_by(2.0);
    assert_eq!(state.pan_offset(), [0.0, 0.0]);
    assert_eq!(state.zoom_factor(), 1.0);
    assert_eq!(state.target_zoom_factor(), 2.0);
    assert!(state.view_is_settling());
    assert!(state.advance(0.016));
    assert!(state.pan_offset()[0] > 0.0 && state.pan_offset()[0] < 12.0);
    assert!(state.zoom_factor() > 1.0 && state.zoom_factor() < 2.0);
    settle(&mut state);

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
    assert_eq!(state.target_zoom_factor(), 8.0);
    state.zoom_by(0.0001);
    assert_eq!(state.target_zoom_factor(), 0.2);
    assert!(state.toggle_error());
    assert!(state.force_error());
    state.pan_by(9.0, 8.0);
    settle(&mut state);
    state.reset_view();
    assert!(state.view_is_settling());
    settle(&mut state);
    assert_eq!(state.pan_offset(), [0.0, 0.0]);
    assert_eq!(state.zoom_factor(), 1.0);
    assert!(state.force_error());
}

#[test]
fn paused_scene_still_settles_camera_motion() {
    let mut state = LabState::default();
    assert!(state.toggle_paused());
    state.pan_by(40.0, -12.0);
    state.zoom_by(1.8);
    assert!(state.view_is_settling());
    assert!(state.advance(0.016));
    settle(&mut state);
    assert_eq!(state.pan_offset(), [40.0, -12.0]);
    assert_eq!(state.zoom_factor(), 1.8);
    assert!(!state.advance(0.016));
}

fn settle(state: &mut LabState) {
    for _ in 0..120 {
        if !state.view_is_settling() {
            return;
        }
        assert!(state.advance(0.016));
    }
    panic!("camera did not settle");
}
