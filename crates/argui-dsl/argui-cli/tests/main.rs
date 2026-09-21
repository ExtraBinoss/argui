use std::{
    fs,
    io::{BufRead, BufReader},
    net::{TcpListener, TcpStream},
    path::Path,
    process::{Child, Command, Output, Stdio},
    sync::mpsc,
    time::{Duration, Instant},
};

use argui_dsl_protocol::{LiveMessage, read_frame, write_frame};

/// Runs one terminating CLI command from `root` and returns its process result.
fn run(root: &Path, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_argui"))
        .args(arguments)
        .current_dir(root)
        .output()
        .unwrap()
}

/// Child guard to keep an interrupted integration test from leaving a watcher.
struct RunningCli(Child);

impl RunningCli {
    /// Requests normal shutdown so coverage and terminal cleanup can flush.
    fn stop(&mut self) {
        #[cfg(unix)]
        {
            let status = Command::new("kill")
                .args(["-INT", &self.0.id().to_string()])
                .status()
                .unwrap();
            assert!(status.success());
            assert!(self.0.wait().unwrap().success());
        }
        #[cfg(not(unix))]
        {
            self.0.kill().unwrap();
            let _ = self.0.wait().unwrap();
        }
    }
}

impl Drop for RunningCli {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}

/// Finds an unused native address pair for the dev TCP and WebSocket hosts.
fn free_dev_port() -> u16 {
    loop {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        if let Some(websocket) = port.checked_add(1)
            && let Ok(peer) = TcpListener::bind(("127.0.0.1", websocket))
        {
            drop(peer);
            return port;
        }
    }
}

