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
/// Update option errors return before making a network request.
fn update_rejects_unknown_options() {
    let root = tempfile::tempdir().unwrap();
    assert!(
        run_in(root.path(), &["update".into(), "--bogus".into()])
            .unwrap_err()
            .contains("usage: argui update [--check]")
    );
}
