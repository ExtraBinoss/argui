use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};

/// A duplicate registration never replaces the currently published service.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServiceAlreadyRegistered;
impl std::fmt::Display for ServiceAlreadyRegistered {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("a service of this type is already registered")
    }
}
impl std::error::Error for ServiceAlreadyRegistered {}

type Entries = RefCell<HashMap<TypeId, Weak<dyn Any>>>;

#[derive(Default)]
pub(super) struct ServiceRegistry(Rc<Entries>);

/// Owns publication of one service. Retain this in application state or transfer
/// it to a `ResourceScope::own` lease. Dropping it unpublishes the service;
/// consumers that already hold an `Rc` may finish using that value.
/// The registry holds weak values, so publishing a service that retains models
/// cannot create a registry → service → model → runtime ownership cycle.
pub struct ServiceRegistration<T: 'static> {
    value: Rc<T>,
    registry: Weak<Entries>,
}

impl<T: 'static> ServiceRegistration<T> {
    #[must_use]
    /// Returns another shared owner of the registered service value.
    pub fn service(&self) -> Rc<T> {
        self.value.clone()
    }
}
impl<T: 'static> Drop for ServiceRegistration<T> {
    fn drop(&mut self) {
        if let Some(registry) = self.registry.upgrade() {
            registry.borrow_mut().remove(&TypeId::of::<T>());
        }
    }
}

impl super::ModelRuntime {
    /// Publish one typed service in this domain. The caller explicitly owns its
    /// registration; another runtime cannot see it and there is no global fallback.
    ///
    /// `value` is the service value to publish.
    ///
    /// # Errors
    /// Returns [`ServiceAlreadyRegistered`] if this runtime already has a live
    /// registration for the same concrete type.
    pub fn register_service<T: 'static>(
        &self,
        value: T,
    ) -> Result<ServiceRegistration<T>, ServiceAlreadyRegistered> {
        let registry = &self.services().0;
        let mut entries = registry.borrow_mut();
        let id = TypeId::of::<T>();
        if entries.contains_key(&id) {
            return Err(ServiceAlreadyRegistered);
        }
        let value = Rc::new(value);
        let erased: Rc<dyn Any> = value.clone();
        entries.insert(id, Rc::downgrade(&erased));
        Ok(ServiceRegistration {
            value,
            registry: Rc::downgrade(registry),
        })
    }

    #[must_use]
    /// Looks up a service of type `T` in this runtime.
    /// Returns `None` if no live registration exists.
    pub fn service<T: 'static>(&self) -> Option<Rc<T>> {
        self.services()
            .0
            .borrow()
            .get(&TypeId::of::<T>())?
            .upgrade()?
            .downcast()
            .ok()
    }
}

impl<T: 'static> super::Context<T> {
    /// Resolve an application service without borrowing the registry during use.
    #[must_use]
    /// Returns `None` when this context is detached or the service is not registered.
    pub fn service<S: 'static>(&self) -> Option<Rc<S>> {
        self.entity.as_ref()?.upgrade()?.runtime().service()
    }
}
