use argui_animation::{Duration, Frame, Time};
use argui_core::{Insets, Point, Size};
use argui_devtools::DevtoolsHost;
use argui_inspect::{AdapterRecord, FrameRecord, GpuFrameRecord, GpuPassRecord};
use argui_layout::LayoutEngine;
use argui_runtime::{LayoutBounds, LayoutSnapshot, ViewUpdate, WindowEnvironment};
use argui_showcase::{StateShowcase, text_engine};
use argui_ui::{Element, UiEvent, UiEventKind, UiTree};

#[path = "view/dashboard.rs"]
mod dashboard;
#[path = "view/toolbar.rs"]
mod toolbar;

fn event(key: &str) -> UiEvent {
    let tree = UiTree::new(Element::container([]));
    UiEvent::new(
        tree.node_id_at(0).unwrap(),
        Some(key.to_owned()),
        UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
    )
}

#[test]
fn opening_tools_survives_the_transient_zero_height_application_viewport() {
    let host = argui_runtime::Entity::new(DevtoolsHost::new(StateShowcase::default()))
        .mount()
        .unwrap();
    assert_eq!(
        host.update(|tools, cx| {
            cx.notify();
            tools.update(&event("__devtools-toggle"))
        })
        .unwrap(),
        ViewUpdate::Rebuild
    );
    let mut tree = UiTree::new(host.render(Default::default()).unwrap());
    let mut layout = LayoutEngine::new();
    let output = layout
        .compute(&mut tree, &mut text_engine(), Size::new(1_100.0, 230.0))
        .unwrap();
    let snapshot = LayoutSnapshot {
        viewport: output.viewport,
        nodes: output
            .nodes
            .iter()
            .map(|layout| LayoutBounds {
                node: layout.node,
                key: tree.key(layout.node).map(str::to_owned),
                bounds: layout.bounds,
            })
            .collect(),
    };

    assert_eq!(
        host.update(|tools, cx| {
            cx.notify();
            tools.inspect_layout(&snapshot)
        })
        .unwrap(),
        ViewUpdate::None
    );
}

#[test]
fn toggle_button_keeps_its_authored_radius_while_pressed() {
    let host = argui_runtime::Entity::new(DevtoolsHost::new(StateShowcase::default()))
        .mount()
        .unwrap();
    let mut tree = UiTree::new(host.render(Default::default()).unwrap());
    let mut layout = LayoutEngine::new();
    let output = layout
        .compute(&mut tree, &mut text_engine(), Size::new(1_100.0, 700.0))
        .unwrap();
    let (index, node) = tree
        .node_ids()
        .iter()
        .copied()
        .enumerate()
        .find(|(_, node)| tree.key(*node) == Some("__devtools-toggle"))
        .unwrap();
    let initial = tree
        .resolved_quad(node, tree.element_at(index).unwrap())
        .radii;
    let hit = output
        .hit_regions
        .iter()
        .find(|region| region.node == node)
        .unwrap();
    let local = Point::new(
        hit.bounds.origin.x + hit.bounds.size.width * 0.5,
        hit.bounds.origin.y + hit.bounds.size.height * 0.5,
    );
    let point = hit.transform.transform_point(local);

    tree.pointer_moved(point, &output.hit_regions);
    tree.primary_pressed(&output.hit_regions);
    tree.set_reduced_motion(true);

    assert_eq!(
        tree.resolved_quad(node, tree.element_at(index).unwrap())
            .radii,
        initial
    );
}

#[test]
fn open_dock_reserves_application_viewport_space() {
    let host = argui_runtime::Entity::new(DevtoolsHost::new(StateShowcase::default()).open(true))
        .mount()
        .unwrap();
    let mut tree = UiTree::new(
        host.render(WindowEnvironment {
            safe_area_insets: Insets::new(44.0, 12.0, 24.0, 8.0),
            ..WindowEnvironment::default()
        })
        .unwrap(),
    );
    let mut layout = LayoutEngine::new();
    let output = layout
        .compute(&mut tree, &mut text_engine(), Size::new(1_100.0, 700.0))
        .unwrap();
    let keyed = |key: &str| {
        tree.node_ids()
            .iter()
            .copied()
            .enumerate()
            .find_map(|(index, node)| {
                (tree.key(node) == Some(key)).then(|| output.nodes[index].bounds)
            })
            .unwrap()
    };
    let application = keyed("__devtools-app-root");
    let surface = keyed("__devtools-surface");
    let splitter = keyed("__devtools-splitter");
    let (surface_index, surface_node) = tree
        .node_ids()
        .iter()
        .copied()
        .enumerate()
        .find(|(_, node)| tree.key(*node) == Some("__devtools-surface"))
        .unwrap();
    assert!(application.size.height < output.viewport.size.height);
    assert_eq!(
        surface.origin.y + surface.size.height,
        output.viewport.size.height
    );
    assert_eq!(splitter.origin.y, surface.origin.y);
    assert!(
        tree.resolved_quad(surface_node, tree.element_at(surface_index).unwrap())
            .background
            .is_some()
    );
}

