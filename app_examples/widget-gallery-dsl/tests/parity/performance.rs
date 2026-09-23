//! Repeatable CPU-side gallery workloads, without a GPU or wall-clock assertions.

use std::time::{Duration, Instant};

use argui::{core::ScrollDelta, runtime::Render};
use argui_testing::TestApp;

use super::{Main, Point, live_gallery, navigate_to_page};

/// Every demonstration keeps changing in later cycles, and pause stops scheduling.
#[test]
fn animation_lab_loops_and_pauses_in_both_backends() {
    exercise_motion(TestApp::new(Main::new()));
    exercise_motion(live_gallery());
}

/// Samples motion in `app`, checking repeated cycles and retained paused state.
fn exercise_motion<R: Render>(mut app: TestApp<R>) {
    use argui::ui::Role;
    navigate_to_page(&mut app, "Animation lab");
    app.advance(Duration::from_millis(1)).unwrap();
    for _ in 0..3 {
        let baseline = motion_signature(&app);
        let mut changed = std::collections::HashSet::new();
        for _ in 0..12 {
            app.advance(Duration::from_millis(250)).unwrap();
            for (key, value) in motion_signature(&app) {
                if baseline.get(&key) != Some(&value) {
                    changed.insert(key);
                }
            }
        }
        assert_eq!(
            changed.len(),
            baseline.len(),
            "tracks that stopped: {:?}",
            baseline
                .keys()
                .filter(|key| !changed.contains(*key))
                .collect::<Vec<_>>()
        );
    }
    app.get_by_role(Role::Button, "Pause animations")
        .click()
        .unwrap();
    let paused = motion_signature(&app);
    assert!(!app.entity().read(Render::wants_animation_frame));
    app.advance(Duration::from_secs(20)).unwrap();
    assert_motion_matches(&app, &paused);
    app.get_by_role(Role::Button, "Resume animations")
        .click()
        .unwrap();
    app.advance(Duration::from_millis(1)).unwrap();
    assert_motion_matches(&app, &paused);
    app.advance(Duration::from_millis(250)).unwrap();
    assert_ne!(motion_signature(&app), paused);
    navigate_to_page(&mut app, "Empty");
    // The navigation button's hover transition may still be finishing.
    app.advance(Duration::from_millis(1)).unwrap();
    app.advance(Duration::from_secs(1)).unwrap();
    assert!(!app.entity().read(Render::wants_animation_frame));
}

/// Compares `app` with `expected` and reports only the names of drifting tracks.
fn assert_motion_matches<R: Render>(
    app: &TestApp<R>,
    expected: &std::collections::BTreeMap<String, String>,
) {
    let current = motion_signature(app);
    let changed = current
        .iter()
        .filter(|(key, value)| expected.get(*key) != Some(*value))
        .map(|(key, _)| key)
        .collect::<Vec<_>>();
    assert!(changed.is_empty(), "paused tracks drifted: {changed:?}");
}

/// Returns presented style/paint values for the identified motion tracks in `app`.
fn motion_signature<R: Render>(app: &TestApp<R>) -> std::collections::BTreeMap<String, String> {
    let tree = app.rendered_tree();
    let values = tree
        .node_ids()
        .iter()
        .filter_map(|id| tree.element_for(*id))
        .filter_map(|element| {
            let key = element.key.as_ref().filter(|key| key.starts_with("lab_"))?;
            Some((
                key.clone(),
                format!(
                    "{:?}{:?}{:?}{:?}{:?}",
                    element.kind, element.style, element.paint, element.transform, element.layer
                ),
            ))
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(values.len(), 14);
    values
}

/// Runs the same scrolling and frame workload through compiled and live rendering.
#[test]
fn gallery_cpu_workloads() {
    measure_gallery("AOT", TestApp::new(Main::new()));
    measure_gallery("live", live_gallery());
}

/// Measures `app` CPU work with deterministic simulated time and labels it `backend`.
/// Reports durations for comparison, without asserting machine-dependent timing.
fn measure_gallery<R: Render>(backend: &str, mut app: TestApp<R>) {
    let sidebar = app.bounds("sidebar").unwrap();
    let pointer = Point::new(sidebar.origin.x + 30.0, sidebar.origin.y + 70.0);
    let started = Instant::now();
    let cpu = cpu_ticks();
    for index in 0..60 {
        let delta = if index % 20 < 10 { -60.0 } else { 60.0 };
        app.wheel(pointer, ScrollDelta::Pixels(Point::new(0.0, delta)))
            .unwrap();
    }
    report(backend, "sidebar 60 events", started, cpu);
    for page in ["Animation lab", "Damage control"] {
        navigate_to_page(&mut app, page);
        app.advance(Duration::from_millis(16)).unwrap();
        let started = Instant::now();
        let cpu = cpu_ticks();
        for _ in 0..120 {
            app.advance(Duration::from_millis(16)).unwrap();
        }
        report(backend, &format!("{page} 120 frames"), started, cpu);
        assert!(app.entity().read(Render::wants_animation_frame));
    }
}

/// Reports elapsed time and Linux process CPU ticks for a measured `workload`.
/// `started` and `cpu` are readings taken immediately before the workload.
fn report(backend: &str, workload: &str, started: Instant, cpu: Option<u64>) {
    let ticks = cpu.zip(cpu_ticks()).map(|(start, end)| end - start);
    eprintln!(
        "PERF {backend} {workload}: wall={:?}, process_cpu_ticks={ticks:?}",
        started.elapsed()
    );
}

/// Returns cumulative process CPU ticks on Linux, or None on other platforms.
/// Tick duration is the host's `getconf CLK_TCK`; this avoids scheduler wait time.
fn cpu_ticks() -> Option<u64> {
    let stat = std::fs::read_to_string("/proc/self/stat").ok()?;
    let (_, fields) = stat.rsplit_once(") ")?;
    let mut fields = fields.split_whitespace().skip(11);
    Some(fields.next()?.parse::<u64>().ok()? + fields.next()?.parse::<u64>().ok()?)
}
