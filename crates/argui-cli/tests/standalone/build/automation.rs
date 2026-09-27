use super::{cli, fixture};
use std::{fs, process::Output};

#[test]
/// Automation reports stage and host failures without losing the original test file.
fn automation_failures_identify_the_stage_and_preserve_source() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root, "react", "native", &["automation"]);
    let app = root.join("app");
    let source = app.join("src/failure.test.ts");
    fs::write(&source, "export default {}\n").unwrap();
    let test = |output: &str, flags: &[(&str, &str)]| {
        cli(
            root,
            &["test", "src/failure.test.ts", "--out", output],
            flags,
        )
    };
    let fails_with = |output: Output, expected: &str| {
        assert!(!output.status.success());
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(expected),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    };

    fs::write(app.join("blocked-output"), "occupied").unwrap();
    fails_with(test("blocked-output", &[]), "blocked-output");

    let bundle = app.join("blocked-bundle/.argui-bundle");
    fs::create_dir_all(bundle.parent().unwrap()).unwrap();
    fs::write(&bundle, "occupied").unwrap();
    fails_with(test("blocked-bundle", &[]), "File exists");

    let script = app.join("node_modules/.argui-build-test.mjs");
    fs::create_dir(&script).unwrap();
    fails_with(test("blocked-script", &[]), ".argui-build-test.mjs");
    fs::remove_dir(&script).unwrap();

    fails_with(
        test("failed-bundle", &[("ARGUI_FAKE_BUNDLE_FAIL", "1")]),
        "test bundle build failed",
    );
    fails_with(
        test("not-executable", &[("ARGUI_FAKE_NO_EXEC", "1")]),
        "Permission denied",
    );
    fails_with(
        test("closed-gate", &[("ARGUI_FAKE_HOST_EARLY_EXIT", "1")]),
        "host start gate",
    );

    fs::create_dir_all(app.join("blocked-report/report.json")).unwrap();
    fails_with(test("blocked-report", &[]), "report.json");

    fs::create_dir_all(app.join("captures/.argui-screenshot.test.ts")).unwrap();
    fails_with(
        cli(root, &["screenshot", "--out", "captures/frame.png"], &[]),
        ".argui-screenshot.test.ts",
    );
    assert_eq!(fs::read_to_string(source).unwrap(), "export default {}\n");
}
