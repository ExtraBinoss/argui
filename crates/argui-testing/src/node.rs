use argui_accessibility::{SemanticAction, SemanticValue};
use argui_runtime::Render;

use crate::{Selector, TestApp, TestError};

/// Deferred operation handle for one uniquely selected application node.
pub struct TestNode<'a, A: Render> {
    app: &'a mut TestApp<A>,
    selector: Selector,
}

impl<'a, A: Render> TestNode<'a, A> {
    pub(crate) const fn new(app: &'a mut TestApp<A>, selector: Selector) -> Self {
        Self { app, selector }
    }

    /// Clicks this node using real layout, pointer input, and hit testing.
    ///
    /// # Errors
    /// Returns a selector, bounds, layout, or stabilization error.
    pub fn click(self) -> Result<(), TestError> {
        self.app.click_selector(&self.selector)
    }

    /// Focuses this node through the retained focus system.
    ///
    /// # Errors
    /// Returns a selector or stabilization error.
    pub fn focus(self) -> Result<(), TestError> {
        self.app.focus(self.selector)
    }

    /// Types `text` into this text-input node.
    ///
    /// # Errors
    /// Returns a selector, input-kind, layout, or stabilization error.
    pub fn type_text(self, text: &str) -> Result<(), TestError> {
        self.app.type_text(self.selector, text)
    }

    /// Replaces this text input's complete value with `value`.
    ///
    /// # Errors
    /// Returns a selector, input-kind, layout, or stabilization error.
    pub fn replace_text(self, value: &str) -> Result<(), TestError> {
        self.app.replace_text(self.selector, value)
    }

    /// Pastes `text` into this text-input node.
    ///
    /// # Errors
    /// Returns a selector, input-kind, layout, or stabilization error.
    pub fn paste(self, text: &str) -> Result<(), TestError> {
        self.app.paste(self.selector, text)
    }

    /// Submits this text input with Enter.
    ///
    /// # Errors
    /// Returns a selector, input-kind, layout, or stabilization error.
    pub fn submit(self) -> Result<(), TestError> {
        self.app.submit(self.selector)
    }

    /// Invokes `action` through the accessibility event path.
    ///
    /// # Errors
    /// Returns a selector, unsupported-action, layout, or stabilization error.
    pub fn accessibility_action(
        self,
        action: SemanticAction,
        value: Option<SemanticValue>,
    ) -> Result<(), TestError> {
        self.app.accessibility_action(self.selector, action, value)
    }
}
