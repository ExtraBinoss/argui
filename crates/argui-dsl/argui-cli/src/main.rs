use std::{
    env, io,
    path::{Path, PathBuf},
};
#[cfg(not(target_arch = "wasm32"))]
use std::{
    io::Read,
    net::{SocketAddr, TcpListener, TcpStream},
    process::{Child, Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};

#[cfg(not(target_arch = "wasm32"))]
use argui_cli::{
    ChangeTracker, WebSocketHub,
    dashboard::{AMBER, DevConsole},
    is_relevant_event,
};
use argui_cli::{
    DevCompilerService, canonical_relative, complete, format, new_project, schema, symbols,
};
use argui_dsl_protocol::LiveMessage;
#[cfg(not(target_arch = "wasm32"))]
use argui_dsl_protocol::{
    ENGINE_COMPATIBILITY_VERSION, IR_FORMAT_VERSION, PROTOCOL_VERSION, read_frame, write_frame,
};
#[cfg(not(target_arch = "wasm32"))]
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
#[cfg(not(target_arch = "wasm32"))]
use ratatui::style::Color;

#[cfg(not(target_arch = "wasm32"))]
static STOP: AtomicBool = AtomicBool::new(false);

fn main() {
    if let Err(error) = run() {
        eprintln!("argui: {error}");
        std::process::exit(1);
    }
}

/// Dispatches the command-line interface without hidden global state.
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = env::args().skip(1).collect::<Vec<_>>();
    let command = if arguments.is_empty() {
        None
    } else {
        Some(arguments.remove(0))
    };
    match command.as_deref() {
        Some("dev") => {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let values = positional(&arguments);
                let entry = values.first().copied().unwrap_or("ui/main.argui");
                let bind = values.get(1).copied().unwrap_or("127.0.0.1:4777").parse()?;
                dev(
                    env::current_dir()?,
                    entry,
                    bind,
                    !arguments.iter().any(|argument| argument == "--no-run"),
                )
            }
            #[cfg(target_arch = "wasm32")]
            {
                Err("argui dev requires a native host".into())
            }
        }
        Some("check") => {
            let entry = positional(&arguments)
                .first()
                .copied()
                .unwrap_or("ui/main.argui");
            check(env::current_dir()?, entry)
        }
        Some("fmt") => {
            let check_only = arguments.iter().any(|argument| argument == "--check");
            let paths = positional(&arguments)
                .into_iter()
                .map(str::to_owned)
                .collect::<Vec<_>>();
            print_json(format(&env::current_dir()?, &paths, check_only)?)
        }
        Some("new") => {
            let target = positional(&arguments)
                .first()
                .copied()
                .ok_or("new requires a project directory")?;
            let path = new_project(&env::current_dir()?, target)?;
            print_json(serde_json::json!({"ok": true, "path": path}))
        }
        Some("schema") => print_json(schema(positional(&arguments).first().copied())?),
        Some("complete") => {
            let values = positional(&arguments);
            let path = values.first().ok_or("complete requires PATH")?;
            let position = values.get(1).ok_or("complete requires LINE:COLUMN")?;
            print_json(complete(&env::current_dir()?, path, position)?)
        }
        Some("symbols") => print_json(symbols(
            &env::current_dir()?,
            positional(&arguments).first().copied(),
        )?),
        Some("help" | "--help" | "-h") | None => {
            print_help();
            Ok(())
        }
        Some(command) => Err(format!("unknown command `{command}`; run `argui help`").into()),
    }
}

/// Filters CLI switches from ordered positional arguments.
fn positional(arguments: &[String]) -> Vec<&str> {
    arguments
        .iter()
        .filter(|argument| !argument.starts_with("--"))
        .map(String::as_str)
        .collect()
}

/// Prints one stable pretty-JSON command result.
fn print_json(value: serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}

