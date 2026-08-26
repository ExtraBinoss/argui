use argui_ui::{Element, ElementKind};

#[test]
fn rust_builders_form_the_future_dsl_lowering_target() {
    let tree = Element::container([Element::text("hello").keyed("greeting")]);

    assert_eq!(tree.kind, ElementKind::Container);
    assert_eq!(tree.children.len(), 1);
    assert_eq!(tree.children[0].key.as_deref(), Some("greeting"));
}
