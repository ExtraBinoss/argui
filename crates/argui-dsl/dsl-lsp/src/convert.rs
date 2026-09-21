use std::path::{Component, Path, PathBuf};

use serde_json::{Value, json};

/// Converts an LSP UTF-16 position to a UTF-8 byte offset.
pub(crate) fn position_to_offset(source: &str, position: &Value) -> Result<u32, String> {
    let target_line = position["line"]
        .as_u64()
        .ok_or("position.line is missing")? as usize;
    let target_character = position["character"]
        .as_u64()
        .ok_or("position.character is missing")? as usize;
    let mut line = 0;
    let mut line_start = 0;
    for (index, byte) in source.bytes().enumerate() {
        if line == target_line {
            break;
        }
        if byte == b'\n' {
            line += 1;
            line_start = index + 1;
        }
    }
    if line != target_line {
        return Err("position line is outside the document".into());
    }
    let line_text = source[line_start..]
        .split_once('\n')
        .map_or(&source[line_start..], |(line, _)| line);
    let mut utf16 = 0;
    for (offset, character) in line_text.char_indices() {
        if utf16 == target_character {
            return u32::try_from(line_start + offset).map_err(|error| error.to_string());
        }
        utf16 += character.len_utf16();
        if utf16 > target_character {
            return Err("position splits a UTF-16 surrogate pair".into());
        }
    }
    if utf16 == target_character {
        return u32::try_from(line_start + line_text.len()).map_err(|error| error.to_string());
    }
    Err("position character is outside the line".into())
}

/// Converts a UTF-8 byte offset to an LSP UTF-16 position.
pub(crate) fn offset_to_position(source: &str, offset: u32) -> Value {
    let offset = usize::try_from(offset)
        .unwrap_or(source.len())
        .min(source.len());
    let mut safe = offset;
    while !source.is_char_boundary(safe) {
        safe = safe.saturating_sub(1);
    }
    let prefix = &source[..safe];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count();
    let line_start = prefix.rfind('\n').map_or(0, |index| index + 1);
    let character = prefix[line_start..].encode_utf16().count();
    json!({"line": line, "character": character})
}

/// Converts a byte range to an LSP range.
pub(crate) fn byte_range(source: &str, start: u32, end: u32) -> Value {
    json!({
        "start": offset_to_position(source, start),
        "end": offset_to_position(source, end),
    })
}

/// Decodes a local `file:` URI into a platform path.
pub(crate) fn uri_to_path(uri: &str) -> Result<PathBuf, String> {
    let encoded = uri
        .strip_prefix("file://")
        .ok_or_else(|| "only file:// document URIs are supported".to_owned())?;
    let decoded = percent_decode(encoded)?;
    #[cfg(target_os = "windows")]
    let decoded = decoded.strip_prefix('/').unwrap_or(&decoded);
    Ok(PathBuf::from(decoded))
}

/// Encodes an absolute platform path as a local `file:` URI.
pub(crate) fn path_to_uri(path: &Path) -> String {
    let text = path.to_string_lossy().replace('\\', "/");
    let prefix = if text.starts_with('/') {
        "file://"
    } else {
        "file:///"
    };
    format!("{prefix}{}", percent_encode(&text))
}

/// Converts a path below a workspace root to a slash-separated module key.
pub(crate) fn module_path(root: &Path, path: &Path) -> Result<String, String> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| "document is outside the workspace root")?;
    let mut parts = Vec::new();
    for component in relative.components() {
        match component {
            Component::Normal(value) => parts.push(
                value
                    .to_str()
                    .ok_or("document path is not valid UTF-8")?
                    .to_owned(),
            ),
            Component::CurDir => {}
            _ => return Err("document path is not project-relative".into()),
        }
    }
    Ok(parts.join("/"))
}

/// Percent-decodes a URI path without accepting malformed octets.
fn percent_decode(value: &str) -> Result<String, String> {
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let hex = bytes
                .get(index + 1..index + 3)
                .ok_or("truncated URI escape")?;
            let text = std::str::from_utf8(hex).map_err(|error| error.to_string())?;
            output.push(u8::from_str_radix(text, 16).map_err(|error| error.to_string())?);
            index += 3;
        } else {
            output.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(output).map_err(|error| error.to_string())
}

/// Percent-encodes URI path bytes outside the unreserved/path set.
fn percent_encode(value: &str) -> String {
    let mut output = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~' | b'/' | b':') {
            output.push(char::from(byte));
        } else {
            output.push_str(&format!("%{byte:02X}"));
        }
    }
    output
}
