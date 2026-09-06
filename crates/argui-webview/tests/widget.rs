use argui_webview::{WebView, WebViewSource, WebViewState};

#[test]
fn rebuilding_a_slot_keeps_identity_and_does_not_own_a_native_instance() {
    let state = WebViewState::new(WebViewSource::html("first"));
    let before = WebView::new(&state).build();
    state.load(WebViewSource::html("second")).unwrap();
    let after = WebView::new(&state.clone()).build();
    assert_eq!(before, after);
    assert!(after.children.is_empty());
    assert_eq!(after.user_select, argui_ui::UserSelect::None);
    let other = WebViewState::new(WebViewSource::html("second"));
    assert_ne!(state.id(), other.id());
    assert_ne!(after.key, WebView::new(&other).build().key);
}
