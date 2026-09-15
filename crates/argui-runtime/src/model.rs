use crate::AppCommand;
use argui_animation::Frame;
use argui_inspect::InspectorHandle;
use argui_paint::{ImageAsset, VectorAsset};
use argui_ui::{Element, EventHandlerId, EventOwnerId, UiEvent};
use std::{
    any::Any,
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

use crate::WindowEnvironment;

pub(crate) mod effects;
mod handler;
use handler::HandlerRegistry;
mod layout;
pub(crate) use effects::PointerCaptureRequest;
pub use effects::ViewUpdate;
use effects::{ContextEffects, merge_effects};
mod context;
pub use context::{Context, Render};
mod editing;
pub use layout::{LayoutBounds, LayoutSnapshot, ScrollRequest};

type HandlerDispatch = dyn Fn(EventHandlerId, &UiEvent) -> ContextEffects;
type Observer = Rc<dyn Fn(&ModelRuntime)>;

mod entity;
mod entity_api;
mod host;
pub use entity::EntityId;
use entity_api::with_event_handler;
use host::ModelRuntimeVisitor;
pub use host::shutdown_presentations;
mod dispatch;
pub use dispatch::ModelRuntime;
mod events;
pub use events::{EventEmitter, EventError};
mod subscription;
pub use subscription::Subscription;
mod scope;
pub use scope::{ResourceLease, ResourceScope, ScopeClosed};
mod cache;
mod services;
mod signal;
use cache::RenderCache;
pub use services::{ServiceAlreadyRegistered, ServiceRegistration};
mod model_context;
mod mount;
pub use model_context::ModelContext;
mod lifecycle;
pub use lifecycle::{MountEvent, MountTransition};
mod presentation;
mod visibility;
pub use mount::{Mount, MountId, WeakMount};
use presentation::Presentation;
#[cfg(feature = "tasks")]
mod tasks;

/// Retained component identity. Cloning an entity never clones its state.
pub struct Entity<T>(Rc<EntityCell<T>>);

struct EntityCell<T> {
    model: Rc<ModelState<T>>,
    presentation: Presentation<T>,
}

struct ModelState<T> {
    commands: RefCell<Vec<AppCommand>>,
    id: EntityId,
    signal: signal::ModelSignal,
    runtime: ModelRuntime,
    events: Rc<events::EventRegistry>,
    resources: ResourceScope,
    value: RefCell<T>,
}

impl<T> Drop for ModelState<T> {
    fn drop(&mut self) {
        self.resources.close();
    }
}

impl<T> Clone for Entity<T> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

/// Non-owning reference to a retained entity and its current presentation.
pub struct WeakEntity<T> {
    model: Weak<ModelState<T>>,
    presentation: Weak<EntityCell<T>>,
}

impl<T> Clone for WeakEntity<T> {
    fn clone(&self) -> Self {
        Self {
            model: self.model.clone(),
            presentation: self.presentation.clone(),
        }
    }
}

#[derive(Clone)]
/// Type-erased retained entity for storage and event routing across model types.
pub struct AnyEntity {
    host_visibility: Rc<dyn Fn(bool)>,
    close_host: Rc<dyn Fn()>,
    model_id: EntityId,
    runtime: ModelRuntime,
    model_runtimes: Rc<ModelRuntimeVisitor>,
    model_effects: Rc<dyn Fn() -> ContextEffects>,
    bind_model_wake: Rc<dyn Fn(&ModelRuntime)>,
    #[cfg(feature = "tasks")]
    task_effects: Rc<dyn Fn() -> ContextEffects>,
    identity: Rc<dyn Any>,
    #[cfg(feature = "tasks")]
    tasks: Rc<dyn Fn(crate::tasks::TaskRuntime)>,
    render: Rc<dyn Fn(WindowEnvironment) -> Element>,
    dispatch_handler: Rc<HandlerDispatch>,
    store: Rc<dyn Fn(ContextEffects)>,
    wants_frame: Rc<dyn Fn() -> bool>,
    frame: Rc<dyn Fn(Frame) -> ContextEffects>,
    layout: Rc<dyn Fn(&LayoutSnapshot) -> ContextEffects>,
    owns: Rc<dyn Fn(EventOwnerId) -> bool>,
    image_assets: Rc<dyn Fn() -> Vec<ImageAsset>>,
    vector_assets: Rc<dyn Fn() -> Vec<VectorAsset>>,
    inspector: Rc<dyn Fn() -> Option<InspectorHandle>>,
    take_effects: Rc<dyn Fn() -> ContextEffects>,
}

fn inherit_environment_use(parent: &Cell<bool>, child: &Cell<bool>) {
    if child.get() {
        parent.set(true);
    }
}

fn update_environment(
    current: &RefCell<WindowEnvironment>,
    used: &Cell<bool>,
    next: &WindowEnvironment,
) -> bool {
    current.replace(next.clone()) != *next && used.get()
}

impl<T: Render> Entity<T> {
    /// Erases the concrete model type while retaining its runtime operations.
    /// Returns a cloneable handle for type-independent storage and routing.
    #[must_use]
    pub fn erase(&self) -> AnyEntity {
        #[cfg(feature = "tasks")]
        let tasks = self.clone();
        #[cfg(feature = "tasks")]
        let task_effects = self.clone();
        let host_visibility = self.clone();
        let close_host = self.clone();
        let render = self.clone();
        let handler = self.clone();
        let store = self.clone();
        let wants_frame = self.clone();
        let frame = self.clone();
        let layout = self.clone();
        let owns = self.clone();
        let image_assets = self.clone();
        let vector_assets = self.clone();
        let inspector = self.clone();
        let take_effects = self.clone();
        let model_runtimes = self.clone();
        let model_effects = self.clone();
        let bind_model_wake = self.clone();
        AnyEntity {
            host_visibility: Rc::new(move |visible| host_visibility.host_visibility(visible)),
            close_host: Rc::new(move || close_host.close_host()),
            model_id: self.id(),
            runtime: self.0.model.runtime.clone(),
            model_runtimes: Rc::new(move |visited| model_runtimes.collect_model_runtimes(visited)),
            model_effects: Rc::new(move || model_effects.collect_model_effects()),
            bind_model_wake: Rc::new(move |runtime| bind_model_wake.bind_model_wake(runtime)),
            #[cfg(feature = "tasks")]
            task_effects: Rc::new(move || task_effects.take_task_effects()),
            identity: self.0.clone(),
            #[cfg(feature = "tasks")]
            tasks: Rc::new(move |runtime| tasks.set_task_runtime(runtime)),
            render: Rc::new(move |environment| render.render_in(environment)),
            dispatch_handler: Rc::new(move |id, event| handler.dispatch_handler(id, event)),
            store: Rc::new(move |effects| store.store_effects(effects)),
            wants_frame: Rc::new(move || wants_frame.wants_frame()),
            frame: Rc::new(move |value| frame.dispatch_frame(value)),
            layout: Rc::new(move |snapshot| layout.dispatch_layout(snapshot)),
            owns: Rc::new(move |owner| owns.owns(owner)),
            image_assets: Rc::new(move || {
                image_assets.read(|value| {
                    #[cfg(all(
                        feature = "hot-reload",
                        debug_assertions,
                        not(target_arch = "wasm32")
                    ))]
                    return crate::hot_reload::image_assets(value);

                    #[cfg(not(all(
                        feature = "hot-reload",
                        debug_assertions,
                        not(target_arch = "wasm32")
                    )))]
                    value.image_assets()
                })
            }),
            vector_assets: Rc::new(move || {
                vector_assets.read(|value| {
                    #[cfg(all(
                        feature = "hot-reload",
                        debug_assertions,
                        not(target_arch = "wasm32")
                    ))]
                    return crate::hot_reload::vector_assets(value);

                    #[cfg(not(all(
                        feature = "hot-reload",
                        debug_assertions,
                        not(target_arch = "wasm32")
                    )))]
                    value.vector_assets()
                })
            }),
            inspector: Rc::new(move || {
                inspector.read(|value| {
                    #[cfg(all(
                        feature = "hot-reload",
                        debug_assertions,
                        not(target_arch = "wasm32")
                    ))]
                    return crate::hot_reload::inspector(value);

                    #[cfg(not(all(
                        feature = "hot-reload",
                        debug_assertions,
                        not(target_arch = "wasm32")
                    )))]
                    value.inspector()
                })
            }),
            take_effects: Rc::new(move || take_effects.take_effects()),
        }
    }

    #[must_use]
    /// Renders this entity using its presentation's most recent environment.
    /// Returns the retained UI subtree, or an empty container when hidden.
    pub fn render(&self) -> Element {
        let environment = self.0.presentation.environment.borrow().clone();
        self.render_in(environment)
    }

    #[must_use]
    /// Renders this entity using `environment` for platform and theme state.
    /// Returns the retained UI subtree, or an empty container when hidden.
    pub fn render_in(&self, environment: WindowEnvironment) -> Element {
        let _transaction = self.0.model.runtime.enter();
        if !self.0.presentation.is_visible() {
            return Element::container([]);
        }
        self.0.presentation.lifecycle.start();
        let environment_changed = update_environment(
            &self.0.presentation.environment,
            &self.0.presentation.environment_used,
            &environment,
        );
        if let Some(element) = self.0.presentation.cache.reusable(environment_changed) {
            return element;
        }
        let observer = self.0.presentation.cache.observer();
        let mut cx = Context {
            entity: Some(self.downgrade()),
            owner: Some((self.0.presentation.id, observer)),
            environment,
            ..Context::default()
        };
        let element = {
            let mut value = self.0.model.value.borrow_mut();
            #[cfg(all(feature = "hot-reload", debug_assertions, not(target_arch = "wasm32")))]
            let element = crate::hot_reload::render(&mut *value, &mut cx);
            #[cfg(not(all(
                feature = "hot-reload",
                debug_assertions,
                not(target_arch = "wasm32")
            )))]
            let element = value.render(&mut cx);
            element.semantic_scope()
        };
        if self.0.presentation.resources.is_closed() {
            return Element::container([]);
        }
        self.0
            .presentation
            .handlers
            .borrow_mut()
            .replace(std::mem::take(&mut cx.handlers));
        let previous_children = self
            .0
            .presentation
            .children
            .replace(std::mem::take(&mut cx.effects.children));
        let previous_routes = self
            .0
            .presentation
            .event_routes
            .replace(std::mem::take(&mut cx.effects.event_routes));
        drop(previous_children);
        drop(previous_routes);
        self.0
            .presentation
            .environment_used
            .set(cx.environment_read.get());
        self.0.presentation.cache.store(&element);
        self.0
            .presentation
            .cache
            .replace_dependencies(std::mem::take(&mut cx.dependencies));
        self.finish(cx);
        element
    }

    /// Dispatches one listener delivery produced by [`argui_ui::UiTree`].
    ///
    /// `event` is the UI event whose current listener should run.
    pub fn dispatch_event(&self, event: &UiEvent) {
        with_event_handler(event, &mut |handler| {
            let effects = self.dispatch_handler(handler, event);
            self.store_effects(effects);
        });
    }

    pub(crate) fn animation_frame(&self, frame: Frame) {
        let effects = self.dispatch_frame(frame);
        self.store_effects(effects);
    }

    pub(crate) fn layout_changed(&self, layout: &LayoutSnapshot) {
        let effects = self.dispatch_layout(layout);
        self.store_effects(effects);
    }

    pub(crate) fn wants_frame(&self) -> bool {
        self.0.presentation.is_visible()
            && ({
                let value = self.0.model.value.borrow();
                #[cfg(all(feature = "hot-reload", debug_assertions, not(target_arch = "wasm32")))]
                let wants_frame = crate::hot_reload::wants_animation_frame(&*value);
                #[cfg(not(all(
                    feature = "hot-reload",
                    debug_assertions,
                    not(target_arch = "wasm32")
                )))]
                let wants_frame = value.wants_animation_frame();
                wants_frame
            } || self
                .0
                .presentation
                .children
                .borrow()
                .iter()
                .any(|child| (child.wants_frame)()))
    }

    fn dispatch_frame(&self, frame: Frame) -> ContextEffects {
        if !self.0.presentation.is_visible() {
            return ContextEffects::default();
        }
        let _transaction = self.0.model.runtime.enter();
        let mut effects = ContextEffects::default();
        let children = self.0.presentation.children.borrow().clone();
        for child in children {
            if (child.wants_frame)() {
                merge_effects(&mut effects, (child.frame)(frame));
            }
        }
        if !self.0.presentation.is_visible() {
            return self.0.presentation.visible_effects(effects);
        }
        let mut cx = Context {
            entity: Some(self.downgrade()),
            environment: self.0.presentation.environment.borrow().clone(),
            ..Context::default()
        };
        let mut value = self.0.model.value.borrow_mut();
        #[cfg(all(feature = "hot-reload", debug_assertions, not(target_arch = "wasm32")))]
        crate::hot_reload::animation_frame(&mut *value, frame, &mut cx);
        #[cfg(not(all(feature = "hot-reload", debug_assertions, not(target_arch = "wasm32"))))]
        value.animation_frame(frame, &mut cx);
        drop(value);
        merge_effects(&mut effects, cx.effects);
        self.0.model.signal.apply_update(effects.update);
        effects
    }

    fn dispatch_layout(&self, layout: &LayoutSnapshot) -> ContextEffects {
        if !self.0.presentation.is_visible() {
            return ContextEffects::default();
        }
        let _transaction = self.0.model.runtime.enter();
        let mut cx = Context {
            entity: Some(self.downgrade()),
            environment: self.0.presentation.environment.borrow().clone(),
            ..Context::default()
        };
        let mut value = self.0.model.value.borrow_mut();
        #[cfg(all(feature = "hot-reload", debug_assertions, not(target_arch = "wasm32")))]
        crate::hot_reload::layout_changed(&mut *value, layout, &mut cx);
        #[cfg(not(all(feature = "hot-reload", debug_assertions, not(target_arch = "wasm32"))))]
        value.layout_changed(layout, &mut cx);
        drop(value);
        self.0.model.signal.apply_update(cx.effects.update);
        cx.effects
    }
}
