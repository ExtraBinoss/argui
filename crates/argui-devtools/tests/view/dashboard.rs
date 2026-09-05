use super::*;
use argui_core::Rect;
use argui_runtime::Inspection;

fn find<'a>(root: &'a Element, key: &str) -> Option<&'a Element> {
    if root.key.as_deref() == Some(key) {
        return Some(root);
    }
    root.children.iter().find_map(|child| find(child, key))
}

fn profiled() -> DevtoolsHost<StateShowcase> {
    let mut host = DevtoolsHost::new(StateShowcase::default()).open(true);
    host.inspector().record_ui(FrameRecord {
        interval: std::time::Duration::from_millis(10),
        gpu: Some(GpuFrameRecord {
            total: std::time::Duration::from_millis(4),
            passes: (0..40)
                .map(|index| GpuPassRecord {
                    label: format!("layer.content.{index}"),
                    start: std::time::Duration::from_micros(index * 100),
                    duration: std::time::Duration::from_micros(80),
                    ..GpuPassRecord::default()
                })
                .collect(),
            ..GpuFrameRecord::default()
        }),
        ..FrameRecord::default()
    });
    host.update(&event("__devtools-profiling"));
    host
}

#[test]
fn controls_stay_outside_scroll_regions_and_narrow_panels_switch_views() {
    for width in [420.0, 800.0, 1220.0] {
        let mut host = profiled();
        host.layout_changed(&LayoutSnapshot {
            viewport: Rect::new(Point::default(), Size::new(width, 700.0)),
            nodes: vec![],
        });
        let mut tree = UiTree::new(host.view());
        let output = LayoutEngine::new()
            .compute(&mut tree, &mut text_engine(), Size::new(width, 700.0))
            .unwrap();
        let snapshot = Inspection::snapshot(&tree, &output);
        assert!(
            !snapshot
                .nodes
                .iter()
                .any(|node| node.key.as_deref() == Some("__devtools-profile-controls"))
        );
        let bounds = |key| {
            output
                .nodes
                .iter()
                .find(|node| tree.key(node.node) == Some(key))
                .unwrap()
                .bounds
        };
        let controls = bounds("__devtools-profile-controls");
        let overview = bounds("__devtools-overview");
        assert!(controls.origin.y + controls.size.height <= overview.origin.y);
        assert!(overview.size.width <= width);
        assert!(overview.size.height > 0.0);
        if width < 760.0 {
            assert!(find(tree.root(), "__devtools-gpu-timeline").is_none());
            host.update(&event("__devtools-profile-gpu"));
            let gpu = host.view();
            assert!(find(&gpu, "__devtools-profile-graph").is_none());
            assert!(find(&gpu, "__devtools-gpu-timeline").is_some());
        } else {
            let details = bounds("__devtools-profile-details-body");
            assert!(overview.origin.x + overview.size.width <= details.origin.x + 0.1);
            let passes = bounds("__devtools-gpu-passes");
            assert!(
                passes.origin.y + passes.size.height
                    <= details.origin.y + details.size.height + 0.1,
                "width {width}: passes {passes:?}; details {details:?}"
            );
        }
    }
}

#[test]
fn graph_updates_animate_paint_without_reflowing_the_plot() {
    let mut host = profiled();
    let graph = find(&host.view(), "__devtools-profile-graph")
        .unwrap()
        .clone();
    let mut tree = UiTree::new(graph);
    host.inspector().record_ui(FrameRecord {
        interval: std::time::Duration::from_millis(30),
        ..FrameRecord::default()
    });
    host.update(&event("__devtools-refresh"));
    let next = find(&host.view(), "__devtools-profile-graph")
        .unwrap()
        .clone();
    assert_eq!(tree.update(next), argui_ui::TreeUpdate::Paint);
    let node = tree
        .node_ids()
        .iter()
        .copied()
        .find(|node| tree.key(*node) == Some("__devtools-graph-bar-59"))
        .unwrap();
    let element = find(tree.root(), "__devtools-graph-bar-59")
        .unwrap()
        .clone();
    tree.advance_animations(Time::from_nanos(1));
    tree.advance_animations(Time::from_nanos(60_000_001));
    let scale = tree.resolved_transform(node, &element).scale.y;
    assert!(scale > 0.2 && scale < 0.6, "intermediate scale {scale}");
    tree.advance_animations(Time::from_nanos(200_000_001));
    assert!(!tree.wants_animation_frame());
}

#[test]
fn gpu_measurements_are_opt_in_and_details_are_reused_between_samples() {
    let mut host = DevtoolsHost::new(StateShowcase::default()).open(true);
    assert!(!host.inspector().gpu_profiling());
    host.update(&event("__devtools-profiling"));
    assert!(host.inspector().gpu_profiling());
    host.update(&event("__devtools-pause"));
    assert!(!host.inspector().gpu_profiling());
    host.update(&event("__devtools-pause"));
    host.update(&event("__devtools-elements"));
    assert!(!host.inspector().gpu_profiling());
    let mut host = profiled();
    let before = host.view();
    host.inspector().record_ui(FrameRecord::default());
    host.animation_frame(Frame {
        now: Time::ZERO,
        elapsed: Duration::from_millis(100),
    });
    let after = host.view();
    assert!(
        find(&before, "__devtools-gpu-timeline")
            .unwrap()
            .ptr_eq(find(&after, "__devtools-gpu-timeline").unwrap())
    );
    host.animation_frame(Frame {
        now: Time::ZERO,
        elapsed: Duration::from_millis(500),
    });
    assert!(find(&host.view(), "__devtools-gpu-timeline").is_none());
}
