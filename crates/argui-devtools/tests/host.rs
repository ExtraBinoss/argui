use std::{cell::Cell, rc::Rc, time::Duration as StdDuration};

use argui_animation::{Duration, Frame, Time};
use argui_core::{Point, Rect, Size};
use argui_devtools::DevtoolsHost;
use argui_inspect::{
    FrameRecord, InspectNodeId, Invalidation, NodeSnapshot, PropertySnapshot, StyleProperty,
    StyleValue, TreeSnapshot,
};
use argui_layout::LayoutEngine;
use argui_runtime::{Context, LayoutBounds, LayoutSnapshot, Render, ViewUpdate};
use argui_showcase::{StateShowcase, text_engine};
use argui_text::TextEngine;
use argui_ui::{Element, UiEvent, UiEventKind, UiTree};

struct App(Rc<Cell<usize>>);

impl Render for App {
    fn render(&mut self, _cx: &mut Context<Self>) -> Element {
        Element::text("application").keyed("app-content")
    }

    fn event(&mut self, _event: &UiEvent, cx: &mut Context<Self>) {
        self.0.set(self.0.get() + 1);
        cx.notify();
    }
}

fn event(key: &str, kind: UiEventKind) -> UiEvent {
    let tree = UiTree::new(Element::container([]));
    UiEvent {
        target: tree.node_id_at(0).unwrap(),
        key: Some(key.to_owned()),
        kind,
    }
}

#[test]
fn host_routes_application_and_devtools_events_independently() {
    let updates = Rc::new(Cell::new(0));
    let mut host = DevtoolsHost::new(App(updates.clone())).open(true);
    assert_eq!(
        host.update(&event("app-content", UiEventKind::Clicked)),
        ViewUpdate::Rebuild
    );
    assert_eq!(updates.get(), 1);
    assert_eq!(
        host.update(&event("__devtools-profiling", UiEventKind::Clicked)),
        ViewUpdate::Rebuild
    );
    assert_eq!(updates.get(), 1);
}

#[test]
fn selected_nodes_receive_reversible_typed_style_overrides() {
    let mut host = DevtoolsHost::new(App(Rc::new(Cell::new(0)))).open(true);
    let inspector = host.inspector();
    inspector.publish_tree(TreeSnapshot {
        revision: 1,
        nodes: vec![NodeSnapshot {
            id: InspectNodeId(42),
            parent: None,
            depth: 0,
            key: Some("panel".into()),
            kind: "container".into(),
            summary: Some("panel content".into()),
            bounds: Rect::new(Point::default(), Size::new(100.0, 80.0)),
            clip: None,
            z_index: 0,
            visible: true,
            painted: true,
            interactive: false,
            child_count: 0,
            properties: vec![PropertySnapshot {
                property: StyleProperty::Background,
                authored: true,
                value: StyleValue::Color([0.2, 0.4, 0.6, 1.0]),
            }],
        }],
    });
    host.update(&event("__devtools-node-42", UiEventKind::Clicked));
    assert!(contains_key(
        &host.view(),
        "__devtools-value-42-background-0"
    ));
    host.update(&event(
        "__devtools-value-42-background-0",
        UiEventKind::TextChanged("0.9".into()),
    ));
    assert!(matches!(
        inspector.property_value(InspectNodeId(42), StyleProperty::Background),
        Some(StyleValue::Color([red, _, _, _])) if red == 0.9
    ));
    host.update(&event("__devtools-style-background", UiEventKind::Clicked));
    assert_eq!(inspector.selected(), Some(InspectNodeId(42)));
    assert_eq!(
        inspector.property_enabled(InspectNodeId(42), StyleProperty::Background),
        Some(false)
    );
    assert!(contains_key(&host.view(), "__devtools-tree"));
}

