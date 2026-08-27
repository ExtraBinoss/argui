use super::*;

#[test]
fn stable_topology_updates_taffy_in_place() {
    let root = |height| Element::container([]).height(Length::Px(height));
    let mut ui = UiTree::new(root(100.0));
    let mut engine = LayoutEngine::new();
    let mut text = TextEngine::new();
    engine
        .compute(&mut ui, &mut text, Size::new(400.0, 300.0))
        .unwrap();
    let taffy_root = engine.root.as_ref().unwrap().id;
    ui.update(root(180.0));
    engine
        .compute(&mut ui, &mut text, Size::new(400.0, 300.0))
        .unwrap();
    assert_eq!(engine.root.as_ref().unwrap().id, taffy_root);
}

#[test]
fn structural_updates_preserve_unchanged_taffy_branches() {
    let root = |key: &str| {
        Element::column([
            Element::text("stable").keyed("stable"),
            Element::text(key).keyed(key),
        ])
    };
    let mut ui = UiTree::new(root("first"));
    let mut engine = LayoutEngine::new();
    let mut text = TextEngine::new();
    engine
        .compute(&mut ui, &mut text, Size::new(400.0, 300.0))
        .unwrap();
    let stable = engine.root.as_ref().unwrap().children[0].id;
    let replaced = engine.root.as_ref().unwrap().children[1].id;

    ui.update(root("second"));
    engine
        .compute(&mut ui, &mut text, Size::new(400.0, 300.0))
        .unwrap();

    assert_eq!(engine.root.as_ref().unwrap().children[0].id, stable);
    assert_ne!(engine.root.as_ref().unwrap().children[1].id, replaced);
    assert_eq!(engine.tree.total_node_count(), 3);
}

#[test]
fn virtual_windows_recycle_only_their_changed_rows() {
    let list = |offset| {
        argui_ui::VirtualList::new(1_000_000, 36.0, 260.0).build("million-list", offset, |index| {
            Element::text(index.to_string()).keyed(format!("row-{index}"))
        })
    };
    let mut ui = UiTree::new(list(0.0));
    let mut engine = LayoutEngine::new();
    let mut text = TextEngine::new();
    engine
        .compute(&mut ui, &mut text, Size::new(400.0, 300.0))
        .unwrap();
    let scroll = engine.root.as_ref().unwrap().id;
    let spacer = engine.root.as_ref().unwrap().children[0].children[0].id;

    ui.update(list(20_000.0));
    engine
        .compute(&mut ui, &mut text, Size::new(400.0, 300.0))
        .unwrap();

    let root = engine.root.as_ref().unwrap();
    assert_eq!(root.id, scroll);
    assert_eq!(root.children[0].children[0].id, spacer);
}

#[test]
fn keyed_intrinsic_nodes_keep_taffy_identity_when_reordered_and_edited() {
    let root = |reverse: bool, suffix: &str| {
        let first = Element::text(format!("first{suffix}")).keyed("first");
        let second = Element::text(format!("second{suffix}")).keyed("second");
        if reverse {
            Element::column([second, first])
        } else {
            Element::column([first, second])
        }
    };
    let mut ui = UiTree::new(root(false, ""));
    let mut engine = LayoutEngine::new();
    let mut text = TextEngine::new();
    engine
        .compute(&mut ui, &mut text, Size::new(400.0, 300.0))
        .unwrap();
    let first = engine.root.as_ref().unwrap().children[0].id;
    let second = engine.root.as_ref().unwrap().children[1].id;

    ui.update(root(true, " edited"));
    engine
        .compute(&mut ui, &mut text, Size::new(400.0, 300.0))
        .unwrap();
    let reordered = engine.root.as_ref().unwrap();
    assert_eq!(reordered.children[0].id, second);
    assert_eq!(reordered.children[1].id, first);

    ui.update(root(true, " edited again"));
    engine
        .compute(&mut ui, &mut text, Size::new(400.0, 300.0))
        .unwrap();
    assert_eq!(engine.root.as_ref().unwrap().children[0].id, second);

    let containers = |reverse: bool| {
        let first = Element::container([]).keyed("first-container");
        let second = Element::container([]).keyed("second-container");
        if reverse {
            Element::column([second, first])
        } else {
            Element::column([first, second])
        }
    };
    let mut container_ui = UiTree::new(containers(false));
    let mut container_engine = LayoutEngine::new();
    container_engine
        .compute(&mut container_ui, &mut text, Size::new(400.0, 300.0))
        .unwrap();
    let first_container = container_engine.root.as_ref().unwrap().children[0].id;
    container_ui.update(containers(true));
    container_engine
        .compute(&mut container_ui, &mut text, Size::new(400.0, 300.0))
        .unwrap();
    assert_eq!(
        container_engine.root.as_ref().unwrap().children[1].id,
        first_container
    );
}
