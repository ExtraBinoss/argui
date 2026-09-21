use crate::{LivePackage, LiveRuntime, ReloadOutcome, RuntimeError};

/// Application-visible result from a native or browser development transport.
#[derive(Clone, Debug, PartialEq)]
pub enum ClientEvent {
    Package(Box<argui_dsl_protocol::LivePackageEnvelope>),
    Diagnostics {
        generation: u64,
        diagnostics: Vec<argui_dsl_protocol::DiagnosticMessage>,
    },
    Committed(ReloadOutcome),
    RestartRequired {
        generation: u64,
        previous_api_hash: u64,
        next_api_hash: u64,
    },
    Disconnected(String),
}

/// Prepares and atomically commits one transport event into a running UI.
pub(crate) fn apply_event(
    runtime: &mut LiveRuntime,
    event: ClientEvent,
) -> Result<ClientEvent, RuntimeError> {
    let ClientEvent::Package(envelope) = event else {
        return Ok(event);
    };
    let package = LivePackage::from_envelope(*envelope)?;
    let prepared = runtime.prepare_reload(package)?;
    Ok(ClientEvent::Committed(runtime.commit_reload(prepared)))
}

/// Converts a wire result to the bounded client event surface.
pub(crate) fn protocol_event(message: argui_dsl_protocol::LiveMessage) -> ClientEvent {
    use argui_dsl_protocol::LiveMessage;
    match message {
        LiveMessage::Package(package) => ClientEvent::Package(package),
        LiveMessage::Diagnostics {
            generation,
            diagnostics,
        } => ClientEvent::Diagnostics {
            generation,
            diagnostics,
        },
        LiveMessage::RestartRequired {
            generation,
            previous_api_hash,
            next_api_hash,
        } => ClientEvent::RestartRequired {
            generation,
            previous_api_hash,
            next_api_hash,
        },
        LiveMessage::Rejected {
            generation,
            message,
        } => ClientEvent::Diagnostics {
            generation,
            diagnostics: vec![argui_dsl_protocol::DiagnosticMessage {
                path: None,
                start: None,
                end: None,
                severity: argui_dsl_protocol::Severity::Error,
                code: "rejected".into(),
                message,
            }],
        },
        LiveMessage::Committed { generation } => ClientEvent::Diagnostics {
            generation,
            diagnostics: Vec::new(),
        },
        LiveMessage::Hello { .. } => {
            ClientEvent::Disconnected("unexpected repeated hello frame".into())
        }
    }
}

/// Converts a non-package initial event into a concise connection failure.
pub(crate) fn client_event_message(event: &ClientEvent) -> String {
    match event {
        ClientEvent::Diagnostics { diagnostics, .. } => diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.as_str())
            .collect::<Vec<_>>()
            .join("; "),
        ClientEvent::RestartRequired { .. } => "development host requested a restart".into(),
        ClientEvent::Disconnected(message) => message.clone(),
        ClientEvent::Committed(_) => "development host sent a commit without a package".into(),
        ClientEvent::Package(_) => "initial package".into(),
    }
}
