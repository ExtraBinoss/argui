use argui_text::TextStyle;
use argui_ui::{
    Border, ClipBehavior, Color, CornerRadii, Direction, Element, ElementKind, Interaction,
    LayerStyle, Length, TreeUpdate, UiTree, VectorId,
};

#[test]
fn rust_builders_form_the_future_dsl_lowering_target() {
    let root = Element::row([Element::text("hello")
        .keyed("greeting")
        .width(Length::Percent(0.5))
        .text_style(TextStyle {
            weight: 700,
            ..TextStyle::default()
        })])
    .background(Color::rgb(0.1, 0.2, 0.3))
    .border(Border::all(2.0, Color::WHITE))
    .radius(CornerRadii::all(8.0))
    .paint_opacity(0.9)
    .clip(ClipBehavior::Bounds);
    let mut tree = UiTree::new(root.clone());

    assert_eq!(tree.root().kind, ElementKind::Container);
    assert_eq!(tree.root().style.direction, Direction::Row);
    assert_eq!(tree.root().children.len(), 1);
    assert_eq!(tree.root().children[0].key.as_deref(), Some("greeting"));
    assert!(tree.root().paint.is_visible());
    assert!(tree.layout_dirty());
    tree.mark_layout_clean();
    assert!(!tree.layout_dirty());
    assert!(!tree.replace(root));
    assert!(tree.replace(Element::text("changed")));
    assert_eq!(tree.revision(), 1);
    assert!(tree.layout_dirty());
}

#[test]
fn vector_builder_keeps_a_gpu_morph_progress_value() {
    let element = Element::vector(VectorId(9)).vector_progress(0.75);
    assert_eq!(
        element.kind,
        ElementKind::Vector {
            vector: VectorId(9),
            progress: 0.75,
        }
    );
}

#[test]
fn vector_morph_progress_is_a_paint_only_update() {
    let mut tree = UiTree::new(Element::vector(VectorId(9)).vector_progress(0.0));
    let node = tree.node_id_at(0);
    tree.mark_layout_clean();
    assert_eq!(
        tree.update(Element::vector(VectorId(9)).vector_progress(0.5)),
        TreeUpdate::Paint
    );
    assert_eq!(tree.node_id_at(0), node);
    assert!(!tree.layout_dirty());
}

#[test]
fn tree_updates_distinguish_paint_from_layout() {
    let base = Element::container([])
        .keyed("panel")
        .background(Color::rgb(0.1, 0.2, 0.3));
    let mut tree = UiTree::new(base.clone());
    tree.mark_layout_clean();
    let revision = tree.revision();
    let node = tree.node_id_at(0);

    assert_eq!(
        tree.update(base.clone().background(Color::rgb(0.3, 0.2, 0.1))),
        TreeUpdate::Paint
    );
    assert_eq!(tree.revision(), revision);
    assert_eq!(tree.node_id_at(0), node);
    assert!(!tree.layout_dirty());

    assert_eq!(
        tree.update(
            base.clone()
                .text_effect(LayerStyle::new(Default::default()).opacity(0.7))
        ),
        TreeUpdate::Paint
    );
    assert_eq!(tree.revision(), revision);

    assert_eq!(
        tree.update(base.width(Length::Px(200.0))),
        TreeUpdate::Layout
    );
    assert!(tree.revision() > revision);
    assert!(tree.layout_dirty());
}

#[test]
fn enabling_retained_interaction_only_requires_repaint() {
    let mut tree =
        UiTree::new(Element::container([]).interaction(Interaction::blocker().enabled(false)));
    assert_eq!(
        tree.update(Element::container([]).interaction(Interaction::blocker())),
        TreeUpdate::Paint
    );
}
