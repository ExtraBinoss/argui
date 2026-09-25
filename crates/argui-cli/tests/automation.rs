#![cfg(unix)]

use std::{fs, os::unix::fs::PermissionsExt, path::Path, process::Command};

/// Writes an executable process fixture under `path`.
fn script(path: &Path, contents: &str) {
    fs::write(path, contents).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

/// Creates a tiny native project with fake build tools and a report-writing host.
fn fixture(root: &Path) {
    for path in [
        "apps/demo/src",
        "apps/demo/node_modules/@argui/solid",
        "apps/gallery/quickjs-host",
        "apps/gallery/src",
        "apps/gallery/node_modules/@argui/solid",
        "apps/gallery/dist",
        "packages/host",
        "node_modules/typescript",
        "node_modules/vite",
        "bin",
    ] {
        fs::create_dir_all(root.join(path)).unwrap();
    }
    fs::write(root.join("packages/host/package.json"), "{}").unwrap();
    fs::write(root.join("apps/gallery/quickjs-host/Cargo.toml"), "").unwrap();
    fs::write(root.join("apps/gallery/dist/gallery-core.mjs"), "").unwrap();
    fs::write(
        root.join("apps/gallery/argui.json"),
        r#"{"name":"gallery","framework":"solid"}"#,
    )
    .unwrap();
    fs::write(
        root.join("apps/gallery/src/main.tsx"),
        "export function mountGallery() {}\n",
    )
    .unwrap();
    fs::write(
        root.join("apps/demo/argui.json"),
        r#"{"name":"demo","framework":"solid"}"#,
    )
    .unwrap();
    fs::write(
        root.join("apps/demo/src/main.tsx"),
        "export function mountGallery() {}\n",
    )
    .unwrap();
    fs::write(root.join("demo.test.ts"), "export default {}\n").unwrap();
    script(
        &root.join("bin/bun"),
        r##"#!/bin/sh
case "$1" in
  --version) echo 1.4.0 ;;
  *vite.js) mkdir -p dist; printf 'export function mountGallery() {}\n' > dist/app.mjs; if [ "${FAKE_BUN_DELETE:-}" = 1 ]; then rm "$0"; fi ;;
  *) if [ "${FAKE_TEST_BUILD_FAIL:-}" = 1 ]; then exit 2; fi; exit 0 ;;
esac
"##,
    );
    script(
        &root.join("bin/cargo"),
        r##"#!/bin/sh
while [ "$#" -gt 0 ]; do
  if [ "$1" = --target-dir ]; then shift; target="$1"; fi
  shift
done
mkdir -p "$target/debug"
cat > "$target/debug/argui-gallery-quickjs" <<'HOST'
#!/bin/sh
if [ "${FAKE_HOST_EARLY_EXIT:-}" = 1 ]; then exit 3; fi
dd bs=1 count=1 of=/dev/null 2>/dev/null
sleep 0.35
if [ "${FAKE_HOST_NOREPORT:-}" = 1 ]; then exit 0; fi
started=$(($(date +%s) * 1000))
if [ "${FAKE_HOST_EMPTY_STEPS:-}" = 1 ]; then
  printf '{"ok":true,"startedUnixMs":%s,"steps":[]}\n' "$started" > "$ARGUI_AUTOMATION_OUT/report.json"
  exit 0
fi
if [ "${FAKE_AUTOMATION_FAIL:-}" = 1 ]; then
  printf '{"ok":false,"error":"assertion failed","startedUnixMs":%s,"steps":[{"started_ms":0,"duration_ms":1,"ok":false}]}\n' "$started" > "$ARGUI_AUTOMATION_OUT/report.json"
  exit 2
fi
printf '{"ok":true,"startedUnixMs":%s,"steps":[{"started_ms":0,"duration_ms":1,"ok":true}],"frames":[{}],"galleryAssets":%s}\n' "$started" "${ARGUI_AUTOMATION_GALLERY_ASSETS:-0}" > "$ARGUI_AUTOMATION_OUT/report.json"
HOST
chmod +x "$target/debug/argui-gallery-quickjs"
if [ "${FAKE_HOST_UNEXEC:-}" = 1 ]; then chmod 644 "$target/debug/argui-gallery-quickjs"; fi
"##,
    );
}