/// Runs the external watcher, incremental compiler, transports, and optional application.
#[cfg(not(target_arch = "wasm32"))]
fn dev(
    root: PathBuf,
    entry: &str,
    bind: SocketAddr,
    launch_application: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let entry = entry_path(&root, entry)?;
    let mut service = DevCompilerService::open(&root, entry)?;
    let mut changes = ChangeTracker::open(&root)?;
    let stdlib_root = service.stdlib_root().map(Path::to_path_buf);
    let mut stdlib_changes = stdlib_root.as_ref().map(ChangeTracker::open).transpose()?;
    let listener = TcpListener::bind(bind)?;
    listener.set_nonblocking(true)?;
    let (sender, receiver) = mpsc::channel();
    let mut watcher = RecommendedWatcher::new(
        move |event| {
            let _ = sender.send(event);
        },
        Config::default(),
    )?;
    watcher.watch(&root, RecursiveMode::Recursive)?;
    if let Some(path) = &stdlib_root {
        watcher.watch(path, RecursiveMode::Recursive)?;
    }
    let mut clients = Vec::new();
    let (status_sender, status_receiver) = mpsc::channel();
    let mut latest = service.compile().message;
    let websocket_address = SocketAddr::new(bind.ip(), bind.port().saturating_add(1));
    let websocket = WebSocketHub::bind(websocket_address, &latest)?;
    let mut console = DevConsole::new(&root, bind, websocket_address)?;
    ctrlc::set_handler(|| STOP.store(true, Ordering::Relaxed))?;
    if !console.interactive() {
        console.note(
            format!("◆ Argui dev · watching {}", root.display()),
            Color::Cyan,
        )?;
        console.note(format!("◇ Native tcp://{bind}"), Color::Cyan)?;
        console.note(format!("◇ Browser ws://{websocket_address}"), Color::Cyan)?;
    }
    console.compilation(&latest)?;
    let (mut application, application_logs) = if launch_application {
        console.native_build_started()?;
        let (application, logs) = launch_dev_application(&root, bind, console.interactive())?;
        (Some(application), logs)
    } else {
        (None, None)
    };
    loop {
        if console.pump_input()? || STOP.load(Ordering::Relaxed) {
            return Ok(());
        }
        if let Some(logs) = &application_logs {
            for line in logs.try_iter().take(128) {
                console.application_output(line)?;
            }
        }
        console.native_build_tick()?;
        let status = application
            .as_mut()
            .map(|application| application.child.try_wait())
            .transpose()?
            .flatten();
        if let Some(status) = status {
            return if status.success() {
                Ok(())
            } else {
                Err(format!("development application exited with {status}").into())
            };
        }
        accept_clients(
            &listener,
            &mut clients,
            &latest,
            &status_sender,
            &mut console,
        )?;
        report_client_statuses(&status_receiver, &mut console)?;
        match receiver.recv_timeout(Duration::from_millis(100)) {
            Ok(Ok(event)) if is_relevant_event(&event) => {
                let paths = settled_paths(event, &receiver, &mut console)?;
                let mut detected = changes.refresh(paths.iter().cloned())?;
                if let Some(tracker) = &mut stdlib_changes {
                    let mut library = tracker.refresh(paths)?;
                    for change in &mut library {
                        change.path = format!("@argui/ui/{}", change.path);
                    }
                    detected.extend(library);
                }
                if detected.is_empty() {
                    continue;
                }
                for change in &detected {
                    console.change(change)?;
                }
                service.refresh_sources()?;
                latest = service.compile().message;
                console.compilation(&latest)?;
                broadcast(&mut clients, &latest);
                websocket.broadcast(&latest)?;
                console.clients(clients.len())?;
                if clients.is_empty() {
                    console.note(
                        "No native client connected; compilation cannot update a window",
                        AMBER,
                    )?;
                }
            }
            Ok(Err(error)) => console.note(format!("Watcher: {error}"), Color::Red)?,
            Ok(Ok(_)) | Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err("filesystem watcher disconnected".into());
            }
        }
    }
}

