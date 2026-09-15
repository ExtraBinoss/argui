//! Owned asynchronous work with event-driven delivery on the UI thread.
use futures_util::future::{AbortHandle, AbortRegistration};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::{cell::RefCell, rc::Rc};

mod dispatcher;
pub use dispatcher::TaskRuntime;
mod spawn;
pub use spawn::{TaskFuture, TaskOutput};

#[derive(Clone, Debug, Eq, PartialEq)]
/// Failure encountered while scheduling or delivering asynchronous work.
pub enum TaskError {
    /// A model or presentation resource scope closed before work could be started.
    ScopeClosed,
    /// No task executor was available or the task owner had been detached.
    Unavailable,
    /// The runtime could not reserve capacity for another task.
    CapacityExceeded,
    /// The executor could not be initialized or used.
    Runtime(String),
    /// The future or blocking work panicked before producing a result.
    Panicked,
}
impl std::fmt::Display for TaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for TaskError {}

#[derive(Clone, Debug, Default)]
/// Shared cancellation state that task code can check cooperatively.
pub struct CancellationToken(Arc<AtomicBool>);
impl CancellationToken {
    /// Returns whether cancellation has been requested.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
    /// Marks this token as cancelled.
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
}

pub(crate) struct TaskState {
    token: CancellationToken,
    abort: AbortHandle,
    finished: AtomicBool,
}
impl TaskState {
    pub(crate) fn active(&self) -> bool {
        !self.finished.load(Ordering::Acquire) && !self.token.is_cancelled()
    }
    pub(crate) fn cancel(&self) {
        self.token.cancel();
        self.abort.abort();
    }
}

/// Retain the handle while work is wanted. Dropping it cancels delivery.
#[must_use]
pub struct TaskHandle {
    pub(crate) state: Arc<TaskState>,
    pub(crate) scopes: Rc<RefCell<Vec<crate::ResourceLease>>>,
}
impl TaskHandle {
    /// Also cancel when this UI-owned resource scope ends. Existing owner and
    /// handle cancellation remain in force.
    ///
    /// `scope` is the resource scope that owns the additional cancellation hook.
    ///
    /// # Errors
    /// Returns [`crate::ScopeClosed`] if the scope has already been closed.
    pub fn in_scope(self, scope: &crate::ResourceScope) -> Result<Self, crate::ScopeClosed> {
        if scope.is_closed() {
            return Err(crate::ScopeClosed);
        }
        if self.is_finished() {
            return Ok(self);
        }
        let weak = Arc::downgrade(&self.state);
        let scopes = Rc::downgrade(&self.scopes);
        let lease = scope.defer(move || {
            if let Some(state) = weak.upgrade() {
                state.cancel();
            }
            if let Some(scopes) = scopes.upgrade() {
                let leases = std::mem::take(&mut *scopes.borrow_mut());
                drop(leases);
            }
        })?;
        self.scopes.borrow_mut().push(lease);
        Ok(self)
    }
    /// Cancels the task and releases any resource-scope registrations.
    pub fn cancel(&self) {
        self.state.cancel();
        let leases = std::mem::take(&mut *self.scopes.borrow_mut());
        drop(leases);
    }
    #[must_use]
    /// Reports whether this task has completed or has been cancelled.
    pub fn is_finished(&self) -> bool {
        !self.state.active()
    }
    #[must_use]
    /// Returns a token that task code can use to observe cancellation.
    pub fn cancellation_token(&self) -> CancellationToken {
        self.state.token.clone()
    }
    pub(crate) fn pair(token: CancellationToken) -> (Self, AbortRegistration) {
        let (abort, registration) = AbortHandle::new_pair();
        (
            Self {
                scopes: Rc::default(),
                state: Arc::new(TaskState {
                    token,
                    abort,
                    finished: AtomicBool::new(false),
                }),
            },
            registration,
        )
    }
}
impl Drop for TaskHandle {
    fn drop(&mut self) {
        self.cancel();
    }
}

/// One replaceable operation. Replacing it cancels even an already queued result.
#[derive(Default)]
pub struct TaskSlot {
    handle: Option<TaskHandle>,
}
impl TaskSlot {
    /// Cancels and removes the operation currently stored in this slot.
    pub fn cancel(&mut self) {
        self.handle = None;
    }
    #[must_use]
    /// Returns whether this slot contains a task that has not completed.
    pub fn is_running(&self) -> bool {
        self.handle.as_ref().is_some_and(|task| !task.is_finished())
    }
    pub(crate) fn replace(&mut self, handle: TaskHandle) {
        self.handle = Some(handle);
    }
}

/// Suspends the current task for at least the requested duration.
///
/// `duration` is the interval to wait before resuming.
pub async fn sleep(duration: std::time::Duration) {
    #[cfg(not(target_arch = "wasm32"))]
    tokio::time::sleep(duration).await;
    #[cfg(target_arch = "wasm32")]
    {
        let mut millis = duration.as_millis();
        while millis > 0 {
            let part = millis.min(u128::from(u32::MAX)) as u32;
            gloo_timers::future::TimeoutFuture::new(part).await;
            millis -= u128::from(part);
        }
    }
}
/// Yields execution so other ready work can run before this task resumes.
pub async fn yield_now() {
    #[cfg(not(target_arch = "wasm32"))]
    tokio::task::yield_now().await;
    #[cfg(target_arch = "wasm32")]
    gloo_timers::future::TimeoutFuture::new(0).await;
}
