use super::{Context, Entity, Subscription, subscription};
use std::{
    any::{Any, TypeId},
    cell::{Cell, RefCell},
    collections::BTreeMap,
    rc::{Rc, Weak},
};

/// Explicitly declares a model's public event vocabulary.
pub trait EventEmitter<E: 'static>: 'static {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Failure to connect or deliver a typed model event.
pub enum EventError {
    /// An entity or presentation resource scope is closed.
    ScopeClosed,
    /// The operation was requested without an entity-backed context.
    DetachedContext,
    /// The source and receiver belong to different model runtimes.
    DifferentRuntime,
    /// The runtime's bounded event queue cannot accept more work.
    CapacityExceeded,
}
impl std::fmt::Display for EventError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::ScopeClosed => "event owner resource scope is closed",
            Self::DetachedContext => "events require an entity context",
            Self::DifferentRuntime => "event endpoints must share a model runtime",
            Self::CapacityExceeded => "model event queue is full",
        })
    }
}
impl std::error::Error for EventError {}

type Callback = Rc<dyn Fn(&dyn Any)>;

impl<T: 'static> super::Mount<T> {
    /// Retain the returned handle for as long as this presentation should receive
    /// events. Closing the mount also detaches it, including queued deliveries.
    ///
    /// `source` is the entity that emits `E`; `callback` handles each delivered event.
    ///
    /// # Errors
    /// Returns [`EventError::ScopeClosed`] if this mount's resource scope has closed,
    /// or the corresponding context error if the entities cannot be connected.
    pub fn subscribe<S, E>(
        &self,
        source: &Entity<S>,
        callback: impl Fn(&mut T, &E, &mut Context<T>) + 'static,
    ) -> Result<Subscription, EventError>
    where
        S: EventEmitter<E>,
        E: 'static,
    {
        if self.resources().is_closed() {
            return Err(EventError::ScopeClosed);
        }
        self.entity
            .update_view(|_, cx| cx.subscribe(source, callback))
    }
}

#[derive(Clone)]
struct Listener {
    kind: TypeId,
    state: Weak<subscription::State>,
    callback: Callback,
}

#[derive(Default)]
pub(super) struct EventRegistry {
    next: Cell<u64>,
    listeners: RefCell<BTreeMap<u64, Listener>>,
}

impl EventRegistry {
    fn insert(self: &Rc<Self>, kind: TypeId, callback: Callback) -> Subscription {
        let id = self.next.get();
        self.next.set(
            id.checked_add(1)
                .expect("subscription identity space exhausted"),
        );
        let weak = Rc::downgrade(self);
        let subscription = Subscription::new(move || {
            if let Some(registry) = weak.upgrade() {
                registry.listeners.borrow_mut().remove(&id);
            }
        });
        self.listeners.borrow_mut().insert(
            id,
            Listener {
                kind,
                callback,
                state: subscription.weak(),
            },
        );
        subscription
    }
}

