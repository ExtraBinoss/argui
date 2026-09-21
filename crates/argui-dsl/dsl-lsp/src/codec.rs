use std::io::{BufRead, Write};

use serde_json::Value;

/// Stateful reader for `Content-Length` framed JSON-RPC messages.
pub struct MessageReader<R> {
    input: R,
}

impl<R: BufRead> MessageReader<R> {
    /// Wraps one buffered LSP input stream.
    ///
    /// * `input` — buffered byte stream containing LSP frames.
    #[must_use]
    pub const fn new(input: R) -> Self {
        Self { input }
    }

    /// Reads and decodes the next JSON-RPC message, or EOF.
    ///
    /// # Errors
    ///
    /// Returns malformed-header, truncated-body, UTF-8, or JSON errors.
    pub fn read_message(&mut self) -> Result<Option<Value>, Box<dyn std::error::Error>> {
        let mut length = None;
        loop {
            let mut line = String::new();
            if self.input.read_line(&mut line)? == 0 {
                return if length.is_none() {
                    Ok(None)
                } else {
                    Err("unexpected EOF in LSP headers".into())
                };
            }
            if line == "\r\n" || line == "\n" {
                break;
            }
            if let Some(value) = line
                .strip_prefix("Content-Length:")
                .or_else(|| line.strip_prefix("content-length:"))
            {
                length = Some(value.trim().parse::<usize>()?);
            }
        }
        let length = length.ok_or("LSP frame lacks Content-Length")?;
        let mut body = vec![0; length];
        self.input.read_exact(&mut body)?;
        Ok(Some(serde_json::from_slice(&body)?))
    }
}

/// Writes one JSON-RPC value using standard LSP framing.
///
/// * `output` — destination byte stream.
/// * `message` — JSON-RPC request, response, or notification.
///
/// # Errors
///
/// Returns JSON serialization or stream write errors.
pub fn write_message(
    output: &mut impl Write,
    message: &Value,
) -> Result<(), Box<dyn std::error::Error>> {
    let body = serde_json::to_vec(message)?;
    write!(output, "Content-Length: {}\r\n\r\n", body.len())?;
    output.write_all(&body)?;
    output.flush()?;
    Ok(())
}
