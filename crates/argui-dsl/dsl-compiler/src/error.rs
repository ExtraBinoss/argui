use std::path::PathBuf;

use argui_dsl_syntax::Span;

/// Failure produced before an AOT artifact can be committed.
#[derive(Debug)]
pub enum CompilerError {
    Schema(argui_schema::SchemaError),
    Semantic(Vec<argui_dsl_semantic::Diagnostic>),
    Lower(Vec<argui_dsl_ir::LowerError>),
    MissingEntry(String),
    Asset {
        path: PathBuf,
        message: String,
        source_span: Option<Span>,
    },
    Codegen(String),
}

impl std::fmt::Display for CompilerError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Schema(error) => write!(formatter, "native schema: {error}"),
            Self::Semantic(diagnostics) => {
                write!(formatter, "{} DSL diagnostic(s)", diagnostics.len())
            }
            Self::Lower(errors) => write!(formatter, "{} IR lowering error(s)", errors.len()),
            Self::MissingEntry(path) => write!(formatter, "entry module `{path}` was not found"),
            Self::Asset { path, message, .. } => {
                write!(formatter, "asset `{}`: {message}", path.display())
            }
            Self::Codegen(message) => write!(formatter, "Rust AOT generation: {message}"),
        }
    }
}

impl std::error::Error for CompilerError {}

impl From<argui_schema::SchemaError> for CompilerError {
    fn from(error: argui_schema::SchemaError) -> Self {
        Self::Schema(error)
    }
}
