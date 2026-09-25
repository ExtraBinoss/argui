use std::fmt;

/// One-based source position in the developer-authored WGSL file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShaderSourcePosition {
    pub source_name: String,
    pub line: u32,
    pub column: u32,
}

/// Source-mapped shader diagnostic suitable for CLI, build, LSP, and live reload output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShaderDiagnostic {
    pub position: ShaderSourcePosition,
    pub message: String,
}

impl fmt::Display for ShaderDiagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}:{}:{}: {}",
            self.position.source_name, self.position.line, self.position.column, self.message
        )
    }
}

/// Shader metadata or WGSL validation failure.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ShaderError {
    #[error("invalid effect parameter metadata: {0}")]
    InvalidParameter(String),
    #[error("{0}")]
    Diagnostic(ShaderDiagnostic),
}
