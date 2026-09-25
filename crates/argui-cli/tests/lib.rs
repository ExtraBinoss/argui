#![cfg(unix)]

use argui_cli::run_in;
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::Path,
    process::Command,
    thread,
    time::{Duration, SystemTime},
};

/// Writes one executable shell fixture to `path`.
fn script(path: &Path, contents: &str) {
    fs::write(path, contents).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

/// Creates a project and fake process tools so CLI lifecycle tests stay headless.
fn fixture(root: &Path) {
    fs::create_dir_all(root.join("apps/demo/src")).unwrap();
    fs::create_dir_all(root.join("apps/demo/node_modules/@argui/solid")).unwrap();
    fs::create_dir_all(root.join("apps/gallery/quickjs-host")).unwrap();
    fs::create_dir_all(root.join("apps/gallery/dist")).unwrap();
    fs::create_dir_all(root.join("packages/host")).unwrap();
    fs::create_dir_all(root.join("node_modules/typescript")).unwrap();
    fs::create_dir_all(root.join("node_modules/vite")).unwrap();
    fs::create_dir_all(root.join("bin")).unwrap();
    fs::write(root.join("packages/host/package.json"), "{}").unwrap();
    fs::write(root.join("apps/gallery/quickjs-host/Cargo.toml"), "").unwrap();
    fs::write(root.join("apps/gallery/dist/gallery-core.mjs"), "").unwrap();
    fs::write(
        root.join("apps/demo/argui.json"),
        r#"{"name":"demo","framework":"solid"}"#,
    )
    .unwrap();
    script(
        &root.join("bin/bun"),
        r##"#!/bin/sh
case "$1" in
  --version) if [ "${FAKE_BUN_DELETE:-}" = 1 ]; then rm "$0"; fi; echo 1.4.0 ;;
  *tsc) if [ "${FAKE_TSC_FAIL:-}" = 1 ]; then echo 'src/main.tsx(2,3): error TS0001: sample error'; exit 2; fi ;;
  *vite.js) if [ "${FAKE_VITE_FAIL:-}" = 1 ]; then exit 2; fi; mkdir -p dist; if [ "${FAKE_BUNDLE_MISSING:-}" != 1 ]; then printf 'export function mountGallery() {}\n' > dist/app.mjs; fi; if [ -n "${ARGUI_TEST_VITE_LOG:-}" ]; then printf 'build\n' >> "$ARGUI_TEST_VITE_LOG"; fi ;;
  run) mkdir -p apps/gallery/dist; printf 'export function mountGallery() {}\n' > apps/gallery/dist/gallery-core.mjs ;;
  *) exit 3 ;;
esac
"##,
    );
    script(
        &root.join("bin/cargo"),
        r##"#!/bin/sh
profile=debug
if [ "${FAKE_CARGO_FAIL:-}" = 1 ]; then exit 2; fi
while [ "$#" -gt 0 ]; do
  if [ "$1" = '--target-dir' ]; then shift; target="$1"; fi
  if [ "$1" = '--release' ]; then profile=release; fi
  shift
done
mkdir -p "$target/$profile"
cat > "$target/$profile/argui-gallery-quickjs" <<'HOST'
#!/bin/sh
printf '%s\n' "$ARGUI_APP_BUNDLE" > "$ARGUI_TEST_LOG"
if [ -n "${ARGUI_TEST_READY:-}" ]; then touch "$ARGUI_TEST_READY"; sleep 3; fi
if [ "${FAKE_HOST_FAIL:-}" = 1 ]; then exit 2; fi
HOST
chmod +x "$target/$profile/argui-gallery-quickjs"
if [ "${FAKE_HOST_MISSING:-}" = 1 ]; then rm "$target/$profile/argui-gallery-quickjs"; fi
if [ "${FAKE_HOST_UNEXEC:-}" = 1 ]; then chmod 644 "$target/$profile/argui-gallery-quickjs"; fi
"##,
    );
}

