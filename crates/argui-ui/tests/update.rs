use argui_ui::{Color, Element, TreeUpdate, UiTree, length};

fn screen(color: Color) -> Element {
    Element::row([
        Element::column([Element::text("Volume"), Element::text("55 %")]).keyed("controls"),
        Element::container([])
            .keyed("animation")
            .width(length(80.0))
            .background(color),
    ])
}

#[test]
fn rebuilding_an_animation_retains_unchanged_controls_and_their_descendants() {
    let mut tree = UiTree::new(screen(Color::BLACK));
    let controls = tree.root().children[0].clone();
    let animation = tree.root().children[1].clone();
    let ids = tree.node_ids().to_vec();
    assert_eq!(tree.update(screen(Color::WHITE)), TreeUpdate::Paint);
    assert!(tree.root().children[0].ptr_eq(&controls));
    assert!(tree.root().children[0].children[1].ptr_eq(&controls.children[1]));
    assert!(!tree.root().children[1].ptr_eq(&animation));
    assert_eq!(tree.node_ids(), ids);
    assert_eq!(tree.update(screen(Color::WHITE)), TreeUpdate::None);
    assert!(tree.root().children[0].ptr_eq(&controls));
}

#[test]
fn retaining_equal_subtrees_does_not_hide_semantic_or_layout_edits() {
    let mut tree = UiTree::new(screen(Color::BLACK));
    let mut next = screen(Color::WHITE);
    next.children[0].semantics =
        Some(argui_ui::Semantics::new(argui_ui::Role::Group).label("Mixer"));
    assert_eq!(tree.update(next.clone()), TreeUpdate::Paint);
    assert_eq!(
        tree.root().children[0].semantics,
        next.children[0].semantics
    );
    next.children[0].children.push(Element::text("Pan"));
    assert_eq!(tree.update(next), TreeUpdate::Layout);
    assert_eq!(tree.root().children[0].children.len(), 3);
}
