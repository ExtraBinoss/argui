use super::{Element, Subscription, signal::ModelSignal};
use std::cell::{Cell, RefCell};

pub(super) struct RenderCache {
    signal: ModelSignal,
    local: ModelSignal,
    rendered_local_revision: Cell<u64>,
    rendered_revision: Cell<u64>,
    element: RefCell<Option<Element>>,
    dependencies: RefCell<Vec<Subscription>>,
}
impl RenderCache {
    pub(super) fn clear(&self) {
        let element = self.element.borrow_mut().take();
        let dependencies = self.dependencies.take();
        drop(dependencies);
        drop(element);
    }
    pub(super) fn new(signal: ModelSignal) -> Self {
        Self {
            local: signal.local(),
            rendered_local_revision: Cell::new(0),
            signal,
            rendered_revision: Cell::new(0),
            element: RefCell::new(None),
            dependencies: RefCell::new(Vec::new()),
        }
    }
    pub(super) fn observer(&self) -> super::Observer {
        self.local.observer()
    }
    pub(super) fn apply_update(&self, update: super::ViewUpdate) {
        self.local.apply_update(update);
    }
    pub(super) fn subscribe(&self, observer: super::Observer) -> Subscription {
        self.local.subscribe(observer)
    }
    pub(super) fn replace_dependencies(&self, dependencies: Vec<Subscription>) {
        *self.dependencies.borrow_mut() = dependencies;
    }
    pub(super) fn reusable(&self, environment_changed: bool) -> Option<Element> {
        if self.rendered_revision.get() == self.signal.revision()
            && self.rendered_local_revision.get() == self.local.revision()
            && !environment_changed
        {
            self.element.borrow().clone()
        } else {
            None
        }
    }
    pub(super) fn store(&self, element: &Element) {
        *self.element.borrow_mut() = Some(element.clone());
        self.rendered_revision.set(self.signal.revision());
        self.rendered_local_revision.set(self.local.revision());
    }
    pub(super) fn needs_rebuild(&self) -> bool {
        (self.rendered_revision.get() != self.signal.revision()
            || self.rendered_local_revision.get() != self.local.revision())
            && self.element.borrow().is_some()
    }
}
