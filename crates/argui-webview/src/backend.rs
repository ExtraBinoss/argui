use std::fmt;

use argui_core::Rect;

use crate::{WebViewEventSink, WebViewSource, WebViewState};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WebViewError {
    InvalidUrl,
    PolicyMismatch,
    CapacityExceeded,
    DuplicateMount,
    InvalidBounds,
    InvalidOptions(String),
    UnsupportedOptions(String),
    UnsupportedPlatform,
    MissingHost,
    Native(String),
}

impl fmt::Display for WebViewError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "WebView error: {self:?}")
    }
}

impl std::error::Error for WebViewError {}

/// Native handles stay on the UI thread. Implementations must not redraw Argui.
pub trait NativeWebView {
    fn load(&mut self, source: &WebViewSource) -> Result<(), WebViewError>;
    fn set_bounds(&mut self, bounds: Rect) -> Result<(), WebViewError>;
    fn set_visible(&mut self, visible: bool) -> Result<(), WebViewError>;
    fn focus(&mut self) -> Result<(), WebViewError>;
}

/// The same lifecycle contract is used by every OS backend and deterministic tests.
pub trait WebViewBackend {
    type View: NativeWebView;

    /// Create hidden, without loading content. The pool sets bounds before showing it.
    fn create(
        &mut self,
        host: u64,
        state: &WebViewState,
        events: WebViewEventSink,
    ) -> Result<Self::View, WebViewError>;
}
