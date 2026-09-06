use argui_webview::email_document;

#[test]
fn hostile_email_cannot_replace_policy_or_execute_content() {
    let document = email_document(
        r#"</body></html><meta http-equiv="refresh" content="0;url=https://evil.test"><base href="https://evil.test"><script>alert(1)</script><iframe src="https://evil.test"></iframe><form action="https://evil.test"><input></form><p onclick="alert(1)">Hello</p><a href="javascript:alert(1)">bad</a><a href="https://example.com">good</a><style>body{background:url(https://evil.test)}</style>"#,
    );
    for forbidden in [
        "<script",
        "<iframe",
        "<form",
        "<input",
        "<base",
        "onclick",
        "javascript:",
        "http-equiv=\"refresh\"",
        "background:url",
    ] {
        assert!(!document.contains(forbidden), "{forbidden}");
    }
    assert_eq!(document.matches("Content-Security-Policy").count(), 1);
    assert!(document.contains("default-src 'none'"));
    assert!(document.contains("script-src 'none'"));
    assert!(document.contains("img-src data:"));
    assert!(document.contains("Hello"));
    assert!(document.contains("href=\"https://example.com\""));
    assert!(document.contains("noopener noreferrer"));
}

#[test]
fn empty_and_unicode_emails_are_complete_documents() {
    assert!(email_document("").ends_with("<body></body></html>"));
    assert!(email_document("<p>Bonjour 👋 &amp; merci</p>").contains("Bonjour 👋 &amp; merci"));
}
