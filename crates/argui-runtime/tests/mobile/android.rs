#[test]
fn target_detection_matches_the_compiler_target() {
    assert!(argui_runtime::mobile::android::is_android());
}

/// Accepts Android's activity handle to type-check the exported macro contract.
///
/// `app` is supplied by Android; this test entry returns immediately.
fn launch(_app: argui_runtime::mobile::android::AndroidApp) -> Result<(), std::io::Error> {
    Ok(())
}

argui_runtime::android_main!(launch);
