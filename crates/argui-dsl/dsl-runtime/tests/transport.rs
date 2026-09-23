#[cfg(not(target_arch = "wasm32"))]
use std::{collections::HashMap, time::Duration};

use argui_dsl_protocol::LiveMessage;
#[cfg(not(target_arch = "wasm32"))]
use argui_dsl_protocol::{DiagnosticMessage, Severity};
#[cfg(not(target_arch = "wasm32"))]
use argui_dsl_runtime::{ClientEvent, LiveClient, LivePackage, LiveRuntime, RuntimeError};

#[cfg(not(target_arch = "wasm32"))]
fn runtime() -> LiveRuntime {
    let package = LivePackage::prepare(
        1,
        1,
        argui_dsl_ir::IrProject {
            native_schema_hash: 0,
            modules: Vec::new(),
            structs: Vec::new(),
            enums: Vec::new(),
            components: Vec::new(),
            themes: Vec::new(),
            styles: Vec::new(),
            effects: Vec::new(),
            assets: Vec::new(),
        },
        HashMap::new(),
    )
    .unwrap();
    LiveRuntime::new(package).unwrap()
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn client_apply_preserves_diagnostics_restart_disconnect_and_commit_events() {
    let events = [
        ClientEvent::Diagnostics {
            generation: 4,
            diagnostics: vec![DiagnosticMessage {
                path: Some("main.argui".into()),
                start: Some(1),
                end: Some(2),
                severity: Severity::Warning,
                code: "warning".into(),
                message: "check".into(),
            }],
        },
        ClientEvent::RestartRequired {
            generation: 5,
            previous_api_hash: 1,
            next_api_hash: 2,
        },
        ClientEvent::Disconnected("host closed".into()),
        ClientEvent::Committed(argui_dsl_runtime::ReloadOutcome {
            previous_generation: 1,
            generation: 2,
            migrated_instances: 0,
        }),
    ];
    for event in events {
        let copy = event.clone();
        assert_eq!(LiveClient::apply(&mut runtime(), event).unwrap(), copy);
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn connection_timeout_and_failed_endpoint_are_bounded() {
    assert!(matches!(
        LiveClient::connect(("127.0.0.1", 0)),
        Err(RuntimeError::IncompatiblePackage(_))
    ));
    assert!(matches!(
        LiveRuntime::connect(("127.0.0.1", 0), Duration::from_millis(1)),
        Err(RuntimeError::IncompatiblePackage(_))
    ));
}

#[test]
fn protocol_status_messages_keep_their_wire_shape() {
    let messages = [
        LiveMessage::Committed { generation: 2 },
        LiveMessage::Rejected {
            generation: 2,
            message: "bad package".into(),
        },
        LiveMessage::RestartRequired {
            generation: 2,
            previous_api_hash: 1,
            next_api_hash: 2,
        },
    ];
    for message in messages {
        let encoded = argui_dsl_protocol::encode(&message).unwrap();
        assert_eq!(
            argui_dsl_protocol::decode::<LiveMessage>(&encoded).unwrap(),
            message
        );
    }
}