/// Invokes the CLI binary in `root` with the fixture tools ahead of PATH.
fn cli(root: &Path, args: &[&str], extra: &[(&str, &str)]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_argui"))
        .args(args)
        .current_dir(root)
        .env(
            "PATH",
            format!("{}:/usr/bin:/bin", root.join("bin").display()),
        )
        .envs(extra.iter().copied())
        .output()
        .unwrap()
}

#[test]
fn test_command_reports_arguments_and_native_process_metrics() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root);
    for args in [
        vec!["test"],
        vec!["test", "demo.txt", "--out", "artifacts"],
        vec!["test", "missing.test.ts", "--out", "artifacts"],
        vec![
            "test",
            "apps/demo",
            "demo.test.ts",
            "--out",
            "artifacts",
            "--out",
            "again",
        ],
        vec!["test", "apps/demo", "demo.test.ts", "--out"],
    ] {
        assert!(!cli(root, &args, &[]).status.success());
    }
    fs::write(
        root.join("apps/demo/argui.json"),
        r#"{"name":"demo","framework":"solid","target":"web"}"#,
    )
    .unwrap();
    let web = cli(
        root,
        &["test", "apps/demo", "demo.test.ts", "--out", "web-test"],
        &[],
    );
    assert!(!web.status.success());
    assert!(String::from_utf8_lossy(&web.stderr).contains("native TSX apps"));
    fs::write(
        root.join("apps/demo/argui.json"),
        r#"{"name":"demo","framework":"solid"}"#,
    )
    .unwrap();
    let args = ["test", "apps/demo", "demo.test.ts", "--out", "artifacts"];
    let passed = cli(root, &args, &[]);
    assert!(
        passed.status.success(),
        "{}",
        String::from_utf8_lossy(&passed.stderr)
    );
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("artifacts/report.json")).unwrap()).unwrap();
    assert_eq!(report["ok"], true);
    assert_eq!(report["galleryAssets"], 0);
    assert!(report["processMetrics"]["hostPid"].as_u64().unwrap() > 0);
    assert!(report["processMetrics"]["peakRssBytes"].as_u64().unwrap() > 0);
    assert!(
        report["processMetrics"]["samples"]
            .as_array()
            .unwrap()
            .len()
            >= 2
    );
    let failed = cli(root, &args, &[("FAKE_AUTOMATION_FAIL", "1")]);
    assert!(!failed.status.success());
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("artifacts/report.json")).unwrap()).unwrap();
    assert_eq!(report["ok"], false);
    assert!(
        report["error"]
            .as_str()
            .unwrap()
            .contains("assertion failed")
    );
    let empty_steps = cli(root, &args, &[("FAKE_HOST_EMPTY_STEPS", "1")]);
    assert!(empty_steps.status.success());
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("artifacts/report.json")).unwrap()).unwrap();
    assert!(report["steps"].as_array().unwrap().is_empty());
    assert!(report["processMetrics"]["samples"].as_array().is_some());
    fs::write(root.join("demo.test.tsx"), "export default {}\n").unwrap();
    let tsx = cli(
        root,
        &["test", "apps/demo", "demo.test.tsx", "--out", "tsx"],
        &[],
    );
    assert!(tsx.status.success());
    let gallery = cli(
        root,
        &[
            "test",
            "apps/gallery",
            "demo.test.ts",
            "--out",
            "gallery-assets",
        ],
        &[],
    );
    assert!(
        gallery.status.success(),
        "{}",
        String::from_utf8_lossy(&gallery.stderr)
    );
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("gallery-assets/report.json")).unwrap())
            .unwrap();
    assert_eq!(report["galleryAssets"], 1);
}

