use argui_dsl_syntax::Span;

/// Stable machine-readable semantic diagnostic category.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DiagnosticCode {
    Parse,
    DuplicateDefinition,
    DuplicateMember,
    UnresolvedImport,
    PrivateImport,
    ImportCycle,
    UnknownType,
    UnknownName,
    TypeMismatch,
    UnitMismatch,
    UnknownComponent,
    UnknownProperty,
    MissingProperty,
    UnknownEvent,
    InvalidTwoWayBinding,
    BindingCycle,
    ThemeCycle,
    MissingRepeaterKey,
    InvalidEffect,
    InvalidAsset,
}

/// Diagnostic importance used consistently by CLI, LSP, and live development.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Severity {
    Warning,
    Error,
}

/// Source-located semantic or parser diagnostic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub severity: Severity,
    pub message: String,
    pub primary: Span,
    pub related: Vec<(Span, String)>,
}

impl Diagnostic {
    /// Creates an error diagnostic at `primary`.
    ///
    /// * `code` — stable diagnostic category.
    /// * `message` — actionable explanation.
    /// * `primary` — primary source span.
    #[must_use]
    pub fn error(code: DiagnosticCode, message: impl Into<String>, primary: Span) -> Self {
        Self {
            code,
            severity: Severity::Error,
            message: message.into(),
            primary,
            related: Vec::new(),
        }
    }

    /// Creates a warning diagnostic at `primary`.
    ///
    /// * `code` — stable diagnostic category.
    /// * `message` — actionable explanation.
    /// * `primary` — primary source span.
    #[must_use]
    pub fn warning(code: DiagnosticCode, message: impl Into<String>, primary: Span) -> Self {
        Self {
            code,
            severity: Severity::Warning,
            message: message.into(),
            primary,
            related: Vec::new(),
        }
    }

    /// Attaches a related source location such as another cycle member.
    ///
    /// * `span` — related source span.
    /// * `message` — relationship explanation.
    #[must_use]
    pub fn related(mut self, span: Span, message: impl Into<String>) -> Self {
        self.related.push((span, message.into()));
        self
    }
}
