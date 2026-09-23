use std::{
    net::{TcpStream, ToSocketAddrs},
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, RecvTimeoutError},
    },
    time::Duration,
};

use argui_dsl_protocol::{LiveMessage, PackageHeader, RuntimeVersions, read_frame, write_frame};

use crate::{
    ClientEvent, LivePackage, LiveRuntime, RuntimeError,
    transport::{apply_event, client_event_message, protocol_event},
};

/// Background TCP receiver; compilation and file watching remain out-of-process.
pub struct LiveClient {
    receiver: Receiver<ClientEvent>,
    writer: Mutex<TcpStream>,
}

impl LiveClient {
    /// Connects to `argui dev`, validates its exact versions, and starts a reader thread.
    ///
    /// # Errors
    ///
    /// Returns connection, handshake, framing, or compatibility failures.
    pub fn connect(address: impl ToSocketAddrs) -> Result<Self, RuntimeError> {
        let mut stream = TcpStream::connect(address)
            .map_err(|error| RuntimeError::IncompatiblePackage(error.to_string()))?;
        let hello: LiveMessage = read_frame(&mut stream)
            .map_err(|error| RuntimeError::IncompatiblePackage(error.to_string()))?;
        let LiveMessage::Hello {
            protocol_version,
            ir_format_version,
            engine_version,
        } = hello
        else {
            return Err(RuntimeError::IncompatiblePackage(
                "development host did not send a hello frame".into(),
            ));
        };
        RuntimeVersions::current()
            .check(&PackageHeader {
                protocol_version,
                ir_format_version,
                engine_version,
                public_api_hash: 0,
                generation: 1,
            })
            .map_err(|error| RuntimeError::IncompatiblePackage(error.to_string()))?;
        let writer = stream
            .try_clone()
            .map_err(|error| RuntimeError::IncompatiblePackage(error.to_string()))?;
        let (sender, receiver) = mpsc::channel();
        std::thread::Builder::new()
            .name("argui-dsl-live-client".into())
            .spawn(move || {
                loop {
                    match read_frame::<LiveMessage>(&mut stream) {
                        Ok(message) => {
                            let event = protocol_event(message);
                            if sender.send(event).is_err() {
                                break;
                            }
                        }
                        Err(error) => {
                            let _ = sender.send(ClientEvent::Disconnected(error.to_string()));
                            break;
                        }
                    }
                }
            })
            .map_err(|error| RuntimeError::IncompatiblePackage(error.to_string()))?;
        Ok(Self {
            receiver,
            writer: Mutex::new(writer),
        })
    }

    /// Waits for a compiler result without polling files or requesting frames.
    ///
    /// Returns `None` on timeout and a disconnection event when the reader exits.
    #[must_use]
    pub fn receive_timeout(&self, timeout: Duration) -> Option<ClientEvent> {
        match self.receiver.recv_timeout(timeout) {
            Ok(event) => Some(event),
            Err(RecvTimeoutError::Timeout) => None,
            Err(RecvTimeoutError::Disconnected) => {
                Some(ClientEvent::Disconnected("transport thread stopped".into()))
            }
        }
    }

    /// Prepares and atomically commits one package event into a running UI.
    ///
    /// Non-package events are returned unchanged for application diagnostics.
    ///
    /// # Errors
    ///
    /// Returns compatibility, preparation, migration, asset, or shader failures
    /// while the previous generation remains active.
    pub fn apply(
        runtime: &mut LiveRuntime,
        event: ClientEvent,
    ) -> Result<ClientEvent, RuntimeError> {
        apply_event(runtime, event)
    }

    /// Confirms a committed generation or reports a transactional rejection to the host.
    ///
    /// `message` is a protocol status corresponding to a received package.
    ///
    /// # Errors
    /// Returns a poisoned writer or transport I/O failure.
    pub fn acknowledge(&self, message: &LiveMessage) -> Result<(), RuntimeError> {
        let mut writer = self.writer.lock().map_err(|_| {
            RuntimeError::IncompatiblePackage("live status writer lock was poisoned".into())
        })?;
        write_frame(&mut *writer, message)
            .map_err(|error| RuntimeError::IncompatiblePackage(error.to_string()))
    }
}

impl LiveRuntime {
    /// Connects to `argui dev`, mounts the first exported entry component, and listens idle.
    ///
    /// * `address` — native TCP development host address.
    /// * `timeout` — maximum duration allowed for the initial validated package.
    ///
    /// # Errors
    ///
    /// Returns transport, compatibility, initial diagnostics, missing-root, or mount failures.
    pub fn connect(address: impl ToSocketAddrs, timeout: Duration) -> Result<Self, RuntimeError> {
        let registry = argui_schema::builtin::registry()
            .map_err(|error| RuntimeError::Schema(error.to_string()))?;
        Self::connect_with_registry(address, timeout, registry)
    }

