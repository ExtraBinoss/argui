use super::{ResourceLease, ResourceScope, ScopeClosed};
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

type Cancellation = Box<dyn FnOnce()>;
pub(super) struct State {
    cancel: RefCell<Option<Cancellation>>,
    leases: RefCell<Vec<ResourceLease>>,
}
impl State {
    pub(super) fn active(&self) -> bool {
        self.cancel.borrow().is_some()
    }
    fn cancel(&self) {
        let cancel = self.cancel.borrow_mut().take();
        let leases = std::mem::take(&mut *self.leases.borrow_mut());
        drop(leases);
        if let Some(cancel) = cancel {
            cancel();
        }
    }
}
impl Drop for State {
    fn drop(&mut self) {
        self.cancel();
    }
}

/// Retain this handle for the desired subscription lifetime. Neither endpoint is kept alive.
#[must_use]
pub struct Subscription(Rc<State>);
impl Subscription {
    pub(super) fn new(cancel: impl FnOnce() + 'static) -> Self {
        Self(Rc::new(State {
            cancel: RefCell::new(Some(Box::new(cancel))),
            leases: RefCell::new(Vec::new()),
        }))
    }
    /// Cancels this subscription and any queued callback deliveries.
    pub fn cancel(&self) {
        self.0.cancel();
    }
    #[must_use]
    /// Returns whether this subscription can still deliver callbacks.
    pub fn is_active(&self) -> bool {
        self.0.active()
    }
    pub(super) fn weak(&self) -> Weak<State> {
        Rc::downgrade(&self.0)
    }

    /// Adds a lifetime constraint without removing the entity/handle lifetimes.
    ///
    /// `scope` is an additional owner whose closure cancels this subscription.
    ///
    /// # Errors
    /// Returns [`ScopeClosed`] if `scope` is already closed.
    pub fn in_scope(self, scope: &ResourceScope) -> Result<Self, ScopeClosed> {
        self.attach(scope)?;
        Ok(self)
    }
    pub(super) fn attach(&self, scope: &ResourceScope) -> Result<(), ScopeClosed> {
        if scope.is_closed() {
            return Err(ScopeClosed);
        }
        if !self.is_active() {
            return Ok(());
        }
        let weak = self.weak();
        let lease = scope.defer(move || {
            if let Some(state) = weak.upgrade() {
                state.cancel();
            }
        })?;
        self.0.leases.borrow_mut().push(lease);
        Ok(())
    }
}
