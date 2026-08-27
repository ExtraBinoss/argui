use std::time::Duration;

use argui_core::{Point, Rect, Size};
use argui_inspect::{
    FrameRecord, InspectNodeId, InspectorHandle, NodeSnapshot, PropertySnapshot, StyleLength,
    StyleProperty, StyleUnit, StyleValue, TreeSnapshot,
};

fn node(id: u64, parent: Option<u64>, depth: usize, bounds: Rect, z_index: i32) -> NodeSnapshot {
    NodeSnapshot {
        id: InspectNodeId(id),
        parent: parent.map(InspectNodeId),
        depth,
        key: None,
        kind: "container".into(),
        summary: None,
        bounds,
        clip: None,
        z_index,
        visible: true,
        properties: Vec::new(),
    }
}

#[test]
fn frame_history_is_bounded_pauseable_and_clearable() {
    let inspector = InspectorHandle::new(2);
    inspector.record_ui(FrameRecord::default());
    inspector.record_ui(FrameRecord::default());
    inspector.record_ui(FrameRecord::default());
    assert_eq!(inspector.frames().len(), 2);
    inspector.set_paused(true);
    inspector.record_ui(FrameRecord::default());
    assert_eq!(inspector.frames().len(), 2);
    inspector.clear_frames();
    assert!(inspector.frames().is_empty());
}

#[test]
fn overrides_are_reversible_and_selection_is_stable() {
    let inspector = InspectorHandle::default();
    let node = InspectNodeId(7);
    inspector.select(Some(node));
    assert_eq!(inspector.selected(), Some(node));
    assert!(!inspector.toggle(node, StyleProperty::Background));
    assert_eq!(
        inspector.property_enabled(node, StyleProperty::Background),
        Some(false)
    );
    assert!(inspector.toggle(node, StyleProperty::Background));
    inspector.clear_overrides();
    assert_eq!(
        inspector.property_enabled(node, StyleProperty::Background),
        None
    );

    let mut value = StyleValue::Color([0.1, 0.2, 0.3, 1.0]);
    assert!(value.set_field(1, 0.75));
    inspector.set_property_value(node, StyleProperty::Background, value.clone());
    assert_eq!(
        inspector.property_value(node, StyleProperty::Background),
        Some(value)
    );
    assert_eq!(
        inspector.property_enabled(node, StyleProperty::Background),
        Some(true)
    );
}

#[test]
fn style_labels_and_frame_totals_are_complete() {
    let labels = StyleProperty::ALL.map(StyleProperty::label);
    assert_eq!(
        labels,
        [
            "background",
            "border",
            "opacity",
            "clip",
            "transform",
            "layer",
            "effects",
            "width",
            "height",
        ]
    );
    let frame = FrameRecord {
        model: Duration::from_millis(1),
        tree: Duration::from_millis(2),
        paint: Duration::from_millis(3),
        render_cpu: Duration::from_millis(4),
        ..FrameRecord::default()
    };
    assert_eq!(frame.total_cpu(), Duration::from_millis(10));
}

#[test]
fn inspection_hit_testing_prefers_frontmost_deep_nodes_inside_the_viewport() {
    let inspector = InspectorHandle::default();
    let mut clipped = node(
        4,
        None,
        0,
        Rect::new(Point::new(30.0, 30.0), Size::new(80.0, 80.0)),
        10,
    );
    clipped.clip = Some(Rect::new(Point::new(100.0, 100.0), Size::new(20.0, 20.0)));
    let mut invisible = node(
        5,
        None,
        0,
        Rect::new(Point::new(20.0, 20.0), Size::new(80.0, 80.0)),
        100,
    );
    invisible.visible = false;
    inspector.publish_tree(TreeSnapshot {
        revision: 1,
        nodes: vec![
            node(
                1,
                None,
                0,
                Rect::new(Point::default(), Size::new(200.0, 200.0)),
                0,
            ),
            node(
                2,
                Some(1),
                1,
                Rect::new(Point::new(20.0, 20.0), Size::new(80.0, 80.0)),
                0,
            ),
            node(
                3,
                None,
                0,
                Rect::new(Point::new(30.0, 30.0), Size::new(80.0, 80.0)),
                5,
            ),
            clipped,
            invisible,
        ],
    });
    let viewport = Rect::new(Point::default(), Size::new(150.0, 150.0));
    assert_eq!(
        inspector.hit_test(Point::new(40.0, 40.0), viewport),
        Some(InspectNodeId(3))
    );
    assert_eq!(
        inspector.hit_test(Point::new(25.0, 25.0), viewport),
        Some(InspectNodeId(2))
    );
    assert_eq!(inspector.hit_test(Point::new(170.0, 40.0), viewport), None);
}

#[test]
fn snapshots_and_render_metrics_share_one_bounded_record() {
    let inspector = InspectorHandle::new(1);
    let tree = TreeSnapshot {
        revision: 9,
        nodes: vec![NodeSnapshot {
            id: InspectNodeId(3),
            parent: None,
            depth: 0,
            key: Some("root".into()),
            kind: "container".into(),
            summary: Some("one child".into()),
            bounds: Rect::new(Point::new(2.0, 3.0), Size::new(40.0, 50.0)),
            clip: None,
            z_index: 7,
            visible: true,
            properties: vec![PropertySnapshot {
                property: StyleProperty::Width,
                authored: true,
                value: StyleValue::Length(StyleLength {
                    value: 40.0,
                    unit: StyleUnit::Px,
                }),
            }],
        }],
    };
    inspector.publish_tree(tree.clone());
    assert_eq!(inspector.tree(), tree);

    inspector.record_ui(FrameRecord {
        model: Duration::from_millis(1),
        ..FrameRecord::default()
    });
    inspector.record_render(FrameRecord {
        render_cpu: Duration::from_millis(2),
        layers: 3,
        passes: 4,
        offscreen_pixels: 5,
        textures: 6,
        reused_textures: 7,
        texture_bytes: 8,
        ..FrameRecord::default()
    });
    let frame = inspector.frames()[0];
    assert_eq!(frame.model, Duration::from_millis(1));
    assert_eq!(frame.render_cpu, Duration::from_millis(2));
    assert_eq!(frame.layers, 3);
    assert_eq!(frame.passes, 4);
    assert_eq!(frame.offscreen_pixels, 5);
    assert_eq!(frame.textures, 6);
    assert_eq!(frame.reused_textures, 7);
    assert_eq!(frame.texture_bytes, 8);
}

#[test]
fn empty_paused_and_zero_capacity_histories_are_safe() {
    let empty = InspectorHandle::new(2);
    empty.record_render(FrameRecord {
        passes: 2,
        ..FrameRecord::default()
    });
    assert_eq!(empty.frames()[0].passes, 2);

    empty.set_paused(true);
    assert!(empty.paused());
    empty.record_render(FrameRecord::default());
    assert_eq!(empty.frames().len(), 1);
    empty.select(None);
    assert_eq!(empty.selected(), None);
    assert_eq!(
        empty.property_enabled(InspectNodeId(99), StyleProperty::Height),
        None
    );

    let disabled = InspectorHandle::new(0);
    disabled.record_ui(FrameRecord::default());
    disabled.record_render(FrameRecord::default());
    assert!(disabled.frames().is_empty());
}
