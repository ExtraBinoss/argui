use super::interaction::{click_count, key_input};
use super::*;

#[test]
fn selected_nodes_receive_reversible_typed_style_overrides() {
    let host = Entity::new(DevtoolsHost::new(App(Rc::new(Cell::new(0)))).open(true))
        .mount()
        .unwrap();
    let inspector = host.read(|tools| tools.inspector());
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
            portal: None,
            visible: true,
            painted: true,
            interactive: false,
            child_count: 0,
            properties: vec![PropertySnapshot {
                property: StyleProperty::Background,
                authored: true,
                value: StyleValue::Srgba([0.2, 0.4, 0.6, 1.0]),
            }],
        }],
    });
    change_tools(&host, |tools| {
        tools.update(&event(
            "__devtools-node-42",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        ))
    });
    assert!(contains_key(
        &host.render(Default::default()).unwrap(),
        "__devtools-swatch-42-background"
    ));
    change_tools(&host, |tools| {
        tools.update(&event(
            "__devtools-swatch-42-background",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        ))
    });
    change_tools(&host, |tools| {
        tools.update(&event(
            "__devtools-color-42-background::field::0",
            UiEventKind::TextChanged("#ff3366".into()),
        ))
    });
    assert!(matches!(
        inspector.property_value(InspectNodeId(42), StyleProperty::Background),
        Some(StyleValue::Srgba([red, _, _, _])) if (red - 1.0).abs() < 0.0001
    ));
    change_tools(&host, |tools| {
        tools.update(&event(
            "__devtools-style-background",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        ))
    });
    assert_eq!(inspector.selected(), Some(InspectNodeId(42)));
    assert_eq!(
        inspector.property_enabled(InspectNodeId(42), StyleProperty::Background),
        Some(false)
    );
    assert!(contains_key(
        &host.render(Default::default()).unwrap(),
        "__devtools-tree"
    ));
}

#[test]
fn elements_render_selection_highlight_and_every_style_control() {
    let host = Entity::new(populated_host()).mount().unwrap();
    let inspector = host.read(|tools| tools.inspector());
    change_tools(&host, |tools| {
        tools.update(&event(
            "__devtools-node-2",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        ))
    });
    let view = host.render(Default::default()).unwrap();
    assert!(contains_text(&view, "Reset overrides"));
    assert!(contains_key(&view, "__devtools-style-effects"));

    for property in StyleProperty::ALL {
        change_tools(&host, |tools| {
            tools.update(&event(
                &format!("__devtools-style-{}", property.label()),
                UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
            ))
        });
        assert_eq!(
            inspector.property_enabled(InspectNodeId(2), property),
            Some(false)
        );
    }
    inspector.select(Some(InspectNodeId(999)));
    assert!(contains_text(
        &host.render(Default::default()).unwrap(),
        "Select an element to inspect its styles"
    ));
    change_tools(&host, |tools| {
        tools.update(&event(
            "__devtools-node-not-a-number",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        ))
    });
    assert_eq!(inspector.selected(), None);
    change_tools(&host, |tools| {
        tools.update(&event(
            "__devtools-style-not-real",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        ))
    });
}

#[test]
fn live_profiler_records_do_not_invalidate_the_visible_snapshot() {
    let host = Entity::new(populated_host()).mount().unwrap();
    change_tools(&host, |tools| {
        tools.update(&event(
            "__devtools-profiling",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        ))
    });
    let mut tree = UiTree::new(host.render(Default::default()).unwrap());
    tree.mark_layout_clean();
    host.read(|tools| tools.inspector()).record_ui(FrameRecord {
        interval: StdDuration::from_millis(400),
        ..FrameRecord::default()
    });
    assert_eq!(
        tree.update(host.render(Default::default()).unwrap()),
        argui_ui::TreeUpdate::None
    );
    change_tools(&host, |tools| {
        tools.update(&event(
            "__devtools-refresh",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        ))
    });
    assert_eq!(
        tree.update(host.render(Default::default()).unwrap()),
        argui_ui::TreeUpdate::Layout
    );
}

