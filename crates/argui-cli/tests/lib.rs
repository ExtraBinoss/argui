#![cfg(not(target_arch = "wasm32"))]

use argui_cli::run_in;
use std::{fs, process::Command};

#[test]
/// CLI routing accepts only standalone v2 project forms.
fn legacy_project_commands_are_unreachable() {
    let root = tempfile::tempdir().unwrap();
    for args in [
        vec!["init", "counter-web"],
        vec!["init", "solid", "old-name"],
        vec!["init", "web"],
    ] {
        let args = args.into_iter().map(str::to_owned).collect::<Vec<_>>();
        assert!(run_in(root.path(), &args).is_err(), "{args:?}");
    }
    let old = root.path().join("old");
    fs::create_dir(&old).unwrap();
    fs::write(
        old.join("argui.json"),
        r#"{"name":"old","framework":"solid"}"#,
    )
    .unwrap();
    for args in [
        vec!["check"],
        vec!["build", "release"],
        vec!["dev"],
        vec!["run", "dev"],
    ] {
        let args = args.into_iter().map(str::to_owned).collect::<Vec<_>>();
        assert!(run_in(&old, &args).is_err(), "{args:?}");
    }
}

#[test]
/// Help and overview describe the standalone commands without requiring a project.
fn help_and_overview_work_outside_an_app() {
    let root = tempfile::tempdir().unwrap();
    let binary = env!("CARGO_BIN_EXE_argui");
    let overview = Command::new(binary)
        .current_dir(root.path())
        .output()
        .unwrap();
    assert!(overview.status.success());
    let overview = String::from_utf8(overview.stdout).unwrap();
    assert!(overview.contains("argui init solid --dir"));
    assert!(overview.contains("argui doctor"));
    let help = Command::new(binary)
        .arg("--help")
        .current_dir(root.path())
        .output()
        .unwrap();
    assert!(help.status.success());
    let help = String::from_utf8(help.stdout).unwrap();
    assert!(help.contains("argui add"));
    assert!(help.contains("argui format [path] [--check]"));
    assert!(help.contains("[--no-install]"));
    assert!(help.contains("argui screenshot"));
    assert!(help.contains("argui update [--check]"));
    assert!(!help.contains("counter-web"));
}

#[test]
/// The start screen identifies the selected application's framework and targets.
fn overview_identifies_the_current_project() {
    let root = tempfile::tempdir().unwrap();
    run_in(
        root.path(),
        &[
            "init".into(),
            "solid".into(),
            "--dir".into(),
            "demo".into(),
            "--yes".into(),
            "--no-install".into(),
        ],
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_argui"))
        .current_dir(root.path().join("demo"))
        .output()
        .unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("demo (solid / native, web)"), "{text}");
}

#[test]
/// Update option errors return before making a network request.
fn update_rejects_unknown_options() {
    let root = tempfile::tempdir().unwrap();
    assert!(
        run_in(root.path(), &["update".into(), "--bogus".into()])
            .unwrap_err()
            .contains("usage: argui update [--check]")
    );
}

#[test]
/// Test and screenshot commands reject malformed requests before building an app.
fn automation_commands_validate_inputs_before_build() {
    let root = tempfile::tempdir().unwrap();
    let run = |args: &[&str]| {
        run_in(
            root.path(),
            &args.iter().map(|value| (*value).into()).collect::<Vec<_>>(),
        )
    };
    assert!(run(&["test"]).unwrap_err().contains("--out"));
    assert!(
        run(&["test", "sample.test.ts", "--out", "a", "--out", "b"])
            .unwrap_err()
            .contains("only once")
    );
    assert!(
        run(&["test", "sample.ts", "--out", "reports"])
            .unwrap_err()
            .contains(".test.ts")
    );
    assert!(
        run(&["test", "sample.test.ts", "--out", "reports"])
            .unwrap_err()
            .contains("test file missing")
    );
    assert!(
        run(&["screenshot", "--out", "capture.jpg"])
            .unwrap_err()
            .contains(".png")
    );
    assert!(
        run(&["screenshot", "one", "two", "--out", "capture.png"])
            .unwrap_err()
            .contains("usage: argui screenshot")
    );
    assert!(
        run(&["screenshot", "--out", "capture.png"])
            .unwrap_err()
            .contains("src/main.tsx")
    );
}

#[cfg(unix)]
#[test]
/// Doctor reports missing tools and accepts a complete PATH in isolated child processes.
fn doctor_checks_native_and_web_prerequisites() {
    use std::os::unix::fs::PermissionsExt;

    let directory = tempfile::tempdir().unwrap();
    let binary = env!("CARGO_BIN_EXE_argui");
    for args in [vec!["doctor"], vec!["doctor", "web"]] {
        let missing = Command::new(binary)
            .args(&args)
            .env("PATH", directory.path())
            .output()
            .unwrap();
        assert!(!missing.status.success());
        assert!(String::from_utf8_lossy(&missing.stderr).contains("missing"));
    }
    for tool in [
        "bun",
        "cargo",
        "rustc",
        "git",
        "node",
        "clang",
        "cc",
        "pkg-config",
        "wasm-pack",
        "wasm-opt",
        "rustup",
    ] {
        let executable = directory.path().join(tool);
        fs::write(
            &executable,
            "#!/bin/sh\nif [ \"$1\" = target ]; then echo wasm32-unknown-unknown; fi\n",
        )
        .unwrap();
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).unwrap();
    }
    for args in [vec!["doctor"], vec!["doctor", "web"]] {
        let ready = Command::new(binary)
            .args(&args)
            .env("PATH", directory.path())
            .output()
            .unwrap();
        assert!(
            ready.status.success(),
            "{}",
            String::from_utf8_lossy(&ready.stderr)
        );
    }
}

#[test]
/// Catalog and component options report conflicting or incomplete selectors.
fn component_commands_validate_selectors() {
    let root = tempfile::tempdir().unwrap();
    let run = |args: &[&str]| {
        run_in(
            root.path(),
            &args.iter().map(|value| (*value).into()).collect::<Vec<_>>(),
        )
    };
    for args in [
        vec!["list", "components", "--solid", "--react"],
        vec!["list", "components", "--project"],
        vec!["list", "components", "--other"],
        vec!["add", "--solid", "--react", "button"],
        vec!["add", "--project"],
        vec!["add", "--unknown", "button"],
    ] {
        assert!(run(&args).is_err(), "{args:?}");
    }
}
