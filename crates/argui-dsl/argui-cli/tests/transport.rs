#![cfg(not(target_arch = "wasm32"))]

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use argui_cli::WebSocketHub;
use argui_dsl_protocol::{LiveMessage, decode};
use tungstenite::Message;

#[test]
fn browser_transport_receives_hello_current_and_future_generations() {
    let initial = LiveMessage::Diagnostics {
        generation: 1,
        diagnostics: Vec::new(),
    };
    let hub = WebSocketHub::bind(
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
        &initial,
    )
    .unwrap();
    let (mut client, _) = tungstenite::connect(format!("ws://{}", hub.local_addr())).unwrap();
    let hello = binary(client.read().unwrap());
    assert!(matches!(
        decode::<LiveMessage>(&hello).unwrap(),
        LiveMessage::Hello { .. }
    ));
    let current = binary(client.read().unwrap());
    assert_eq!(decode::<LiveMessage>(&current).unwrap(), initial);

    let next = LiveMessage::Diagnostics {
        generation: 2,
        diagnostics: Vec::new(),
    };
    hub.broadcast(&next).unwrap();
    let received = binary(client.read().unwrap());
    assert_eq!(decode::<LiveMessage>(&received).unwrap(), next);
}

#[test]
fn browser_transport_drops_disconnected_clients_and_rejects_bad_handshakes() {
    let initial = LiveMessage::Diagnostics {
        generation: 1,
        diagnostics: Vec::new(),
    };
    let hub = WebSocketHub::bind(
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
        &initial,
    )
    .unwrap();

    let mut malformed = std::net::TcpStream::connect(hub.local_addr()).unwrap();
    std::io::Write::write_all(&mut malformed, b"not a websocket\r\n\r\n").unwrap();
    drop(malformed);
    let (mut client, _) = tungstenite::connect(format!("ws://{}", hub.local_addr())).unwrap();
    let _ = client.read().unwrap();
    let _ = client.read().unwrap();
    drop(client);
    std::thread::sleep(std::time::Duration::from_millis(20));

    hub.broadcast(&LiveMessage::Diagnostics {
        generation: 2,
        diagnostics: Vec::new(),
    })
    .unwrap();
}

/// Removes a client whose TCP connection is explicitly shut down before broadcast.
#[test]
fn browser_transport_prunes_explicitly_shutdown_clients() {
    let initial = LiveMessage::Diagnostics {
        generation: 1,
        diagnostics: Vec::new(),
    };
    let hub = WebSocketHub::bind(
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
        &initial,
    )
    .unwrap();
    let (mut client, _) = tungstenite::connect(format!("ws://{}", hub.local_addr())).unwrap();
    let _ = client.read().unwrap();
    let _ = client.read().unwrap();
    if let tungstenite::stream::MaybeTlsStream::Plain(stream) = client.get_mut() {
        stream.shutdown(std::net::Shutdown::Both).unwrap();
    }
    drop(client);
    std::thread::sleep(std::time::Duration::from_millis(20));

    hub.broadcast(&LiveMessage::Diagnostics {
        generation: 2,
        diagnostics: Vec::new(),
    })
    .unwrap();
}

#[test]
fn browser_transport_reports_address_binding_errors() {
    let initial = LiveMessage::Diagnostics {
        generation: 1,
        diagnostics: Vec::new(),
    };
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let error = match WebSocketHub::bind(address, &initial) {
        Ok(_) => panic!("binding an occupied address should fail"),
        Err(error) => error,
    };
    assert!(matches!(error.kind(), std::io::ErrorKind::AddrInUse));
}

/// Extracts binary WebSocket frames from the protocol client stream.
fn binary(message: Message) -> Vec<u8> {
    match message {
        Message::Binary(bytes) => bytes.to_vec(),
        other => panic!("expected binary protocol frame, got {other:?}"),
    }
}