#[test]
fn test_command_names_build_host_and_report_failures() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root);
    fs::write(root.join("blocked"), "file").unwrap();
    assert!(
        !cli(
            root,
            &["test", "apps/demo", "demo.test.ts", "--out", "blocked"],
            &[]
        )
        .status
        .success()
    );
    fs::create_dir_all(root.join("bad-bundle")).unwrap();
    fs::write(root.join("bad-bundle/.argui-bundle"), "file").unwrap();
    assert!(
        !cli(
            root,
            &["test", "apps/demo", "demo.test.ts", "--out", "bad-bundle"],
            &[]
        )
        .status
        .success()
    );
    let args = ["test", "apps/demo", "demo.test.ts", "--out", "host-failure"];
    let unexecutable = cli(root, &args, &[("FAKE_HOST_UNEXEC", "1")]);
    assert!(!unexecutable.status.success());
    assert!(String::from_utf8_lossy(&unexecutable.stderr).contains("argui-gallery-quickjs"));
    let early = cli(root, &args, &[("FAKE_HOST_EARLY_EXIT", "1")]);
    assert!(!early.status.success());
    let missing_report = cli(
        root,
        &["test", "apps/demo", "demo.test.ts", "--out", "no-report"],
        &[("FAKE_HOST_NOREPORT", "1")],
    );
    assert!(!missing_report.status.success());
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("no-report/report.json")).unwrap()).unwrap();
    assert!(
        report["error"]
            .as_str()
            .unwrap()
            .contains("before writing its report")
    );
    fs::create_dir_all(root.join("report-blocked/report.json")).unwrap();
    let report_blocked = cli(
        root,
        &[
            "test",
            "apps/demo",
            "demo.test.ts",
            "--out",
            "report-blocked",
        ],
        &[],
    );
    assert!(!report_blocked.status.success());
    assert!(String::from_utf8_lossy(&report_blocked.stderr).contains("report.json"));
    let failed_bundle = cli(
        root,
        &[
            "test",
            "apps/demo",
            "demo.test.ts",
            "--out",
            "failed-bundle",
        ],
        &[("FAKE_TEST_BUILD_FAIL", "1")],
    );
    assert!(!failed_bundle.status.success());
    assert!(String::from_utf8_lossy(&failed_bundle.stderr).contains("test bundle build failed"));

    let separate = tempfile::tempdir().unwrap();
    fixture(separate.path());
    let unavailable_bun = cli(
        separate.path(),
        &["test", "apps/demo", "demo.test.ts", "--out", "bun-failure"],
        &[("FAKE_BUN_DELETE", "1")],
    );
    assert!(!unavailable_bun.status.success());
    assert!(String::from_utf8_lossy(&unavailable_bun.stderr).contains("Bun could not build"));
}

#[test]
fn screenshot_command_rejects_invalid_outputs_before_building() {
    let temp = tempfile::tempdir().unwrap();
    fixture(temp.path());
    for args in [
        vec!["screenshot", "apps/demo"],
        vec!["screenshot", "apps/demo", "--out", "capture.txt"],
        vec!["screenshot", "apps/demo", "extra", "--out", "capture.png"],
    ] {
        assert!(!cli(temp.path(), &args, &[]).status.success());
    }
    let output = cli(
        temp.path(),
        &["screenshot", "apps/demo", "--out", "captures/current.png"],
        &[],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(temp.path().join("captures/report.json").is_file());
    assert!(
        !temp
            .path()
            .join("captures/.argui-screenshot.test.ts")
            .exists()
    );
    let occupied = temp.path().join("captures/.argui-screenshot.test.ts");
    fs::create_dir(&occupied).unwrap();
    let blocked_script = cli(
        temp.path(),
        &["screenshot", "apps/demo", "--out", "captures/blocked.png"],
        &[],
    );
    assert!(!blocked_script.status.success());
    assert!(String::from_utf8_lossy(&blocked_script.stderr).contains(".argui-screenshot.test.ts"));
    fs::remove_dir(&occupied).unwrap();
    fs::write(temp.path().join("blocked"), "file").unwrap();
    assert!(
        !cli(
            temp.path(),
            &["screenshot", "apps/demo", "--out", "blocked/current.png"],
            &[]
        )
        .status
        .success()
    );
    fs::remove_file(temp.path().join("apps/demo/src/main.tsx")).unwrap();
    assert!(
        !cli(
            temp.path(),
            &[
                "screenshot",
                "apps/demo",
                "--out",
                "missing-source/current.png"
            ],
            &[]
        )
        .status
        .success()
    );
}
