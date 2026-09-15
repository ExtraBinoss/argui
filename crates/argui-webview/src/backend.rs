use std::fmt;

use argui_core::Rect;

use crate::{WebViewEventSink, WebViewSource, WebViewState};

#[derive(Clone, Debug, Eq, PartialEq)]
/// Error returned by WebView policy, pool, or native backend operations.
pub enum WebViewError {
    /// Source URL is invalid or unsupported.
    InvalidUrl,
    /// Source policy does not match the retained session.
    PolicyMismatch,
    /// The pool cannot host all active sessions.
    CapacityExceeded,
    /// The same session appears more than once in the active mounts.
    DuplicateMount,
    /// A mount has non-finite or otherwise invalid bounds.
    InvalidBounds,
    /// Requested options violate the source policy.
    InvalidOptions(String),
    /// The selected backend cannot enforce an option.
    UnsupportedOptions(String),
    /// This target has no supported WebView implementation.
    UnsupportedPlatform,
    /// The backend has no registered native host for the mount.
    MissingHost,
    /// Native WebView engine operation failed.
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
    /// Loads a source into this native view.
    ///
    /// # Errors
    /// Returns an error if the native engine cannot load the source.
    /// `source` is the content to load into this view.
    /// `source` is validated against the retained session policy before loading.
    fn load(&mut self, source: &WebViewSource) -> Result<(), WebViewError>;
    /// Sets the view's bounds in host coordinates.
    ///
    /// # Errors
    /// Returns an error if the native engine rejects the bounds.
    /// `bounds` are logical coordinates within the host.
    fn set_bounds(&mut self, bounds: Rect) -> Result<(), WebViewError>;
    /// Shows or hides this native view.
    ///
    /// # Errors
    /// Returns an error if visibility cannot be changed.
    /// `visible` selects whether the view is shown.
    fn set_visible(&mut self, visible: bool) -> Result<(), WebViewError>;
    /// Requests keyboard focus for this native view.
    ///
    /// # Errors
    /// Returns an error if the native engine cannot focus the view.
    fn focus(&mut self) -> Result<(), WebViewError>;
}

/// The same lifecycle contract is used by every OS backend and deterministic tests.
pub trait WebViewBackend {
    type View: NativeWebView;

    /// Create hidden, without loading content. The pool sets bounds before showing it.
    ///
    /// # Errors
    /// Returns an error if the host is unknown or the backend cannot create a view.
    /// `host` identifies the parent, `state` configures the session, and `events` reports backend events.
    fn create(
        &mut self,
        host: u64,
        state: &WebViewState,
        events: WebViewEventSink,
    ) -> Result<Self::View, WebViewError>;
}