/// Runs `args` through the built CLI with the fixture tools on PATH.
fn cli(root: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_argui"))
        .args(args)
        .current_dir(root.join("apps/demo"))
        .env(
            "PATH",
            format!(
                "{}:{}",
                root.join("bin").display(),
                std::env::var("PATH").unwrap()
            ),
        )
        .env("ARGUI_TEST_LOG", root.join("host.log"))
        .output()
        .unwrap()
}

#[test]
/// Option errors keep framework selectors, paths, and catalog filters unambiguous.
fn command_option_edges_report_clear_errors() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let run = |args: &[&str]| {
        run_in(
            root,
            &args
                .iter()
                .map(|value| (*value).to_owned())
                .collect::<Vec<_>>(),
        )
    };
    for (args, expected) in [
        (vec!["add", "--unknown", "button"], "unknown add option"),
        (vec!["add", "--project"], "needs a directory"),
        (vec!["add", "button", "solid"], "cannot locate argui.json"),
        (vec!["add", "--solid", "solid"], "cannot locate argui.json"),
        (
            vec!["list", "components", "--solid", "--react"],
            "choose only one",
        ),
        (vec!["list", "components", "--project"], "needs a directory"),
        (
            vec!["list", "components", "--unknown"],
            "unknown list components option",
        ),
        (vec!["run", "missing", "release"], "argui.json"),
        (vec!["run", "release", "missing"], "argui.json"),
    ] {
        let error = run(&args).unwrap_err();
        assert!(error.contains(expected), "{args:?}: {error}");
    }
    run_in(
        root,
        &[
            "init".into(),
            "rust".into(),
            "--dir".into(),
            "rust-app".into(),
            "--yes".into(),
        ],
    )
    .unwrap();
    let error = run_in(&root.join("rust-app"), &["add".into(), "button".into()]).unwrap_err();
    assert!(error.contains("pure Rust"));
}

#[test]
/// The overview reports unavailable tools when its PATH contains no build executables.
fn overview_reports_missing_tools() {
    let temp = tempfile::tempdir().unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_argui"))
        .current_dir(temp.path())
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(result.status.success());
    let output = String::from_utf8(result.stdout).unwrap();
    assert!(output.contains("Bun missing"));
    assert!(output.contains("Rust missing"));
}

#[test]
/// The CLI emits both readable and JSON TypeScript diagnostics.
fn check_reports_success_and_failure() {
    let temp = tempfile::tempdir().unwrap();
    fixture(temp.path());
    assert!(cli(temp.path(), &["check"]).status.success());
    let failed = Command::new(env!("CARGO_BIN_EXE_argui"))
        .args(["check", "--json"])
        .current_dir(temp.path().join("apps/demo"))
        .env(
            "PATH",
            format!(
                "{}:{}",
                temp.path().join("bin").display(),
                std::env::var("PATH").unwrap()
            ),
        )
        .env("FAKE_TSC_FAIL", "1")
        .output()
        .unwrap();
    assert!(!failed.status.success());
    let json: serde_json::Value = serde_json::from_slice(&failed.stdout).unwrap();
    assert_eq!(json["ok"], false);
    assert!(
        json["diagnostics"][0]
            .as_str()
            .unwrap()
            .contains("src/main.tsx")
    );
    let unavailable = Command::new(env!("CARGO_BIN_EXE_argui"))
        .arg("check")
        .current_dir(temp.path().join("apps/demo"))
        .env(
            "PATH",
            format!("{}:/usr/bin:/bin", temp.path().join("bin").display()),
        )
        .env("FAKE_BUN_DELETE", "1")
        .output()
        .unwrap();
    assert!(!unavailable.status.success());
    assert!(String::from_utf8_lossy(&unavailable.stderr).contains("bun unavailable"));
}

