use std::{
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{Arc, Mutex},
};

use argui_dsl_protocol::{
    ENGINE_COMPATIBILITY_VERSION, IR_FORMAT_VERSION, LiveMessage, PROTOCOL_VERSION, encode,
};
use tungstenite::{Message, WebSocket};

/// Browser/device WebSocket broadcaster sharing the transport-independent protocol.
pub struct WebSocketHub {
    address: SocketAddr,
    clients: Arc<Mutex<Vec<WebSocket<TcpStream>>>>,
    latest: Arc<Mutex<Vec<u8>>>,
}

impl WebSocketHub {
    /// Starts a WebSocket acceptor on a host-visible development address.
    ///
    /// # Errors
    ///
    /// Returns address binding or accept-thread creation failures.
    pub fn bind(address: SocketAddr, latest: &LiveMessage) -> Result<Self, std::io::Error> {
        let listener = TcpListener::bind(address)?;
        let address = listener.local_addr()?;
        let clients = Arc::new(Mutex::new(Vec::new()));
        let latest = Arc::new(Mutex::new(encode(latest).map_err(std::io::Error::other)?));
        let thread_clients = Arc::clone(&clients);
        let thread_latest = Arc::clone(&latest);
        std::thread::Builder::new()
            .name("argui-dsl-websocket-host".into())
            .spawn(move || accept_loop(listener, &thread_clients, &thread_latest))?;
        Ok(Self {
            address,
            clients,
            latest,
        })
    }

    /// Returns the actual listening address, including an OS-selected port.
    #[must_use]
    pub const fn local_addr(&self) -> SocketAddr {
        self.address
    }

    /// Broadcasts a compiler result and retains only writable clients.
    ///
    /// # Errors
    ///
    /// Returns serialization or poisoned-lock failures.
    pub fn broadcast(&self, message: &LiveMessage) -> Result<(), std::io::Error> {
        let bytes = encode(message).map_err(std::io::Error::other)?;
        *self
            .latest
            .lock()
            .map_err(|_| std::io::Error::other("latest WebSocket frame lock poisoned"))? =
            bytes.clone();
        self.clients
            .lock()
            .map_err(|_| std::io::Error::other("WebSocket client lock poisoned"))?
            .retain_mut(|client| client.send(Message::Binary(bytes.clone().into())).is_ok());
        Ok(())
    }
}

/// Accepts WebSockets and primes each one with hello plus the last compiler result.
fn accept_loop(
    listener: TcpListener,
    clients: &Mutex<Vec<WebSocket<TcpStream>>>,
    latest: &Mutex<Vec<u8>>,
) {
    for stream in listener.incoming() {
        let Ok(stream) = stream else {
            continue;
        };
        let Ok(mut socket) = tungstenite::accept(stream) else {
            continue;
        };
        let hello = LiveMessage::Hello {
            protocol_version: PROTOCOL_VERSION,
            ir_format_version: IR_FORMAT_VERSION,
            engine_version: ENGINE_COMPATIBILITY_VERSION.into(),
        };
        let Ok(hello) = encode(&hello) else {
            continue;
        };
        let current = match latest.lock() {
            Ok(value) => value.clone(),
            Err(_) => continue,
        };
        if socket.send(Message::Binary(hello.into())).is_err()
            || socket.send(Message::Binary(current.into())).is_err()
        {
            continue;
        }
        let _ = socket
            .get_mut()
            .set_write_timeout(Some(std::time::Duration::from_millis(100)));
        if let Ok(mut clients) = clients.lock() {
            clients.push(socket);
        }
    }
}
