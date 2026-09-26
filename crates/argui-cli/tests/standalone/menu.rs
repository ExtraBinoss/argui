#![cfg(unix)]

use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

const DRIVE_PTY: &str = r#"
import os, pty, select, subprocess, sys, threading, time
root, binary, encoded, *args = sys.argv[1:]
master, slave = pty.openpty()
child = subprocess.Popen([binary, *args], cwd=root, stdin=slave, stdout=slave, stderr=slave)
os.close(slave)
chunks = []
def collect():
    while True:
        ready, _, _ = select.select([master], [], [], 0.1)
        if not ready:
            if child.poll() is not None: break
            continue
        try: chunks.append(os.read(master, 65536))
        except OSError: break
reader = threading.Thread(target=collect, daemon=True)
reader.start()
time.sleep(0.3)
for token in encoded.split(','):
    os.write(master, bytes.fromhex(token))
    time.sleep(0.15)
try: status = child.wait(timeout=8)
except subprocess.TimeoutExpired:
    child.kill()
    status = child.wait()
    chunks.append(b'PTY timeout')
reader.join(timeout=1)
os.close(master)
sys.stdout.buffer.write(b''.join(chunks))
sys.exit(status)
"#;

/// Runs interactive init inside a private pseudoterminal with real key input.
fn drive(root: &Path, args: &[&str], keys: &[&[u8]]) -> Output {
    let encoded = keys
        .iter()
        .map(|key| {
            key.iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join(",");
    Command::new("python3")
        .args([
            "-c",
            DRIVE_PTY,
            root.to_str().unwrap(),
            env!("CARGO_BIN_EXE_argui"),
            &encoded,
        ])
        .args(args)
        .arg("--no-install")
        .output()
        .unwrap()
}

#[test]
/// Arrow, Space, Enter, and scrolling select a Web React app and tasks.
fn interactive_menu_selects_framework_targets_and_capability() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let output = drive(
        root,
        &["init", "--dir", "app"],
        &[
            b"\x1b[B", b" ", b"\r", // React
            b" ", b"\r", // Web without desktop
            b"\x1b[B", b"\x1b[B", b"\x1b[B", b" ", b"\r", // tasks
            b"\r", // summary
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let state: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("app/argui.json")).unwrap()).unwrap();
    assert_eq!(state["framework"], "react");
    assert_eq!(state["targets"], serde_json::json!(["web"]));
    assert_eq!(state["features"], serde_json::json!(["tasks"]));
    assert!(String::from_utf8_lossy(&output.stdout).contains("Mobile (Android + iOS)"));
}

#[test]
/// Escape cancels without leaving a generated manifest.
fn interactive_menu_can_cancel_cleanly() {
    let temp = tempfile::tempdir().unwrap();
    let output = drive(temp.path(), &["init", "--dir", "app"], &[b"\x1b"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Language and framework"));
    assert!(!temp.path().join("app").exists());
}

#[test]
/// Disabled mobile and automation rows cannot change a pure Rust app.
fn interactive_rust_menu_keeps_unavailable_choices_disabled() {
    let temp = tempfile::tempdir().unwrap();
    let output = drive(
        temp.path(),
        &["init", "rust", "--dir", "app"],
        &[
            b"\x1b[B", b"\x1b[B", b" ", b"\r", // Mobile is disabled.
            b"\x1b[B", b"\x1b[B", b" ", b"\r", // Automation is disabled.
            b"\r",
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let state: serde_json::Value =
        serde_json::from_slice(&fs::read(temp.path().join("app/argui.json")).unwrap()).unwrap();
    assert_eq!(state["framework"], "rust");
    assert_eq!(state["targets"], serde_json::json!(["native"]));
    assert_eq!(state["features"], serde_json::json!([]));
}

#[test]
/// Explicit framework, targets, and capability remain selected in the TTY summary.
fn interactive_menu_preserves_explicit_flags() {
    let temp = tempfile::tempdir().unwrap();
    let output = drive(
        temp.path(),
        &[
            "init",
            "solid",
            "--dir",
            "app",
            "--targets",
            "native,web",
            "--feature",
            "automation",
        ],
        &[b"\r", b"\r"],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let state: serde_json::Value =
        serde_json::from_slice(&fs::read(temp.path().join("app/argui.json")).unwrap()).unwrap();
    assert_eq!(state["framework"], "solid");
    assert_eq!(state["targets"], serde_json::json!(["native", "web"]));
    assert_eq!(state["features"], serde_json::json!(["automation"]));
}

#[test]
/// The target menu refuses an empty selection and accepts it after Web is restored.
fn interactive_menu_requires_one_target() {
    let temp = tempfile::tempdir().unwrap();
    let output = drive(
        temp.path(),
        &["init", "--dir", "app"],
        &[
            b"\x1b[A", b"\x1b[B", b"?", b"\r", // Navigate and keep Solid.
            b" ", b"\x1b[B", b" ", b"\r", // Deselect both; Enter stays here.
            b" ", b"\r", // Restore Web and continue.
            b"\r", b"\r", // Capabilities and summary.
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let state: serde_json::Value =
        serde_json::from_slice(&fs::read(temp.path().join("app/argui.json")).unwrap()).unwrap();
    assert_eq!(state["framework"], "solid");
    assert_eq!(state["targets"], serde_json::json!(["web"]));
}
