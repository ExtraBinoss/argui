use argui_layout::LayoutError;

#[test]
fn missing_roots_have_actionable_context() {
    let error = LayoutError::MissingRoot;

    assert_eq!(error.to_string(), "layout tree has no root");
    assert!(std::error::Error::source(&error).is_none());
}
