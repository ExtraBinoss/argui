use argui_ui::{TextEdit, TextEditError};

#[test]
fn edits_apply_only_the_addressed_utf8_range() {
    let mut value = "zero café tail".to_owned();
    TextEdit::new(5..10, "tea").apply_to(&mut value).unwrap();
    assert_eq!(value, "zero tea tail");
}

#[test]
fn invalid_ranges_leave_the_value_unchanged() {
    let mut value = "éclair".to_owned();
    assert_eq!(
        TextEdit::new(1..2, "x").apply_to(&mut value),
        Err(TextEditError::NotCharBoundary { index: 1 })
    );
    assert_eq!(value, "éclair");
    assert!(matches!(
        TextEdit::new(8..9, "x").apply_to(&mut value),
        Err(TextEditError::OutOfBounds { .. })
    ));
    assert_eq!(value, "éclair");
}
