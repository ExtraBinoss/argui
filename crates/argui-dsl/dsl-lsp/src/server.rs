use std::{io::BufReader, path::PathBuf};

use argui_dsl_semantic::{Location, Severity};
use serde_json::{Value, json};

use crate::{
    codec::{MessageReader, write_message},
    convert::{byte_range, path_to_uri, position_to_offset, uri_to_path},
    protocol::{
        RpcError, capabilities, completion_kind, document_uri, required_str, symbol_kind,
        token_segments,
    },
    workspace::Workspace,
};

/// Fatal language-server startup or transport failure.
#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("language server I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("language server failed: {0}")]
    Other(String),
}

impl From<Box<dyn std::error::Error>> for ServerError {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        Self::Other(error.to_string())
    }
}

/// In-process Argui LSP state machine suitable for stdio and tests.
pub struct LanguageServer {
    workspace: Workspace,
    shutdown: bool,
    exited: bool,
}

impl LanguageServer {
    /// Creates a language server and eagerly indexes `root`.
    ///
    /// # Errors
    ///
    /// Returns schema construction or workspace traversal errors.
    pub fn new(root: PathBuf) -> Result<Self, ServerError> {
        Ok(Self {
            workspace: Workspace::load(root)?,
            shutdown: false,
            exited: false,
        })
    }

    /// Creates a language server with the application's exact native registry.
    ///
    /// * `root` — project source root.
    /// * `registry` — built-ins plus application primitive contracts.
    ///
    /// # Errors
    ///
    /// Returns workspace traversal errors while indexing sources.
    pub fn with_registry(
        root: PathBuf,
        registry: argui_schema::SchemaRegistry,
    ) -> Result<Self, ServerError> {
        Ok(Self {
            workspace: Workspace::load_with_registry(root, registry)?,
            shutdown: false,
            exited: false,
        })
    }

