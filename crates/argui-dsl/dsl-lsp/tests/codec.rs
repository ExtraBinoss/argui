use std::io::Cursor;

use argui_dsl_lsp::{MessageReader, write_message};
use serde_json::json;

#[test]
fn content_length_codec_round_trips_unicode_json_rpc() {
    let message = json!({"jsonrpc": "2.0", "id": 7, "result": "héllo"});
    let mut bytes = Vec::new();
    write_message(&mut bytes, &message).unwrap();
    let mut reader = MessageReader::new(Cursor::new(bytes));
    assert_eq!(reader.read_message().unwrap(), Some(message));
    assert_eq!(reader.read_message().unwrap(), None);
}

#[test]
fn content_length_reader_rejects_incomplete_headers_bodies_and_json() {
    let cases = [
        b"Content-Length: 2\r\n\r\n".as_slice(),
        b"X-Test: yes\r\n\r\n".as_slice(),
        b"Content-Length: nope\r\n\r\n{}".as_slice(),
        b"Content-Length: 2\r\n\r\n{".as_slice(),
        b"Content-Length: 1\r\n\r\n{".as_slice(),
    ];
    for bytes in cases {
        let result = MessageReader::new(Cursor::new(bytes)).read_message();
        assert!(
            result.is_err(),
            "expected malformed frame to fail: {bytes:?}"
        );
    }
    let error = MessageReader::new(Cursor::new(b"Content-Length: 2\r\n")).read_message();
    assert!(error.unwrap_err().to_string().contains("unexpected EOF"));
}

#[test]
fn content_length_reader_accepts_lowercase_header_and_reports_body_errors() {
    let body = br#"{"ok":1}"#;
    let frame = format!(
        "content-length: {}\r\nX-Trace: test\r\n\r\n{}",
        body.len(),
        std::str::from_utf8(body).unwrap()
    );
    let mut reader = MessageReader::new(Cursor::new(frame.into_bytes()));
    assert_eq!(reader.read_message().unwrap(), Some(json!({"ok": 1})));
    assert_eq!(reader.read_message().unwrap(), None);

    let error = MessageReader::new(Cursor::new(b"Content-Length: 1\r\n\r\n{"))
        .read_message()
        .unwrap_err();
    assert!(error.to_string().contains("EOF") || error.to_string().contains("JSON"));
}
