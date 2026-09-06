use argui_ui::{Element, NativeContent};
use std::{cell::Cell, rc::Rc};

#[test]
fn native_content_keeps_typed_retained_identity_across_element_rebuilds() {
    let state = Rc::new(Cell::new(1));
    let content = NativeContent::new(42, state.clone());
    let element = Element::container([]).native_content(content.clone());
    state.set(3);
    assert_eq!(content.id(), 42);
    assert_eq!(
        element
            .native_content
            .as_ref()
            .unwrap()
            .downcast_ref::<Rc<Cell<i32>>>()
            .unwrap()
            .get(),
        3
    );
    assert_eq!(content, NativeContent::new(42, state.clone()));
    assert_ne!(content, NativeContent::new(43, state));
    assert_ne!(content, NativeContent::new(42, "different type"));
    assert!(content.downcast_ref::<String>().is_none());
    assert!(format!("{content:?}").contains("42"));
    assert_eq!(element, element.clone());
}