    /// Processes one decoded JSON-RPC message and returns outbound messages.
    ///
    /// Requests produce one response; document notifications may produce diagnostics.
    pub fn handle_message(&mut self, message: &Value) -> Vec<Value> {
        let Some(method) = message["method"].as_str() else {
            return Vec::new();
        };
        let id = message.get("id").cloned();
        let parameters = message.get("params").cloned().unwrap_or(Value::Null);
        match self.dispatch(method, &parameters) {
            Ok((result, notifications)) => {
                let mut output = Vec::new();
                if let Some(id) = id {
                    output.push(json!({"jsonrpc": "2.0", "id": id, "result": result}));
                }
                output.extend(notifications);
                output
            }
            Err(error) => id.map_or_else(
                || {
                    vec![json!({
                        "jsonrpc": "2.0",
                        "method": "window/logMessage",
                        "params": {"type": 1, "message": error.message},
                    })]
                },
                |id| {
                    vec![json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": {"code": error.code, "message": error.message},
                    })]
                },
            ),
        }
    }

    /// Returns whether the client sent an `exit` notification.
    #[must_use]
    pub const fn exited(&self) -> bool {
        self.exited
    }

    /// Dispatches one LSP method to semantic, formatting, or lifecycle logic.
    fn dispatch(
        &mut self,
        method: &str,
        parameters: &Value,
    ) -> Result<(Value, Vec<Value>), RpcError> {
        if self.shutdown && method != "exit" {
            return Err(RpcError::shutdown());
        }
        match method {
            "initialize" => self.initialize(parameters),
            "initialized" => Ok((Value::Null, Vec::new())),
            "shutdown" => {
                self.shutdown = true;
                Ok((Value::Null, Vec::new()))
            }
            "exit" => {
                self.exited = true;
                Ok((Value::Null, Vec::new()))
            }
            "textDocument/didOpen" => self.did_open(parameters),
            "textDocument/didChange" => self.did_change(parameters),
            "textDocument/didClose" => self.did_close(parameters),
            "textDocument/completion" => self.completion(parameters),
            "textDocument/hover" => self.hover(parameters),
            "textDocument/definition" => self.definition(parameters),
            "textDocument/references" => self.references(parameters),
            "textDocument/rename" => self.rename(parameters),
            "textDocument/documentSymbol" => self.document_symbols(parameters),
            "workspace/symbol" => self.workspace_symbols(),
            "textDocument/semanticTokens/full" => self.semantic_tokens(parameters),
            "textDocument/formatting" => self.formatting(parameters),
            "textDocument/documentColor" => self.document_colors(parameters),
            "textDocument/colorPresentation" => self.color_presentation(parameters),
            "textDocument/codeAction" => self.code_actions(parameters),
            _ => Err(RpcError::method(method)),
        }
    }

    /// Configures the workspace and advertises the complete supported LSP surface.
    fn initialize(&mut self, parameters: &Value) -> Result<(Value, Vec<Value>), RpcError> {
        let uri = parameters["rootUri"].as_str();
        let path = parameters["rootPath"].as_str();
        if let Some(root) = uri
            .map(uri_to_path)
            .transpose()
            .map_err(RpcError::invalid)?
            .or_else(|| path.map(PathBuf::from))
        {
            self.workspace.set_root(root).map_err(RpcError::internal)?;
        }
        Ok((capabilities(), Vec::new()))
    }

    /// Applies an opened document and publishes its checked diagnostics.
    fn did_open(&mut self, parameters: &Value) -> Result<(Value, Vec<Value>), RpcError> {
        let document = &parameters["textDocument"];
        self.update_document(
            required_str(document, "uri")?,
            required_str(document, "text")?.to_owned(),
        )
    }

    /// Applies the latest full-document change and publishes diagnostics.
    fn did_change(&mut self, parameters: &Value) -> Result<(Value, Vec<Value>), RpcError> {
        let uri = required_str(&parameters["textDocument"], "uri")?;
        let text = parameters["contentChanges"]
            .as_array()
            .and_then(|changes| changes.last())
            .and_then(|change| change["text"].as_str())
            .ok_or_else(|| RpcError::invalid("didChange lacks full document text"))?;
        self.update_document(uri, text.to_owned())
    }

    /// Restores a closed document from disk and republishes diagnostics.
    fn did_close(&mut self, parameters: &Value) -> Result<(Value, Vec<Value>), RpcError> {
        let uri = required_str(&parameters["textDocument"], "uri")?;
        let module = self
            .workspace
            .close_document(uri)
            .map_err(RpcError::invalid)?;
        Ok((Value::Null, vec![self.diagnostics(uri, &module)]))
    }

    /// Updates one full text overlay and returns its diagnostic notification.
    fn update_document(
        &mut self,
        uri: &str,
        source: String,
    ) -> Result<(Value, Vec<Value>), RpcError> {
        let module = self
            .workspace
            .set_document(uri, source)
            .map_err(RpcError::invalid)?;
        Ok((Value::Null, vec![self.diagnostics(uri, &module)]))
    }

    /// Produces context-sensitive completion items from the semantic database.
    fn completion(&mut self, parameters: &Value) -> Result<(Value, Vec<Value>), RpcError> {
        let (module, offset) = self.document_position(parameters)?;
        let items = self
            .workspace
            .database()
            .completions(&module, offset)
            .map_err(RpcError::internal)?
            .into_iter()
            .map(|item| {
                json!({
                    "label": item.label,
                    "kind": completion_kind(item.kind),
                    "detail": item.detail,
                    "documentation": {"kind": "markdown", "value": item.documentation},
                })
            })
            .collect::<Vec<_>>();
        Ok((json!({"isIncomplete": false, "items": items}), Vec::new()))
    }

    /// Returns Markdown hover contents for the symbol under the cursor.
    fn hover(&mut self, parameters: &Value) -> Result<(Value, Vec<Value>), RpcError> {
        let (module, offset) = self.document_position(parameters)?;
        let value = self
            .workspace
            .database()
            .hover(&module, offset)
            .map_err(RpcError::internal)?
            .map_or(
                Value::Null,
                |value| json!({"contents": {"kind": "markdown", "value": value}}),
            );
        Ok((value, Vec::new()))
    }

    /// Resolves go-to-definition for one source position.
    fn definition(&mut self, parameters: &Value) -> Result<(Value, Vec<Value>), RpcError> {
        let (module, offset) = self.document_position(parameters)?;
        let location = self
            .workspace
            .database()
            .definition(&module, offset)
            .map_err(RpcError::internal)?;
        Ok((
            location.map_or(Value::Null, |value| self.location(value)),
            Vec::new(),
        ))
    }

    /// Resolves every semantic reference for one source position.
    fn references(&mut self, parameters: &Value) -> Result<(Value, Vec<Value>), RpcError> {
        let (module, offset) = self.document_position(parameters)?;
        let values = self
            .workspace
            .database()
            .references(&module, offset)
            .map_err(RpcError::internal)?
            .into_iter()
            .map(|location| self.location(location))
            .collect::<Vec<_>>();
        Ok((Value::Array(values), Vec::new()))
    }

    /// Produces a validated workspace edit for semantic rename.
    fn rename(&mut self, parameters: &Value) -> Result<(Value, Vec<Value>), RpcError> {
        let (module, offset) = self.document_position(parameters)?;
        let replacement = required_str(parameters, "newName")?;
        let edits = self
            .workspace
            .database()
            .rename(&module, offset, replacement)
            .map_err(RpcError::invalid)?;
        let mut changes = serde_json::Map::new();
        for edit in edits {
            let uri = self.uri_for_module(&edit.location.path);
            changes
                .entry(uri)
                .or_insert_with(|| Value::Array(Vec::new()));
            if let Some(values) = changes
                .get_mut(&self.uri_for_module(&edit.location.path))
                .and_then(Value::as_array_mut)
            {
                let source = self
                    .workspace
                    .source(&edit.location.path)
                    .unwrap_or_default();
                values.push(json!({
                    "range": byte_range(source, edit.location.start, edit.location.end),
                    "newText": edit.replacement,
                }));
            }
        }
        Ok((json!({"changes": changes}), Vec::new()))
    }

    /// Returns source symbols for one document.
    fn document_symbols(&mut self, parameters: &Value) -> Result<(Value, Vec<Value>), RpcError> {
        let uri = document_uri(parameters)?;
        let module = self
            .workspace
            .module_for_uri(uri)
            .map_err(RpcError::invalid)?;
        self.symbols(Some(&module))
    }

    /// Returns source symbols for the entire user workspace.
    fn workspace_symbols(&mut self) -> Result<(Value, Vec<Value>), RpcError> {
        self.symbols(None)
    }

    /// Converts semantic symbols to LSP symbol information.
    fn symbols(&mut self, selected: Option<&str>) -> Result<(Value, Vec<Value>), RpcError> {
        let symbols = self
            .workspace
            .database()
            .symbols(selected)
            .map_err(RpcError::internal)?
            .into_iter()
            .map(|symbol| {
                let location = self.location(symbol.location);
                json!({"name": symbol.name, "kind": symbol_kind(symbol.kind), "location": location})
            })
            .collect::<Vec<_>>();
        Ok((Value::Array(symbols), Vec::new()))
    }

    /// Produces deterministic full semantic tokens with UTF-16 delta encoding.
    fn semantic_tokens(&mut self, parameters: &Value) -> Result<(Value, Vec<Value>), RpcError> {
        let uri = document_uri(parameters)?;
        let module = self
            .workspace
            .module_for_uri(uri)
            .map_err(RpcError::invalid)?;
        let source = self
            .workspace
            .source(&module)
            .unwrap_or_default()
            .to_owned();
        let highlights = self
            .workspace
            .database()
            .semantic_highlights(&module)
            .map_err(RpcError::internal)?;
        let mut absolute = Vec::new();
        for highlight in highlights {
            absolute.extend(token_segments(
                &source,
                highlight.start,
                highlight.end,
                highlight.class,
            ));
        }
        absolute.sort_unstable();
        let mut data = Vec::with_capacity(absolute.len() * 5);
        let (mut previous_line, mut previous_start) = (0_u32, 0_u32);
        for (line, start, length, kind) in absolute {
            let delta_line = line - previous_line;
            let delta_start = if delta_line == 0 {
                start - previous_start
            } else {
                start
            };
            data.extend([delta_line, delta_start, length, kind, 0]);
            previous_line = line;
            previous_start = start;
        }
        Ok((json!({"data": data}), Vec::new()))
    }

    /// Formats a complete document from its lossless CST.
    fn formatting(&mut self, parameters: &Value) -> Result<(Value, Vec<Value>), RpcError> {
        let uri = document_uri(parameters)?;
        let module = self
            .workspace
            .module_for_uri(uri)
            .map_err(RpcError::invalid)?;
        let source = self.workspace.source(&module).unwrap_or_default();
        let formatted = argui_dsl_parser::format_source(source);
        let edits = if formatted == source {
            Vec::new()
        } else {
            vec![json!({
                "range": byte_range(source, 0, u32::try_from(source.len()).unwrap_or(u32::MAX)),
                "newText": formatted,
            })]
        };
        Ok((Value::Array(edits), Vec::new()))
    }

    /// Finds typed hexadecimal color literals in one document.
    fn document_colors(&mut self, parameters: &Value) -> Result<(Value, Vec<Value>), RpcError> {
        let uri = document_uri(parameters)?;
        let module = self
            .workspace
            .module_for_uri(uri)
            .map_err(RpcError::invalid)?;
        let source = self
            .workspace
            .source(&module)
            .unwrap_or_default()
            .to_owned();
        let starts = source
            .match_indices('#')
            .map(|(offset, _)| offset)
            .collect::<Vec<_>>();
        let mut colors = Vec::new();
        for start in starts {
            if let Some(color) = self
                .workspace
                .database()
                .color_presentation(&module, u32::try_from(start + 1).unwrap_or(u32::MAX))
                .map_err(RpcError::internal)?
            {
                colors.push(json!({
                    "range": byte_range(&source, color.location.start, color.location.end),
                    "color": {"red": color.red, "green": color.green, "blue": color.blue, "alpha": color.alpha},
                }));
            }
        }
        Ok((Value::Array(colors), Vec::new()))
    }

    /// Returns normalized replacement text for a selected color literal.
    fn color_presentation(&mut self, parameters: &Value) -> Result<(Value, Vec<Value>), RpcError> {
        let uri = document_uri(parameters)?;
        let module = self
            .workspace
            .module_for_uri(uri)
            .map_err(RpcError::invalid)?;
        let source = self
            .workspace
            .source(&module)
            .unwrap_or_default()
            .to_owned();
        let offset = position_to_offset(&source, &parameters["range"]["start"])
            .map_err(RpcError::invalid)?;
        let values = self
            .workspace
            .database()
            .color_presentation(&module, offset)
            .map_err(RpcError::internal)?
            .into_iter()
            .map(|color| json!({
                "label": color.label,
                "textEdit": {"range": byte_range(&source, color.location.start, color.location.end), "newText": color.label},
            }))
            .collect::<Vec<_>>();
        Ok((Value::Array(values), Vec::new()))
    }

    /// Returns semantic quick fixes for common unknown-name diagnostics.
    fn code_actions(&mut self, parameters: &Value) -> Result<(Value, Vec<Value>), RpcError> {
        let uri = document_uri(parameters)?;
        let module = self
            .workspace
            .module_for_uri(uri)
            .map_err(RpcError::invalid)?;
        let source = self
            .workspace
            .source(&module)
            .unwrap_or_default()
            .to_owned();
        let actions = self
            .workspace
            .database()
            .code_actions(&module)
            .map_err(RpcError::internal)?
            .into_iter()
            .map(|action| json!({
                "title": action.title,
                "kind": "quickfix",
                "edit": {"changes": {(uri): [{
                    "range": byte_range(&source, action.edit.location.start, action.edit.location.end),
                    "newText": action.edit.replacement,
                }]}},
            }))
            .collect::<Vec<_>>();
        Ok((Value::Array(actions), Vec::new()))
    }

    /// Resolves a request's document URI and UTF-16 cursor to semantic coordinates.
    fn document_position(&self, parameters: &Value) -> Result<(String, u32), RpcError> {
        let uri = document_uri(parameters)?;
        let module = self
            .workspace
            .module_for_uri(uri)
            .map_err(RpcError::invalid)?;
        let source = self
            .workspace
            .source(&module)
            .ok_or_else(|| RpcError::invalid("document is not loaded"))?;
        let offset =
            position_to_offset(source, &parameters["position"]).map_err(RpcError::invalid)?;
        Ok((module, offset))
    }

    /// Builds a source-aware LSP location from one semantic location.
    fn location(&self, location: Location) -> Value {
        let source = self.workspace.source(&location.path).unwrap_or_default();
        json!({
            "uri": self.uri_for_module(&location.path),
            "range": byte_range(source, location.start, location.end),
        })
    }

    /// Resolves one module key to its absolute file URI.
    fn uri_for_module(&self, module: &str) -> String {
        path_to_uri(&self.workspace.root().join(module))
    }

    /// Produces current diagnostics for one document after a source update.
    fn diagnostics(&mut self, uri: &str, module: &str) -> Value {
        let file = self.workspace.database().file_id(module);
        let project = self.workspace.database().check();
        let source = self.workspace.source(module).unwrap_or_default();
        let diagnostics = project
            .diagnostics
            .iter()
            .filter(|diagnostic| Some(diagnostic.primary.file) == file)
            .map(|diagnostic| {
                json!({
                    "range": byte_range(source, diagnostic.primary.range.start().into(), diagnostic.primary.range.end().into()),
                    "severity": if diagnostic.severity == Severity::Error { 1 } else { 2 },
                    "code": format!("{:?}", diagnostic.code),
                    "source": "argui",
                    "message": diagnostic.message,
                })
            })
            .collect::<Vec<_>>();
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/publishDiagnostics",
            "params": {"uri": uri, "diagnostics": diagnostics},
        })
    }
}

/// Runs the language server until EOF or a client `exit` notification.
///
/// # Errors
///
/// Returns startup, framing, JSON, or output errors.
pub fn run_stdio() -> Result<(), ServerError> {
    let root = std::env::current_dir()?;
    let mut server = LanguageServer::new(root)?;
    let input = std::io::stdin().lock();
    let mut reader = MessageReader::new(BufReader::new(input));
    let mut output = std::io::stdout().lock();
    while let Some(message) = reader.read_message()? {
        for response in server.handle_message(&message) {
            write_message(&mut output, &response)?;
        }
        if server.exited() {
            break;
        }
    }
    Ok(())
}