#[test]
fn dock_controls_cover_filter_scroll_pause_clear_and_resize() {
    let mut host = populated_host();
    let inspector = host.inspector();
    assert_eq!(
        host.update(&event(
            "__devtools-search",
            UiEventKind::TextChanged("button".into())
        )),
        ViewUpdate::Rebuild
    );
    let filtered = host.view();
    assert!(contains_text(&filtered, "button  #save"));
    assert!(!contains_text(&filtered, "container  #panel"));
    assert_eq!(
        host.update(&event(
            "__devtools-tree",
            UiEventKind::Scrolled {
                delta: Point::new(0.0, 20.0),
                offset: Point::new(0.0, 40.0),
            }
        )),
        ViewUpdate::None
    );
    assert_eq!(
        host.update(&event(
            "__devtools-frames",
            UiEventKind::Scrolled {
                delta: Point::new(0.0, 12.0),
                offset: Point::new(0.0, 24.0),
            }
        )),
        ViewUpdate::None
    );
    host.update(&event("__devtools-copy", UiEventKind::Clicked));
    assert!(matches!(
        host.take_clipboard_request(),
        Some(argui_ui::ClipboardRequest::Write(trace))
            if trace.contains("argui-gpu-trace-v1")
    ));
    assert_eq!(host.take_clipboard_request(), None);
    host.update(&event("__devtools-section-0", UiEventKind::Clicked));
    assert!(host.wants_animation_frame());
    assert_eq!(
        host.animation_frame(Frame {
            now: Time::ZERO,
            elapsed: Duration::from_millis(16),
        }),
        ViewUpdate::Rebuild
    );
    host.update(&event("__devtools-section-nope", UiEventKind::Clicked));
    host.update(&event("__devtools-section-99", UiEventKind::Clicked));
    host.update(&event("__devtools-pause", UiEventKind::Clicked));
    assert!(inspector.paused());
    host.update(&event("__devtools-pause", UiEventKind::Clicked));
    assert!(!inspector.paused());
    inspector.record_ui(FrameRecord::default());
    host.update(&event("__devtools-clear", UiEventKind::Clicked));
    assert!(inspector.frames().is_empty());
    host.update(&event("__devtools-reset", UiEventKind::Clicked));
    let layout = LayoutSnapshot {
        viewport: Rect::new(Point::default(), Size::new(900.0, 700.0)),
        nodes: Vec::new(),
    };
    host.layout_changed(&layout);
    host.update(&event("__devtools-splitter", UiEventKind::Pressed));
    assert_eq!(
        host.update(&event(
            "__devtools-splitter",
            UiEventKind::PointerMoved(Point::new(30.0, 250.0))
        )),
        ViewUpdate::Rebuild
    );
    host.update(&event("__devtools-splitter", UiEventKind::Released));
    assert_eq!(
        host.update(&event(
            "__devtools-splitter",
            UiEventKind::PointerMoved(Point::new(30.0, 300.0))
        )),
        ViewUpdate::None
    );
}

#[test]
fn elements_render_selection_highlight_and_every_style_control() {
    let mut host = populated_host();
    let inspector = host.inspector();
    host.update(&event("__devtools-node-2", UiEventKind::Clicked));
    let view = host.view();
    assert!(contains_text(&view, "Reset overrides"));
    assert!(contains_key(&view, "__devtools-style-effects"));

    for property in StyleProperty::ALL {
        host.update(&event(
            &format!("__devtools-style-{}", property.label()),
            UiEventKind::Clicked,
        ));
        assert_eq!(
            inspector.property_enabled(InspectNodeId(2), property),
            Some(false)
        );
    }
    inspector.select(Some(InspectNodeId(999)));
    assert!(contains_text(
        &host.view(),
        "Select an element to inspect its styles"
    ));
    host.update(&event("__devtools-node-not-a-number", UiEventKind::Clicked));
    assert_eq!(inspector.selected(), None);
    host.update(&event("__devtools-style-not-real", UiEventKind::Clicked));
}

#[test]
fn profiling_and_closed_views_keep_the_dock_tree_retained() {
    let mut host = populated_host();
    let inspector = host.inspector();
    inspector.record_ui(FrameRecord {
        interval: StdDuration::from_millis(16),
        model: StdDuration::from_micros(100),
        tree: StdDuration::from_micros(200),
        paint: StdDuration::from_micros(300),
        update: Invalidation::Paint,
        ..FrameRecord::default()
    });
    inspector.record_ui(FrameRecord {
        interval: StdDuration::from_millis(25),
        update: Invalidation::Layout,
        ..FrameRecord::default()
    });
    inspector.record_render(FrameRecord {
        render_cpu: StdDuration::from_millis(2),
        layers: 3,
        passes: 5,
        offscreen_pixels: 4096,
        textures: 2,
        reused_textures: 1,
        texture_bytes: 1_048_576,
        ..FrameRecord::default()
    });
    host.update(&event("__devtools-profiling", UiEventKind::Clicked));
    let profile = host.view();
    assert!(contains_text(&profile, "5 passes"));
    assert!(contains_text(&profile, "1.0 MiB"));
    assert!(contains_text(&profile, "invalidation Layout"));

    host.update(&event("__devtools-elements", UiEventKind::Clicked));
    assert!(contains_key(&host.view(), "__devtools-tree"));
    host.update(&event("__devtools-toggle", UiEventKind::Clicked));
    assert!(contains_key(&host.view(), "__devtools-tree"));
    settle(&mut host);
    let closed = host.view();
    assert!(contains_text(&closed, "DevTools"));
    assert!(contains_key(&closed, "__devtools-tree"));
}

