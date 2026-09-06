//! Retained web content and bounded native WebView residency.

mod backend;
#[cfg(target_arch = "wasm32")]
mod browser;
mod mounts;
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
mod native;
mod options;
mod pool;
mod source;
pub use options::{PopupPolicy, WebCompatibility, WebViewOptions};
mod state;
mod web_document;
mod widget;
#[cfg(target_arch = "wasm32")]
pub use browser::BrowserBackend;
pub use mounts::{resolve_clipped_mounts, resolve_mounts};
pub use web_document::email_document;

pub use backend::{NativeWebView, WebViewBackend, WebViewError};
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
pub use native::{WryBackend, WryView};
pub use pool::{PoolConfig, PoolStats, WebViewMount, WebViewPool};
pub use source::{WebViewPolicy, WebViewSource};
pub use state::{WebViewEvent, WebViewEventSink, WebViewId, WebViewState};
pub use widget::WebView;
