#![cfg(unix)]

use std::{
    fs::File,
    io::{ErrorKind, Read, Write},
    net::TcpListener,
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

use nix::{
    fcntl::{FcntlArg, OFlag, fcntl},
    pty::{Winsize, openpty},
};

/// Owns a real terminal-backed dev host and captures its terminal output.
struct PtyDevHost {
    child: Child,
    master: File,
    slave: File,
    captured: Vec<u8>,
    _root: tempfile::TempDir,
}

impl PtyDevHost {
    /// Starts `argui dev --no-run` in a private PTY for input and resize checks.
    fn start() -> Self {
        Self::start_with_term("xterm-256color")
    }

    /// Starts the host with `term` as its terminal capability setting.
    fn start_with_term(term: &str) -> Self {
        Self::start_with_options(term, false)
    }

    /// Starts a terminal-backed host that launches a small child application.
    fn start_with_application() -> Self {
        Self::start_with_options("xterm-256color", true)
    }

    /// Starts the host with the requested terminal and application behavior.
    fn start_with_options(term: &str, launch_application: bool) -> Self {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("ui")).unwrap();
        std::fs::write(
            root.path().join("ui/main.argui"),
            "export component Main {}",
        )
        .unwrap();
        if launch_application {
            std::fs::create_dir(root.path().join("src")).unwrap();
            std::fs::write(
                root.path().join("Cargo.toml"),
                "[package]\nname = \"argui-pty-child\"\nversion = \"0.1.0\"\nedition = \"2024\"\n[features]\nargui-live = []\n",
            )
            .unwrap();
            std::fs::write(
                root.path().join("src/main.rs"),
                "fn main() { println!(\"child ready\"); eprintln!(\"child warning\"); std::thread::sleep(std::time::Duration::from_millis(500)); }\n",
            )
            .unwrap();
        }
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let winsize = Winsize {
            ws_row: 18,
            ws_col: 90,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        let pair = openpty(Some(&winsize), None).unwrap();
        let master = File::from(pair.master);
        let slave = File::from(pair.slave);
        let input = slave.try_clone().unwrap();
        let output = slave.try_clone().unwrap();
        let mut command = Command::new(env!("CARGO_BIN_EXE_argui"));
        command.args(["dev", "ui/main.argui", &format!("127.0.0.1:{port}")]);
        if !launch_application {
            command.arg("--no-run");
        }
        let child = command
            .env("TERM", term)
            .current_dir(root.path())
            .stdin(Stdio::from(input))
            .stdout(Stdio::from(output))
            .stderr(Stdio::from(slave.try_clone().unwrap()))
            .spawn()
            .unwrap();
        let flags = OFlag::from_bits_truncate(fcntl(&master, FcntlArg::F_GETFL).unwrap());
        fcntl(&master, FcntlArg::F_SETFL(flags | OFlag::O_NONBLOCK)).unwrap();
        Self {
            child,
            master,
            slave,
            captured: Vec::new(),
            _root: root,
        }
    }

    /// Reads available terminal bytes until `needle` appears or `timeout` expires.
    fn read_until(&mut self, needle: &[u8], timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            let mut buffer = [0; 4096];
            match self.master.read(&mut buffer) {
                Ok(count) if count > 0 => self.captured.extend_from_slice(&buffer[..count]),
                Ok(_) => {}
                Err(error) if error.kind() == ErrorKind::WouldBlock => {}
                Err(error) => panic!(
                    "PTY read failed: {error}; child={:?}; output={}",
                    self.child.try_wait().unwrap(),
                    String::from_utf8_lossy(&self.captured)
                ),
            }
            if self
                .captured
                .windows(needle.len())
                .any(|window| window == needle)
            {
                return true;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        false
    }
}

/// Interactive dev mode captures both child streams and exits when the child succeeds.
#[test]
fn terminal_dashboard_reports_launched_application_output() {
    let mut host = PtyDevHost::start_with_application();
    assert!(
        host.read_until(b"child warning", Duration::from_secs(20)),
        "terminal output: {}",
        String::from_utf8_lossy(&host.captured)
    );
    assert!(host.read_until(b"child ready", Duration::from_secs(3)));
    let transcript = String::from_utf8_lossy(&host.captured);
    assert!(transcript.contains("BUILDING") && transcript.contains("CLIENT"));
    assert!(transcript.contains("0/") || transcript.contains("1/"));
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        if let Some(status) = host.child.try_wait().unwrap() {
            assert!(status.success());
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("interactive host did not exit after its application");
}

impl Drop for PtyDevHost {
    fn drop(&mut self) {
        if self.child.try_wait().unwrap().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

/// Arrow and scroll escape sequences are consumed; Ctrl+C exits raw mode.
#[test]
fn terminal_input_and_resize_do_not_echo_control_sequences() {
    let mut host = PtyDevHost::start();
    assert!(
        host.read_until(b"WATCHING", Duration::from_secs(3)),
        "terminal output: {}",
        String::from_utf8_lossy(&host.captured)
    );
    host.captured.clear();
    let resize = Command::new("stty")
        .args(["cols", "50", "rows", "9"])
        .stdin(Stdio::from(host.slave.try_clone().unwrap()))
        .status()
        .unwrap();
    assert!(resize.success());
    assert!(
        Command::new("kill")
            .args(["-WINCH", &host.child.id().to_string()])
            .status()
            .unwrap()
            .success()
    );
    assert!(
        host.read_until(b"ARGUI", Duration::from_secs(3)),
        "resize did not redraw: {}",
        String::from_utf8_lossy(&host.captured)
    );
    let redraw = host
        .captured
        .windows(b"ARGUI".len())
        .rposition(|window| window == b"ARGUI")
        .unwrap();
    assert!(
        !host.captured[redraw..]
            .windows(b"Connections".len())
            .any(|window| window == b"Connections")
    );
    host.captured.clear();
    host.master
        .write_all(b"\x1b[B\x1b[B\x1b[<64;10;5M")
        .unwrap();
    host.master.write_all(b"c").unwrap();
    std::thread::sleep(Duration::from_millis(250));
    assert!(!host.read_until(b"^[[B", Duration::from_millis(100)));
    assert!(!host.captured.windows(4).any(|window| window == b"bbbb"));
    assert!(
        host.child.try_wait().unwrap().is_none(),
        "plain c must not stop the host"
    );
    host.master.write_all(b"\x03").unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        if let Some(status) = host.child.try_wait().unwrap() {
            assert!(status.success());
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("Ctrl+C did not stop the interactive host");
}

/// A dumb terminal retains line-oriented output even when both streams are TTYs.
#[test]
fn dumb_terminal_does_not_enter_fullscreen_dashboard() {
    let mut host = PtyDevHost::start_with_term("dumb");
    assert!(
        host.read_until(b"Generation 1 compiled", Duration::from_secs(3)),
        "terminal output: {}",
        String::from_utf8_lossy(&host.captured)
    );
    assert!(
        !host
            .captured
            .windows(b"\x1b[?1049h".len())
            .any(|window| window == b"\x1b[?1049h")
    );
    let signal = Command::new("kill")
        .args(["-INT", &host.child.id().to_string()])
        .status()
        .unwrap();
    assert!(signal.success());
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        if let Some(status) = host.child.try_wait().unwrap() {
            assert!(status.success());
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("SIGINT did not stop the line-oriented host");
}