/// Starts the application under `root` with the live feature and server `bind`.
///
/// `capture_output` routes child output into dashboard lines; the returned
/// receiver is absent for ordinary line-oriented execution.
///
/// # Errors
///
/// Returns process-spawn or output-reader-spawn errors.
#[cfg(not(target_arch = "wasm32"))]
fn launch_dev_application(
    root: &Path,
    bind: SocketAddr,
    capture_output: bool,
) -> Result<(ChildGuard, Option<mpsc::Receiver<String>>), std::io::Error> {
    let mut command = Command::new("cargo");
    command
        .args(["run", "--features", "argui-live"])
        .current_dir(root)
        .env("ARGUI_DEV_ADDRESS", bind.to_string())
        .env("ARGUI_DEV_CLIENT", "1");
    if capture_output {
        command
            .env("CARGO_TERM_PROGRESS_WHEN", "always")
            .env("CARGO_TERM_PROGRESS_WIDTH", "80")
            .env("CARGO_TERM_COLOR", "never");
        command.stdout(Stdio::piped()).stderr(Stdio::piped());
    }
    let mut application = ChildGuard {
        child: command.spawn()?,
    };
    let logs = if capture_output {
        let (sender, receiver) = mpsc::channel();
        for output in [
            application
                .child
                .stdout
                .take()
                .map(|output| Box::new(output) as Box<dyn Read + Send>),
            application
                .child
                .stderr
                .take()
                .map(|output| Box::new(output) as Box<dyn Read + Send>),
        ]
        .into_iter()
        .flatten()
        {
            let sender = sender.clone();
            std::thread::Builder::new()
                .name("argui-dev-application-output".into())
                .spawn(move || forward_application_output(output, sender))?;
        }
        Some(receiver)
    } else {
        None
    };
    Ok((application, logs))
}

/// Forwards both Cargo carriage-return progress and ordinary newline output.
///
/// `output` is one child pipe and `sender` receives each complete UTF-8-lossy frame.
/// Read errors and a dropped receiver end the forwarding thread.
#[cfg(not(target_arch = "wasm32"))]
fn forward_application_output(mut output: Box<dyn Read + Send>, sender: mpsc::Sender<String>) {
    let mut buffer = [0_u8; 4096];
    let mut pending = Vec::new();
    loop {
        let count = match output.read(&mut buffer) {
            Ok(0) | Err(_) => break,
            Ok(count) => count,
        };
        for byte in &buffer[..count] {
            if *byte == b'\r' || *byte == b'\n' {
                if !pending.is_empty() {
                    let line = String::from_utf8_lossy(&pending).into_owned();
                    pending.clear();
                    if sender.send(line).is_err() {
                        return;
                    }
                }
            } else {
                pending.push(*byte);
            }
        }
    }
    if !pending.is_empty() {
        let _ = sender.send(String::from_utf8_lossy(&pending).into_owned());
    }
}

/// Child process guard which prevents an orphaned development application.
#[cfg(not(target_arch = "wasm32"))]
struct ChildGuard {
    child: Child,
}

