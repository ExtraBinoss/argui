use super::{
    AnyEntity, ContextEffects, HandlerRegistry, RenderCache, WindowEnvironment, signal::ModelSignal,
};
use std::cell::{Cell, RefCell};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct PresentationId(super::EntityId);

impl PresentationId {
    pub(super) fn get(self) -> usize {
        self.0.get()
    }
}

/// Retained rendering state, kept separate from the model value and its resources.
pub(super) struct Presentation<T> {
    pub(super) lifecycle: super::lifecycle::MountLifecycle,
    pub(super) visible: Cell<bool>,
    pub(super) host_visible: Cell<bool>,
    pub(super) resources: super::ResourceScope,
    pub(super) model_lease: RefCell<Option<super::ResourceLease>>,
    pub(super) parent_lease: RefCell<Option<super::ResourceLease>>,
    pub(super) cleanup_lease: RefCell<Option<super::ResourceLease>>,
    pub(super) id: PresentationId,
    pub(super) cache: RenderCache,
    pub(super) pending: RefCell<ContextEffects>,
    pub(super) children: RefCell<Vec<AnyEntity>>,
    pub(super) event_routes: RefCell<Vec<AnyEntity>>,
    pub(super) environment: RefCell<WindowEnvironment>,
    pub(super) environment_used: Cell<bool>,
    pub(super) handlers: RefCell<HandlerRegistry<T>>,
}

impl<T> Presentation<T> {
    pub(super) fn new(signal: ModelSignal) -> Self {
        let id = PresentationId(super::EntityId::next());
        Self {
            lifecycle: super::lifecycle::MountLifecycle::new(
                signal.runtime(),
                signal.id(),
                super::MountId(id.get()),
            ),
            visible: Cell::new(true),
            host_visible: Cell::new(true),
            resources: super::ResourceScope::default(),
            model_lease: RefCell::default(),
            parent_lease: RefCell::default(),
            cleanup_lease: RefCell::default(),
            id,
            cache: RenderCache::new(signal),
            pending: RefCell::default(),
            children: RefCell::default(),
            event_routes: RefCell::default(),
            environment: RefCell::default(),
            environment_used: Cell::new(false),
            handlers: RefCell::new(HandlerRegistry::default()),
        }
    }

    pub(super) fn clear(&self) {
        self.lifecycle.end();
        self.cache.clear();
        let handlers = self.handlers.replace(HandlerRegistry::default());
        let children = self.children.take();
        let routes = self.event_routes.take();
        let pending = self.pending.take();
        let model_lease = self.model_lease.take();
        let parent_lease = self.parent_lease.take();
        drop((
            handlers,
            children,
            routes,
            pending,
            model_lease,
            parent_lease,
        ));
    }
}

impl<T> Drop for Presentation<T> {
    fn drop(&mut self) {
        let result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.resources.close()));
        self.lifecycle.end();
        if let Err(payload) = result {
            std::panic::resume_unwind(payload);
        }
    }
}