#[test]
/// Dev and release builds produce a bundle, native host, and portable launcher.
fn build_and_run_use_the_project_bundle() {
    let temp = tempfile::tempdir().unwrap();
    fixture(temp.path());
    assert!(cli(temp.path(), &["run", "dev"]).status.success());
    assert_eq!(
        fs::read_to_string(temp.path().join("host.log"))
            .unwrap()
            .trim(),
        temp.path()
            .join("apps/demo/dist/app.mjs")
            .display()
            .to_string()
    );
    assert!(cli(temp.path(), &["build", "release"]).status.success());
    let package = temp.path().join("apps/demo/dist/desktop");
    assert!(package.join("argui-gallery-quickjs").is_file());
    assert!(package.join("app.mjs").is_file());
    assert!(package.join("run.sh").is_file());
    let launched = Command::new("sh")
        .arg(package.join("run.sh"))
        .env("ARGUI_TEST_LOG", temp.path().join("packaged-host.log"))
        .output()
        .unwrap();
    assert!(launched.status.success());
    assert_eq!(
        fs::read_to_string(temp.path().join("packaged-host.log"))
            .unwrap()
            .trim(),
        package.join("app.mjs").display().to_string()
    );
}

#[test]
/// Native project commands accept an app path from a multi-app checkout.
fn native_commands_select_project_path() {
    let temp = tempfile::tempdir().unwrap();
    fixture(temp.path());
    let path = format!(
        "{}:{}",
        temp.path().join("bin").display(),
        std::env::var("PATH").unwrap()
    );
    for args in [
        &["check", "apps/demo"][..],
        &["build", "apps/demo", "dev"],
        &["dev", "apps/demo"],
    ] {
        let result = Command::new(env!("CARGO_BIN_EXE_argui"))
            .args(args)
            .current_dir(temp.path())
            .env("PATH", &path)
            .env("ARGUI_TEST_LOG", temp.path().join("host.log"))
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}: {}",
            args.join(" "),
            String::from_utf8_lossy(&result.stderr)
        );
    }
    assert_eq!(
        fs::read_to_string(temp.path().join("host.log"))
            .unwrap()
            .trim(),
        temp.path()
            .join("apps/demo/dist/app.mjs")
            .display()
            .to_string()
    );
}

#[test]
/// Native dev rebuilds changed TSX while the running host watches the bundle.
fn native_dev_rebuilds_changed_tsx() {
    let temp = tempfile::tempdir().unwrap();
    fixture(temp.path());
    fs::write(temp.path().join("apps/demo/src/main.tsx"), "// initial\n").unwrap();
    let path = format!(
        "{}:{}",
        temp.path().join("bin").display(),
        std::env::var("PATH").unwrap()
    );
    let mut child = Command::new(env!("CARGO_BIN_EXE_argui"))
        .args(["dev", "apps/demo"])
        .current_dir(temp.path())
        .env("PATH", &path)
        .env("ARGUI_TEST_LOG", temp.path().join("host.log"))
        .env("ARGUI_TEST_READY", temp.path().join("ready"))
        .env("ARGUI_TEST_VITE_LOG", temp.path().join("vite.log"))
        .spawn()
        .unwrap();
    let deadline = SystemTime::now() + Duration::from_secs(5);
    while !temp.path().join("ready").exists() && SystemTime::now() < deadline {
        thread::sleep(Duration::from_millis(50));
    }
    assert!(
        temp.path().join("ready").exists(),
        "native host never started"
    );
    fs::write(
        temp.path().join("apps/demo/src/main.tsx"),
        "// changed TSX source\n",
    )
    .unwrap();
    assert!(child.wait().unwrap().success());
    let builds = fs::read_to_string(temp.path().join("vite.log")).unwrap();
    assert!(builds.lines().count() >= 2, "TSX was not rebuilt: {builds}");
}

