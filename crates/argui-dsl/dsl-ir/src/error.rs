use argui_dsl_syntax::Span;

/// Typed IR lowering failure. A valid semantic project should not produce one.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LowerError {
    pub message: String,
    pub span: Span,
}

impl LowerError {
    /// Creates a source-located lowering failure.
    ///
    /// * `message` — violated lowering invariant.
    /// * `span` — source that could not be normalized.
    #[must_use]
    pub fn new(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span,
        }
    }
}
