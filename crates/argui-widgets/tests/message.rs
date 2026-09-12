use argui_ui::{Element, FlexDirection, UiTree};
use argui_widgets::Message;

#[test]
fn message_alignment_keeps_author_and_optional_metadata_in_reading_order() {
    for end in [false, true] {
        let mut message = Message::new("m", "Ada", Element::text("Hello"));
        message.end = end;
        message.avatar = Some(Element::text("AL"));
        message.header = Some(Element::text("Yesterday"));
        message.footer = Some(Element::text("Delivered"));
        let root = message.build();
        assert_eq!(
            root.style.flex_direction,
            if end {
                FlexDirection::RowReverse
            } else {
                FlexDirection::Row
            }
        );
        let tree = UiTree::new(root);
        let semantic = tree.semantic_tree(&[], 1.0);
        assert!(
            semantic
                .nodes
                .iter()
                .any(|node| node.semantics.label.as_deref() == Some("Ada"))
        );
        assert!(
            !semantic
                .nodes
                .iter()
                .any(|node| node.semantics.label.as_deref() == Some("AL"))
        );
    }
}
