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
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("ui")).unwrap();
        std::fs::write(
            root.path().join("ui/main.argui"),
            "export component Main {}",
        )
        .unwrap();
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
        let child = Command::new(env!("CARGO_BIN_EXE_argui"))
            .args([
                "dev",
                "--no-run",
                "ui/main.argui",
                &format!("127.0.0.1:{port}"),
            ])
            .env("TERM", term)
            .current_dir(root.path())
            .stdin(Stdio::from(input))
            .stdout(Stdio::from(output))
            .stderr(Stdio::from(slave))
            .spawn()
            .unwrap();
        let flags = OFlag::from_bits_truncate(fcntl(&master, FcntlArg::F_GETFL).unwrap());
        fcntl(&master, FcntlArg::F_SETFL(flags | OFlag::O_NONBLOCK)).unwrap();
        Self {
            child,
            master,
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
