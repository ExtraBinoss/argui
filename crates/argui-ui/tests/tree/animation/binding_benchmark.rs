use argui_animation::{Duration, Motion, Time, Tween};
use argui_ui::{Element, UiTree, property};
use web_time::Instant;

/// Measures registry scans with many idle bindings and one externally started track.
/// Run explicitly with nextest's ignored-test and output flags; timings are not assertions.
#[test]
#[ignore]
fn binding_registry_scan_cost() {
    const SAMPLES: u32 = 1_000;
    for count in [128, 512, 2_048, 8_192] {
        let motions: Vec<_> = (0..count).map(|_| Motion::new(1.0_f32)).collect();
        let mut tree =
            UiTree::new(Element::row(motions.iter().cloned().map(|motion| {
                Element::container([]).bind(property::LayerOpacity, motion)
            })));

        let started = Instant::now();
        for _ in 0..SAMPLES {
            std::hint::black_box(tree.wants_animation_frame());
        }
        let idle_demand = started.elapsed() / SAMPLES;

        let started = Instant::now();
        for _ in 0..SAMPLES {
            std::hint::black_box(tree.next_animation_frame_at());
        }
        let idle_deadline = started.elapsed() / SAMPLES;

        motions
            .last()
            .unwrap()
            .animate_to(0.0, Tween::new(Duration::from_secs(10)));
        tree.advance_animations(Time::ZERO);
        let started = Instant::now();
        for _ in 0..SAMPLES {
            std::hint::black_box(tree.advance_animations(Time::from_nanos(1)));
        }
        let active_sample = started.elapsed() / SAMPLES;

        eprintln!(
            "{count} bindings: idle demand {idle_demand:?}, idle deadline {idle_deadline:?}, one-active sample {active_sample:?}"
        );
    }
}