impl<T: 'static> Context<T> {
    /// Invalidates only this presentation when the source calls `notify`.
    /// Retain the returned handle; unmounting also detaches the observation.
    /// Use `read` for dependencies that follow the latest render's reads.
    ///
    /// `source` is the entity whose notifications should invalidate this presentation.
    ///
    /// # Errors
    /// Returns an error when the context is detached, the entities belong to different
    /// runtimes, or either owning scope has closed.
    pub fn observe<S: 'static>(&mut self, source: &Entity<S>) -> Result<Subscription, EventError> {
        self.observe_owned(source, true)
    }

    pub(super) fn observe_owned<S: 'static>(
        &mut self,
        source: &Entity<S>,
        presentation_owned: bool,
    ) -> Result<Subscription, EventError> {
        let target = self
            .entity
            .as_ref()
            .and_then(|entity| entity.upgrade())
            .ok_or(EventError::DetachedContext)?;
        if !target.0.model.runtime.same(&source.0.model.runtime) {
            return Err(EventError::DifferentRuntime);
        }
        let observer = if presentation_owned {
            target.0.presentation.cache.observer()
        } else {
            target.0.model.signal.observer()
        };
        let subscription = source.0.model.signal.subscribe(observer);
        subscription
            .attach(target.resources())
            .map_err(|_| EventError::ScopeClosed)?;
        subscription
            .attach(source.resources())
            .map_err(|_| EventError::ScopeClosed)?;
        if presentation_owned {
            subscription
                .attach(&target.0.presentation.resources)
                .map_err(|_| EventError::ScopeClosed)?;
        }
        Ok(subscription)
    }

    /// Reads a model and tracks this render's dependency. Rebuilding without this
    /// read detaches the dependency; a cached render keeps it alive.
    ///
    /// `source` is the model to read; `read` computes a result from its shared value,
    /// which is returned unchanged.
    ///
    /// # Panics
    /// Panics when called outside an entity render context or when either model's
    /// resource scope has closed, or when the models belong to different runtimes.
    pub fn read<S: 'static, R>(&mut self, source: &Entity<S>, read: impl FnOnce(&S) -> R) -> R {
        let target = self
            .entity
            .as_ref()
            .and_then(|entity| entity.upgrade())
            .expect("tracked reads require an entity context");
        assert!(
            target.0.model.runtime.same(&source.0.model.runtime),
            "tracked reads require models from the same runtime"
        );
        let observer = self
            .owner
            .as_ref()
            .expect("tracked reads require an owner")
            .1
            .clone();
        let subscription = source.0.model.signal.subscribe(observer);
        subscription
            .attach(source.resources())
            .expect("tracked read source is closed");
        subscription
            .attach(target.resources())
            .expect("tracked read target is closed");
        subscription
            .attach(&target.0.presentation.resources)
            .expect("tracked read presentation is closed");
        self.dependencies.push(subscription);
        source.read(read)
    }

    /// Subscribe for this presentation's lifetime, bounded additionally by the
    /// returned handle and both model owners. Queued delivery is cancelled too.
    ///
    /// `source` emits the event type `E`; `callback` handles each event with mutable
    /// access to this model and its context.
    ///
    /// # Errors
    /// Returns an error if the context is detached, the entities use different runtimes,
    /// or either owning scope has closed.
    pub fn subscribe<S, E>(
        &mut self,
        source: &Entity<S>,
        callback: impl Fn(&mut T, &E, &mut Context<T>) + 'static,
    ) -> Result<Subscription, EventError>
    where
        S: EventEmitter<E>,
        E: 'static,
    {
        self.subscribe_owned(source, callback, true)
    }

    pub(super) fn subscribe_owned<S, E>(
        &mut self,
        source: &Entity<S>,
        callback: impl Fn(&mut T, &E, &mut Context<T>) + 'static,
        presentation_owned: bool,
    ) -> Result<Subscription, EventError>
    where
        S: EventEmitter<E>,
        E: 'static,
    {
        let weak = self.entity.clone().ok_or(EventError::DetachedContext)?;
        let target = weak.upgrade().ok_or(EventError::DetachedContext)?;
        if !target.0.model.runtime.same(&source.0.model.runtime) {
            return Err(EventError::DifferentRuntime);
        }
        let subscription = source.0.model.events.insert(
            TypeId::of::<E>(),
            Rc::new(move |event| {
                if let Some(target) = weak.upgrade()
                    && let Some(event) = event.downcast_ref::<E>()
                {
                    if presentation_owned {
                        target.update_view(|value, cx| callback(value, event, cx));
                    } else {
                        target.update_model(|value, cx| callback(value, event, cx));
                    }
                }
            }),
        );
        subscription
            .attach(target.resources())
            .map_err(|_| EventError::ScopeClosed)?;
        subscription
            .attach(source.resources())
            .map_err(|_| EventError::ScopeClosed)?;
        if presentation_owned {
            subscription
                .attach(&target.0.presentation.resources)
                .map_err(|_| EventError::ScopeClosed)?;
        }
        Ok(subscription)
    }

    /// Captures current listeners. Later subscribers do not receive earlier events.
    /// Cancellation before delivery suppresses even an already queued callback.
    ///
    /// `event` is delivered to listeners registered for its concrete type.
    ///
    /// # Errors
    /// Returns an error when this context is detached or its resource scope is closed,
    /// or if event delivery exceeds runtime capacity.
    pub fn emit<E: 'static>(&mut self, event: E) -> Result<(), EventError>
    where
        T: EventEmitter<E>,
    {
        let source = self.entity.clone().ok_or(EventError::DetachedContext)?;
        let owner = source.upgrade().ok_or(EventError::DetachedContext)?;
        if owner.resources().is_closed() {
            return Err(EventError::ScopeClosed);
        }
        let listeners: Vec<_> = owner
            .0
            .model
            .events
            .listeners
            .borrow()
            .values()
            .filter(|listener| listener.kind == TypeId::of::<E>())
            .cloned()
            .collect();
        if listeners.is_empty() {
            return Ok(());
        }
        let mut listeners = listeners.into_iter();
        owner.0.model.runtime.enqueue(Box::new(move || {
            if source
                .upgrade()
                .is_none_or(|owner| owner.resources().is_closed())
            {
                return true;
            }
            let listener = listeners
                .next()
                .expect("a queued listener batch cannot be empty");
            if listener.state.upgrade().is_some_and(|state| state.active()) {
                (listener.callback)(&event);
            }
            listeners.len() == 0
        }))
    }
}
