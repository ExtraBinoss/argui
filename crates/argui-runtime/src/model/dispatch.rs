use std::{
    cell::{Cell, RefCell},
    collections::{HashSet, VecDeque},
    rc::Rc,
};

use super::{Entity, EntityId};

type Delivery = Box<dyn FnMut() -> bool>;
type InvalidationStep = Box<dyn FnMut(&ModelRuntime) -> bool>;
type Wave = Rc<RefCell<HashSet<EntityId>>>;
struct Invalidation {
    wave: Wave,
    step: InvalidationStep,
}
const EVENT_CAPACITY: usize = 1024;
const DELIVERIES_PER_TURN: usize = 64;

struct Inner {
    linked: RefCell<Vec<std::rc::Weak<Inner>>>,
    lifecycle: super::lifecycle::LifecycleRegistry,
    services: super::services::ServiceRegistry,
    #[cfg(feature = "tasks")]
    tasks: RefCell<Option<crate::tasks::TaskRuntime>>,
    queue: RefCell<VecDeque<Delivery>>,
    invalidations: RefCell<VecDeque<Invalidation>>,
    transaction_wave: RefCell<Option<Wave>>,
    active_wave: RefCell<Option<Wave>>,
    depth: Cell<usize>,
    dispatching: Cell<bool>,
    delivering_event: Cell<bool>,
    wake: RefCell<Option<Rc<dyn Fn()>>>,
    wake_pending: Cell<bool>,
}

/// UI-thread transaction and event owner shared by related models and views.
/// Hosts provide a wake callback and call `dispatch_pending` on that wake.
/// A hostless runtime can be advanced explicitly; it never installs a polling timer.
#[derive(Clone)]
pub struct ModelRuntime(Rc<Inner>);

impl Default for ModelRuntime {
    fn default() -> Self {
        Self::with_wake(None)
    }
}

impl ModelRuntime {
    #[must_use]
    pub fn new(wake: impl Fn() + 'static) -> Self {
        Self::with_wake(Some(Rc::new(wake)))
    }

    fn with_wake(wake: Option<Rc<dyn Fn()>>) -> Self {
        Self(Rc::new(Inner {
            linked: RefCell::default(),
            lifecycle: super::lifecycle::LifecycleRegistry::default(),
            services: super::services::ServiceRegistry::default(),
            #[cfg(feature = "tasks")]
            tasks: RefCell::new(None),
            queue: RefCell::new(VecDeque::new()),
            invalidations: RefCell::new(VecDeque::new()),
            transaction_wave: RefCell::new(None),
            active_wave: RefCell::new(None),
            depth: Cell::new(0),
            dispatching: Cell::new(false),
            delivering_event: Cell::new(false),
            wake: RefCell::new(wake),
            wake_pending: Cell::new(false),
        }))
    }

