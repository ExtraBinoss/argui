/// A complete, script-free document for the browser's restricted email frame.
/// The backend additionally applies sandboxing and intercepts all link clicks.
#[must_use]
pub fn email_document(html: &str) -> String {
    let body = ammonia::Builder::default()
        .url_schemes(["https", "http"].into_iter().collect())
        .clean(html)
        .to_string();
    format!(
        r#"<!doctype html><html><head><meta charset="utf-8"><meta http-equiv="Content-Security-Policy" content="default-src 'none'; script-src 'none'; style-src 'unsafe-inline'; img-src data:; base-uri 'none'; form-action 'none'"><meta name="referrer" content="no-referrer"><style>html{{color-scheme:light dark}}body{{font:16px/1.6 system-ui,sans-serif;margin:24px;overflow-wrap:anywhere}}img{{max-width:100%}}a{{color:#3987ed}}blockquote{{border-left:3px solid #888;padding-left:16px;margin-left:0}}</style></head><body>{body}</body></html>"#
    )
}
