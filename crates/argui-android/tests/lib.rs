#[test]
fn target_detection_matches_the_compiler_target() {
    assert_eq!(argui_android::is_android(), cfg!(target_os = "android"));
}
