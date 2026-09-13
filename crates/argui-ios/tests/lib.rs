#[test]
fn target_detection_matches_the_compiler_target() {
    assert_eq!(argui_ios::is_ios(), cfg!(target_os = "ios"));
}
