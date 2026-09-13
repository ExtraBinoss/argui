use std::{cell::Cell, rc::Rc, time::Duration as StdDuration};

use argui_animation::{Duration, Frame, Time};
use argui_core::{Point, PointerEvent, PointerPhase, Rect, Size};
use argui_devtools::DevtoolsHost;
use argui_inspect::{
    FrameRecord, InspectNodeId, Invalidation, NodeSnapshot, PropertySnapshot, StyleProperty,
    StyleValue, TreeSnapshot,
};
use argui_runtime::{Context, Entity, LayoutBounds, LayoutSnapshot, Render, ViewUpdate};
use argui_ui::{
    Element, EventType, GestureEvent, GestureKind, GesturePhase, UiEvent, UiEventKind, UiTree,
};

#[path = "host/input.rs"]
mod input;
#[path = "host/interaction.rs"]
mod interaction;
#[path = "host/properties.rs"]
mod properties;
#[path = "host/theme.rs"]
mod theme;
#[path = "host/tree.rs"]
mod tree;

struct App(Rc<Cell<usize>>);

impl Render for App {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        Element::text("application")
            .keyed("app-content")
            .on(cx.listener(EventType::Click, |app, _event, cx| {
                app.0.set(app.0.get() + 1);
                cx.notify();
            }))
    }
}

fn event(key: &str, kind: UiEventKind) -> UiEvent {
    let tree = UiTree::new(Element::container([]));
    UiEvent::new(tree.node_id_at(0).unwrap(), Some(key.to_owned()), kind)
}

fn pointer(phase: PointerPhase, point: Point) -> UiEventKind {
    UiEventKind::Pointer(PointerEvent::mouse(phase, point))
}

fn dispatch_key<T: Render>(
    entity: &argui_runtime::Mount<T>,
    tree: &mut UiTree,
    key: &str,
    kind: UiEventKind,
) {
    let target = tree
        .node_ids()
        .iter()
        .copied()
        .find(|node| tree.key(*node) == Some(key))
        .expect("event target must exist");
    for delivery in tree.event_deliveries(target, kind) {
        if delivery.should_dispatch() {
            entity.dispatch_event(&delivery).unwrap();
        }
    }
}

#[test]
fn host_routes_application_and_devtools_events_independently() {
    let updates = Rc::new(Cell::new(0));
    let host = Entity::new(DevtoolsHost::new(App(updates.clone())).open(true))
        .mount()
        .unwrap();
    let mut tree = UiTree::new(host.render(Default::default()).unwrap());
    let target = tree
        .node_ids()
        .iter()
        .copied()
        .find(|node| tree.key(*node) == Some("app-content"))
        .unwrap();
    for delivery in tree.event_deliveries(
        target,
        UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
    ) {
        if delivery.should_dispatch() {
            host.dispatch_event(&delivery).unwrap();
        }
    }
    assert_eq!(updates.get(), 1);
    host.update(|tools, _cx| {
        assert_eq!(
            tools.update(&event(
                "__devtools-profiling",
                UiEventKind::Click(argui_ui::ClickEvent::accessibility())
            )),
            ViewUpdate::Rebuild
        );
    })
    .unwrap();
    assert_eq!(updates.get(), 1);
}

