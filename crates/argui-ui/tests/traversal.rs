use argui_animation::{Duration, Motion, Time, Tween};
use argui_core::{CaretAffinity, Color, TextPosition};
use argui_ui::{
    CaretStyle, Element, ElementKind, TextEditorSpec, TextInputFilter, TreeUpdate, UiTree, property,
};

fn editor(value: &str) -> Element {
    Element::text_editor(TextEditorSpec {
        value: value.to_owned(),
        placeholder: "placeholder".to_owned(),
        multiline: false,
        read_only: false,
        filter: TextInputFilter::Any,
        text: argui_text::TextStyle::default(),
        placeholder_text: argui_text::TextStyle::default(),
        selection: Color::TRANSPARENT,
        caret: CaretStyle::default(),
    })
    .keyed("editor")
}

#[test]
fn retained_preorder_services_nested_editors_and_animations() {
    let opacity = Motion::new(0.25_f32);
    let root = Element::column([
        Element::container([Element::text("label"), editor("initial")]),
        Element::container([Element::text("animated").bind(property::Opacity, opacity.clone())]),
    ]);
    let mut tree = UiTree::new(root.clone());

    assert_eq!(tree.node_ids().len(), 6);
    assert_eq!(
        tree.element_at(0).map(|element| &element.kind),
        Some(&ElementKind::Container)
    );
    let editor_node = tree.node_id_at(3).expect("nested editor");
    assert_eq!(tree.parent_of(editor_node), tree.node_id_at(1));
    assert_eq!(tree.key(editor_node), Some("editor"));
    assert_eq!(tree.text_input_value(editor_node), Some("initial"));
    assert_eq!(
        tree.text_input_display(editor_node).as_deref(),
        Some("initial")
    );
    assert_eq!(tree.text_input_cursor(editor_node), Some("initial".len()));
    assert_eq!(
        tree.text_input_position(editor_node),
        Some(TextPosition::new("initial".len(), CaretAffinity::Before))
    );
    assert_eq!(tree.animation_count(), 1);
    assert!(tree.layout_animation_indices().is_empty());

    opacity.animate_to(0.75, Tween::new(Duration::from_millis(40)));
    assert_eq!(
        tree.advance_animations(Time::from_nanos(1)),
        TreeUpdate::None
    );
    assert_eq!(
        tree.advance_animations(Time::from_nanos(20_000_001)),
        TreeUpdate::Paint
    );

    let updated = Element::column([
        Element::container([Element::text("label"), editor("updated")]),
        Element::container([Element::text("animated").bind(property::Opacity, opacity)]),
    ]);
    assert_eq!(tree.update(updated), TreeUpdate::Layout);
    assert_eq!(tree.text_input_value(editor_node), Some("updated"));
    assert!(tree.text_input_should_reveal_cursor(editor_node));
    tree.mark_text_input_layout_clean();
    assert!(!tree.text_input_should_reveal_cursor(editor_node));
    assert!(tree.element_at(100).is_none());
    assert!(tree.parent_of(tree.node_id_at(0).unwrap()).is_none());
    assert!(tree.node_id_at(3).is_some());
}
