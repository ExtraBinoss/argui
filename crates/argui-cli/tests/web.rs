#![cfg(unix)]

use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::Path,
    process::Command,
    thread,
    time::{Duration, SystemTime},
};

/// Writes an executable fixture program at `path`.
fn script(path: &Path, source: &str) {
    fs::write(path, source).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

/// Creates an offline checkout with fake browser build tools.
fn fixture(root: &Path) {
    for dir in [
        "bin",
        "packages/host",
        "apps/gallery/quickjs-host",
        "apps/web-host",
        "apps/demo/node_modules/@argui/solid",
        "node_modules/vite",
        "node_modules/typescript",
    ] {
        fs::create_dir_all(root.join(dir)).unwrap();
    }
    fs::write(root.join("packages/host/package.json"), "{}").unwrap();
    fs::write(root.join("apps/gallery/quickjs-host/Cargo.toml"), "").unwrap();
    fs::write(
        root.join("apps/demo/argui.json"),
        r#"{"name":"demo","framework":"solid","target":"web"}"#,
    )
    .unwrap();
    script(
        &root.join("bin/bun"),
        r##"#!/bin/sh
case "$1" in
  --version) echo 1.4.0 ;;
  *vite.js) if [ "$2" = build ]; then if [ "${FAKE_VITE_FAIL:-}" = 1 ]; then exit 2; fi; mkdir -p dist/web; echo html > dist/web/index.html; elif [ -n "${ARGUI_TEST_READY:-}" ]; then touch "$ARGUI_TEST_READY"; sleep 3; fi ;;
  *tsc) : ;;
  *) exit 2 ;;
esac
"##,
    );
    script(
        &root.join("bin/rustup"),
        "#!/bin/sh\nif [ \"${FAKE_NO_WASM:-}\" != 1 ]; then echo wasm32-unknown-unknown; fi\n",
    );
    script(
        &root.join("bin/wasm-pack"),
        r##"#!/bin/sh
if [ "$1" = --version ]; then echo wasm-pack; exit 0; fi
if [ "${FAKE_WASM_FAIL:-}" = 1 ]; then exit 2; fi
if [ -n "${ARGUI_TEST_WASM_LOG:-}" ]; then printf 'build\n' >> "$ARGUI_TEST_WASM_LOG"; fi
while [ "$#" -gt 0 ]; do
  if [ "$1" = --out-dir ]; then shift; output="$1"; fi
  shift
done
mkdir -p "$output"
printf 'wasm source' > "$output/argui_web_host_bg.wasm"
printf 'export class ArguiWebHost {}\n' > "$output/argui_web_host.js"
"##,
    );
    script(
        &root.join("bin/wasm-opt"),
        r##"#!/bin/sh
if [ "$1" = --version ]; then echo wasm-opt; exit 0; fi
if [ "${FAKE_OPT_FAIL:-}" = 1 ]; then exit 2; fi
if [ "${FAKE_OPT_NO_OUTPUT:-}" = 1 ]; then exit 0; fi
while [ "$#" -gt 0 ]; do
  if [ "$1" = -o ]; then shift; output="$1"; else input="$1"; fi
  shift
done
cp "$input" "$output"
"##,
    );
}

/// Runs the built CLI against the isolated browser fixture.
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
        .output()
        .unwrap()
}

#[test]
/// Browser check, optimized release, and dev server use portable tool commands.
fn web_lifecycle_uses_wasm_pack_vite_and_wasm_opt() {
    let temp = tempfile::tempdir().unwrap();
    fixture(temp.path());
    let check = cli(temp.path(), &["check"]);
    assert!(
        check.status.success(),
        "{}",
        String::from_utf8_lossy(&check.stderr)
    );
    let build = cli(temp.path(), &["build", "release"]);
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    assert!(temp.path().join("apps/demo/dist/web/index.html").is_file());
    assert_eq!(
        fs::read_to_string(temp.path().join("apps/web-host/pkg/argui_web_host_bg.wasm")).unwrap(),
        "wasm source"
    );
    let run = cli(temp.path(), &["run", "dev"]);
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    let doctor = cli(temp.path(), &["doctor", "web"]);
    assert!(
        doctor.status.success(),
        "{}",
        String::from_utf8_lossy(&doctor.stderr)
    );
    let missing_target = Command::new(env!("CARGO_BIN_EXE_argui"))
        .args(["doctor", "web"])
        .current_dir(temp.path())
        .env(
            "PATH",
            format!(
                "{}:{}",
                temp.path().join("bin").display(),
                std::env::var("PATH").unwrap()
            ),
        )
        .env("FAKE_NO_WASM", "1")
        .output()
        .unwrap();
    assert!(!missing_target.status.success());
    assert!(String::from_utf8_lossy(&missing_target.stderr).contains("rustup target add"));
}

#[test]
/// Root-relative web commands select one app and dev rebuilds changed Rust.
fn web_dev_path_rebuilds_changed_rust() {
    let temp = tempfile::tempdir().unwrap();
    fixture(temp.path());
    fs::create_dir_all(temp.path().join("crates/example/src")).unwrap();
    fs::write(
        temp.path().join("crates/example/src/lib.rs"),
        "// initial\n",
    )
    .unwrap();
    let path = format!(
        "{}:{}",
        temp.path().join("bin").display(),
        std::env::var("PATH").unwrap()
    );
    let check = Command::new(env!("CARGO_BIN_EXE_argui"))
        .args(["check", "apps/demo"])
        .current_dir(temp.path())
        .env("PATH", &path)
        .output()
        .unwrap();
    assert!(
        check.status.success(),
        "{}",
        String::from_utf8_lossy(&check.stderr)
    );
    let mut child = Command::new(env!("CARGO_BIN_EXE_argui"))
        .args(["dev", "apps/demo"])
        .current_dir(temp.path())
        .env("PATH", &path)
        .env("ARGUI_TEST_READY", temp.path().join("ready"))
        .env("ARGUI_TEST_WASM_LOG", temp.path().join("wasm.log"))
        .spawn()
        .unwrap();
    let deadline = SystemTime::now() + Duration::from_secs(5);
    while !temp.path().join("ready").exists() && SystemTime::now() < deadline {
        thread::sleep(Duration::from_millis(50));
    }
    assert!(
        temp.path().join("ready").exists(),
        "Vite fixture never started"
    );
    fs::write(
        temp.path().join("crates/example/src/lib.rs"),
        "// changed Rust source\n",
    )
    .unwrap();
    let result = child.wait().unwrap();
    assert!(result.success());
    let builds = fs::read_to_string(temp.path().join("wasm.log")).unwrap();
    assert!(
        builds.lines().count() >= 2,
        "WASM was not rebuilt: {builds}"
    );
}

#[test]
/// Web failures identify missing targets and failed build tools.
fn web_build_failures_are_actionable() {
    let temp = tempfile::tempdir().unwrap();
    fixture(temp.path());
    let path = format!(
        "{}:{}",
        temp.path().join("bin").display(),
        std::env::var("PATH").unwrap()
    );
    for (variable, args, expected) in [
        ("FAKE_NO_WASM", vec!["build", "dev"], "rustup target add"),
        ("FAKE_WASM_FAIL", vec!["build", "dev"], "wasm-pack build"),
        ("FAKE_OPT_FAIL", vec!["build", "release"], "wasm-opt"),
        ("FAKE_OPT_NO_OUTPUT", vec!["build", "release"], "opt.wasm"),
        ("FAKE_VITE_FAIL", vec!["build", "dev"], "vite build"),
    ] {
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
