use super::{AnyEntity, Context, Entity, Render};
use crate::tasks::{TaskError, TaskFuture, TaskHandle, TaskOutput, TaskRuntime, TaskSlot};

impl<T: 'static> super::Mount<T> {
    /// Work owned by this exact presentation and its model. Delivery never
    /// recreates a dead presentation through a weak model reference.
    pub fn spawn<F>(
        &self,
        future: F,
        on_complete: impl FnOnce(&mut T, Result<F::Output, TaskError>, &mut Context<T>) + 'static,
    ) -> Result<TaskHandle, TaskError>
    where
        F: TaskFuture,
        F::Output: TaskOutput,
    {
        if self.resources().is_closed() || self.entity.resources().is_closed() {
            return Err(TaskError::ScopeClosed);
        }
        self.entity
            .update_view(|_, cx| cx.spawn(future, on_complete))
    }

    pub fn spawn_latest<F>(
        &self,
        slot: &mut TaskSlot,
        future: F,
        on_complete: impl FnOnce(&mut T, Result<F::Output, TaskError>, &mut Context<T>) + 'static,
    ) -> Result<(), TaskError>
    where
        F: TaskFuture,
        F::Output: TaskOutput,
    {
        slot.cancel();
        slot.replace(self.spawn(future, on_complete)?);
        Ok(())
    }
}

