use super::{TaskError, TaskState};
use futures_channel::mpsc;
use futures_util::SinkExt;
use std::{
    any::Any,
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
    sync::Arc,
};

#[cfg(not(target_arch = "wasm32"))]
pub(super) type Value = Box<dyn Any + Send>;
#[cfg(target_arch = "wasm32")]
pub(super) type Value = Box<dyn Any>;
pub(super) type Completion = Result<Value, TaskError>;
type Message = (u64, Option<Completion>);
type Registration = (u64, mpsc::Sender<Message>, Wake);
#[cfg(not(target_arch = "wasm32"))]
pub(super) type Wake = Arc<dyn Fn() + Send + Sync>;
#[cfg(target_arch = "wasm32")]
pub(super) type Wake = Rc<dyn Fn()>;
type Callback = Box<dyn FnOnce(Completion)>;

struct Entry {
    state: Arc<TaskState>,
    scopes: Rc<RefCell<Vec<crate::ResourceLease>>>,
    callback: Callback,
}
impl Entry {
    fn release_scopes(&self) {
        let leases = std::mem::take(&mut *self.scopes.borrow_mut());
        for lease in leases {
            lease.disarm();
        }
    }
}
struct Inner {
    closed: Cell<bool>,
    wake_pending: Arc<std::sync::atomic::AtomicBool>,
    entries: RefCell<HashMap<u64, Entry>>,
    next: Cell<u64>,
    sender: mpsc::Sender<(u64, Option<Completion>)>,
    receiver: RefCell<mpsc::Receiver<(u64, Option<Completion>)>>,
    wake: Wake,
    #[cfg(not(target_arch = "wasm32"))]
    executor: RefCell<Option<tokio::runtime::Runtime>>,
}
impl Drop for Inner {
    fn drop(&mut self) {
        for entry in self.entries.get_mut().values() {
            entry.state.cancel();
            entry.release_scopes();
        }
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(runtime) = self.executor.get_mut().take() {
            runtime.shutdown_background();
        }
    }
}

/// A UI-thread dispatcher. Custom hosts call drain after their wake callback fires.
/// Clone this dispatcher across windows to share one lazy native executor.
#[derive(Clone)]
pub struct TaskRuntime(Rc<Inner>);
impl TaskRuntime {
    #[cfg(not(target_arch = "wasm32"))]
    /// Creates a task dispatcher whose completion wake can run on a native host.
    ///
    /// `wake` schedules a later call to [`Self::drain`] when completions are ready.
    pub fn new(wake: impl Fn() + Send + Sync + 'static) -> Self {
        Self::with_wake(Arc::new(wake))
    }
    #[cfg(target_arch = "wasm32")]
    /// Creates a browser task dispatcher with a local completion wake callback.
    ///
    /// `wake` schedules a later call to [`Self::drain`] when completions are ready.
    pub fn new(wake: impl Fn() + 'static) -> Self {
        Self::with_wake(Rc::new(wake))
    }
    fn with_wake(wake: Wake) -> Self {
        let wake_pending = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let pending = wake_pending.clone();
        #[cfg(not(target_arch = "wasm32"))]
        let wake: Wake = Arc::new(move || {
            if !pending.swap(true, std::sync::atomic::Ordering::AcqRel) {
                wake();
            }
        });
        #[cfg(target_arch = "wasm32")]
        let wake: Wake = Rc::new(move || {
            if !pending.swap(true, std::sync::atomic::Ordering::AcqRel) {
                wake();
            }
        });
        let (sender, receiver) = mpsc::channel(64);
        Self(Rc::new(Inner {
            closed: Cell::new(false),
            wake_pending,
            entries: RefCell::new(HashMap::new()),
            next: Cell::new(0),
            sender,
            receiver: RefCell::new(receiver),
            wake,
            #[cfg(not(target_arch = "wasm32"))]
            executor: RefCell::new(None),
        }))
    }
    #[must_use]
    /// Returns the number of registered tasks awaiting completion or cancellation.
    pub fn pending(&self) -> usize {
        self.0.entries.borrow().len()
    }

    /// Stop accepting work and cancel all deliveries, even if owners outlive the host.
    /// Repeated calls have no additional effect.
    pub fn shutdown(&self) {
        self.0.closed.set(true);
        let entries = std::mem::take(&mut *self.0.entries.borrow_mut());
        for entry in entries.into_values() {
            entry.state.cancel();
            entry.release_scopes();
        }
        self.0.receiver.borrow_mut().close();
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(runtime) = self.0.executor.borrow_mut().take() {
            runtime.shutdown_background();
        }
    }

    /// Deliver at most 64 completions, without keeping a registry borrow over callbacks.
    /// Requeues the wake callback if more completions remain.
    ///
    /// # Panics
    /// Propagates a panic raised by a task completion callback or the wake callback.
    pub fn drain(&self) {
        self.0
            .wake_pending
            .store(false, std::sync::atomic::Ordering::Release);
        for _ in 0..64 {
            let message = self.0.receiver.borrow_mut().try_recv();
            let Ok((id, result)) = message else {
                return;
            };
            let entry = self.0.entries.borrow_mut().remove(&id);
            if let Some(entry) = entry {
                entry.release_scopes();
                entry
                    .state
                    .finished
                    .store(true, std::sync::atomic::Ordering::Release);
                if !entry.state.token.is_cancelled()
                    && let Some(result) = result
                {
                    (entry.callback)(result);
                }
            }
        }
        (self.0.wake)();
    }

    pub(super) fn register(
        &self,
        state: Arc<TaskState>,
        scopes: Rc<RefCell<Vec<crate::ResourceLease>>>,
        callback: Callback,
    ) -> Result<Registration, TaskError> {
        if self.0.closed.get() {
            return Err(TaskError::Unavailable);
        }
        let mut entries = self.0.entries.borrow_mut();
        if entries.len() >= 256 {
            return Err(TaskError::CapacityExceeded);
        }
        let id = self.0.next.get();
        self.0
            .next
            .set(id.checked_add(1).ok_or(TaskError::CapacityExceeded)?);
        entries.insert(
            id,
            Entry {
                state,
                scopes,
                callback,
            },
        );
        Ok((id, self.0.sender.clone(), self.0.wake.clone()))
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn executor(&self) -> Result<tokio::runtime::Handle, TaskError> {
        if self.0.closed.get() {
            return Err(TaskError::Unavailable);
        }
        let mut executor = self.0.executor.borrow_mut();
        if executor.is_none() {
            *executor = Some(
                tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(
                        std::thread::available_parallelism().map_or(1, |count| count.get().min(4)),
                    )
                    .enable_all()
                    .build()
                    .map_err(|error| TaskError::Runtime(error.to_string()))?,
            );
        }
        Ok(executor.as_ref().unwrap().handle().clone())
    }
}

pub(super) async fn deliver(
    mut sender: mpsc::Sender<(u64, Option<Completion>)>,
    wake: Wake,
    id: u64,
    result: Option<Completion>,
) {
    if sender.send((id, result)).await.is_ok() {
        wake();
    }
}