#[test]
fn closed_dock_keeps_a_visible_overlay_button_without_stealing_app_height() {
    let host = argui_runtime::Entity::new(DevtoolsHost::new(StateShowcase::default()))
        .mount()
        .unwrap();
    let mut tree = UiTree::new(host.render(Default::default()).unwrap());
    let mut layout = LayoutEngine::new();
    let output = layout
        .compute(&mut tree, &mut text_engine(), Size::new(1_100.0, 700.0))
        .unwrap();
    let bounds = |key: &str| {
        tree.node_ids()
            .iter()
            .copied()
            .enumerate()
            .find_map(|(index, node)| {
                (tree.key(node) == Some(key)).then(|| output.nodes[index].bounds)
            })
            .unwrap()
    };
    let application = bounds("__devtools-app-root");
    let surface = bounds("__devtools-surface");
    let toggle = bounds("__devtools-toggle");
    assert_eq!(application.size.height, 700.0);
    assert_eq!(surface.size.height, 0.0);
    assert!(toggle.origin.x >= 0.0);
    assert!(toggle.origin.y >= 0.0);
    assert!(toggle.origin.x + toggle.size.width <= 1_100.0);
    assert!(toggle.origin.y + toggle.size.height <= 700.0);
    assert!((1_100.0 - toggle.origin.x - toggle.size.width - 14.0).abs() < 0.01);
}

#[test]
fn devtools_overlay_respects_native_safe_area_insets() {
    let host = argui_runtime::Entity::new(DevtoolsHost::new(StateShowcase::default()))
        .mount()
        .unwrap();
    let insets = Insets::new(44.0, 12.0, 24.0, 8.0);
    let mut tree = UiTree::new(
        host.render(WindowEnvironment {
            safe_area_insets: insets,
            ..WindowEnvironment::default()
        })
        .unwrap(),
    );
    let output = LayoutEngine::new()
        .compute(&mut tree, &mut text_engine(), Size::new(390.0, 844.0))
        .unwrap();
    let bounds = |key: &str| {
        tree.node_ids()
            .iter()
            .copied()
            .enumerate()
            .find_map(|(index, node)| {
                (tree.key(node) == Some(key)).then(|| output.nodes[index].bounds)
            })
            .unwrap()
    };
    let toggle = bounds("__devtools-toggle");
    let application = bounds("__devtools-app-root");
    let surface = bounds("__devtools-surface");

    assert!((toggle.origin.y - insets.top - 14.0).abs() < 0.01);
    assert!((390.0 - insets.right - toggle.origin.x - toggle.size.width - 14.0).abs() < 0.01);
    assert_eq!(application.size.height, 844.0);
    assert_eq!(surface.size.height, 0.0);
}

#[test]
fn real_showcase_lowers_both_devtools_button_and_page_scrollbar() {
    let host = argui_runtime::Entity::new(DevtoolsHost::new(StateShowcase::default()))
        .mount()
        .unwrap();
    let mut tree = UiTree::new(host.render(Default::default()).unwrap());
    let toggle = tree
        .node_ids()
        .iter()
        .copied()
        .find(|node| tree.key(*node) == Some("__devtools-toggle"))
        .unwrap();
    let page = tree
        .node_ids()
        .iter()
        .copied()
        .find(|node| tree.key(*node) == Some("page-scroll"))
        .unwrap();
    let mut layout = LayoutEngine::new();
    let output = layout
        .compute(&mut tree, &mut text_engine(), Size::new(1_100.0, 700.0))
        .unwrap();

    assert!(output.text.blocks().iter().any(|block| {
        block.content.as_str() == "DevTools"
            && block.bounds.origin.x >= 0.0
            && block.bounds.origin.y >= 0.0
            && block.bounds.origin.x + block.bounds.size.width <= output.viewport.size.width
            && block.bounds.origin.y + block.bounds.size.height <= output.viewport.size.height
    }));
    assert!(
        output
            .hit_regions
            .iter()
            .any(|region| region.node == toggle)
    );
    let page_scroll = output
        .scroll_regions
        .iter()
        .find(|region| region.node == page)
        .unwrap();
    assert!(page_scroll.max_offset.y > 0.0);
    assert!(page_scroll.scrollbar.is_some());
}