    /// Connects to `argui dev` using the application's native extension registry.
    ///
    /// * `address` — native TCP development host address.
    /// * `timeout` — maximum wait for the first accepted package.
    /// * `registry` — exact versioned native contract used by the dev compiler.
    ///
    /// # Errors
    ///
    /// Returns connection, package, ABI, or mount errors before displaying a mismatched UI.
    pub fn connect_with_registry(
        address: impl ToSocketAddrs,
        timeout: Duration,
        registry: argui_schema::SchemaRegistry,
    ) -> Result<Self, RuntimeError> {
        let client = LiveClient::connect(address)?;
        let initial = client.receive_timeout(timeout).ok_or_else(|| {
            RuntimeError::IncompatiblePackage(
                "development host did not send an initial package".into(),
            )
        })?;
        let ClientEvent::Package(envelope) = initial else {
            return Err(RuntimeError::IncompatiblePackage(client_event_message(
                &initial,
            )));
        };
        let package = LivePackage::from_envelope(*envelope)?;
        let root = package.roots.first().copied().ok_or_else(|| {
            RuntimeError::IncompatiblePackage(
                "entry module must export at least one component".into(),
            )
        })?;
        let mut runtime = Self::new_with_registry(package, registry)?;
        runtime.mount(root, [])?;
        client.acknowledge(&LiveMessage::Committed {
            generation: runtime.generation(),
        })?;
        runtime.live_client = Some(Arc::new(Mutex::new(client)));
        Ok(runtime)
    }

    /// Starts one presentation-owned blocking receive without requesting idle frames.
    pub(crate) fn ensure_client_listener(&mut self, context: &mut argui_runtime::Context<Self>) {
        if self
            .live_task
            .as_ref()
            .is_some_and(|task| !task.is_finished())
        {
            return;
        }
        let Some(client) = self.live_client.clone() else {
            return;
        };
        self.live_task = None;
        let task = context.spawn_blocking(
            move |cancellation| loop {
                if cancellation.is_cancelled() {
                    return ClientEvent::Disconnected("live listener cancelled".into());
                }
                let event = client
                    .lock()
                    .map_err(|_| "live transport lock was poisoned".to_owned())
                    .and_then(|client| {
                        client
                            .receive_timeout(Duration::from_millis(250))
                            .ok_or_else(|| "timeout".to_owned())
                    });
                match event {
                    Ok(event) => return event,
                    Err(message) if message == "timeout" => {}
                    Err(message) => return ClientEvent::Disconnected(message),
                }
            },
            |runtime, result, context| {
                runtime.live_task = None;
                let event = match result {
                    Ok(event) => {
                        let generation = match &event {
                            ClientEvent::Package(package) => Some(package.header.generation),
                            _ => None,
                        };
                        match LiveClient::apply(runtime, event) {
                            Ok(event) => {
                                if let ClientEvent::Committed(outcome) = &event {
                                    runtime.send_client_status(&LiveMessage::Committed {
                                        generation: outcome.generation,
                                    });
                                }
                                event
                            }
                            Err(RuntimeError::RestartRequired { previous, next }) => {
                                let generation = generation.unwrap_or(runtime.generation());
                                runtime.send_client_status(&LiveMessage::RestartRequired {
                                    generation,
                                    previous_api_hash: previous,
                                    next_api_hash: next,
                                });
                                ClientEvent::RestartRequired {
                                    generation,
                                    previous_api_hash: previous,
                                    next_api_hash: next,
                                }
                            }
                            Err(error) => {
                                let generation = generation.unwrap_or(runtime.generation());
                                runtime.send_client_status(&LiveMessage::Rejected {
                                    generation,
                                    message: error.to_string(),
                                });
                                ClientEvent::Diagnostics {
                                    generation,
                                    diagnostics: vec![argui_dsl_protocol::DiagnosticMessage {
                                        path: None,
                                        start: None,
                                        end: None,
                                        severity: argui_dsl_protocol::Severity::Error,
                                        code: "runtime".into(),
                                        message: error.to_string(),
                                    }],
                                }
                            }
                        }
                    }
                    Err(error) => ClientEvent::Disconnected(error.to_string()),
                };
                let keep_listening = !matches!(event, ClientEvent::Disconnected(_));
                let rebuild = matches!(event, ClientEvent::Committed(_));
                runtime.last_client_event = Some(event);
                if rebuild {
                    context.notify();
                }
                if keep_listening {
                    runtime.ensure_client_listener(context);
                }
            },
        );
        match task {
            Ok(task) => self.live_task = Some(task),
            Err(error) => {
                self.last_client_event = Some(ClientEvent::Disconnected(error.to_string()));
            }
        }
    }

    /// Sends one status without turning a successful UI commit into a transport failure.
    fn send_client_status(&self, message: &LiveMessage) {
        if let Some(client) = &self.live_client
            && let Ok(client) = client.lock()
        {
            let _ = client.acknowledge(message);
        }
    }
}
