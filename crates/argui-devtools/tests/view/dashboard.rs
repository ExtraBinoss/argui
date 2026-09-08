use super::*;
use argui_core::Rect;
use argui_runtime::Inspection;

fn find<'a>(root: &'a Element, key: &str) -> Option<&'a Element> {
    if root.key.as_deref() == Some(key) {
        return Some(root);
    }
    root.children.iter().find_map(|child| find(child, key))
}

fn profiled() -> argui_runtime::Mount<DevtoolsHost<StateShowcase>> {
    let host = argui_runtime::Entity::new(DevtoolsHost::new(StateShowcase::default()).open(true))
        .mount()
        .unwrap();
    host.read(|tools| tools.inspector()).record_ui(FrameRecord {
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
    host.update(|tools, cx| {
        cx.notify();
        tools.update(&event("__devtools-profiling"))
    })
    .unwrap();
    host
}

#[test]
fn controls_stay_outside_scroll_regions_and_narrow_panels_switch_views() {
    for width in [420.0, 800.0, 1220.0] {
        let host = profiled();
        host.update(|tools, cx| {
            cx.notify();
            tools.inspect_layout(&LayoutSnapshot {
                viewport: Rect::new(Point::default(), Size::new(width, 700.0)),
                nodes: vec![],
            })
        })
        .unwrap();
        let mut tree = UiTree::new(host.render(Default::default()).unwrap());
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
            host.update(|tools, cx| {
                cx.notify();
                tools.update(&event("__devtools-profile-gpu"))
            })
            .unwrap();
            let gpu = host.render(Default::default()).unwrap();
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
    let host = profiled();
    let graph = find(
        &host.render(Default::default()).unwrap(),
        "__devtools-profile-graph",
    )
    .unwrap()
    .clone();
    let mut tree = UiTree::new(graph);
    host.read(|tools| tools.inspector()).record_ui(FrameRecord {
        interval: std::time::Duration::from_millis(30),
        ..FrameRecord::default()
    });
    host.update(|tools, cx| {
        cx.notify();
        tools.update(&event("__devtools-refresh"))
    })
    .unwrap();
    let next = find(
        &host.render(Default::default()).unwrap(),
        "__devtools-profile-graph",
    )
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
    let host = argui_runtime::Entity::new(DevtoolsHost::new(StateShowcase::default()).open(true))
        .mount()
        .unwrap();
    assert!(!host.read(|tools| tools.inspector()).gpu_profiling());
    host.update(|tools, cx| {
        cx.notify();
        tools.update(&event("__devtools-profiling"))
    })
    .unwrap();
    assert!(host.read(|tools| tools.inspector()).gpu_profiling());
    host.update(|tools, cx| {
        cx.notify();
        tools.update(&event("__devtools-pause"))
    })
    .unwrap();
    assert!(!host.read(|tools| tools.inspector()).gpu_profiling());
    host.update(|tools, cx| {
        cx.notify();
        tools.update(&event("__devtools-pause"))
    })
    .unwrap();
    host.update(|tools, cx| {
        cx.notify();
        tools.update(&event("__devtools-elements"))
    })
    .unwrap();
    assert!(!host.read(|tools| tools.inspector()).gpu_profiling());
    let host = profiled();
    let before = host.render(Default::default()).unwrap();
    host.read(|tools| tools.inspector())
        .record_ui(FrameRecord::default());
    host.update(|tools, cx| {
        cx.notify();
        tools.animation_frame(Frame {
            now: Time::ZERO,
            elapsed: Duration::from_millis(100),
        })
    })
    .unwrap();
    let after = host.render(Default::default()).unwrap();
    assert!(
        find(&before, "__devtools-gpu-timeline")
            .unwrap()
            .ptr_eq(find(&after, "__devtools-gpu-timeline").unwrap())
    );
    host.update(|tools, cx| {
        cx.notify();
        tools.animation_frame(Frame {
            now: Time::ZERO,
            elapsed: Duration::from_millis(500),
        })
    })
    .unwrap();
    assert!(
        find(
            &host.render(Default::default()).unwrap(),
            "__devtools-gpu-timeline"
        )
        .is_none()
    );
}

#[test]
fn profiling_has_one_scrollport_per_pane_and_reserved_scrollbar_space() {
    let host = profiled();
    host.read(|tools| tools.inspector()).record_ui(FrameRecord {
        interval: std::time::Duration::from_millis(999),
        ..host.read(|tools| tools.inspector()).frames()[0].clone()
    });
    host.update(|tools, cx| {
        cx.notify();
        tools.update(&event("__devtools-refresh"))
    })
    .unwrap();
    let mut tree = UiTree::new(host.render(Default::default()).unwrap());
    let mut engine = LayoutEngine::new();
    let mut text = text_engine();
    let mut output = engine
        .compute(&mut tree, &mut text, Size::new(1220.0, 700.0))
        .unwrap();
    let mut settled = false;
    for _ in 0..5 {
        let snapshot = LayoutSnapshot {
            viewport: output.viewport,
            nodes: output
                .nodes
                .iter()
                .map(|node| LayoutBounds {
                    node: node.node,
                    key: tree.key(node.node).map(str::to_owned),
                    bounds: node.bounds,
                })
                .collect(),
        };
        if host
            .update(|tools, cx| {
                cx.notify();
                tools.inspect_layout(&snapshot)
            })
            .unwrap()
            == ViewUpdate::None
        {
            settled = true;
            break;
        }
        tree.update(host.render(Default::default()).unwrap());
        output = engine
            .compute(&mut tree, &mut text, Size::new(1220.0, 700.0))
            .unwrap();
    }
    assert!(settled, "measured header heights must converge");
    assert!(!contains_text(tree.root(), "Live recording"));
    for key in ["__devtools-overview", "__devtools-profile-details-body"] {
        assert!(find(tree.root(), key).unwrap().scroll.is_none());
    }
    let list = find(tree.root(), "__devtools-frames").unwrap();
    assert_eq!(
        list.style.scrollbar_gutter,
        argui_ui::ScrollbarGutter::Stable
    );
    let region = output
        .scroll_regions
        .iter()
        .find(|region| tree.key(region.node) == Some("__devtools-frames"))
        .unwrap();
    let track = region.scrollbar.as_ref().unwrap().vertical.unwrap().track;
    let bounds = |element: &Element| {
        let index = (0..tree.node_ids().len())
            .find(|index| tree.element_at(*index).unwrap().ptr_eq(element))
            .unwrap();
        output
            .nodes
            .iter()
            .find(|node| node.node == tree.node_ids()[index])
            .unwrap()
            .bounds
    };
    let first = find(tree.root(), "__devtools-frame-1").unwrap();
    let second = find(tree.root(), "__devtools-frame-0").unwrap();
    for (a, b) in first.children[0]
        .children
        .iter()
        .zip(&second.children[0].children)
    {
        let a = bounds(a);
        let b = bounds(b);
        assert!((a.origin.x - b.origin.x).abs() < 0.1);
        assert!((a.size.width - b.size.width).abs() < 0.1);
        assert!(a.origin.x + a.size.width <= track.origin.x + 0.1);
    }
    for key in ["__devtools-pause", "__devtools-refresh", "__devtools-clear"] {
        let button = find(tree.root(), key).unwrap();
        let outer = bounds(button);
        let label = bounds(&button.children[0]);
        assert!(
            (outer.origin.x + outer.size.width * 0.5 - label.origin.x - label.size.width * 0.5)
                .abs()
                < 0.6
        );
        assert!(
            (outer.origin.y + outer.size.height * 0.5 - label.origin.y - label.size.height * 0.5)
                .abs()
                < 0.6,
            "{key}: outer {outer:?}, label {label:?}"
        );
    }
    let gpu_region = output
        .scroll_regions
        .iter()
        .find(|region| tree.key(region.node) == Some("__devtools-gpu-passes"))
        .unwrap();
    let gpu_track = gpu_region
        .scrollbar
        .as_ref()
        .unwrap()
        .vertical
        .unwrap()
        .track;
    let gpu_row = find(tree.root(), "__devtools-gpu-pass-0").unwrap();
    let chart = bounds(gpu_row.children.last().unwrap());
    assert!(chart.origin.x + chart.size.width <= gpu_track.origin.x + 0.1);
    fn scrollports(element: &Element) -> usize {
        usize::from(element.scroll.is_some())
            + element.children.iter().map(scrollports).sum::<usize>()
    }
    assert_eq!(
        scrollports(find(tree.root(), "__devtools-profiling-panel").unwrap()),
        2
    );
    let select = find(tree.root(), "__devtools-dock").unwrap();
    fn has_vector(element: &Element) -> bool {
        matches!(element.kind, argui_ui::ElementKind::Vector { .. })
            || element.children.iter().any(has_vector)
    }
    assert!(has_vector(select));
}
