#![cfg(all(feature = "native", not(target_arch = "wasm32")))]
use argui_updater::{
    http::{Config, HttpBackend},
    install::{Format, Installer},
    *,
};
use std::{
    collections::VecDeque,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    path::Path,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

const KEY: &str = include_str!("fixtures/update.pub");
const SIGNATURE: &str = include_str!("fixtures/update.minisig");
fn payload() -> Vec<u8> {
    b"argui signed update\n".repeat(8192)
}
struct Reply {
    status: u16,
    headers: String,
    body: Vec<u8>,
}
impl Reply {
    fn ok(body: impl Into<Vec<u8>>) -> Self {
        let body = body.into();
        Self {
            status: 200,
            headers: format!("Content-Length: {}\r\n", body.len()),
            body,
        }
    }
}
struct Server {
    url: String,
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
    requests: Arc<Mutex<Vec<String>>>,
}
impl Server {
    fn new(replies: impl FnOnce(&str) -> Vec<Reply>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let mut replies = VecDeque::from(replies(&url));
        let stop = Arc::new(AtomicBool::new(false));
        let shutdown = stop.clone();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let recorded = requests.clone();
        let thread = thread::spawn(move || {
            while !shutdown.load(Ordering::Acquire) {
                let Ok((stream, _)) = listener.accept() else {
                    thread::sleep(Duration::from_millis(1));
                    continue;
                };
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                let mut reader = BufReader::new(stream);
                let mut line = String::new();
                if reader.read_line(&mut line).is_err() {
                    continue;
                }
                if line.is_empty() {
                    continue;
                }
                recorded.lock().unwrap().push(line.clone());
                loop {
                    line.clear();
                    if reader.read_line(&mut line).unwrap_or(0) == 0 || line == "\r\n" {
                        break;
                    }
                }
                let Some(reply) = replies.pop_front() else {
                    continue;
                };
                let mut stream = reader.into_inner();
                let _ = write!(
                    stream,
                    "HTTP/1.1 {} Response\r\n{}Connection: close\r\n\r\n",
                    reply.status, reply.headers
                );
                let _ = stream.write_all(&reply.body);
            }
        });
        Self {
            url,
            stop,
            thread: Some(thread),
            requests,
        }
    }
    fn updater(&self) -> Updater<HttpBackend<RecordingInstaller>> {
        Updater::new(HttpBackend::new(config(&self.url), RecordingInstaller).unwrap())
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        let _ = TcpStream::connect(self.url.trim_start_matches("http://"));
        self.thread.take().unwrap().join().unwrap();
    }
}
struct RecordingInstaller;
impl Installer for RecordingInstaller {
    fn supports(&self, format: Format) -> Result<()> {
        if format == Format::Executable {
            Ok(())
        } else {
            Err(Error::backend("unsupported format"))
        }
    }
    fn install(&self, _: Format, path: &Path) -> Result<InstallOutcome> {
        assert_eq!(std::fs::read(path).unwrap(), payload());
        Ok(InstallOutcome::RestartRequired)
    }
}
fn config(url: &str) -> Config {
    let mut config = Config::new("1.0.0", url, KEY).unwrap();
    config.target = "linux-x86_64".into();
    config.timeout = Duration::from_secs(2);
    config
}
fn manifest(url: &str, version: &str) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "version": version, "notes": "Faster startup",
        "platforms": { "linux-x86_64": { "url": format!("{url}/artifact"), "signature": SIGNATURE, "format": "executable" } }
    })).unwrap()
}

#[test]
fn signed_streams_report_cumulative_bytes_with_known_and_unknown_lengths() {
    for known in [true, false] {
        let server = Server::new(|url| {
            let mut artifact = Reply::ok(payload());
            if !known {
                artifact.headers.clear();
            }
            vec![Reply::ok(manifest(url, "1.2.0")), artifact]
        });
        let mut updater = server.updater();
        assert!(updater.check(|_| {}).unwrap());
        assert_eq!(
            server.requests.lock().unwrap().len(),
            1,
            "checking must not download"
        );
        let mut counts = Vec::new();
        updater
            .download(&CancellationToken::default(), |state| {
                if let State::Downloading { progress, .. } = state {
                    counts.push(*progress);
                }
            })
            .unwrap();
        assert!(counts.len() > 3);
        assert!(
            counts
                .windows(2)
                .all(|pair| pair[0].downloaded <= pair[1].downloaded)
        );
        assert_eq!(counts.last().unwrap().downloaded, payload().len() as u64);
        assert_eq!(
            counts.last().unwrap().total,
            known.then_some(payload().len() as u64)
        );
        updater.install(|_| {}).unwrap();
        assert_eq!(server.requests.lock().unwrap().len(), 2);
    }
}

