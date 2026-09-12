use argui_webview::WebViewError;

#[test]
fn webview_error_display_keeps_variant_context() {
    let error = WebViewError::Native(String::from("engine stopped"));
    assert_eq!(
        error.to_string(),
        "WebView error: Native(\"engine stopped\")"
    );

    let source: &dyn std::error::Error = &error;
    assert_eq!(source.to_string(), error.to_string());
}
