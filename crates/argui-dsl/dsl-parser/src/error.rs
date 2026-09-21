use argui_dsl_syntax::{SyntaxKind, TextRange};

/// Recoverable lexer or parser diagnostic attached to an exact source range.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseDiagnostic {
    pub message: String,
    pub range: TextRange,
    pub expected: Vec<SyntaxKind>,
}

impl ParseDiagnostic {
    /// Creates a parser diagnostic.
    ///
    /// * `message` — actionable human-readable explanation.
    /// * `range` — offending or insertion source range.
    /// * `expected` — syntax kinds that would be valid at the range.
    #[must_use]
    pub fn new(
        message: impl Into<String>,
        range: TextRange,
        expected: impl IntoIterator<Item = SyntaxKind>,
    ) -> Self {
        Self {
            message: message.into(),
            range,
            expected: expected.into_iter().collect(),
        }
    }
}
