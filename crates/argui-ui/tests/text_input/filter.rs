use super::*;

#[test]
fn code_filters_apply_before_paste_ime_and_typing() {
    let mut input = Input::new(
        "field",
        "",
        "",
        InputStyle::new(PaintStyle::default(), TextStyle::default()),
    )
    .build();
    if let ElementKind::TextEditor { filter, .. } = &mut input.kind {
        *filter = argui_ui::TextInputFilter::Digits { max_length: 6 };
    }
    let (mut tree, region) = with_region(UiTree::new(listens(input)));
    focus(&mut tree, &region);
    assert!(tree.paste_text(None, "12345").layout_changed);
    assert!(!tree.paste_text(None, "ab").layout_changed);
    assert!(!tree.ime_input(ImeInput::Commit("６".into())).layout_changed);
    tree.edit_text_input(&key(
        Key::Character("6".into()),
        Some("6"),
        Modifiers::default(),
    ));
    assert_eq!(tree.text_input_value(region.node), Some("123456"));
    assert!(!tree.paste_text(None, "7").layout_changed);
    assert_eq!(tree.text_input_value(region.node), Some("123456"));
}