/// Help, diagnostics and generated project commands work through the real binary.
#[test]
fn binary_dispatches_documented_commands_and_rejects_unknown_ones() {
    let root = tempfile::tempdir().unwrap();
    let help = run(root.path(), &["help"]);
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("argui dev"));
    assert!(run(root.path(), &[]).status.success());
    assert!(run(root.path(), &["--help"]).status.success());
    assert!(run(root.path(), &["-h"]).status.success());
    let unknown = run(root.path(), &["not-a-command"]);
    assert!(!unknown.status.success());
    assert!(String::from_utf8_lossy(&unknown.stderr).contains("unknown command"));

    let created = run(root.path(), &["new", "sample"]);
    assert!(
        created.status.success(),
        "{}",
        String::from_utf8_lossy(&created.stderr)
    );
    assert!(root.path().join("sample/ui/main.argui").is_file());
    let missing_name = run(root.path(), &["new"]);
    assert!(!missing_name.status.success());
    assert!(!run(root.path(), &["new", "sample"]).status.success());
    let project = root.path().join("sample");
    for arguments in [
        vec!["check"],
        vec!["schema", "Button"],
        vec!["schema"],
        vec!["symbols"],
        vec!["symbols", "ui/main.argui"],
        vec!["complete", "ui/main.argui", "1:1"],
        vec!["fmt", "ui/main.argui"],
        vec!["fmt", "ui/main.argui", "--check"],
    ] {
        let output = run(&project, &arguments);
        assert!(
            output.status.success(),
            "{arguments:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let invalid = run(&project, &["complete", "ui/main.argui"]);
    assert!(!invalid.status.success());
    assert!(!run(&project, &["complete"]).status.success());
    assert!(
        !run(&project, &["dev", "ui/main.argui", "invalid-bind"])
            .status
            .success()
    );
}

/// The binary accepts absolute in-project entries and reports invalid DSL diagnostics.
#[test]
fn binary_check_handles_absolute_paths_and_invalid_source() {
    let root = tempfile::tempdir().unwrap();
    assert!(run(root.path(), &["new", "sample"]).status.success());
    let project = root.path().join("sample");
    let entry = project.join("ui/main.argui");
    assert!(
        run(&project, &["check", entry.to_str().unwrap()])
            .status
            .success()
    );
    fs::write(&entry, "export component Main { invalid }").unwrap();
    let invalid = run(&project, &["check"]);
    assert!(!invalid.status.success());
    assert!(String::from_utf8_lossy(&invalid.stdout).contains("Diagnostics"));
}

/// The command dispatcher rejects paths outside the project and malformed queries.
#[test]
fn binary_reports_invalid_project_paths_and_query_arguments() {
    let root = tempfile::tempdir().unwrap();
    assert!(run(root.path(), &["new", "sample"]).status.success());
    let project = root.path().join("sample");
    let outside = root.path().join("outside.argui");
    fs::write(&outside, "export component Outside {}").unwrap();

    let invalid_entry = run(&project, &["check", outside.to_str().unwrap()]);
    assert!(!invalid_entry.status.success());
    let invalid_schema = run(&project, &["schema", "NoSuchComponent"]);
    assert!(!invalid_schema.status.success());
    assert!(
        String::from_utf8_lossy(&invalid_schema.stderr)
            .contains("unknown component `NoSuchComponent`")
    );

    for arguments in [
        ["complete", "ui/main.argui", "0:1"],
        ["complete", "ui/main.argui", "1:0"],
        ["complete", "ui/main.argui", "999:1"],
        ["complete", "ui/main.argui", "1:999"],
        ["complete", "ui/main.argui", "not-a-position"],
    ] {
        assert!(
            !run(&project, &arguments).status.success(),
            "invalid completion position {arguments:?} unexpectedly succeeded"
        );
    }
}

/// The real no-run dev host broadcasts an initial and a changed DSL generation.
#[test]
fn binary_dev_no_run_watches_and_publishes_changes() {
    let root = tempfile::tempdir().unwrap();
    assert!(run(root.path(), &["new", "sample"]).status.success());
    let project = root.path().join("sample");
    let port = free_dev_port();
    let address = format!("127.0.0.1:{port}");
    let mut child = RunningCli(
        Command::new(env!("CARGO_BIN_EXE_argui"))
            .args(["dev", "ui/main.argui", &address, "--no-run"])
            .current_dir(&project)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(8);
    let mut stream = loop {
        if let Ok(stream) = TcpStream::connect(&address) {
            break stream;
        }
        assert!(
            child.0.try_wait().unwrap().is_none(),
            "dev host exited early"
        );
        assert!(Instant::now() < deadline, "dev host did not bind in time");
        std::thread::sleep(Duration::from_millis(25));
    };
    stream
        .set_read_timeout(Some(Duration::from_secs(8)))
        .unwrap();
    assert!(matches!(
        read_frame::<LiveMessage>(&mut stream).unwrap(),
        LiveMessage::Hello { .. }
    ));
    assert!(matches!(
        read_frame::<LiveMessage>(&mut stream).unwrap(),
        LiveMessage::Package(_)
    ));
    let source = project.join("ui/main.argui");
    let previous = fs::read_to_string(&source).unwrap();
    fs::write(
        &source,
        previous.replace("Welcome to Argui", "Welcome updated"),
    )
    .unwrap();
    fs::write(project.join("README.md"), "unrelated source edit\n").unwrap();
    loop {
        match read_frame::<LiveMessage>(&mut stream).unwrap() {
            LiveMessage::Package(package) if package.header.generation >= 2 => {
                write_frame(
                    &mut stream,
                    &LiveMessage::Committed {
                        generation: package.header.generation,
                    },
                )
                .unwrap();
                break;
            }
            LiveMessage::Diagnostics { .. } => {}
            unexpected => panic!("unexpected watcher message: {unexpected:?}"),
        }
    }
    fs::write(&source, "export component Main { invalid }").unwrap();
    assert!(matches!(
        read_frame::<LiveMessage>(&mut stream).unwrap(),
        LiveMessage::Diagnostics { .. }
    ));
    child.stop();
}

/// An edit without a native client is reported as compiled but not applied.
#[test]
fn binary_dev_distinguishes_compilation_from_application() {
    let root = tempfile::tempdir().unwrap();
    assert!(run(root.path(), &["new", "sample"]).status.success());
    let project = root.path().join("sample");
    let address = format!("127.0.0.1:{}", free_dev_port());
    let mut child = RunningCli(
        Command::new(env!("CARGO_BIN_EXE_argui"))
            .args(["dev", "ui/main.argui", &address, "--no-run"])
            .current_dir(&project)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    let stderr = child.0.stderr.take().unwrap();
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(stderr).lines() {
            let Ok(line) = line else { break };
            if sender.send(line).is_err() {
                break;
            }
        }
    });
    let mut saw_initial = false;
    while !saw_initial {
        let line = receiver.recv_timeout(Duration::from_secs(8)).unwrap();
        saw_initial = line.contains("Generation 1 compiled");
    }
    let source = project.join("ui/main.argui");
    let previous = fs::read_to_string(&source).unwrap();
    fs::write(&source, &previous).unwrap();
    assert!(matches!(
        receiver.recv_timeout(Duration::from_millis(800)),
        Err(mpsc::RecvTimeoutError::Timeout)
    ));
    fs::write(
        &source,
        previous.replace("Welcome to Argui", "Welcome updated"),
    )
    .unwrap();
    let mut saw_change = false;
    let mut saw_compilation = false;
    loop {
        let line = receiver.recv_timeout(Duration::from_secs(8)).unwrap();
        saw_change |= line.contains("Change detected");
        saw_compilation |= line.contains("Generation 2 compiled");
        if line.contains("No native client connected") {
            break;
        }
    }
    assert!(saw_change);
    assert!(saw_compilation);
    child.stop();
}

/// Application-launch failures are reported without leaving a watcher running.
#[test]
fn binary_dev_reports_child_startup_failure() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("main.argui"), "export component Main {}").unwrap();
    let address = format!("127.0.0.1:{}", free_dev_port());
    let output = run(root.path(), &["dev", "main.argui", &address]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("development application exited"));
}

/// A successfully launched dev application exits without leaving its watcher alive.
#[test]
fn binary_dev_exits_after_a_successful_child_application() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join("ui")).unwrap();
    fs::create_dir(root.path().join("src")).unwrap();
    fs::write(
        root.path().join("ui/main.argui"),
        "export component Main {}",
    )
    .unwrap();
    fs::write(
        root.path().join("Cargo.toml"),
        "[package]\nname = \"argui-dev-smoke\"\nversion = \"0.1.0\"\nedition = \"2024\"\n[features]\nargui-live = []\n",
    )
    .unwrap();
    fs::write(
        root.path().join("src/main.rs"),
        "fn main() { println!(\"child ready\"); }\n",
    )
    .unwrap();
    let address = format!("127.0.0.1:{}", free_dev_port());
    let output = run(root.path(), &["dev", "ui/main.argui", &address]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("child ready"));
}
