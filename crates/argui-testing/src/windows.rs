use argui_platform::WindowKey;
use argui_runtime::{Entity, Render};

use crate::{TestApp, TestError};

/// Headless presentations that share one retained application entity.
///
/// Each window has independent focus, layout, event handlers, and
/// presentation-scoped resources while mutations affect the shared model.
pub struct TestWindows<A: Render> {
    entity: Entity<A>,
    windows: Vec<TestApp<A>>,
}

impl<A: Render> TestWindows<A> {
    /// Creates a multi-window harness with a settled main window.
    ///
    /// # Panics
    /// Panics if the main presentation cannot be initialized. Use
    /// [`Self::try_new`] to handle the failure explicitly.
    #[must_use]
    pub fn new(app: A) -> Self {
        Self::try_new(app).unwrap_or_else(|error| panic!("{error}"))
    }

    /// Creates a multi-window harness and returns initialization failures.
    ///
    /// # Errors
    /// Returns a layout, closed-presentation, or stabilization error.
    pub fn try_new(app: A) -> Result<Self, TestError> {
        let entity = Entity::new(app);
        let main = TestApp::from_entity_in_window(entity.clone(), WindowKey::main())?;
        Ok(Self {
            entity,
            windows: vec![main],
        })
    }

    /// Returns the retained entity shared by all windows.
    #[must_use]
    pub const fn entity(&self) -> &Entity<A> {
        &self.entity
    }

    /// Opens and settles an independent presentation named by `key`.
    ///
    /// # Errors
    /// Returns a duplicate-key, layout, closed-presentation, or stabilization error.
    pub fn open(&mut self, key: WindowKey) -> Result<&mut TestApp<A>, TestError> {
        if self
            .windows
            .iter()
            .any(|window| window.window_key() == &key)
        {
            return Err(TestError::DuplicateWindow { window: key });
        }
        self.windows
            .push(TestApp::from_entity_in_window(self.entity.clone(), key)?);
        Ok(self.windows.last_mut().expect("a window was just inserted"))
    }

    /// Returns the open window named by `key`.
    ///
    /// # Errors
    /// Returns [`TestError::MissingWindow`] when no presentation has that key.
    pub fn window(&mut self, key: &WindowKey) -> Result<&mut TestApp<A>, TestError> {
        self.windows
            .iter_mut()
            .find(|window| window.window_key() == key)
            .ok_or_else(|| TestError::MissingWindow {
                window: key.clone(),
            })
    }

    /// Closes the presentation named by `key` and cancels its scoped resources.
    ///
    /// # Errors
    /// Returns [`TestError::MissingWindow`] when no presentation has that key.
    pub fn close(&mut self, key: &WindowKey) -> Result<(), TestError> {
        let index = self
            .windows
            .iter()
            .position(|window| window.window_key() == key)
            .ok_or_else(|| TestError::MissingWindow {
                window: key.clone(),
            })?;
        self.windows.remove(index);
        Ok(())
    }

    /// Settles every open window after shared state changes.
    ///
    /// # Errors
    /// Returns the first layout or stabilization error.
    pub fn settle(&mut self) -> Result<(), TestError> {
        for window in &mut self.windows {
            window.settle()?;
        }
        Ok(())
    }

    /// Returns the currently open application-window keys.
    #[must_use]
    pub fn window_keys(&self) -> Vec<WindowKey> {
        self.windows
            .iter()
            .map(|window| window.window_key().clone())
            .collect()
    }
}
