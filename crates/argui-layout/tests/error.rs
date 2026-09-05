use argui_layout::LayoutError;

#[test]
fn missing_roots_have_actionable_context() {
    let error = LayoutError::MissingRoot;

    assert_eq!(error.to_string(), "layout tree has no root");
    assert!(std::error::Error::source(&error).is_none());
}

#[test]
fn identity_and_container_query_errors_explain_the_failed_layout() {
    let identity = LayoutError::MissingNodeIdentity(17);
    assert_eq!(identity.to_string(), "UI node 17 has no stable identity");
    assert!(std::error::Error::source(&identity).is_none());

    let queries = LayoutError::NonConvergentContainerQueries;
    assert_eq!(
        queries.to_string(),
        "container queries did not converge after four layout passes"
    );
    assert!(std::error::Error::source(&queries).is_none());
}
