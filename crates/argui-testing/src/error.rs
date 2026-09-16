use crate::Selector;

/// Observed cardinality for a selector that must resolve uniquely.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectorCount {
    None,
    Multiple(usize),
}

/// Structured failure returned by a headless application operation.
#[derive(Clone, Debug, thiserror::Error)]
pub enum TestError {
    #[error("the requested application presentation is closed")]
    ClosedPresentation,
    #[error("task runtime failed: {message}")]
    Task { message: String },
    #[error("application window {window:?} is already open")]
    DuplicateWindow { window: argui_platform::WindowKey },
    #[error("application window {window:?} is not open")]
    MissingWindow { window: argui_platform::WindowKey },
    #[error("layout failed: {message}")]
    Layout { message: String },
    #[error("selector {selector} matched {count:?}; focused={focused:?}\n{candidates}\n{tree}")]
    Selector {
        selector: Selector,
        count: SelectorCount,
        focused: Option<String>,
        candidates: String,
        tree: String,
    },
    #[error("application did not settle within {limit} iterations\n{tree}")]
    DidNotSettle { limit: usize, tree: String },
    #[error("node selected by {selector} has no usable layout bounds\n{tree}")]
    MissingBounds { selector: Selector, tree: String },
    #[error("operation requires a text input, but {selector} selected {role:?}\n{tree}")]
    NotTextInput {
        selector: Selector,
        role: argui_accessibility::Role,
        tree: String,
    },
    #[error("semantic action {action:?} is unavailable for {selector}\n{tree}")]
    UnsupportedAction {
        selector: Selector,
        action: argui_accessibility::SemanticAction,
        tree: String,
    },
}
