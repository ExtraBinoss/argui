#![cfg(unix)]

use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::Path,
    process::{Command, Output},
};

/// Writes a fake Bun executable that records every invocation.
fn fake_bun(root: &Path) {
    fs::create_dir_all(root.join("bin")).unwrap();
    let path = root.join("bin/bun");
    fs::write(
        &path,
        "#!/bin/sh\nprintf '%s\\n' \"$1\" >> \"$ARGUI_FAKE_LOG\"\nif [ \"$1\" = --version ]; then echo 1.4.0; exit 0; fi\nif [ \"$1\" = install ] && [ \"${ARGUI_FAKE_INSTALL_FAIL:-}\" = 1 ]; then echo 'registry unavailable' >&2; exit 7; fi\nif [ \"$1\" = install ]; then exit 0; fi\nexit 8\n",
    )
    .unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

/// Runs init with one isolated tool path and captures its user-facing result.
fn init(root: &Path, framework: &str, extra: &[&str], fake: bool, fail: bool) -> Output {
    if fake {
        fake_bun(root);
    } else {
        fs::create_dir_all(root.join("bin")).unwrap();
    }
    Command::new(env!("CARGO_BIN_EXE_argui"))
        .args(["init", framework, "--dir", "app", "--yes"])
        .args(extra)
        .current_dir(root)
        .env("PATH", root.join("bin"))
        .env("ARGUI_FAKE_LOG", root.join("bun.log"))
        .env("ARGUI_FAKE_INSTALL_FAIL", if fail { "1" } else { "0" })
        .output()
        .unwrap()
}

#[test]
/// Solid and React each install their own pinned Bun dependencies at init.
fn init_installs_generated_tsx_dependencies() {
    for framework in ["solid", "react"] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let output = init(root, framework, &[], true, false);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let log = fs::read_to_string(root.join("bun.log")).unwrap();
        assert_eq!(log, "--version\ninstall\n");
        let package: serde_json::Value =
            serde_json::from_slice(&fs::read(root.join("app/package.json")).unwrap()).unwrap();
        assert_eq!(package["devDependencies"]["oxfmt"], "0.70.0");
        assert!(root.join("app/.oxfmtrc.json").is_file());
        assert!(String::from_utf8_lossy(&output.stdout).contains("Installed Bun dependencies"));
    }
}

#[test]
/// Skipping install does not probe Bun and leaves an explicit install step.
fn init_no_install_preserves_manual_setup() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let output = init(root, "solid", &["--no-install"], true, false);
    assert!(output.status.success());
    assert!(!root.join("bun.log").exists());
    assert!(String::from_utf8_lossy(&output.stdout).contains("bun install && argui check"));
}

#[test]
/// Missing Bun or a failed install keeps the generated app and prints a retry.
fn init_keeps_project_when_install_does_not_finish() {
    for (fake, fail) in [(false, false), (true, true)] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let output = init(root, "react", &[], fake, fail);
        assert!(output.status.success());
        assert!(root.join("app/argui.json").is_file());
        assert!(root.join("app/package.json").is_file());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("bun install"), "{stderr}");
        assert!(String::from_utf8_lossy(&output.stdout).contains("bun install && argui check"));
    }
}