    #[must_use]
    pub fn entity<T: 'static>(&self, value: T) -> Entity<T> {
        Entity::in_runtime(value, self.clone())
    }

    pub(super) fn collect_linked(&self, visited: &mut Vec<Self>) {
        if visited.iter().any(|runtime| runtime.same(self)) {
            return;
        }
        visited.push(self.clone());
        let linked: Vec<_> = self
            .0
            .linked
            .borrow()
            .iter()
            .filter_map(std::rc::Weak::upgrade)
            .collect();
        self.0
            .linked
            .borrow_mut()
            .retain(|weak| weak.strong_count() > 0);
        for inner in linked {
            Self(inner).collect_linked(visited);
        }
    }

    /// Disconnects a terminating host, cancels queued model work and stops its executor.
    /// Data and explicitly owned services remain valid. Final lifecycle events are delivered.
    pub fn shutdown_host(&self) {
        #[cfg(feature = "tasks")]
        if let Some(tasks) = self.task_runtime() {
            tasks.shutdown();
        }
        self.detach_host();
        let result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.lifecycle().finish()));
        self.detach_host();
        if let Err(payload) = result
            && !std::thread::panicking()
        {
            std::panic::resume_unwind(payload);
        }
    }

    pub(super) fn detach_host(&self) {
        self.0.wake.borrow_mut().take();
        self.0.wake_pending.set(false);
        self.0.queue.borrow_mut().clear();
        self.0.invalidations.borrow_mut().clear();
        self.0.linked.borrow_mut().clear();
    }

    pub(super) fn lifecycle(&self) -> &super::lifecycle::LifecycleRegistry {
        &self.0.lifecycle
    }
    pub(super) fn enqueue_mount_event(&self, event: super::MountEvent) {
        self.0.lifecycle.enqueue(event);
        self.wake_if_pending();
    }

    pub(super) fn services(&self) -> &super::services::ServiceRegistry {
        &self.0.services
    }

    /// Bind this application's event-loop wake. Already queued work requests a
    /// wake immediately; replacing a host does not inherit its outstanding wake.
    /// The callback schedules a later `dispatch_pending`, never a synchronous drain.
    pub fn set_wake(&self, wake: impl Fn() + 'static) {
        self.bind_wake(Rc::new(wake));
    }

    pub(super) fn inherit_wake(&self, parent: &Self) {
        if !self.same(parent) {
            let mut linked = parent.0.linked.borrow_mut();
            linked.retain(|weak| weak.strong_count() > 0);
            if !linked
                .iter()
                .any(|weak| weak.ptr_eq(&Rc::downgrade(&self.0)))
            {
                linked.push(Rc::downgrade(&self.0));
            }
        }
        let wake = parent.0.wake.borrow().clone();
        if let Some(wake) = wake {
            self.bind_wake(wake);
        }
    }

    fn bind_wake(&self, wake: Rc<dyn Fn()>) {
        if self
            .0
            .wake
            .borrow()
            .as_ref()
            .is_some_and(|old| Rc::ptr_eq(old, &wake))
        {
            return;
        }
        *self.0.wake.borrow_mut() = Some(wake);
        self.0.wake_pending.set(false);
        self.wake_if_pending();
    }

    fn wake_if_pending(&self) {
        if self.pending_events() == 0
            && self.pending_invalidations() == 0
            && self.pending_mount_events() == 0
        {
            return;
        }
        self.request_wake();
    }

    pub(super) fn request_wake(&self) {
        let wake = self.0.wake.borrow().clone();
        if let Some(wake) = wake
            && !self.0.wake_pending.replace(true)
        {
            wake();
        }
    }

    /// Attach the host executor to every model in this runtime, including models
    /// created before attachment and models that are never rendered.
    #[cfg(feature = "tasks")]
    pub fn set_task_runtime(&self, runtime: crate::tasks::TaskRuntime) {
        *self.0.tasks.borrow_mut() = Some(runtime);
    }

    #[cfg(feature = "tasks")]
    pub(super) fn task_runtime(&self) -> Option<crate::tasks::TaskRuntime> {
        self.0.tasks.borrow().clone()
    }

    /// Defers event callbacks until every nested transaction has released its borrows.
    /// This batches delivery, not data rollback.
    pub fn transaction<R>(&self, update: impl FnOnce() -> R) -> R {
        let _guard = self.enter();
        update()
    }

    #[must_use]
    pub fn pending_events(&self) -> usize {
        self.0.queue.borrow().len()
    }

    #[must_use]
    pub fn pending_invalidations(&self) -> usize {
        self.0.invalidations.borrow().len()
    }

    /// Runs a bounded batch, yielding to the host even for self-emitting listeners.
    pub fn dispatch_pending(&self) {
        self.0.wake_pending.set(false);
        self.dispatch_batch(true);
    }

    fn dispatch_batch(&self, lifecycle: bool) {
        if self.0.depth.get() != 0 || self.0.dispatching.replace(true) {
            return;
        }
        let _guard = DispatchGuard(self);
        for index in 0..DELIVERIES_PER_TURN {
            if index % 3 == 0 && self.dispatch_invalidation() {
                continue;
            }
            if lifecycle && index % 3 == 1 && self.0.lifecycle.dispatch() {
                continue;
            }
            let delivery = self.0.queue.borrow_mut().pop_front();
            if let Some(mut delivery) = delivery {
                self.0.delivering_event.set(true);
                let guard = EventGuard(&self.0.delivering_event);
                let finished = delivery();
                drop(guard);
                if !finished {
                    self.0.queue.borrow_mut().push_front(delivery);
                }
            } else if !self.dispatch_invalidation() && (!lifecycle || !self.0.lifecycle.dispatch())
            {
                break;
            }
        }
    }

    fn dispatch_invalidation(&self) -> bool {
        let work = self.0.invalidations.borrow_mut().pop_front();
        let Some(mut work) = work else {
            return false;
        };
        let previous = self.0.active_wave.replace(Some(work.wave.clone()));
        let _wave_guard = WaveGuard(&self.0.active_wave, previous);
        if !(work.step)(self) {
            self.0.invalidations.borrow_mut().push_back(work);
        }
        true
    }

    pub(super) fn invalidate(&self, entity: EntityId, step: InvalidationStep, propagated: bool) {
        let wave = if propagated {
            self.0.active_wave.borrow().clone()
        } else {
            None
        }
        .unwrap_or_else(|| {
            if self.0.depth.get() == 0 {
                Rc::new(RefCell::new(HashSet::new()))
            } else {
                self.0
                    .transaction_wave
                    .borrow_mut()
                    .get_or_insert_with(Default::default)
                    .clone()
            }
        });
        let first = wave.borrow_mut().insert(entity);
        if first {
            self.0
                .invalidations
                .borrow_mut()
                .push_back(Invalidation { wave, step });
        }
    }

    pub(super) fn enter(&self) -> Transaction<'_> {
        if self.0.depth.get() == 0 {
            self.0.transaction_wave.borrow_mut().take();
        }
        self.0.depth.set(self.0.depth.get() + 1);
        Transaction(self)
    }

    pub(super) fn same(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }

    pub(super) fn enqueue(&self, delivery: Delivery) -> Result<(), super::EventError> {
        let mut queue = self.0.queue.borrow_mut();
        if queue.len() + usize::from(self.0.delivering_event.get()) >= EVENT_CAPACITY {
            return Err(super::EventError::CapacityExceeded);
        }
        queue.push_back(delivery);
        Ok(())
    }
}

struct EventGuard<'a>(&'a Cell<bool>);
impl Drop for EventGuard<'_> {
    fn drop(&mut self) {
        self.0.set(false);
    }
}

struct WaveGuard<'a>(&'a RefCell<Option<Wave>>, Option<Wave>);
impl Drop for WaveGuard<'_> {
    fn drop(&mut self) {
        *self.0.borrow_mut() = self.1.take();
    }
}

pub(super) struct Transaction<'a>(&'a ModelRuntime);
impl Drop for Transaction<'_> {
    fn drop(&mut self) {
        self.0.0.depth.set(self.0.0.depth.get() - 1);
        if self.0.0.depth.get() == 0 {
            self.0.0.transaction_wave.borrow_mut().take();
        }
        if !std::thread::panicking() {
            self.0.dispatch_batch(false);
        }
    }
}

struct DispatchGuard<'a>(&'a ModelRuntime);
impl Drop for DispatchGuard<'_> {
    fn drop(&mut self) {
        self.0.0.dispatching.set(false);
        if !std::thread::panicking() {
            self.0.wake_if_pending();
        }
    }
}