#[test]
/// Project selection rejects bad paths and doctor reports setup gaps in English.
fn command_errors_and_doctor_explain_next_steps() {
    let temp = tempfile::tempdir().unwrap();
    fixture(temp.path());
    let path = format!(
        "{}:{}",
        temp.path().join("bin").display(),
        std::env::var("PATH").unwrap()
    );
    for (args, expected) in [
        (vec!["build", "apps/missing", "dev"], "argui.json"),
        (vec!["dev", "apps/missing"], "argui.json"),
        (vec!["build", "apps/demo", "oops"], "usage:"),
        (vec!["check", "apps/demo", "oops"], "usage:"),
        (vec!["doctor"], "QuickJS is bundled"),
    ] {
        let result = Command::new(env!("CARGO_BIN_EXE_argui"))
            .args(&args)
            .current_dir(temp.path())
            .env("PATH", &path)
            .output()
            .unwrap();
        let message = format!(
            "{}{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(message.contains(expected), "{}: {message}", args.join(" "));
    }
    let manifest = temp.path().join("apps/demo/argui.json");
    fs::write(&manifest, "not JSON").unwrap();
    let malformed = cli(temp.path(), &["check"]);
    assert!(!malformed.status.success());
    assert!(String::from_utf8_lossy(&malformed.stderr).contains("argui.json"));
    fs::write(&manifest, r#"{"name":"../bad","framework":"solid"}"#).unwrap();
    let unsafe_name = cli(temp.path(), &["check"]);
    assert!(!unsafe_name.status.success());
    assert!(String::from_utf8_lossy(&unsafe_name.stderr).contains("invalid project name"));
    fs::write(&manifest, r#"{"name":"demo","framework":"solid"}"#).unwrap();
    fs::remove_dir_all(temp.path().join("apps/demo/node_modules")).unwrap();
    let missing = cli(temp.path(), &["check"]);
    assert!(!missing.status.success());
    assert!(String::from_utf8_lossy(&missing.stderr).contains("bun install"));
    let doctor = Command::new(env!("CARGO_BIN_EXE_argui"))
        .arg("doctor")
        .current_dir(temp.path())
        .env("PATH", temp.path().join("bin"))
        .output()
        .unwrap();
    assert!(!doctor.status.success());
    assert!(String::from_utf8_lossy(&doctor.stderr).contains("missing prerequisites"));
}

#[test]
/// Native build and launch failures identify the missing stage or artifact.
fn native_failures_name_the_build_stage() {
    let temp = tempfile::tempdir().unwrap();
    fixture(temp.path());
    let path = format!(
        "{}:{}",
        temp.path().join("bin").display(),
        std::env::var("PATH").unwrap()
    );
    for (variable, args, expected) in [
        ("FAKE_VITE_FAIL", vec!["build", "dev"], "vite build"),
        ("FAKE_CARGO_FAIL", vec!["build", "dev"], "cargo build"),
        (
            "FAKE_BUNDLE_MISSING",
            vec!["run", "dev"],
            "TSX bundle missing",
        ),
        (
            "FAKE_HOST_MISSING",
            vec!["run", "dev"],
            "native host missing",
        ),
        ("FAKE_HOST_FAIL", vec!["run", "dev"], "native host exited"),
        ("FAKE_HOST_UNEXEC", vec!["run", "dev"], "native host:"),
        (
            "FAKE_HOST_MISSING",
            vec!["build", "release"],
            "argui-gallery-quickjs",
        ),
    ] {
        let _ = fs::remove_file(temp.path().join("apps/demo/dist/app.mjs"));
        let result = Command::new(env!("CARGO_BIN_EXE_argui"))
            .args(&args)
            .current_dir(temp.path().join("apps/demo"))
            .env("PATH", &path)
            .env(variable, "1")
            .output()
            .unwrap();
        assert!(
            !result.status.success(),
            "{} unexpectedly passed",
            args.join(" ")
        );
        assert!(
            String::from_utf8_lossy(&result.stderr).contains(expected),
            "{}: {}",
            args.join(" "),
            String::from_utf8_lossy(&result.stderr)
        );
    }
}

#[test]
/// Release packaging names each file conflict without replacing user files.
fn release_packaging_reports_conflicts() {
    for stage in ["desktop", "bundle", "icon", "launcher"] {
        let temp = tempfile::tempdir().unwrap();
        fixture(temp.path());
        let app = temp.path().join("apps/demo");
        match stage {
            "desktop" => {
                fs::create_dir_all(app.join("dist")).unwrap();
                fs::write(app.join("dist/desktop"), "occupied").unwrap();
            }
            "icon" => {
                fs::create_dir_all(app.join("icons")).unwrap();
                fs::write(app.join("icons/icon-256.png"), "icon").unwrap();
                fs::create_dir_all(app.join("dist/desktop/icon-256.png")).unwrap();
            }
            "launcher" => {
                fs::create_dir_all(app.join("dist/desktop/run.sh")).unwrap();
            }
            _ => {}
        }
        let result = Command::new(env!("CARGO_BIN_EXE_argui"))
            .args(["build", "release"])
            .current_dir(&app)
            .env(
                "PATH",
                format!(
                    "{}:{}",
                    temp.path().join("bin").display(),
                    std::env::var("PATH").unwrap()
                ),
            )
            .env(
                "FAKE_BUNDLE_MISSING",
                if stage == "bundle" { "1" } else { "0" },
            )
            .output()
            .unwrap();
        assert!(!result.status.success(), "{stage} unexpectedly passed");
        let error = String::from_utf8_lossy(&result.stderr);
        assert!(
            error.contains(match stage {
                "desktop" => "File exists",
                "bundle" => "No such file",
                "icon" => "icon-256.png",
                _ => "run.sh",
            }),
            "{stage}: {error}"
        );
    }
}

#[test]
/// Global initialization creates an application without cloning a checkout.
fn init_outside_checkout_creates_standalone_app() {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir_all(temp.path().join("bin")).unwrap();
    script(&temp.path().join("bin/git"), "#!/bin/sh\nexit 97\n");
    let result = Command::new(env!("CARGO_BIN_EXE_argui"))
        .args([
            "init",
            "react",
            "--dir",
            "outside-app",
            "--name",
            "outside-app",
            "--yes",
        ])
        .current_dir(temp.path())
        .env(
            "PATH",
            format!(
                "{}:{}",
                temp.path().join("bin").display(),
                std::env::var("PATH").unwrap()
            ),
        )
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(temp.path().join("outside-app/src/main.tsx").is_file());
    assert!(temp.path().join("outside-app/argui.json").is_file());
}

#[test]
/// The library builds both profiles with explicit fixture tools on PATH.
fn library_builds_with_checkout_tools() {
    if let Ok(root) = std::env::var("ARGUI_CLI_TEST_CHILD_ROOT") {
        let root = Path::new(&root);
        let app = root.join("apps/demo");
        run_in(&app, &["check".into()]).unwrap();
        run_in(&app, &["build".into(), "dev".into()]).unwrap();
        run_in(&app, &["build".into(), "release".into()]).unwrap();
        return;
    }
    let temp = tempfile::tempdir().unwrap();
    fixture(temp.path());
    fs::remove_file(temp.path().join("apps/gallery/dist/gallery-core.mjs")).unwrap();
    let child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "library_builds_with_checkout_tools",
            "--nocapture",
        ])
        .env("ARGUI_CLI_TEST_CHILD_ROOT", temp.path())
        .env(
            "PATH",
            format!(
                "{}:{}",
                temp.path().join("bin").display(),
                std::env::var("PATH").unwrap()
            ),
        )
        .output()
        .unwrap();
    assert!(
        child.status.success(),
        "{}{}",
        String::from_utf8_lossy(&child.stdout),
        String::from_utf8_lossy(&child.stderr)
    );
}
