use argui_webview::{PopupPolicy, WebCompatibility, WebViewOptions, WebViewSource, WebViewState};

#[test]
fn email_cannot_be_relaxed_or_used_for_webpages() {
    let email = WebViewSource::html("<p>Mail</p>");
    let relay = WebCompatibility::compatible("https://relay.example/").unwrap();
    for options in [
        WebViewOptions::email().compatibility(relay),
        WebViewOptions::email().popups(PopupPolicy::Sandboxed),
        WebViewOptions::email().allow_downloads(true),
        WebViewOptions::webpage(),
    ] {
        assert!(WebViewState::with_options(email.clone(), options).is_err());
    }
    assert_eq!(WebViewOptions::email().sandbox(), "allow-same-origin");
    assert!(
        WebViewOptions::email()
            .validate(&WebViewSource::url("https://example.com").unwrap())
            .is_err()
    );
}

#[test]
fn webpage_permissions_are_explicit_and_persist_across_navigation() {
    let options = WebViewOptions::webpage()
        .compatibility(WebCompatibility::compatible("https://relay.example/").unwrap())
        .popups(PopupPolicy::External)
        .allow_downloads(true);
    assert_eq!(
        options.sandbox(),
        "allow-scripts allow-forms allow-same-origin allow-popups allow-popups-to-escape-sandbox allow-downloads"
    );
    assert!(options.validate_native().is_err());
    let state = WebViewState::with_options(
        WebViewSource::url("https://example.com").unwrap(),
        options.clone(),
    )
    .unwrap();
    state
        .load(WebViewSource::url("https://example.org").unwrap())
        .unwrap();
    state.reload();
    assert_eq!(state.options(), options);
    assert!(state.load(WebViewSource::html("Mail")).is_err());
    assert_eq!(state.options(), options);
    assert!(WebViewOptions::webpage().validate_native().is_ok());
    assert_eq!(
        WebViewOptions::webpage()
            .popups(PopupPolicy::Sandboxed)
            .sandbox(),
        "allow-scripts allow-forms allow-popups"
    );
}

#[test]
fn relay_rejects_unsafe_and_ambiguous_urls() {
    for url in [
        "/relay",
        "file:///relay",
        "javascript:alert(1)",
        "https://u:p@relay.example",
        "https://relay.example/?app=other",
        "https://relay.example/#other",
    ] {
        assert!(WebCompatibility::compatible(url).is_err(), "{url}");
    }
    let forged = WebCompatibility::Compatible {
        relay: url::Url::parse("file:///relay").unwrap(),
    };
    assert!(
        WebViewOptions::webpage()
            .compatibility(forged)
            .validate(&WebViewSource::url("https://example.com").unwrap())
            .is_err()
    );
    let state = WebViewState::new(WebViewSource::url("https://example.com").unwrap());
    assert_eq!(state.options().sandbox(), "allow-scripts allow-forms");
    assert!(!state.options().downloads_allowed());
    assert_eq!(state.options().popup_policy(), PopupPolicy::Block);
}
