//! Deterministic, renderer-independent testing for complete Argui applications.
//!
//! [`TestApp`] renders through the retained model runtime, computes real layout,
//! hit-tests pointer input, routes ordinary UI events, and settles resulting
//! application work without creating a native window or GPU surface.
//!
//! The harness uses embedded fonts and an in-memory clipboard. Native tests can
//! therefore exercise complete controlled views on CI hosts without a display
//! server; renderer and operating-system integration remain separate concerns.
//!
//! ```ignore
//! let mut app = argui_testing::TestApp::new(Counter::default());
//! app.click("increment")?;
//! app.assert_text("Count: 1");
//! app.assert_quiescent();
//! # Ok::<(), argui_testing::TestError>(())
//! ```

mod actions;
mod app;
mod error;
mod inspect;
mod node;
mod query;
mod windows;

pub use app::TestApp;
pub use error::{SelectorCount, TestError};
pub use node::TestNode;
pub use query::{Selector, SemanticMatcher};
pub use windows::TestWindows;
