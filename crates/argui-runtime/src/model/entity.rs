use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Process-unique identity, never reused after an entity is destroyed.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EntityId(usize);

impl EntityId {
    pub(super) fn next() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(1);
        Self(
            NEXT.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
                .expect("entity identity space exhausted"),
        )
    }

    #[must_use]
    /// Returns the numeric value of this process-unique identity.
    pub const fn get(self) -> usize {
        self.0
    }
}

impl<T: 'static> Entity<T> {
    #[must_use]
    /// Returns this entity's stable identity.
    pub fn id(&self) -> EntityId {
        self.0.model.id
    }

    /// Invalidation revision, independent of rendering. Advances on `notify`
    /// and dependency invalidation, not on every write or typed event.
    #[must_use]
    pub fn revision(&self) -> u64 {
        self.0.model.signal.revision()
    }

    #[must_use]
    /// Creates a new entity with its own model runtime.
    ///
    /// `value` is the initial model state.
    ///
    /// # Panics
    /// Panics if the process exhausts the available entity identity values.
    pub fn new(value: T) -> Self {
        ModelRuntime::default().entity(value)
    }

    #[must_use]
    /// Returns the runtime shared by this entity and related entities.
    pub fn runtime(&self) -> ModelRuntime {
        self.0.model.runtime.clone()
    }

    #[must_use]
    /// Returns the scope that owns resources associated with this model.
    pub fn resources(&self) -> &ResourceScope {
        &self.0.model.resources
    }

    pub(super) fn in_runtime(value: T, runtime: ModelRuntime) -> Self {
        let id = EntityId::next();
        let signal = signal::ModelSignal::new(id, runtime.clone());
        Self(Rc::new(EntityCell {
            presentation: Presentation::new(signal.clone()),
            model: Rc::new(ModelState {
                commands: RefCell::default(),
                id,
                signal,
                runtime,
                events: Rc::new(events::EventRegistry::default()),
                resources: ResourceScope::default(),
                value: RefCell::new(value),
            }),
        }))
    }

    #[must_use]
    /// Creates a non-owning reference that can later be upgraded.
    pub fn downgrade(&self) -> WeakEntity<T> {
        WeakEntity {
            model: Rc::downgrade(&self.0.model),
            presentation: Rc::downgrade(&self.0),
        }
    }

    /// Mutates the model inside a runtime transaction.
    ///
    /// `update` receives mutable model state and its restricted model context; its
    /// return value is returned unchanged.
    ///
    /// # Panics
    /// Propagates a panic from `update`; reentrant access that conflicts with the
    /// active mutable model borrow also panics.
    pub fn update<R>(&self, update: impl FnOnce(&mut T, &mut ModelContext<'_, T>) -> R) -> R {
        self.update_model(|value, cx| update(value, &mut ModelContext::new(cx)))
    }

    pub(super) fn update_model<R>(&self, update: impl FnOnce(&mut T, &mut Context<T>) -> R) -> R {
        let _transaction = self.0.model.runtime.enter();
        let mut cx = Context {
            entity: Some(self.downgrade()),
            ..Context::default()
        };
        let result = update(&mut self.0.model.value.borrow_mut(), &mut cx);
        if cx.effects.update != ViewUpdate::None || !cx.effects.commands.is_empty() {
            self.0.model.runtime.request_wake();
        }
        self.0
            .model
            .commands
            .borrow_mut()
            .append(&mut cx.effects.commands);
        self.0.model.signal.apply_update(cx.effects.update);
        result
    }

    pub(crate) fn update_view<R>(&self, update: impl FnOnce(&mut T, &mut Context<T>) -> R) -> R {
        let _transaction = self.0.model.runtime.enter();
        let observer = self.0.model.signal.observer();
        let mut cx = Context {
            entity: Some(self.downgrade()),
            owner: Some((self.0.presentation.id, observer)),
            environment: self.0.presentation.environment.borrow().clone(),
            ..Context::default()
        };
        let result = update(&mut self.0.model.value.borrow_mut(), &mut cx);
        self.finish(cx);
        result
    }

    pub(super) fn finish(&self, cx: Context<T>) {
        self.store_effects(cx.effects);
    }

    pub(crate) fn store_effects(&self, mut effects: ContextEffects) {
        self.0
            .model
            .commands
            .borrow_mut()
            .append(&mut effects.commands);
        self.0.model.signal.apply_update(effects.update);
        if self.0.presentation.resources.is_closed() {
            return;
        }
        let mut pending = self.0.presentation.pending.borrow_mut();
        merge_effects(&mut pending, effects);
    }

    pub(super) fn owner_id(&self) -> EventOwnerId {
        EventOwnerId(self.0.presentation.id.get())
    }

    pub(super) fn owns(&self, owner: EventOwnerId) -> bool {
        owner == self.owner_id()
            || self
                .0
                .presentation
                .children
                .borrow()
                .iter()
                .any(|child| (child.owns)(owner))
            || self
                .0
                .presentation
                .event_routes
                .borrow()
                .iter()
                .any(|route| (route.owns)(owner))
    }

    pub(crate) fn take_effects(&self) -> ContextEffects {
        let mut effects = std::mem::take(&mut *self.0.presentation.pending.borrow_mut());
        effects
            .commands
            .append(&mut self.0.model.commands.borrow_mut());
        if self.0.presentation.cache.needs_rebuild() {
            effects.update = ViewUpdate::Rebuild;
        }
        effects
    }

    /// Reads the current model value without changing it.
    ///
    /// `read` receives a shared reference to the model and computes the result.
    ///
    /// # Panics
    /// Propagates a panic from `read` or a conflicting reentrant mutable borrow.
    pub fn read<R>(&self, read: impl FnOnce(&T) -> R) -> R {
        let _transaction = self.0.model.runtime.enter();
        read(&self.0.model.value.borrow())
    }
}

impl super::AnyEntity {
    pub(crate) fn invalidate(&self) {
        // AppModel can change outside a component callback (for example a global shortcut).
        // Its explicit rebuild must invalidate the retained root, not just schedule a frame.
        (self.store)(ContextEffects {
            update: ViewUpdate::Rebuild,
            ..Default::default()
        });
    }
}
