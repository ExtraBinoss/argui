use argui_dsl_semantic::{SemanticClass, SymbolKind};
use serde_json::{Value, json};

use crate::convert::offset_to_position;

/// Lightweight JSON-RPC error used to preserve standard error codes.
pub(crate) struct RpcError {
    pub(crate) code: i32,
    pub(crate) message: String,
}

impl RpcError {
    /// Creates an invalid-parameters response.
    pub(crate) fn invalid(error: impl std::fmt::Display) -> Self {
        Self {
            code: -32602,
            message: error.to_string(),
        }
    }

    /// Creates an internal-error response.
    pub(crate) fn internal(error: impl std::fmt::Display) -> Self {
        Self {
            code: -32603,
            message: error.to_string(),
        }
    }

    /// Creates a method-not-found response.
    pub(crate) fn method(method: &str) -> Self {
        Self {
            code: -32601,
            message: format!("unsupported LSP method `{method}`"),
        }
    }

    /// Creates an invalid-request response after orderly shutdown.
    pub(crate) fn shutdown() -> Self {
        Self {
            code: -32600,
            message: "language server is shutting down".into(),
        }
    }
}

/// Returns the advertised static LSP capabilities.
pub(crate) fn capabilities() -> Value {
    json!({
        "capabilities": {
            "textDocumentSync": {"openClose": true, "change": 1},
            "completionProvider": {"triggerCharacters": [".", "-"]},
            "hoverProvider": true,
            "definitionProvider": true,
            "referencesProvider": true,
            "renameProvider": {"prepareProvider": false},
            "documentSymbolProvider": true,
            "workspaceSymbolProvider": true,
            "semanticTokensProvider": {
                "legend": {"tokenTypes": ["keyword", "type", "class", "property", "event", "variable", "string", "number", "comment", "enumMember"], "tokenModifiers": []},
                "full": true,
            },
            "documentFormattingProvider": true,
            "colorProvider": true,
            "codeActionProvider": true,
        },
        "serverInfo": {"name": "argui-lsp", "version": env!("CARGO_PKG_VERSION")},
    })
}

/// Reads the standard text-document URI from request parameters.
pub(crate) fn document_uri(parameters: &Value) -> Result<&str, RpcError> {
    required_str(&parameters["textDocument"], "uri")
}

/// Reads one required string field from a JSON object.
pub(crate) fn required_str<'a>(value: &'a Value, field: &str) -> Result<&'a str, RpcError> {
    value[field]
        .as_str()
        .ok_or_else(|| RpcError::invalid(format!("missing string field `{field}`")))
}

/// Maps semantic completion kinds to LSP completion item kinds.
pub(crate) fn completion_kind(kind: SymbolKind) -> u8 {
    match kind {
        SymbolKind::Component | SymbolKind::Native => 7,
        SymbolKind::Property => 10,
        SymbolKind::Callback => 23,
        SymbolKind::Slot | SymbolKind::Keyword => 14,
        SymbolKind::Struct
        | SymbolKind::Enum
        | SymbolKind::Theme
        | SymbolKind::Style
        | SymbolKind::Effect => 7,
        SymbolKind::Function => 3,
        SymbolKind::Token => 21,
    }
}

/// Maps semantic symbols to LSP symbol kinds.
pub(crate) fn symbol_kind(kind: SymbolKind) -> u8 {
    match kind {
        SymbolKind::Component | SymbolKind::Native => 5,
        SymbolKind::Property => 7,
        SymbolKind::Callback => 24,
        SymbolKind::Slot => 8,
        SymbolKind::Struct => 23,
        SymbolKind::Enum => 10,
        SymbolKind::Theme | SymbolKind::Style | SymbolKind::Effect => 3,
        SymbolKind::Function => 12,
        SymbolKind::Token => 22,
        SymbolKind::Keyword => 14,
    }
}

/// Splits one byte-range highlight into valid single-line semantic token segments.
pub(crate) fn token_segments(
    source: &str,
    start: u32,
    end: u32,
    class: SemanticClass,
) -> Vec<(u32, u32, u32, u32)> {
    let start = usize::try_from(start)
        .unwrap_or(source.len())
        .min(source.len());
    let end = usize::try_from(end)
        .unwrap_or(source.len())
        .min(source.len());
    let mut output = Vec::new();
    let mut cursor = start;
    while cursor < end {
        let line_end = source[cursor..end]
            .find('\n')
            .map_or(end, |offset| cursor + offset);
        if line_end > cursor {
            let position = offset_to_position(source, u32::try_from(cursor).unwrap_or(u32::MAX));
            output.push((
                position["line"].as_u64().unwrap_or_default() as u32,
                position["character"].as_u64().unwrap_or_default() as u32,
                u32::try_from(source[cursor..line_end].encode_utf16().count()).unwrap_or(u32::MAX),
                semantic_kind(class),
            ));
        }
        cursor = line_end.saturating_add(1);
    }
    output
}

/// Maps semantic highlight classes to the advertised legend indices.
const fn semantic_kind(class: SemanticClass) -> u32 {
    match class {
        SemanticClass::Keyword => 0,
        SemanticClass::Type => 1,
        SemanticClass::Component => 2,
        SemanticClass::Property => 3,
        SemanticClass::Callback => 4,
        SemanticClass::Variable => 5,
        SemanticClass::String => 6,
        SemanticClass::Number => 7,
        SemanticClass::Comment => 8,
        SemanticClass::ThemeToken => 9,
    }
}