#[test]
fn dock_controls_cover_filter_scroll_pause_clear_and_resize() {
    let host = Entity::new(populated_host()).mount().unwrap();
    let inspector = host.read(|tools| tools.inspector());
    assert_eq!(
        change_tools(&host, |tools| tools.update(&event(
            "__devtools-search",
            UiEventKind::TextChanged("button".into())
        ))),
        ViewUpdate::Rebuild
    );
    let filtered = host.render(Default::default()).unwrap();
    assert!(contains_text(&filtered, "button  #save"));
    assert!(!contains_text(&filtered, "container  #panel"));
    assert_eq!(
        change_tools(&host, |tools| tools.update(&event(
            "__devtools-tree",
            UiEventKind::Scrolled {
                delta: Point::new(0.0, 20.0),
                offset: Point::new(0.0, 40.0),
            }
        ))),
        ViewUpdate::None
    );
    assert_eq!(
        change_tools(&host, |tools| tools.update(&event(
            "__devtools-frames",
            UiEventKind::Scrolled {
                delta: Point::new(0.0, 12.0),
                offset: Point::new(0.0, 24.0),
            }
        ))),
        ViewUpdate::None
    );
    change_tools(&host, |tools| {
        tools.update(&event(
            "__devtools-copy",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        ))
    });
    assert!(matches!(
        change_tools(&host, |tools| tools.take_clipboard_request()),
        Some(argui_ui::ClipboardRequest::Write(trace))
            if trace.contains("argui-gpu-trace-v3")
    ));
    assert_eq!(
        change_tools(&host, |tools| tools.take_clipboard_request()),
        None
    );
    change_tools(&host, |tools| {
        tools.update(&event(
            "__devtools-section-0",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        ))
    });
    assert!(host.read(|tools| tools.wants_animation_frame()));
    assert_eq!(
        change_tools(&host, |tools| tools.animation_frame(Frame {
            now: Time::ZERO,
            elapsed: Duration::from_millis(16),
        })),
        ViewUpdate::Rebuild
    );
    change_tools(&host, |tools| {
        tools.update(&event(
            "__devtools-section-nope",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        ))
    });
    change_tools(&host, |tools| {
        tools.update(&event(
            "__devtools-section-99",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        ))
    });
    change_tools(&host, |tools| {
        tools.update(&event(
            "__devtools-pause",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        ))
    });
    assert!(inspector.paused());
    change_tools(&host, |tools| {
        tools.update(&event(
            "__devtools-pause",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        ))
    });
    assert!(!inspector.paused());
    inspector.record_ui(FrameRecord::default());
    change_tools(&host, |tools| {
        tools.update(&event(
            "__devtools-clear",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        ))
    });
    assert!(inspector.frames().is_empty());
    change_tools(&host, |tools| {
        tools.update(&event(
            "__devtools-reset",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        ))
    });
    let layout = LayoutSnapshot {
        viewport: Rect::new(Point::default(), Size::new(900.0, 700.0)),
        nodes: Vec::new(),
    };
    change_tools(&host, |tools| tools.inspect_layout(&layout));
    let mut tree = UiTree::new(host.render(Default::default()).unwrap());
    let splitter = tree
        .node_ids()
        .iter()
        .copied()
        .find(|node| tree.key(*node) == Some("__devtools-splitter"))
        .unwrap();
    dispatch_key(
        &host,
        &mut tree,
        "__devtools-splitter",
        UiEventKind::Gesture(GestureEvent {
            target: splitter,
            pointer: argui_core::PointerId::MOUSE,
            phase: GesturePhase::Started,
            delivery: Default::default(),
            kind: GestureKind::Pan {
                position: Point::new(30.0, 250.0),
                delta: Point::default(),
                total: Point::default(),
                velocity: Point::default(),
            },
        }),
    );
    dispatch_key(
        &host,
        &mut tree,
        "__devtools-splitter",
        UiEventKind::Gesture(GestureEvent {
            target: splitter,
            pointer: argui_core::PointerId::MOUSE,
            phase: GesturePhase::Changed,
            delivery: Default::default(),
            kind: GestureKind::Pan {
                position: Point::new(30.0, 200.0),
                delta: Point::new(0.0, -50.0),
                total: Point::new(0.0, -50.0),
                velocity: Point::default(),
            },
        }),
    );
    assert_eq!(
        tree.update(host.render(Default::default()).unwrap()),
        argui_ui::TreeUpdate::Layout
    );
}

#[test]
fn profiling_and_closed_views_keep_the_dock_tree_retained() {
    let host = Entity::new(populated_host()).mount().unwrap();
    let inspector = host.read(|tools| tools.inspector());
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
    change_tools(&host, |tools| {
        tools.update(&event(
            "__devtools-profiling",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        ))
    });
    let profile = host.render(Default::default()).unwrap();
    assert!(contains_text(&profile, "Passes"));
    assert!(contains_text(&profile, "MiB"));
    assert!(contains_text(&profile, "1.0"));
    change_tools(&host, |tools| {
        tools.update(&event(
            "__devtools-profile-details",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        ))
    });
    let details = host.render(Default::default()).unwrap();
    assert!(contains_text(&details, "Invalidation"));
    assert!(contains_text(&details, "Layout"));

    change_tools(&host, |tools| {
        tools.update(&event(
            "__devtools-elements",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        ))
    });
    assert!(contains_key(
        &host.render(Default::default()).unwrap(),
        "__devtools-tree"
    ));
    change_tools(&host, |tools| {
        tools.update(&event(
            "__devtools-toggle",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        ))
    });
    assert!(contains_key(
        &host.render(Default::default()).unwrap(),
        "__devtools-tree"
    ));
    settle(&host);
    let closed = host.render(Default::default()).unwrap();
    assert!(contains_text(&closed, "DevTools"));
    assert!(contains_key(&closed, "__devtools-tree"));
}

#[test]
fn host_animation_and_layout_delegation_keep_the_app_viewport_explicit() {
    let updates = Rc::new(Cell::new(0));
    let host = Entity::new(DevtoolsHost::new(App(updates.clone())))
        .mount()
        .unwrap();
    assert!(!host.read(|tools| tools.wants_animation_frame()));
    assert_eq!(
        change_tools(&host, |tools| tools.animation_frame(Frame {
            now: Time::ZERO,
            elapsed: Duration::ZERO,
        })),
        ViewUpdate::None
    );
    change_tools(&host, |tools| {
        tools.update(&event(
            "__devtools-toggle",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        ))
    });
    assert!(host.read(|tools| tools.wants_animation_frame()));
    assert_eq!(
        change_tools(&host, |tools| tools.animation_frame(Frame {
            now: Time::ZERO,
            elapsed: Duration::from_millis(16),
        })),
        ViewUpdate::Rebuild
    );

    let tree = UiTree::new(Element::container([]));
    let app_bounds = Rect::new(Point::new(0.0, 0.0), Size::new(800.0, 400.0));
    assert_eq!(
        change_tools(&host, |tools| tools.inspect_layout(&LayoutSnapshot {
            viewport: Rect::new(Point::default(), Size::new(800.0, 700.0)),
            nodes: vec![LayoutBounds {
                node: tree.node_id_at(0).unwrap(),
                key: Some("__devtools-app-root".into()),
                bounds: app_bounds,
            }],
        })),
        ViewUpdate::None
    );
    assert!(host.read(|tools| tools.image_assets()).is_empty());
    assert_eq!(host.read(|tools| tools.vector_assets()).len(), 8);
    assert!(host.read(Render::inspector).is_some());
}

