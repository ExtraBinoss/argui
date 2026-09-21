use std::fmt;

/// Error produced while evaluating a reactive binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReactiveError {
    /// A computed value recursively depended on itself through `path`.
    Cycle { path: Vec<String> },
    /// The binding callback rejected evaluation with an application-defined message.
    Evaluation(String),
}

impl ReactiveError {
    /// Creates an application-defined evaluation error.
    ///
    /// * `message` — explanation suitable for a compiler or development diagnostic.
    #[must_use]
    pub fn evaluation(message: impl Into<String>) -> Self {
        Self::Evaluation(message.into())
    }
}

impl fmt::Display for ReactiveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cycle { path } => write!(formatter, "reactive cycle: {}", path.join(" -> ")),
            Self::Evaluation(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for ReactiveError {}