#[test]
fn host_animation_and_layout_delegation_keep_the_app_viewport_explicit() {
    let updates = Rc::new(Cell::new(0));
    let mut host = DevtoolsHost::new(App(updates.clone()));
    assert!(!host.wants_animation_frame());
    assert_eq!(
        host.animation_frame(Frame {
            now: Time::ZERO,
            elapsed: Duration::ZERO,
        }),
        ViewUpdate::None
    );
    host.update(&event("__devtools-toggle", UiEventKind::Clicked));
    assert!(host.wants_animation_frame());
    assert_eq!(
        host.animation_frame(Frame {
            now: Time::ZERO,
            elapsed: Duration::from_millis(16),
        }),
        ViewUpdate::Rebuild
    );

    let tree = UiTree::new(Element::container([]));
    let app_bounds = Rect::new(Point::new(0.0, 0.0), Size::new(800.0, 400.0));
    assert_eq!(
        host.layout_changed(&LayoutSnapshot {
            viewport: Rect::new(Point::default(), Size::new(800.0, 700.0)),
            nodes: vec![LayoutBounds {
                node: tree.node_id_at(0).unwrap(),
                key: Some("__devtools-app-root".into()),
                bounds: app_bounds,
            }],
        }),
        ViewUpdate::None
    );
    assert!(host.image_assets().is_empty());
    assert_eq!(host.vector_assets().len(), 3);
    assert!(Render::inspector(&host).is_some());
}

#[test]
fn toolbar_exposes_and_animates_the_gpu_path_morph() {
    let mut host = DevtoolsHost::new(App(Rc::new(Cell::new(0)))).open(true);
    assert!(contains_text(&host.view(), "GPU morph"));
    assert_eq!(
        host.update(&event("__devtools-morph", UiEventKind::Clicked)),
        ViewUpdate::Rebuild
    );
    assert!(host.wants_animation_frame());
    assert_eq!(
        host.animation_frame(Frame {
            now: Time::ZERO,
            elapsed: Duration::from_millis(100),
        }),
        ViewUpdate::Rebuild
    );
}

#[test]
fn picker_hit_tests_the_application_and_selects_without_clicking_through() {
    let mut host = populated_host();
    let inspector = host.inspector();
    host.layout_changed(&LayoutSnapshot {
        viewport: Rect::new(Point::default(), Size::new(900.0, 700.0)),
        nodes: vec![LayoutBounds {
            node: UiTree::new(Element::container([])).node_id_at(0).unwrap(),
            key: Some("__devtools-app-root".into()),
            bounds: Rect::new(Point::default(), Size::new(900.0, 380.0)),
        }],
    });

    assert_eq!(
        host.update(&event("__devtools-picker", UiEventKind::Clicked)),
        ViewUpdate::Rebuild
    );
    assert!(contains_key(&host.view(), "__devtools-picker-surface"));
    assert_eq!(
        host.update(&event(
            "__devtools-picker-surface",
            UiEventKind::PointerMoved(Point::new(40.0, 40.0)),
        )),
        ViewUpdate::Paint
    );
    assert_eq!(inspector.selected(), None);
    assert_eq!(
        host.update(&event("__devtools-picker-surface", UiEventKind::Clicked)),
        ViewUpdate::Rebuild
    );
    assert_eq!(inspector.selected(), Some(InspectNodeId(2)));
    let request = host.take_scroll_request().expect("picker reveals tree row");
    assert_eq!(request.key, "__devtools-tree");
    assert!(!contains_key(&host.view(), "__devtools-picker-surface"));

    host.update(&event("__devtools-picker", UiEventKind::Clicked));
    host.update(&event(
        "__devtools-picker-surface",
        UiEventKind::PointerMoved(Point::new(40.0, 40.0)),
    ));
    host.update(&event("__devtools-picker-surface", UiEventKind::Clicked));
    assert_eq!(
        inspector.selected(),
        Some(InspectNodeId(1)),
        "repeating the picker at one point cycles to the structural fallback"
    );
}

#[test]
fn sheet_relayouts_incrementally_and_vectors_stay_paint_only() {
    let mut host = DevtoolsHost::new(App(Rc::new(Cell::new(0))));
    let mut tree = UiTree::new(host.view());
    host.update(&event("__devtools-toggle", UiEventKind::Clicked));
    assert_eq!(tree.update(host.view()), argui_ui::TreeUpdate::Paint);
    host.animation_frame(Frame {
        now: Time::ZERO,
        elapsed: Duration::from_millis(16),
    });
    assert_eq!(tree.update(host.view()), argui_ui::TreeUpdate::Layout);

    settle(&mut host);
    tree.update(host.view());

    host.update(&event("__devtools-morph", UiEventKind::Clicked));
    host.animation_frame(Frame {
        now: Time::ZERO,
        elapsed: Duration::from_millis(16),
    });
    assert_eq!(tree.update(host.view()), argui_ui::TreeUpdate::Paint);
}

