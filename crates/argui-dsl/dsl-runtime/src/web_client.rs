use std::{cell::RefCell, rc::Rc};

use futures_channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
use futures_util::StreamExt;
use wasm_bindgen::{JsCast, closure::Closure};

use crate::{
    ClientEvent, LivePackage, LiveRuntime, RuntimeError,
    transport::{apply_event, client_event_message, protocol_event},
};

/// Browser WebSocket receiver retaining its JavaScript event closures.
pub(crate) struct WebLiveClient {
    receiver: RefCell<UnboundedReceiver<Result<argui_dsl_protocol::LiveMessage, String>>>,
    _socket: web_sys::WebSocket,
    _message: Closure<dyn FnMut(web_sys::MessageEvent)>,
    _error: Closure<dyn FnMut(web_sys::ErrorEvent)>,
    _close: Closure<dyn FnMut(web_sys::CloseEvent)>,
}

impl WebLiveClient {
    /// Opens a binary WebSocket and connects its browser events to an async channel.
    fn connect(url: &str) -> Result<Self, RuntimeError> {
        let socket = web_sys::WebSocket::new(url)
            .map_err(|error| RuntimeError::IncompatiblePackage(format!("{error:?}")))?;
        socket.set_binary_type(web_sys::BinaryType::Arraybuffer);
        let (sender, receiver) = unbounded();
        let message = message_callback(sender.clone());
        let error_sender = sender.clone();
        let error = Closure::new(move |event: web_sys::ErrorEvent| {
            send(&error_sender, Err(event.message()));
        });
        let close = Closure::new(move |event: web_sys::CloseEvent| {
            send(
                &sender,
                Err(format!(
                    "WebSocket closed with code {}: {}",
                    event.code(),
                    event.reason()
                )),
            );
        });
        socket.set_onmessage(Some(message.as_ref().unchecked_ref()));
        socket.set_onerror(Some(error.as_ref().unchecked_ref()));
        socket.set_onclose(Some(close.as_ref().unchecked_ref()));
        Ok(Self {
            receiver: RefCell::new(receiver),
            _socket: socket,
            _message: message,
            _error: error,
            _close: close,
        })
    }

    /// Waits for and validates the next non-handshake compiler event.
    async fn receive(&self) -> ClientEvent {
        loop {
            let Some(message) = self.receiver.borrow_mut().next().await else {
                return ClientEvent::Disconnected("browser transport stopped".into());
            };
            let message = match message {
                Ok(message) => message,
                Err(error) => return ClientEvent::Disconnected(error),
            };
            if let argui_dsl_protocol::LiveMessage::Hello {
                protocol_version,
                ir_format_version,
                engine_version,
            } = message
            {
                let header = argui_dsl_protocol::PackageHeader {
                    protocol_version,
                    ir_format_version,
                    engine_version,
                    public_api_hash: 0,
                    generation: 1,
                };
                if let Err(error) = argui_dsl_protocol::RuntimeVersions::current().check(&header) {
                    return ClientEvent::Disconnected(error.to_string());
                }
                continue;
            }
            return protocol_event(message);
        }
    }
}

impl LiveRuntime {
    /// Connects a browser runtime to the host WebSocket and mounts its first entry root.
    ///
    /// * `url` — WebSocket URL exposed by `argui dev`.
    ///
    /// # Errors
    ///
    /// Returns JavaScript transport, compatibility, initial diagnostics, or mount failures.
    pub async fn connect_web(url: &str) -> Result<Self, RuntimeError> {
        let client = Rc::new(WebLiveClient::connect(url)?);
        let initial = client.receive().await;
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
        let mut runtime = Self::new(package)?;
        runtime.mount(root, [])?;
        runtime.web_client = Some(client);
        Ok(runtime)
    }

    /// Starts one presentation-owned browser receive without requesting idle frames.
    pub(crate) fn ensure_web_listener(&mut self, context: &mut argui_runtime::Context<Self>) {
        if self
            .live_task
            .as_ref()
            .is_some_and(|task| !task.is_finished())
        {
            return;
        }
        let Some(client) = self.web_client.clone() else {
            return;
        };
        self.live_task = None;
        let task = context.spawn(
            async move { client.receive().await },
            |runtime, result, context| {
                runtime.live_task = None;
                let event = match result {
                    Ok(event) => match apply_event(runtime, event) {
                        Ok(event) => event,
                        Err(error) => ClientEvent::Disconnected(error.to_string()),
                    },
                    Err(error) => ClientEvent::Disconnected(error.to_string()),
                };
                let keep_listening = !matches!(event, ClientEvent::Disconnected(_));
                let rebuild = matches!(event, ClientEvent::Committed(_));
                runtime.last_client_event = Some(event);
                if rebuild {
                    context.notify();
                }
                if keep_listening {
                    runtime.ensure_web_listener(context);
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
}

/// Creates a browser message callback which decodes binary protocol frames.
fn message_callback(
    sender: UnboundedSender<Result<argui_dsl_protocol::LiveMessage, String>>,
) -> Closure<dyn FnMut(web_sys::MessageEvent)> {
    Closure::new(move |event: web_sys::MessageEvent| {
        let result = event
            .data()
            .dyn_into::<js_sys::ArrayBuffer>()
            .map_err(|_| "WebSocket message is not a binary ArrayBuffer".to_owned())
            .and_then(|buffer| {
                let bytes = js_sys::Uint8Array::new(&buffer).to_vec();
                argui_dsl_protocol::decode(&bytes).map_err(|error| error.to_string())
            });
        send(&sender, result);
    })
}

/// Sends a browser callback result while ignoring a closed application channel.
fn send(
    sender: &UnboundedSender<Result<argui_dsl_protocol::LiveMessage, String>>,
    value: Result<argui_dsl_protocol::LiveMessage, String>,
) {
    let _ = sender.unbounded_send(value);
}