#[test]
fn large_virtual_scrolls_rebuild_only_when_the_visible_window_changes() {
    let host = Entity::new(populated_host()).mount().unwrap();
    let mut snapshot = host.read(|tools| tools.inspector()).tree();
    snapshot.revision += 1;
    let template = snapshot.nodes[1].clone();
    for index in 3..600 {
        let mut node = template.clone();
        node.id = InspectNodeId(index);
        node.key = Some(format!("node-{index}"));
        snapshot.nodes.push(node);
    }
    host.read(|tools| tools.inspector()).publish_tree(snapshot);
    assert_eq!(
        change_tools(&host, |tools| tools.update(&event(
            "__devtools-tree",
            UiEventKind::Scrolled {
                delta: Point::new(0.0, 1_500.0),
                offset: Point::new(0.0, 1_500.0),
            },
        ))),
        ViewUpdate::Rebuild
    );

    for _ in 0..100 {
        host.read(|tools| tools.inspector())
            .record_ui(FrameRecord::default());
    }
    assert_eq!(
        change_tools(&host, |tools| tools.update(&event(
            "__devtools-refresh",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        ))),
        ViewUpdate::Rebuild
    );
    assert_eq!(
        change_tools(&host, |tools| tools.update(&event(
            "__devtools-frames",
            UiEventKind::Scrolled {
                delta: Point::new(0.0, 1_500.0),
                offset: Point::new(0.0, 1_500.0),
            },
        ))),
        ViewUpdate::Rebuild
    );
}

#[test]
fn tree_actions_cover_collapse_expand_parent_and_sibling_navigation() {
    let host = Entity::new(populated_host()).mount().unwrap();
    assert_eq!(
        change_tools(&host, |tools| tools
            .update(&event("__devtools-node-1", click_count(2)))),
        ViewUpdate::Rebuild
    );
    assert!(!contains_text(
        &host.render(Default::default()).unwrap(),
        "button  #save"
    ));

    assert_eq!(
        change_tools(&host, |tools| tools.update(&event(
            "__devtools-node-1",
            key_input(argui_core::Key::ArrowRight)
        ))),
        ViewUpdate::Rebuild
    );
    assert!(contains_text(
        &host.render(Default::default()).unwrap(),
        "button  #save"
    ));
    assert_eq!(
        change_tools(&host, |tools| tools.update(&event(
            "__devtools-node-1",
            key_input(argui_core::Key::ArrowRight)
        ))),
        ViewUpdate::Rebuild
    );
    assert_eq!(
        host.read(|tools| tools.inspector()).selected(),
        Some(InspectNodeId(2))
    );
    assert!(matches!(
        change_tools(&host, |tools| tools.take_focus_request()),
        Some(argui_ui::FocusRequest::Focus(argui_ui::FocusTarget::Key(key)))
            if key == "__devtools-node-2"
    ));
    assert_eq!(
        change_tools(&host, |tools| tools.update(&event(
            "__devtools-node-2",
            key_input(argui_core::Key::ArrowLeft)
        ))),
        ViewUpdate::Rebuild
    );
    assert_eq!(
        host.read(|tools| tools.inspector()).selected(),
        Some(InspectNodeId(1))
    );
    for navigation in [argui_core::Key::ArrowDown, argui_core::Key::End] {
        assert_eq!(
            change_tools(&host, |tools| tools
                .update(&event("__devtools-node-1", key_input(navigation)))),
            ViewUpdate::Rebuild
        );
        assert_eq!(
            host.read(|tools| tools.inspector()).selected(),
            Some(InspectNodeId(2))
        );
    }
    assert_eq!(
        change_tools(&host, |tools| tools.update(&event(
            "__devtools-node-2",
            key_input(argui_core::Key::Home)
        ))),
        ViewUpdate::Rebuild
    );
    assert_eq!(
        host.read(|tools| tools.inspector()).selected(),
        Some(InspectNodeId(1))
    );
}
