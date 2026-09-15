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
    /// Marks the model dirty so its presentations rebuild on the next frame.
    pub fn notify(&mut self) {
        self.inner.notify();
    }
    #[must_use]
    /// Returns a shared application service of type `S`, if one is registered.
    pub fn service<S: 'static>(&self) -> Option<std::rc::Rc<S>> {
        self.inner.service()
    }
    /// Queues an application command for the host to process.
    ///
    /// `command` describes the requested host operation.
    pub fn command(&mut self, command: AppCommand) {
        self.inner.command(command);
    }
    #[must_use]
    /// Returns the strongest view update requested through this context.
    pub fn view_update(&self) -> ViewUpdate {
        self.inner.view_update()
    }
    #[must_use]
    /// Creates a retained entity in this model's runtime.
    ///
    /// `value` is the initial state stored in the new entity.
    pub fn new_entity<U: 'static>(&mut self, value: U) -> Entity<U> {
        self.inner.new_entity(value)
    }
    /// Observe at model scope, invalidating all its presentations. The returned
    /// handle and both model owners bound the observation, not an individual view.
    ///
    /// `source` is the entity whose notifications should invalidate this model.
    ///
    /// # Errors
    /// Returns an error if this context is detached, the entities belong to different
    /// runtimes, or an owning resource scope has closed.
    pub fn observe<S: 'static>(&mut self, source: &Entity<S>) -> Result<Subscription, EventError> {
        self.inner.observe_owned(source, false)
    }
    /// Queues `event` for current subscribers of its declared event type.
    ///
    /// # Errors
    /// Returns an error if this context is detached, its model scope has closed, or
    /// the runtime cannot accept another queued delivery.
    pub fn emit<E: 'static>(&mut self, event: E) -> Result<(), EventError>
    where
        T: EventEmitter<E>,
    {
        self.inner.emit(event)
    }
    /// Subscribes this model to events emitted by `source`.
    ///
    /// `source` emits events of type `E`; `callback` handles each delivery with model
    /// context access. The returned handle controls the subscription lifetime.
    ///
    /// # Errors
    /// Returns an error if the context is detached, the models use different runtimes,
    /// or an owning resource scope has closed.
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
    /// Spawns asynchronous work owned by this model.
    ///
    /// `future` computes the result; `callback` applies that result to the model when
    /// delivered. Returns a handle that can cancel the work.
    ///
    /// # Errors
    /// Returns a task error if no executor is attached, the model scope has closed,
    /// or the runtime cannot accept the task.
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
    /// Spawns work whose delivery is also bounded by `scope`.
    ///
    /// `scope` supplies an additional lifetime boundary; `future` computes the result;
    /// `callback` applies it to the model when delivered.
    ///
    /// # Errors
    /// Returns a task error if the scope/model is closed, no executor is attached, or
    /// the runtime cannot accept the task.
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
    /// Replaces the operation in `slot` with newly spawned work.
    ///
    /// `slot` tracks the replaceable operation; `future` computes its result and
    /// `callback` handles delivery.
    ///
    /// # Errors
    /// Returns a task error if spawning the replacement fails.
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
    /// Runs blocking work on the native blocking executor.
    ///
    /// `work` receives a cooperative cancellation token; `callback` handles the result
    /// on the model context. Returns a handle for cancellation.
    ///
    /// # Errors
    /// Returns a task error if no executor is attached, the model scope has closed, or
    /// the runtime cannot accept the task.
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
