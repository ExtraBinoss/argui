use std::io::{Read, Write};

use serde::{Serialize, de::DeserializeOwned};

/// Maximum accepted frame size, preventing unbounded allocation from a peer.
pub const MAX_FRAME_BYTES: usize = 64 * 1024 * 1024;

/// Serializes one protocol value into canonical compact JSON bytes.
///
/// # Errors
///
/// Returns an error when a value cannot be represented by the wire schema.
pub fn encode(value: &impl Serialize) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(value)
}

/// Decodes one complete protocol value without best-effort version coercion.
///
/// # Errors
///
/// Returns an error for malformed, incomplete, or schema-incompatible input.
pub fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, serde_json::Error> {
    serde_json::from_slice(bytes)
}

/// Writes one big-endian length-prefixed protocol frame.
///
/// # Errors
///
/// Returns invalid-data for oversized values or propagates serialization/I/O errors.
pub fn write_frame(writer: &mut impl Write, value: &impl Serialize) -> Result<(), std::io::Error> {
    let bytes = encode(value).map_err(std::io::Error::other)?;
    let length = u32::try_from(bytes.len())
        .ok()
        .filter(|length| *length as usize <= MAX_FRAME_BYTES)
        .ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "frame is too large")
        })?;
    writer.write_all(&length.to_be_bytes())?;
    writer.write_all(&bytes)
}

/// Reads one complete big-endian length-prefixed protocol frame.
///
/// # Errors
///
/// Returns invalid-data before allocation for oversized frames and propagates decode/I/O errors.
pub fn read_frame<T: DeserializeOwned>(reader: &mut impl Read) -> Result<T, std::io::Error> {
    let mut length = [0_u8; 4];
    reader.read_exact(&mut length)?;
    let length = u32::from_be_bytes(length) as usize;
    if length > MAX_FRAME_BYTES {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "frame is too large",
        ));
    }
    let mut bytes = vec![0; length];
    reader.read_exact(&mut bytes)?;
    decode(&bytes).map_err(std::io::Error::other)
}
