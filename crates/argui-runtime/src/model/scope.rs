use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    rc::{Rc, Weak},
};

type Cleanup = Box<dyn FnOnce()>;

#[derive(Default)]
struct Inner {
    closed: Cell<bool>,
    next: Cell<u64>,
    resources: RefCell<BTreeMap<u64, Cleanup>>,
}

impl Inner {
    fn close(&self) {
        if self.closed.replace(true) {
            return;
        }
        let resources = std::mem::take(&mut *self.resources.borrow_mut());
        let mut panic = None;
        for (_, cleanup) in resources.into_iter().rev() {
            if let Err(payload) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(cleanup))
                && panic.is_none()
            {
                panic = Some(payload);
            }
        }
        if !std::thread::panicking()
            && let Some(payload) = panic
        {
            std::panic::resume_unwind(payload);
        }
    }
}
impl Drop for Inner {
    fn drop(&mut self) {
        self.close();
    }
}

/// UI-thread resource lifetime. Closing a scope is idempotent and releases its
/// registrations in reverse order, without retaining registry borrows over cleanup.
/// Cloning shares the lifetime; explicit owner destruction must call `close`.
#[derive(Clone, Default)]
pub struct ResourceScope(Rc<Inner>);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScopeClosed;
impl std::fmt::Display for ScopeClosed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("resource scope is closed")
    }
}
impl std::error::Error for ScopeClosed {}

impl ResourceScope {
    pub fn close(&self) {
        self.0.close();
    }
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.0.closed.get()
    }
    #[must_use]
    pub fn resource_count(&self) -> usize {
        self.0.resources.borrow().len()
    }

    /// Runs cleanup when either the scope closes or the returned lease is dropped.
    pub fn defer(&self, cleanup: impl FnOnce() + 'static) -> Result<ResourceLease, ScopeClosed> {
        if self.is_closed() {
            return Err(ScopeClosed);
        }
        let id = self.0.next.get();
        self.0.next.set(
            id.checked_add(1)
                .expect("resource identity space exhausted"),
        );
        self.0.resources.borrow_mut().insert(id, Box::new(cleanup));
        Ok(ResourceLease {
            scope: Rc::downgrade(&self.0),
            id,
        })
    }

    /// Owns an arbitrary resource until either the scope or its lease ends.
    pub fn own<T: 'static>(&self, resource: T) -> Result<ResourceLease, ScopeClosed> {
        self.defer(move || drop(resource))
    }
}

/// Does not keep its scope alive. Dropping it releases its registration immediately.
#[must_use]
pub struct ResourceLease {
    scope: Weak<Inner>,
    id: u64,
}
impl ResourceLease {
    /// Remove an already completed resource without running its cancellation.
    #[cfg(feature = "tasks")]
    pub(crate) fn disarm(self) {
        if let Some(scope) = self.scope.upgrade() {
            let cleanup = scope.resources.borrow_mut().remove(&self.id);
            drop(cleanup);
        }
    }

    #[must_use]
    pub fn is_active(&self) -> bool {
        self.scope
            .upgrade()
            .is_some_and(|scope| scope.resources.borrow().contains_key(&self.id))
    }
}
impl Drop for ResourceLease {
    fn drop(&mut self) {
        if let Some(scope) = self.scope.upgrade() {
            let cleanup = scope.resources.borrow_mut().remove(&self.id);
            if let Some(cleanup) = cleanup {
                cleanup();
            }
        }
    }
}
