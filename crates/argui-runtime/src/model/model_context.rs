use super::{AppCommand, Context, Entity, EventEmitter, EventError, Subscription, ViewUpdate};

/// Model-only capabilities during a mutation. No access to window environment,
/// focus, layout, rendering, selection or other presentation services is exposed.
/// Even a renderable model needs a live mount to access presentation services.
#[doc = include_str!("../../tests/model/model_context.md")]
pub struct ModelContext<'a, T> {
    inner: &'a mut Context<T>,
}

impl<'a, T: 'static> ModelContext<'a, T> {
    pub(super) fn new(inner: &'a mut Context<T>) -> Self {
        Self { inner }
    }
    pub fn notify(&mut self) {
        self.inner.notify();
    }
    #[must_use]
    pub fn service<S: 'static>(&self) -> Option<std::rc::Rc<S>> {
        self.inner.service()
    }
    pub fn command(&mut self, command: AppCommand) {
        self.inner.command(command);
    }
    #[must_use]
    pub fn view_update(&self) -> ViewUpdate {
        self.inner.view_update()
    }
    #[must_use]
    pub fn new_entity<U: 'static>(&mut self, value: U) -> Entity<U> {
        self.inner.new_entity(value)
    }
    /// Observe at model scope, invalidating all its presentations. The returned
    /// handle and both model owners bound the observation, not an individual view.
    pub fn observe<S: 'static>(&mut self, source: &Entity<S>) -> Result<Subscription, EventError> {
        self.inner.observe_owned(source, false)
    }
    pub fn emit<E: 'static>(&mut self, event: E) -> Result<(), EventError>
    where
        T: EventEmitter<E>,
    {
        self.inner.emit(event)
    }
    pub fn subscribe<S, E>(
        &mut self,
        source: &Entity<S>,
        callback: impl Fn(&mut T, &E, &mut ModelContext<'_, T>) + 'static,
    ) -> Result<Subscription, EventError>
    where
        S: EventEmitter<E>,
        E: 'static,
    {
        self.inner.subscribe_owned(
            source,
            move |value, event, cx| callback(value, event, &mut ModelContext::new(cx)),
            false,
        )
    }
}

#[cfg(feature = "tasks")]
impl<T: 'static> ModelContext<'_, T> {
    pub fn spawn<F>(
        &mut self,
        future: F,
        callback: impl FnOnce(
            &mut T,
            Result<F::Output, crate::tasks::TaskError>,
            &mut ModelContext<'_, T>,
        ) + 'static,
    ) -> Result<crate::tasks::TaskHandle, crate::tasks::TaskError>
    where
        F: crate::tasks::TaskFuture,
        F::Output: crate::tasks::TaskOutput,
    {
        self.inner.spawn_token(
            future,
            crate::tasks::CancellationToken::default(),
            move |value, result, cx| callback(value, result, &mut ModelContext::new(cx)),
            false,
        )
    }
    pub fn spawn_in<F>(
        &mut self,
        scope: &super::ResourceScope,
        future: F,
        callback: impl FnOnce(
            &mut T,
            Result<F::Output, crate::tasks::TaskError>,
            &mut ModelContext<'_, T>,
        ) + 'static,
    ) -> Result<crate::tasks::TaskHandle, crate::tasks::TaskError>
    where
        F: crate::tasks::TaskFuture,
        F::Output: crate::tasks::TaskOutput,
    {
        if scope.is_closed() {
            return Err(crate::tasks::TaskError::ScopeClosed);
        }
        self.spawn(future, callback)?
            .in_scope(scope)
            .map_err(|_| crate::tasks::TaskError::ScopeClosed)
    }
    pub fn spawn_latest<F>(
        &mut self,
        slot: &mut crate::tasks::TaskSlot,
        future: F,
        callback: impl FnOnce(
            &mut T,
            Result<F::Output, crate::tasks::TaskError>,
            &mut ModelContext<'_, T>,
        ) + 'static,
    ) -> Result<(), crate::tasks::TaskError>
    where
        F: crate::tasks::TaskFuture,
        F::Output: crate::tasks::TaskOutput,
    {
        slot.cancel();
        slot.replace(self.spawn(future, callback)?);
        Ok(())
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn spawn_blocking<R: Send + 'static>(
        &mut self,
        work: impl FnOnce(crate::tasks::CancellationToken) -> R + Send + 'static,
        callback: impl FnOnce(&mut T, Result<R, crate::tasks::TaskError>, &mut ModelContext<'_, T>)
        + 'static,
    ) -> Result<crate::tasks::TaskHandle, crate::tasks::TaskError> {
        self.inner.spawn_blocking_owned(
            work,
            move |value, result, cx| callback(value, result, &mut ModelContext::new(cx)),
            false,
        )
    }
}
