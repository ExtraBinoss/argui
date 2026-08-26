use argui_text::TextStyle;
use argui_ui::{
    Border, ClipBehavior, Color, CornerRadii, Direction, Element, ElementKind, Length, UiTree,
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