impl<T: 'static> Entity<T> {
    pub(crate) fn take_task_effects(&self) -> super::effects::ContextEffects
    where
        T: Render,
    {
        if self.0.presentation.resources.is_closed() {
            return super::ContextEffects::default();
        }
        self.update_view(|value, cx| {
            #[cfg(all(feature = "hot-reload", debug_assertions, not(target_arch = "wasm32")))]
            crate::hot_reload::tasks_ready(value, cx);
            #[cfg(not(all(
                feature = "hot-reload",
                debug_assertions,
                not(target_arch = "wasm32")
            )))]
            value.tasks_ready(cx);
        });
        let mut effects = self.take_effects();
        let children: Vec<_> = self
            .0
            .presentation
            .children
            .borrow()
            .iter()
            .chain(self.0.presentation.event_routes.borrow().iter())
            .cloned()
            .collect();
        for child in children {
            super::effects::merge_effects(&mut effects, child.take_task_effects());
        }
        self.0.presentation.cache.apply_update(effects.update);
        self.0.presentation.visible_effects(effects)
    }
    /// Attach an event-driven executor before invoking a component's task-producing methods.
    pub fn set_task_runtime(&self, runtime: TaskRuntime) {
        self.0.model.runtime.set_task_runtime(runtime.clone());
        for child in self
            .0
            .presentation
            .children
            .borrow()
            .iter()
            .chain(self.0.presentation.event_routes.borrow().iter())
        {
            child.set_task_runtime(runtime.clone());
        }
    }
}
impl AnyEntity {
    pub(crate) fn take_task_effects(&self) -> super::effects::ContextEffects {
        (self.task_effects)()
    }
    pub fn set_task_runtime(&self, runtime: TaskRuntime) {
        (self.tasks)(runtime);
    }
}
impl<T: 'static> Context<T> {
    /// The operation ends with its presentation, model owner or explicit scope.
    pub fn spawn_in<F>(
        &mut self,
        scope: &super::ResourceScope,
        future: F,
        on_complete: impl FnOnce(&mut T, Result<F::Output, TaskError>, &mut Context<T>) + 'static,
    ) -> Result<TaskHandle, TaskError>
    where
        F: TaskFuture,
        F::Output: TaskOutput,
    {
        if scope.is_closed() {
            return Err(TaskError::ScopeClosed);
        }
        self.spawn(future, on_complete)?
            .in_scope(scope)
            .map_err(|_| TaskError::ScopeClosed)
    }

    pub fn spawn<F>(
        &mut self,
        future: F,
        on_complete: impl FnOnce(&mut T, Result<F::Output, TaskError>, &mut Context<T>) + 'static,
    ) -> Result<TaskHandle, TaskError>
    where
        F: TaskFuture,
        F::Output: TaskOutput,
    {
        self.spawn_token(
            future,
            crate::tasks::CancellationToken::default(),
            on_complete,
            true,
        )
    }

    pub(super) fn spawn_token<F>(
        &mut self,
        future: F,
        token: crate::tasks::CancellationToken,
        on_complete: impl FnOnce(&mut T, Result<F::Output, TaskError>, &mut Context<T>) + 'static,
        presentation_owned: bool,
    ) -> Result<TaskHandle, TaskError>
    where
        F: TaskFuture,
        F::Output: TaskOutput,
    {
        let weak = self.entity.clone().ok_or(TaskError::Unavailable)?;
        let owner = weak.upgrade().ok_or(TaskError::Unavailable)?;
        if owner.resources().is_closed()
            || (presentation_owned && owner.0.presentation.resources.is_closed())
        {
            return Err(TaskError::ScopeClosed);
        }
        let runtime = owner
            .0
            .model
            .runtime
            .task_runtime()
            .ok_or(TaskError::Unavailable)?;
        let task = runtime
            .spawn(future, token, move |result| {
                if let Some(owner) = weak.upgrade() {
                    if presentation_owned {
                        owner.update_view(|value, cx| on_complete(value, result, cx));
                    } else {
                        owner.update_model(|value, cx| on_complete(value, result, cx));
                    }
                }
            })?
            .in_scope(owner.resources())
            .map_err(|_| TaskError::ScopeClosed)?;
        if presentation_owned {
            return task
                .in_scope(&owner.0.presentation.resources)
                .map_err(|_| TaskError::ScopeClosed);
        }
        Ok(task)
    }

    /// Cooperative cancellation only: an already-running blocking function cannot be killed.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn spawn_blocking<R: Send + 'static>(
        &mut self,
        work: impl FnOnce(crate::tasks::CancellationToken) -> R + Send + 'static,
        on_complete: impl FnOnce(&mut T, Result<R, TaskError>, &mut Context<T>) + 'static,
    ) -> Result<TaskHandle, TaskError> {
        self.spawn_blocking_owned(work, on_complete, true)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn spawn_blocking_owned<R: Send + 'static>(
        &mut self,
        work: impl FnOnce(crate::tasks::CancellationToken) -> R + Send + 'static,
        on_complete: impl FnOnce(&mut T, Result<R, TaskError>, &mut Context<T>) + 'static,
        presentation_owned: bool,
    ) -> Result<TaskHandle, TaskError> {
        let token = crate::tasks::CancellationToken::default();
        let worker_token = token.clone();
        self.spawn_token(
            async move {
                tokio::task::spawn_blocking(move || work(worker_token))
                    .await
                    .map_err(|_| TaskError::Panicked)
            },
            token,
            move |owner, result, cx| on_complete(owner, result.and_then(|result| result), cx),
            presentation_owned,
        )
    }
    pub fn spawn_latest<F>(
        &mut self,
        slot: &mut TaskSlot,
        future: F,
        on_complete: impl FnOnce(&mut T, Result<F::Output, TaskError>, &mut Context<T>) + 'static,
    ) -> Result<(), TaskError>
    where
        F: TaskFuture,
        F::Output: TaskOutput,
    {
        slot.cancel();
        slot.replace(self.spawn(future, on_complete)?);
        Ok(())
    }
    pub(super) fn inherit_tasks<U: Render>(&self, entity: &Entity<U>) {
        if let Some(runtime) = self
            .entity
            .as_ref()
            .and_then(|owner| owner.upgrade())
            .and_then(|owner| owner.0.model.runtime.task_runtime())
        {
            entity.set_task_runtime(runtime);
        }
    }
}
