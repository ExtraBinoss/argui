#![cfg(not(target_arch = "wasm32"))]

#[test]
fn empty_private_session_clipboard_reports_an_error_without_panicking() {
    if std::env::var_os("ARGUI_NATIVE_TESTS").is_none() {
        return;
    }
    assert_eq!(std::env::var("ARGUI_HIDDEN_DISPLAY").as_deref(), Ok("1"));
    assert!(std::env::var_os("DISPLAY").is_none());
    assert!(
        std::path::Path::new(&std::env::var("XDG_RUNTIME_DIR").unwrap())
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with("argui-display.")
    );
    let error = argui_platform::Clipboard::new().read_text().unwrap_err();
    assert!(!error.to_string().is_empty());
}