#[test]
fn picker_hit_tests_the_application_and_selects_without_clicking_through() {
    let host = Entity::new(populated_host()).mount().unwrap();
    let inspector = host.read(|tools| tools.inspector());
    change_tools(&host, |tools| {
        tools.inspect_layout(&LayoutSnapshot {
            viewport: Rect::new(Point::default(), Size::new(900.0, 700.0)),
            nodes: vec![LayoutBounds {
                node: UiTree::new(Element::container([])).node_id_at(0).unwrap(),
                key: Some("__devtools-app-root".into()),
                bounds: Rect::new(Point::default(), Size::new(900.0, 380.0)),
            }],
        })
    });

    assert_eq!(
        change_tools(&host, |tools| tools.update(&event(
            "__devtools-picker",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility())
        ))),
        ViewUpdate::Rebuild
    );
    assert!(contains_key(
        &host.render(Default::default()).unwrap(),
        "__devtools-picker-surface"
    ));
    assert_eq!(
        change_tools(&host, |tools| tools.update(&event(
            "__devtools-picker-surface",
            pointer(PointerPhase::Moved, Point::new(40.0, 40.0)),
        ))),
        ViewUpdate::Paint
    );
    assert_eq!(inspector.selected(), None);
    assert_eq!(
        change_tools(&host, |tools| tools.update(&event(
            "__devtools-picker-surface",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility())
        ))),
        ViewUpdate::Rebuild
    );
    assert_eq!(inspector.selected(), Some(InspectNodeId(2)));
    let request =
        change_tools(&host, |tools| tools.take_scroll_request()).expect("picker reveals tree row");
    assert!(matches!(
        request.target,
        argui_ui::ScrollTarget::Offset {
            container: argui_ui::FocusTarget::Key(key),
            ..
        } if key == "__devtools-tree"
    ));
    assert!(!contains_key(
        &host.render(Default::default()).unwrap(),
        "__devtools-picker-surface"
    ));

    change_tools(&host, |tools| {
        tools.update(&event(
            "__devtools-picker",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        ))
    });
    change_tools(&host, |tools| {
        tools.update(&event(
            "__devtools-picker-surface",
            pointer(PointerPhase::Moved, Point::new(40.0, 40.0)),
        ))
    });
    change_tools(&host, |tools| {
        tools.update(&event(
            "__devtools-picker-surface",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        ))
    });
    assert_eq!(
        inspector.selected(),
        Some(InspectNodeId(1)),
        "repeating the picker at one point cycles to the structural fallback"
    );
}

#[test]
fn non_click_tool_events_are_consumed_without_reaching_the_application() {
    let updates = Rc::new(Cell::new(0));
    let host = Entity::new(DevtoolsHost::new(App(updates.clone())).open(true))
        .mount()
        .unwrap();
    assert_eq!(
        change_tools(&host, |tools| tools.update(&event(
            "__devtools-unknown",
            pointer(PointerPhase::Moved, Point::new(1.0, 2.0))
        ))),
        ViewUpdate::None
    );
    assert_eq!(updates.get(), 0);
    assert_eq!(
        change_tools(&host, |tools| tools.update(&event(
            "unrelated",
            pointer(PointerPhase::Left, Point::default())
        ))),
        ViewUpdate::None
    );
    assert_eq!(updates.get(), 0);
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
                portal: None,
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
                portal: None,
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
    matches!(&element.kind, argui_ui::ElementKind::Text { content, .. } if content.as_str().contains(needle))
        || element
            .children
            .iter()
            .any(|child| contains_text(child, needle))
}

fn settle(host: &argui_runtime::Mount<DevtoolsHost<App>>) {
    for _ in 0..60 {
        host.animation_frame(Frame {
            now: Time::ZERO,
            elapsed: Duration::from_millis(16),
        })
        .unwrap();
    }
}

fn change_tools<A: Render, R>(
    host: &argui_runtime::Mount<DevtoolsHost<A>>,
    change: impl FnOnce(&mut DevtoolsHost<A>) -> R,
) -> R {
    host.update(|tools, cx| {
        cx.notify();
        change(tools)
    })
    .unwrap()
}
