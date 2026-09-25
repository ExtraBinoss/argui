#[test]
fn target_detection_matches_the_compiler_target() {
    assert!(argui_runtime::mobile::ios::is_ios());
}

/// Type-checks the C-callable iOS entry macro without starting UIKit.
///
/// Returns immediately because this is a compile-time entry contract check.
fn launch() -> Result<(), std::io::Error> {
    Ok(())
}

argui_runtime::ios_main!(start_argui_test_app, launch);