#[test]
fn live_profiler_records_do_not_invalidate_the_visible_snapshot() {
    let mut host = populated_host();
    host.update(&event("__devtools-profiling", UiEventKind::Clicked));
    let mut tree = UiTree::new(host.view());
    tree.mark_layout_clean();
    host.inspector().record_ui(FrameRecord {
        interval: StdDuration::from_millis(400),
        ..FrameRecord::default()
    });
    assert_eq!(tree.update(host.view()), argui_ui::TreeUpdate::None);
    host.update(&event("__devtools-refresh", UiEventKind::Clicked));
    assert_eq!(tree.update(host.view()), argui_ui::TreeUpdate::Layout);
}

#[test]
fn non_click_tool_events_are_consumed_without_reaching_the_application() {
    let updates = Rc::new(Cell::new(0));
    let mut host = DevtoolsHost::new(App(updates.clone())).open(true);
    assert_eq!(
        host.update(&event(
            "__devtools-unknown",
            UiEventKind::PointerMoved(Point::new(1.0, 2.0))
        )),
        ViewUpdate::None
    );
    assert_eq!(updates.get(), 0);
    assert_eq!(
        host.update(&event("unrelated", UiEventKind::PointerLeft)),
        ViewUpdate::Rebuild
    );
    assert_eq!(updates.get(), 1);
}

#[test]
fn open_dock_reserves_application_viewport_space() {
    let mut host = populated_host();
    let mut tree = UiTree::new(host.view());
    let mut layout = LayoutEngine::new();
    let output = layout
        .compute(&mut tree, &mut TextEngine::new(), Size::new(1_100.0, 700.0))
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
    assert!(application.size.height < output.viewport.size.height);
    assert_eq!(
        surface.origin.y + surface.size.height,
        output.viewport.size.height
    );
    assert_eq!(splitter.origin.y, surface.origin.y);
}

#[test]
fn closed_dock_keeps_a_visible_overlay_button_without_stealing_app_height() {
    let mut host = DevtoolsHost::new(App(Rc::new(Cell::new(0))));
    let mut tree = UiTree::new(host.view());
    let mut layout = LayoutEngine::new();
    let output = layout
        .compute(&mut tree, &mut TextEngine::new(), Size::new(1_100.0, 700.0))
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
}

#[test]
fn real_showcase_lowers_both_devtools_button_and_page_scrollbar() {
    let mut host = DevtoolsHost::new(StateShowcase::default());
    let mut tree = UiTree::new(host.view());
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
        block.text == "DevTools"
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

fn populated_host() -> DevtoolsHost<App> {
    let host = DevtoolsHost::new(App(Rc::new(Cell::new(0)))).open(true);
    host.inspector().publish_tree(TreeSnapshot {
        revision: 4,
        nodes: vec![
            NodeSnapshot {
                id: InspectNodeId(1),
                parent: None,
                depth: 0,
                key: Some("panel".into()),
                kind: "container".into(),
                summary: Some("1 child".into()),
                bounds: Rect::new(Point::new(10.0, 12.0), Size::new(300.0, 200.0)),
                clip: None,
                z_index: 0,
                visible: true,
                painted: false,
                interactive: false,
                child_count: 1,
                properties: vec![],
            },
            NodeSnapshot {
                id: InspectNodeId(2),
                parent: Some(InspectNodeId(1)),
                depth: 1,
                key: Some("save".into()),
                kind: "button".into(),
                summary: Some("Save changes".into()),
                bounds: Rect::new(Point::new(20.0, 30.0), Size::new(80.0, 32.0)),
                clip: None,
                z_index: 2,
                visible: true,
                painted: true,
                interactive: true,
                child_count: 0,
                properties: StyleProperty::ALL
                    .into_iter()
                    .map(|property| PropertySnapshot {
                        property,
                        authored: property == StyleProperty::Background,
                        value: StyleValue::Summary("test value".into()),
                    })
                    .collect(),
            },
        ],
    });
    host
}

fn contains_key(element: &Element, key: &str) -> bool {
    element.key.as_deref() == Some(key)
        || element
            .children
            .iter()
            .any(|child| contains_key(child, key))
}

fn contains_text(element: &Element, needle: &str) -> bool {
    matches!(&element.kind, argui_ui::ElementKind::Text { content, .. } if content.contains(needle))
        || element
            .children
            .iter()
            .any(|child| contains_text(child, needle))
}

fn settle(host: &mut DevtoolsHost<App>) {
    for _ in 0..60 {
        host.animation_frame(Frame {
            now: Time::ZERO,
            elapsed: Duration::from_millis(16),
        });
    }
}