#[test]
fn toolbar_exposes_docking_and_has_no_demo_animation() {
    let host = argui_runtime::Entity::new(DevtoolsHost::new(StateShowcase::default()).open(true))
        .mount()
        .unwrap();
    let root = host.render(Default::default()).unwrap();
    assert!(!contains_text(&root, "GPU transform"));
    assert!(contains_text(&root, "Bottom"));
    assert_eq!(
        host.update(|tools, cx| {
            cx.notify();
            tools.update(&event("__devtools-icon-transform"))
        })
        .unwrap(),
        ViewUpdate::None
    );
    assert!(!host.read(|tools| tools.wants_animation_frame()));
}

#[test]
fn sheet_relayouts_incrementally_and_returns_to_idle() {
    let host = argui_runtime::Entity::new(DevtoolsHost::new(StateShowcase::default()))
        .mount()
        .unwrap();
    let mut tree = UiTree::new(host.render(Default::default()).unwrap());
    host.update(|tools, cx| {
        cx.notify();
        tools.update(&event("__devtools-toggle"))
    })
    .unwrap();
    assert_eq!(
        tree.update(host.render(Default::default()).unwrap()),
        argui_ui::TreeUpdate::Paint
    );
    host.update(|tools, cx| {
        cx.notify();
        tools.animation_frame(Frame {
            now: Time::ZERO,
            elapsed: Duration::from_millis(16),
        })
    })
    .unwrap();
    assert_eq!(
        tree.update(host.render(Default::default()).unwrap()),
        argui_ui::TreeUpdate::Layout
    );

    for _ in 0..60 {
        host.update(|tools, cx| {
            cx.notify();
            tools.animation_frame(Frame {
                now: Time::ZERO,
                elapsed: Duration::from_millis(16),
            })
        })
        .unwrap();
    }
    tree.update(host.render(Default::default()).unwrap());

    assert!(!host.read(|tools| tools.wants_animation_frame()));
    assert_eq!(
        tree.update(host.render(Default::default()).unwrap()),
        argui_ui::TreeUpdate::None
    );
}

#[test]
fn profiling_view_presents_gpu_passes_and_unavailable_timestamp_state() {
    let host = argui_runtime::Entity::new(DevtoolsHost::new(StateShowcase::default()).open(true))
        .mount()
        .unwrap();
    host.read(|tools| tools.inspector()).record_ui(FrameRecord {
        interval: std::time::Duration::from_millis(18),
        model: std::time::Duration::from_millis(1),
        tree: std::time::Duration::from_millis(2),
        paint: std::time::Duration::from_millis(3),
        passes: 2,
        texture_bytes: 2_097_152,
        adapter: AdapterRecord {
            name: "test-adapter".into(),
            backend: "test-backend".into(),
            features: "timestamps".into(),
            timestamp_queries: true,
            ..AdapterRecord::default()
        },
        gpu: Some(GpuFrameRecord {
            sequence: 7,
            total: std::time::Duration::from_millis(4),
            passes: vec![GpuPassRecord {
                label: "blur pass".into(),
                start: std::time::Duration::ZERO,
                duration: std::time::Duration::from_millis(2),
                pixels: 4_096,
                object_domain: Some("effect".into()),
                object_id: Some(7),
            }],
        }),
        ..FrameRecord::default()
    });
    assert_eq!(
        host.update(|tools, cx| {
            cx.notify();
            tools.update(&event("__devtools-profiling"))
        })
        .unwrap(),
        ViewUpdate::Rebuild
    );
    let profiling = host.render(Default::default()).unwrap();
    assert!(contains_text(&profiling, "GPU timeline"));
    assert!(contains_text(&profiling, "blur pass"));
    assert!(!contains_text(&profiling, "Most expensive passes"));
    host.update(|tools, cx| {
        cx.notify();
        tools.update(&event("__devtools-profile-details"))
    })
    .unwrap();
    assert!(contains_text(
        &host.render(Default::default()).unwrap(),
        "test-adapter"
    ));
    host.update(|tools, cx| {
        cx.notify();
        tools.update(&event("__devtools-profile-gpu"))
    })
    .unwrap();

    host.read(|tools| tools.inspector()).clear_frames();
    host.read(|tools| tools.inspector()).record_ui(FrameRecord {
        adapter: AdapterRecord {
            timestamp_queries: false,
            ..AdapterRecord::default()
        },
        ..FrameRecord::default()
    });
    assert_eq!(
        host.update(|tools, cx| {
            cx.notify();
            tools.update(&event("__devtools-refresh"))
        })
        .unwrap(),
        ViewUpdate::Rebuild
    );
    assert!(contains_text(
        &host.render(Default::default()).unwrap(),
        "GPU timestamps unavailable on this adapter"
    ));
}

fn contains_text(element: &Element, needle: &str) -> bool {
    matches!(&element.kind, argui_ui::ElementKind::Text { content, .. } if content.as_str().contains(needle))
        || element
            .children
            .iter()
            .any(|child| contains_text(child, needle))
}
