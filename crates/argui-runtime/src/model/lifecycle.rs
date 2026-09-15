use super::{EntityId, ModelRuntime, MountId, Subscription};
use std::{
    cell::{Cell, RefCell},
    collections::{BTreeMap, VecDeque},
    rc::Rc,
};

/// Presentation transitions, delivered after the enclosing model transaction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MountTransition {
    /// A presentation became mounted.
    Mounted,
    /// A mounted presentation changed visibility.
    VisibilityChanged {
        /// Whether the presentation is visible after the transition.
        visible: bool,
    },
    /// A mounted presentation was closed.
    Unmounted,
}

/// A historical transition. IDs do not retain or recreate the model or mount.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MountEvent {
    /// Identity of the shared model.
    pub model: EntityId,
    /// Identity of the presentation that changed.
    pub mount: MountId,
    /// Transition that occurred.
    pub transition: MountTransition,
}

type Listener = Rc<dyn Fn(MountEvent)>;
struct Delivery {
    event: MountEvent,
    next: u64,
    cutoff: u64,
}
#[derive(Default)]
struct Inner {
    listeners: RefCell<BTreeMap<u64, Listener>>,
    next: Cell<u64>,
    queue: RefCell<VecDeque<Delivery>>,
}
#[derive(Clone, Default)]
pub(super) struct LifecycleRegistry(Rc<Inner>);
impl LifecycleRegistry {
    pub(super) fn finish(&self) {
        let queue = self.0.queue.take();
        let mut panic = None;
        for delivery in queue {
            let mut next = delivery.next;
            loop {
                let listener = self
                    .0
                    .listeners
                    .borrow()
                    .range(next..delivery.cutoff)
                    .next()
                    .map(|(&id, callback)| (id, callback.clone()));
                let Some((id, callback)) = listener else {
                    break;
                };
                next = id + 1;
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    callback(delivery.event)
                }));
                if panic.is_none() {
                    panic = result.err();
                }
            }
        }
        self.0.queue.borrow_mut().clear();
        if let Some(payload) = panic
            && !std::thread::panicking()
        {
            std::panic::resume_unwind(payload);
        }
    }

    pub(super) fn subscribe(&self, callback: impl Fn(MountEvent) + 'static) -> Subscription {
        let id = self.0.next.get();
        self.0.next.set(
            id.checked_add(1)
                .expect("lifecycle listener identity exhausted"),
        );
        self.0.listeners.borrow_mut().insert(id, Rc::new(callback));
        let weak = Rc::downgrade(&self.0);
        Subscription::new(move || {
            if let Some(inner) = weak.upgrade() {
                inner.listeners.borrow_mut().remove(&id);
            }
        })
    }
    pub(super) fn enqueue(&self, event: MountEvent) {
        if !self.0.listeners.borrow().is_empty() {
            self.0.queue.borrow_mut().push_back(Delivery {
                event,
                next: 0,
                cutoff: self.0.next.get(),
            });
        }
    }
    pub(super) fn pending(&self) -> usize {
        self.0.queue.borrow().len()
    }
    pub(super) fn dispatch(&self) -> bool {
        let Some(mut delivery) = self.0.queue.borrow_mut().pop_front() else {
            return false;
        };
        let listener = self
            .0
            .listeners
            .borrow()
            .range(delivery.next..delivery.cutoff)
            .next()
            .map(|(&id, callback)| (id, callback.clone()));
        if let Some((id, callback)) = listener {
            let event = delivery.event;
            delivery.next = id + 1;
            // Advance before invoking user code, preserving remaining listeners if it panics.
            if self
                .0
                .listeners
                .borrow()
                .range(delivery.next..delivery.cutoff)
                .next()
                .is_some()
            {
                self.0.queue.borrow_mut().push_front(delivery);
            }
            callback(event);
        }
        true
    }
}

impl ModelRuntime {
    /// Observe future transitions in this domain. Dropping/cancelling the returned
    /// subscription also cancels queued delivery. Use weak model handles in callbacks
    /// when the subscription owner itself is retained by that model.
    ///
    /// `callback` receives each transition queued after registration.
    pub fn observe_mounts(&self, callback: impl Fn(MountEvent) + 'static) -> Subscription {
        self.lifecycle().subscribe(callback)
    }
    /// Returns the number of lifecycle event deliveries waiting in this runtime.
    #[must_use]
    pub fn pending_mount_events(&self) -> usize {
        self.lifecycle().pending()
    }
}

pub(super) struct MountLifecycle {
    runtime: ModelRuntime,
    model: EntityId,
    mount: MountId,
    started: Cell<bool>,
    ended: Cell<bool>,
}
impl MountLifecycle {
    pub(super) fn new(runtime: ModelRuntime, model: EntityId, mount: MountId) -> Self {
        Self {
            runtime,
            model,
            mount,
            started: Cell::new(false),
            ended: Cell::new(false),
        }
    }
    fn emit(&self, transition: MountTransition) {
        self.runtime.enqueue_mount_event(MountEvent {
            model: self.model,
            mount: self.mount,
            transition,
        });
    }
    pub(super) fn start(&self) {
        if !self.started.replace(true) {
            self.emit(MountTransition::Mounted);
        }
    }
    pub(super) fn visibility(&self, visible: bool) {
        if self.started.get() && !self.ended.get() {
            self.emit(MountTransition::VisibilityChanged { visible });
        }
    }
    pub(super) fn end(&self) {
        if self.started.get() && !self.ended.replace(true) {
            self.emit(MountTransition::Unmounted);
        }
    }
}
