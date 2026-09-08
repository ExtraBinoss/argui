use argui_animation::{Duration, Frame, Time};
use argui_core::{Point, Rect, Size};
use argui_devtools::DevtoolsHost;
use argui_inspect::{
    InspectNodeId, NodeSnapshot, PortalSnapshot, PropertySnapshot, StyleField, StyleLength,
    StyleProperty, StyleUnit, StyleValue, TreeSnapshot,
};
use argui_runtime::{Context, Render, ViewUpdate};
use argui_ui::{Element, UiEvent, UiEventKind, UiTree};

struct App;

impl Render for App {
    fn render(&mut self, _: &mut Context<Self>) -> Element {
        Element::container([])
    }
}

fn event(key: &str) -> UiEvent {
    let tree = UiTree::new(Element::container([]));
    UiEvent::new(
        tree.node_id_at(0).unwrap(),
        Some(key.into()),
        UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
    )
}

fn portal_node(constrained_width: bool, constrained_height: bool) -> NodeSnapshot {
    NodeSnapshot {
        id: InspectNodeId(7),
        parent: None,
        depth: 0,
        key: Some("menu".into()),
        kind: "popover".into(),
        summary: Some("Account menu".into()),
        bounds: Rect::new(Point::new(24.0, 36.0), Size::new(180.0, 120.0)),
        clip: None,
        z_index: 12,
        portal: Some(PortalSnapshot {
            layer: "overlay".into(),
            anchor: Some("account-button".into()),
            requested: Some("bottom-start".into()),
            resolved: Some("top-start".into()),
            available_size: Size::new(320.0, 240.0),
            constrained_width,
            constrained_height,
        }),
        visible: true,
        painted: true,
        interactive: true,
        child_count: 0,
        properties: vec![
            PropertySnapshot {
                property: StyleProperty::Width,
                authored: true,
                value: StyleValue::Length(StyleLength {
                    value: 180.0,
                    unit: StyleUnit::Px,
                }),
            },
            PropertySnapshot {
                property: StyleProperty::Height,
                authored: true,
                value: StyleValue::Length(StyleLength::default()),
            },
            PropertySnapshot {
                property: StyleProperty::Background,
                authored: true,
                value: StyleValue::Srgba([0.1, 0.2, 0.3, 0.9]),
            },
            PropertySnapshot {
                property: StyleProperty::Border,
                authored: false,
                value: StyleValue::Parameters(vec![StyleField {
                    label: "width".into(),
                    value: 1.0,
                }]),
            },
            PropertySnapshot {
                property: StyleProperty::Opacity,
                authored: true,
                value: StyleValue::Number(0.9),
            },
            PropertySnapshot {
                property: StyleProperty::Overflow,
                authored: false,
                value: StyleValue::Choice("clip".into()),
            },
            PropertySnapshot {
                property: StyleProperty::Transform,
                authored: false,
                value: StyleValue::Summary("identity".into()),
            },
            PropertySnapshot {
                property: StyleProperty::Layer,
                authored: false,
                value: StyleValue::Parameters(vec![StyleField {
                    label: "z".into(),
                    value: 12.0,
                }]),
            },
            PropertySnapshot {
                property: StyleProperty::Effects,
                authored: false,
                value: StyleValue::Summary("none".into()),
            },
        ],
    }
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

#[test]
fn portal_metadata_and_collapsible_style_sections_follow_snapshot_state() {
    let host = argui_runtime::Entity::new(DevtoolsHost::new(App).open(true))
        .mount()
        .unwrap();
    let inspector = host.read(|tools| tools.inspector());
    inspector.publish_tree(TreeSnapshot {
        revision: 1,
        nodes: vec![portal_node(true, false)],
    });
    host.update(|tools, cx| {
        cx.notify();
        tools.update(&event("__devtools-node-7"))
    })
    .unwrap();

    let constrained = host.render(Default::default()).unwrap();
    assert!(contains_text(&constrained, "Portal overlay"));
    assert!(contains_text(&constrained, "constrained"));
    assert!(contains_key(&constrained, "__devtools-value-7-width-0"));
    assert!(contains_text(&constrained, "auto"));

    assert_eq!(
        host.update(|tools, cx| {
            cx.notify();
            tools.update(&event("__devtools-section-0"))
        })
        .unwrap(),
        ViewUpdate::Rebuild
    );
    assert!(!contains_key(
        &host.render(Default::default()).unwrap(),
        "__devtools-value-7-width-0"
    ));
    assert!(host.read(|tools| tools.wants_animation_frame()));
    assert_eq!(
        host.update(|tools, cx| {
            cx.notify();
            tools.animation_frame(Frame {
                now: Time::ZERO,
                elapsed: Duration::from_millis(16),
            })
        })
        .unwrap(),
        ViewUpdate::Rebuild
    );

    for (revision, width, height, text) in [(2, false, false, false), (3, false, true, true)] {
        inspector.publish_tree(TreeSnapshot {
            revision,
            nodes: vec![portal_node(width, height)],
        });
        host.update(|_, cx| cx.notify()).unwrap();
        let view = host.render(Default::default()).unwrap();
        assert_eq!(contains_text(&view, "constrained"), text);
    }

    host.update(|tools, cx| {
        cx.notify();
        tools.update(&event("__devtools-section-0"))
    })
    .unwrap();
    assert!(contains_key(
        &host.render(Default::default()).unwrap(),
        "__devtools-value-7-width-0"
    ));
    host.update(|tools, cx| {
        cx.notify();
        tools.inspect_layout(&argui_runtime::LayoutSnapshot {
            viewport: Rect::new(Point::default(), Size::new(500.0, 700.0)),
            nodes: vec![],
        })
    })
    .unwrap();
    assert!(contains_key(
        &host.render(Default::default()).unwrap(),
        "__devtools-back"
    ));
    host.update(|tools, cx| {
        cx.notify();
        tools.update(&event("__devtools-back"))
    })
    .unwrap();
    let elements = host.render(Default::default()).unwrap();
    assert!(contains_key(&elements, "__devtools-tree"));
    assert!(!contains_key(&elements, "__devtools-back"));
}
