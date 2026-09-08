//! CPU-only retained UI workload; does not measure GPU query or presentation cost.
use argui_animation::{Duration, Frame, Time};
use argui_core::Size;
use argui_devtools::DevtoolsHost;
use argui_inspect::{FrameRecord, GpuFrameRecord, GpuPassRecord};
use argui_layout::LayoutEngine;
use argui_runtime::{InspectionCache, ViewUpdate};
use argui_showcase::{StateShowcase, text_engine};
use argui_ui::{ClickEvent, Element, TreeUpdate, UiEvent, UiEventKind, UiTree};
use std::time::Duration as StdDuration;
use web_time::Instant;

fn main() {
    let resizing = std::env::args().any(|arg| arg == "--resize");
    let effects = !std::env::args().any(|arg| arg == "--no-scroll-effects");
    for (label, open, profiling) in [
        ("Closed", false, false),
        ("Elements", true, false),
        ("Profiling", true, true),
    ] {
        let host = argui_runtime::Entity::new(
            DevtoolsHost::new(StateShowcase::default())
                .open(open)
                .scroll_effect(effects.then(|| argui_effects::EdgeFade::default().scroll())),
        )
        .mount()
        .unwrap();
        let inspector = host.read(|tools| tools.inspector());
        let mut record = FrameRecord {
            interval: StdDuration::from_micros(16_667),
            gpu: Some(GpuFrameRecord {
                total: StdDuration::from_millis(8),
                passes: (0..80)
                    .map(|index| GpuPassRecord {
                        label: format!("layer.content.{index}"),
                        start: StdDuration::from_micros(index * 100),
                        duration: StdDuration::from_micros(80),
                        ..GpuPassRecord::default()
                    })
                    .collect(),
                ..GpuFrameRecord::default()
            }),
            ..FrameRecord::default()
        };
        if profiling {
            let target = UiTree::new(Element::container([])).node_ids()[0];
            host.update(|tools, cx| {
                let update = tools.update(&UiEvent::new(
                    target,
                    Some("__devtools-profiling".into()),
                    UiEventKind::Click(ClickEvent::accessibility()),
                ));
                if update == ViewUpdate::Rebuild {
                    cx.notify();
                }
                update
            })
            .unwrap();
        }
        let mut tree = UiTree::new(host.render(Default::default()).unwrap());
        let mut engine = LayoutEngine::new();
        let mut text = text_engine();
        let mut cache = InspectionCache::default();
        let mut output = engine
            .compute(&mut tree, &mut text, Size::new(1220.0, 780.0))
            .unwrap();
        let target = tree.node_ids()[0];
        let resize_event = |phase, offset| {
            UiEvent::new(
                target,
                Some("__devtools-splitter".into()),
                UiEventKind::Gesture(argui_ui::GestureEvent {
                    target,
                    pointer: argui_core::PointerId::MOUSE,
                    phase,
                    delivery: argui_ui::GestureDelivery::FrameCoalesced,
                    kind: argui_ui::GestureKind::Pan {
                        position: argui_core::Point::default(),
                        delta: argui_core::Point::default(),
                        total: argui_core::Point::new(0.0, offset),
                        velocity: argui_core::Point::default(),
                    },
                }),
            )
        };
        if resizing && open {
            host.update(|tools, cx| {
                let update = tools.update(&resize_event(argui_ui::GesturePhase::Started, 0.0));
                if update == ViewUpdate::Rebuild {
                    cx.notify();
                }
                update
            })
            .unwrap();
        }
        let mut samples = Vec::new();
        let mut model_samples = Vec::new();
        let mut layout_samples = Vec::new();
        for index in 0..240 {
            record.interval = StdDuration::from_micros(14_000 + (index % 9) * 700);
            if let Some(gpu) = &mut record.gpu {
                gpu.passes[0].duration = StdDuration::from_micros(60 + index % 30);
            }
            inspector.record_ui(record.clone());
            let start = Instant::now();
            let resize_update = if resizing && open {
                host.update(|tools, cx| {
                    let update = tools.update(&resize_event(
                        argui_ui::GesturePhase::Changed,
                        -((index as f32 * 0.05).sin() * 0.5 + 0.5) * 400.0,
                    ));
                    if update == ViewUpdate::Rebuild {
                        cx.notify();
                    }
                    update
                })
                .unwrap()
            } else {
                ViewUpdate::None
            };
            let model_update = host
                .update(|tools, cx| {
                    let update = tools.animation_frame(Frame {
                        now: Time::from_nanos(index * 16_667_000),
                        elapsed: Duration::from_millis(17),
                    });
                    if update == ViewUpdate::Rebuild {
                        cx.notify();
                    }
                    update
                })
                .unwrap();
            let change =
                if model_update == ViewUpdate::Rebuild || resize_update == ViewUpdate::Rebuild {
                    tree.update(host.render(Default::default()).unwrap())
                } else {
                    TreeUpdate::None
                };
            let animated = tree.advance_animations(Time::from_nanos(index * 16_667_000));
            let model_ms = start.elapsed().as_secs_f64() * 1000.0;
            let layout_start = Instant::now();
            if change == TreeUpdate::Layout || animated == TreeUpdate::Layout {
                output = engine
                    .compute(&mut tree, &mut text, Size::new(1220.0, 780.0))
                    .unwrap();
            } else {
                engine.repaint(&tree, &mut output);
            }
            let snapshot = argui_runtime::LayoutSnapshot {
                viewport: output.viewport,
                nodes: output
                    .nodes
                    .iter()
                    .map(|node| argui_runtime::LayoutBounds {
                        node: node.node,
                        key: tree.key(node.node).map(str::to_owned),
                        bounds: node.bounds,
                    })
                    .collect(),
            };
            if host
                .update(|tools, cx| {
                    let update = tools.inspect_layout(&snapshot);
                    if update == ViewUpdate::Rebuild {
                        cx.notify();
                    }
                    update
                })
                .unwrap()
                == ViewUpdate::Rebuild
            {
                tree.update(host.render(Default::default()).unwrap());
                output = engine
                    .compute(&mut tree, &mut text, Size::new(1220.0, 780.0))
                    .unwrap();
            }
            let layout_ms = layout_start.elapsed().as_secs_f64() * 1000.0;
            if inspector.enabled()
                && let Some(snapshot) = cache.snapshot(&tree, &output)
            {
                inspector.publish_tree(snapshot);
            }
            if index >= 40 {
                samples.push(start.elapsed().as_secs_f64() * 1000.0);
                model_samples.push(model_ms);
                layout_samples.push(layout_ms);
            }
        }
        samples.sort_by(f64::total_cmp);
        model_samples.sort_by(f64::total_cmp);
        layout_samples.sort_by(f64::total_cmp);
        println!(
            "{label}: model p95={:.3} ms · layout/paint p95={:.3} ms",
            model_samples[190], layout_samples[190]
        );
        println!(
            "{label}: CPU p50={:.3} ms p95={:.3} ms p99={:.3} ms",
            samples[100], samples[190], samples[198]
        );
    }
}