#[cfg(not(target_arch = "wasm32"))]
impl Drop for ChildGuard {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

/// Compiles once and prints machine-readable diagnostics on failure.
fn check(root: PathBuf, entry: &str) -> Result<(), Box<dyn std::error::Error>> {
    let entry = entry_path(&root, entry)?;
    let mut service = DevCompilerService::open(root, entry)?;
    let attempt = service.compile();
    match attempt.message {
        LiveMessage::Package(_) => {
            println!("{{\"ok\":true,\"generation\":{}}}", attempt.generation);
            Ok(())
        }
        message => {
            println!("{}", serde_json::to_string(&message)?);
            Err("DSL validation failed".into())
        }
    }
}

/// Resolves a user entry argument to a canonical project-relative module path.
fn entry_path(root: &Path, entry: &str) -> Result<String, io::Error> {
    let path = Path::new(entry);
    let joined;
    let path = if path.is_absolute() {
        path
    } else {
        joined = root.join(path);
        &joined
    };
    canonical_relative(root, path)
}

/// Accepts every pending client, sends the current state, and updates `console`.
///
/// # Errors
///
/// Returns terminal output errors while reporting connections or accept failures.
#[cfg(not(target_arch = "wasm32"))]
fn accept_clients(
    listener: &TcpListener,
    clients: &mut Vec<TcpStream>,
    latest: &LiveMessage,
    status_sender: &mpsc::Sender<(SocketAddr, LiveMessage)>,
    console: &mut DevConsole,
) -> io::Result<()> {
    loop {
        match listener.accept() {
            Ok((mut stream, peer)) => {
                let hello = LiveMessage::Hello {
                    protocol_version: PROTOCOL_VERSION,
                    ir_format_version: IR_FORMAT_VERSION,
                    engine_version: ENGINE_COMPATIBILITY_VERSION.into(),
                };
                if write_frame(&mut stream, &hello).is_ok()
                    && write_frame(&mut stream, latest).is_ok()
                {
                    console.connected(peer)?;
                    if let Ok(mut reader) = stream.try_clone() {
                        let sender = status_sender.clone();
                        let _ = std::thread::Builder::new()
                            .name("argui-dev-client-status".into())
                            .spawn(move || {
                                while let Ok(message) = read_frame::<LiveMessage>(&mut reader) {
                                    if sender.send((peer, message)).is_err() {
                                        break;
                                    }
                                }
                            });
                    }
                    clients.push(stream);
                }
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => break,
            Err(error) => {
                console.note(format!("Accept: {error}"), Color::Red)?;
                break;
            }
        }
    }
    console.clients(clients.len())?;
    Ok(())
}

/// Broadcasts one transactional result and removes disconnected clients.
#[cfg(not(target_arch = "wasm32"))]
fn broadcast(clients: &mut Vec<TcpStream>, message: &LiveMessage) {
    clients.retain_mut(|client| write_frame(client, message).is_ok());
}

/// Debounces `first` and later relevant events from `receiver`, reporting errors.
///
/// # Errors
///
/// Returns terminal output errors while reporting watcher failures.
#[cfg(not(target_arch = "wasm32"))]
fn settled_paths(
    first: notify::Event,
    receiver: &mpsc::Receiver<notify::Result<notify::Event>>,
    console: &mut DevConsole,
) -> io::Result<Vec<PathBuf>> {
    let mut paths = first.paths;
    let deadline = Instant::now() + Duration::from_millis(400);
    while Instant::now() < deadline {
        match receiver.recv_timeout(Duration::from_millis(75)) {
            Ok(Ok(event)) if is_relevant_event(&event) => paths.extend(event.paths),
            Ok(Err(error)) => console.note(format!("Watcher: {error}"), Color::Red)?,
            Ok(Ok(_)) => {}
            Err(mpsc::RecvTimeoutError::Timeout | mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    Ok(paths)
}

/// Reports native acknowledgements from `receiver` through `console`.
///
/// # Errors
///
/// Returns terminal output errors.
#[cfg(not(target_arch = "wasm32"))]
fn report_client_statuses(
    receiver: &mpsc::Receiver<(SocketAddr, LiveMessage)>,
    console: &mut DevConsole,
) -> io::Result<()> {
    while let Ok((peer, message)) = receiver.try_recv() {
        console.client_status(peer, &message)?;
    }
    Ok(())
}

/// Prints the stable command surface.
fn print_help() {
    println!(
        "Argui DSL tooling\n\n  argui new DIRECTORY\n  argui dev [ENTRY] [BIND] [--no-run]\n  argui check [ENTRY] [--json]\n  argui fmt [PATH ...] [--check]\n  argui schema [COMPONENT] [--json]\n  argui complete PATH LINE:COLUMN [--json]\n  argui symbols [PATH] [--json]"
    );
}
