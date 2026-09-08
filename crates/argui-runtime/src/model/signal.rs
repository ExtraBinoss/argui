use super::{EntityId, ModelRuntime, Observer, Subscription, ViewUpdate};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    rc::Rc,
};

struct Invalidation {
    id: EntityId,
    revision: Cell<u64>,
    next: Cell<u64>,
    observers: RefCell<BTreeMap<u64, Observer>>,
}
impl Invalidation {
    fn invalidate(self: &Rc<Self>, runtime: &ModelRuntime, propagated: bool) {
        self.revision.set(
            self.revision
                .get()
                .checked_add(1)
                .expect("model revision space exhausted"),
        );
        if self.observers.borrow().is_empty() {
            return;
        }
        let cutoff = self.next.get();
        let weak = Rc::downgrade(self);
        let mut next = 0;
        runtime.invalidate(
            self.id,
            Box::new(move |runtime| {
                let Some(source) = weak.upgrade() else {
                    return true;
                };
                let observer = source
                    .observers
                    .borrow()
                    .range(next..cutoff)
                    .next()
                    .map(|(&id, callback)| (id, callback.clone()));
                let Some((id, observer)) = observer else {
                    return true;
                };
                next = id + 1;
                observer(runtime);
                source
                    .observers
                    .borrow()
                    .range(next..cutoff)
                    .next()
                    .is_none()
            }),
            propagated,
        );
    }
}

#[derive(Clone)]
pub(super) struct ModelSignal {
    runtime: ModelRuntime,
    invalidation: Rc<Invalidation>,
}
impl ModelSignal {
    pub(super) fn runtime(&self) -> ModelRuntime {
        self.runtime.clone()
    }
    pub(super) fn id(&self) -> EntityId {
        self.invalidation.id
    }
    pub(super) fn local(&self) -> Self {
        Self::new(EntityId::next(), self.runtime.clone())
    }
    pub(super) fn new(id: EntityId, runtime: ModelRuntime) -> Self {
        Self {
            runtime,
            invalidation: Rc::new(Invalidation {
                id,
                revision: Cell::new(0),
                next: Cell::new(0),
                observers: RefCell::new(BTreeMap::new()),
            }),
        }
    }
    pub(super) fn revision(&self) -> u64 {
        self.invalidation.revision.get()
    }
    pub(super) fn observer(&self) -> Observer {
        let weak = Rc::downgrade(&self.invalidation);
        Rc::new(move |runtime| {
            if let Some(invalidation) = weak.upgrade() {
                invalidation.invalidate(runtime, true);
            }
        })
    }
    pub(super) fn subscribe(&self, observer: Observer) -> Subscription {
        let id = self.invalidation.next.get();
        self.invalidation.next.set(
            id.checked_add(1)
                .expect("invalidation identity space exhausted"),
        );
        self.invalidation
            .observers
            .borrow_mut()
            .insert(id, observer);
        let weak = Rc::downgrade(&self.invalidation);
        Subscription::new(move || {
            if let Some(invalidation) = weak.upgrade() {
                invalidation.observers.borrow_mut().remove(&id);
            }
        })
    }

    pub(super) fn apply_update(&self, update: ViewUpdate) {
        if update == ViewUpdate::Rebuild {
            self.invalidation.invalidate(&self.runtime, false);
        }
    }
}
