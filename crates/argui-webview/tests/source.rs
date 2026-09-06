use argui_webview::{WebViewPolicy, WebViewSource};

#[test]
fn only_http_and_https_can_be_browser_sources() {
    for value in ["https://example.com", "http://localhost:8080/path"] {
        let source = WebViewSource::url(value).unwrap();
        assert_eq!(source.policy(), WebViewPolicy::Browser);
        assert!(source.sanitized_html().is_none());
        assert!(WebViewPolicy::Browser.allows_navigation(value));
    }
    for value in [
        "relative",
        "file:///etc/passwd",
        "javascript:alert(1)",
        "data:text/html,hi",
        "ftp://example.com",
    ] {
        assert!(WebViewSource::url(value).is_err());
        assert!(!WebViewPolicy::Browser.allows_navigation(value));
    }
}

#[test]
fn html_policy_rejects_external_navigation_and_internal_origin_spoofing() {
    assert!(
        WebViewPolicy::RestrictedHtml
            .allows_navigation("argui-content://localhost/document?revision=1")
    );
    assert_eq!(
        WebViewPolicy::RestrictedHtml.allows_navigation("http://argui-content.localhost/document"),
        cfg!(target_os = "windows")
    );
    for value in [
        "invalid",
        "https://example.com/document",
        "argui-content://evil/document",
        "argui-content://localhost/file",
        "argui-content://user@localhost/document",
        "argui-content://localhost:8080/document",
        "http://argui-content.localhost.evil/document",
        "javascript:alert(1)",
    ] {
        assert!(
            !WebViewPolicy::RestrictedHtml.allows_navigation(value),
            "{value}"
        );
    }
}

#[test]
fn html_sanitization_keeps_text_and_removes_active_markup() {
    let source = WebViewSource::html(
        "<h1>Hello</h1><script>alert(1)</script><img src=x onerror='alert(2)'><iframe src='https://evil'></iframe><form action='https://evil'><input></form><a href='javascript:alert(3)'>link</a>",
    );
    let html = source.sanitized_html().unwrap();
    assert!(html.contains("<h1>Hello</h1>"));
    for forbidden in [
        "<script",
        "alert(",
        "onerror",
        "<iframe",
        "<form",
        "<input",
        "javascript:",
    ] {
        assert!(!html.contains(forbidden), "{html}");
    }
    assert_eq!(source.policy(), WebViewPolicy::RestrictedHtml);
}
