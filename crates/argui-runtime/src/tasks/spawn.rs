use super::{
    TaskError, TaskHandle, TaskRuntime,
    dispatcher::{Value, deliver},
};
use futures_util::{FutureExt, future::Abortable};
use std::{future::Future, panic::AssertUnwindSafe};

/// Native tasks cross worker threads; browser tasks may own local JS handles.
#[cfg(not(target_arch = "wasm32"))]
pub trait TaskFuture: Future + Send + 'static {}
#[cfg(not(target_arch = "wasm32"))]
impl<T: Future + Send + 'static> TaskFuture for T {}
#[cfg(target_arch = "wasm32")]
pub trait TaskFuture: Future + 'static {}
#[cfg(target_arch = "wasm32")]
impl<T: Future + 'static> TaskFuture for T {}
#[cfg(not(target_arch = "wasm32"))]
pub trait TaskOutput: Send + 'static {}
#[cfg(not(target_arch = "wasm32"))]
impl<T: Send + 'static> TaskOutput for T {}
#[cfg(target_arch = "wasm32")]
pub trait TaskOutput: 'static {}
#[cfg(target_arch = "wasm32")]
impl<T: 'static> TaskOutput for T {}

impl TaskRuntime {
    pub(crate) fn spawn<F>(
        &self,
        future: F,
        token: super::CancellationToken,
        callback: impl FnOnce(Result<F::Output, TaskError>) + 'static,
    ) -> Result<TaskHandle, TaskError>
    where
        F: TaskFuture,
        F::Output: TaskOutput,
    {
        #[cfg(not(target_arch = "wasm32"))]
        let executor = self.executor()?;
        let (handle, registration) = TaskHandle::pair(token);
        let (id, sender, wake) = self.register(
            handle.state.clone(),
            handle.scopes.clone(),
            Box::new(move |result| {
                callback(
                    result.map(|value| *value.downcast::<F::Output>().expect("typed task result")),
                );
            }),
        )?;
        let work = async move {
            let result =
                Abortable::new(AssertUnwindSafe(future).catch_unwind(), registration).await;
            let result = match result {
                Ok(Ok(value)) => Some(Ok(Box::new(value) as Value)),
                Ok(Err(_)) => Some(Err(TaskError::Panicked)),
                Err(_) => None,
            };
            deliver(sender, wake, id, result).await;
        };
        #[cfg(not(target_arch = "wasm32"))]
        executor.spawn(work);
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(work);
        Ok(handle)
    }
}