#[test]
fn semver_precedence_and_prerelease_policy_prevent_unwanted_updates() {
    for version in ["0.9.0", "1.0.0", "1.0.0+other", "2.0.0-beta.1"] {
        let server = Server::new(|url| vec![Reply::ok(manifest(url, version))]);
        assert!(!server.updater().check(|_| {}).unwrap());
    }
    let server = Server::new(|url| vec![Reply::ok(manifest(url, "2.0.0-beta.1"))]);
    let mut config = config(&server.url);
    config.allow_prerelease = true;
    let mut updater = Updater::new(HttpBackend::new(config, RecordingInstaller).unwrap());
    assert!(updater.check(|_| {}).unwrap());
    let server = Server::new(|_| {
        vec![Reply {
            status: 204,
            headers: String::new(),
            body: vec![],
        }]
    });
    assert!(!server.updater().check(|_| {}).unwrap());
}

#[test]
fn invalid_or_incompatible_manifests_fail_without_downloading() {
    for case in [
        "missing-target",
        "unsafe-url",
        "signature",
        "format",
        "version",
        "json",
        "large",
        "status",
    ] {
        let server = Server::new(|url| {
            let mut value: serde_json::Value =
                serde_json::from_slice(&manifest(url, "2.0.0")).unwrap();
            match case {
                "missing-target" => value["platforms"] = serde_json::json!({}),
                "unsafe-url" => {
                    value["platforms"]["linux-x86_64"]["url"] = "http://example.com/update".into()
                }
                "signature" => value["platforms"]["linux-x86_64"]["signature"] = "broken".into(),
                "format" => value["platforms"]["linux-x86_64"]["format"] = "nsis".into(),
                "version" => value["version"] = "bad".into(),
                _ => {}
            }
            let reply = match case {
                "json" => Reply::ok("{"),
                "large" => Reply::ok(vec![b' '; 1024 * 1024 + 1]),
                "status" => Reply {
                    status: 503,
                    ..Reply::ok("")
                },
                _ => Reply::ok(serde_json::to_vec(&value).unwrap()),
            };
            vec![reply]
        });
        assert!(server.updater().check(|_| {}).is_err(), "{case}");
        assert_eq!(server.requests.lock().unwrap().len(), 1);
    }
}

#[test]
fn invalid_signature_truncation_limits_and_cancellation_never_produce_installable_packages() {
    for case in [
        "tampered",
        "truncated",
        "known-limit",
        "unknown-limit",
        "cancel",
        "http",
    ] {
        let server = Server::new(|url| {
            let mut data = Reply::ok(payload());
            match case {
                "tampered" => data.body[100] ^= 1,
                "truncated" => {
                    data.body.truncate(100);
                }
                "unknown-limit" => data.headers.clear(),
                "http" => data.status = 503,
                _ => {}
            }
            vec![Reply::ok(manifest(url, "2.0.0")), data]
        });
        let mut config = config(&server.url);
        if case.ends_with("limit") {
            config.max_download_bytes = 100;
        }
        let mut updater = Updater::new(HttpBackend::new(config, RecordingInstaller).unwrap());
        updater.check(|_| {}).unwrap();
        let cancel = CancellationToken::default();
        let result = updater.download(&cancel, |state| {
            if case == "cancel"
                && matches!(state, State::Downloading { progress, .. } if progress.downloaded > 0)
            {
                cancel.cancel();
            }
        });
        assert!(result.is_err(), "{case}");
        assert!(updater.install(|_| {}).is_err(), "{case}");
    }
}

#[test]
fn configuration_and_redirects_enforce_transport_boundaries() {
    for url in [
        "http://example.com/feed",
        "http://localhost/feed",
        "file:///tmp/feed",
        "https://user:password@example.com/feed",
        "bad",
    ] {
        assert!(Config::new("1.0.0", url, KEY).is_err());
    }
    assert!(Config::new("bad", "https://example.com", KEY).is_err());
    assert!(Config::new("1.0.0", "https://example.com", "bad").is_err());
    assert!(Config::new("1.0.0", "http://[::1]/feed", KEY).is_ok());
    for case in ["target", "timeout", "limit"] {
        let mut config = config("https://example.com");
        match case {
            "target" => config.target.clear(),
            "timeout" => config.timeout = Duration::ZERO,
            _ => config.max_download_bytes = 0,
        }
        assert!(HttpBackend::new(config, RecordingInstaller).is_err());
    }
    let server = Server::new(|url| {
        vec![
            Reply {
                status: 302,
                headers: format!("Location: {url}/next\r\n"),
                body: vec![],
            },
            Reply::ok(manifest(url, "2.0.0")),
        ]
    });
    assert!(server.updater().check(|_| {}).unwrap());
    let server = Server::new(|_| {
        vec![Reply {
            status: 302,
            headers: "Location: http://example.com/feed\r\n".into(),
            body: vec![],
        }]
    });
    assert!(server.updater().check(|_| {}).is_err());
    let server = Server::new(|url| {
        (0..11)
            .map(|_| Reply {
                status: 302,
                headers: format!("Location: {url}/loop\r\n"),
                body: vec![],
            })
            .collect()
    });
    assert!(server.updater().check(|_| {}).is_err());
}
