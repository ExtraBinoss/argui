use crate::AppCommand;
use argui_animation::Frame;
use argui_core::{PointerId, Rect};
use argui_inspect::InspectorHandle;
use argui_paint::{ImageAsset, VectorAsset};
use argui_ui::{
    ClipboardRequest, Element, EventHandlerId, EventOwnerId, FocusRequest, FocusTarget,
    TextSelection, TextSelectionRequest, UiEvent,
};
use std::{
    any::Any,
    cell::{Cell, RefCell},
    marker::PhantomData,
    rc::{Rc, Weak},
};

use crate::{ThemeRequest, WindowEnvironment};

pub(crate) mod effects;
mod handler;
mod layout;
pub(crate) use effects::PointerCaptureRequest;
pub use effects::ViewUpdate;
use effects::{ContextEffects, merge_effects, strongest_update};
mod editing;
use handler::{HandlerRegistry, LocalHandler};
pub use layout::{LayoutBounds, LayoutSnapshot, ScrollRequest};

type HandlerDispatch = dyn Fn(EventHandlerId, &UiEvent) -> ContextEffects;
type Observer = Rc<dyn Fn(&ModelRuntime)>;

mod entity;
mod host;
pub use entity::EntityId;
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
use presentation::{Presentation, PresentationId};
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

/// Mutation and scheduling access scoped to one retained component.
pub struct Context<T> {
    entity: Option<WeakEntity<T>>,
    pub(crate) effects: ContextEffects,
    owner: Option<(PresentationId, Observer)>,
    environment: WindowEnvironment,
    environment_read: Cell<bool>,
    event_target: Option<argui_ui::NodeId>,
    handlers: Vec<LocalHandler<T>>,
    dependencies: Vec<Subscription>,
    _marker: PhantomData<fn() -> T>,
}

impl<T> Default for Context<T> {
    fn default() -> Self {
        Self {
            entity: None,
            effects: ContextEffects::default(),
            owner: None,
            environment: WindowEnvironment::default(),
            environment_read: Cell::new(false),
            event_target: None,
            handlers: Vec::new(),
            dependencies: Vec::new(),
            _marker: PhantomData,
        }
    }
}

impl<T: 'static> Context<T> {
    #[must_use]
    pub fn new_entity<U: 'static>(&mut self, value: U) -> Entity<U> {
        match self.entity.as_ref().and_then(WeakEntity::upgrade) {
            Some(owner) => owner.0.model.runtime.entity(value),
            None => Entity::new(value),
        }
    }

    /// Marks this entity dirty. Its cached subtree is rebuilt once at the next frame.
    pub fn notify(&mut self) {
        self.effects.update = ViewUpdate::Rebuild;
    }

    pub fn command(&mut self, command: AppCommand) {
        self.effects.commands.push(command);
    }

    #[must_use]
    pub const fn view_update(&self) -> ViewUpdate {
        self.effects.update
    }

    pub(crate) fn merge_effects(&mut self, child: ContextEffects) {
        merge_effects(&mut self.effects, child);
    }
}

impl<T: Render> Context<T> {
    #[must_use]
    pub fn environment(&self) -> WindowEnvironment {
        self.environment_read.set(true);
        self.environment.clone()
    }

    pub fn request_paint(&mut self) {
        self.effects.update = strongest_update(self.effects.update, ViewUpdate::Paint);
    }

    pub fn request_animation_frame(&mut self) {
        self.effects.animation_frame = true;
        self.request_paint();
    }

    pub fn capture_pointer(&mut self, pointer: PointerId) -> bool {
        let Some(target) = self.event_target else {
            return false;
        };
        self.effects
            .pointer_capture
            .push(PointerCaptureRequest::Capture { pointer, target });
        true
    }

    pub fn release_pointer(&mut self, pointer: PointerId) -> bool {
        let Some(target) = self.event_target else {
            return false;
        };
        self.effects
            .pointer_capture
            .push(PointerCaptureRequest::Release { pointer, target });
        true
    }

    pub fn write_clipboard(&mut self, request: ClipboardRequest) {
        self.effects.clipboard = Some(request);
    }

    pub fn scroll(&mut self, request: ScrollRequest) {
        self.effects.scroll = Some(request);
        self.request_paint();
    }

    pub fn request_focus(&mut self, target: impl Into<FocusTarget>) {
        self.effects.focus = Some(FocusRequest::Focus(target.into()));
        self.request_paint();
    }

    pub fn clear_focus(&mut self) {
        self.effects.focus = Some(FocusRequest::Clear);
        self.request_paint();
    }

    pub fn select_text(&mut self, target: impl Into<FocusTarget>, selection: TextSelection) {
        self.effects.text_selection = Some(TextSelectionRequest::new(target, selection));
        self.request_paint();
    }

    pub fn set_theme(&mut self, request: ThemeRequest) {
        self.effects.theme = Some(request);
        self.notify();
    }

    #[must_use]
    pub fn observe_bounds(&self, layout: &LayoutSnapshot, key: &str) -> Option<Rect> {
        layout.bounds(key)
    }

    /// Connects handlers from an already-rendered retained subtree to this entity.
    pub fn route_events_to(&mut self, entity: AnyEntity) {
        if let Some(owner) = self.entity.as_ref().and_then(WeakEntity::upgrade) {
            (entity.bind_model_wake)(&owner.0.model.runtime);
        }
        #[cfg(feature = "tasks")]
        if let Some(runtime) = self
            .entity
            .as_ref()
            .and_then(WeakEntity::upgrade)
            .and_then(|owner| owner.0.model.runtime.task_runtime())
        {
            entity.set_task_runtime(runtime);
        }
        self.effects.event_routes.push(entity);
    }
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

fn with_event_handler(event: &UiEvent, dispatch: &mut dyn FnMut(EventHandlerId)) {
    if let Some(handler) = event.current_handler() {
        dispatch(handler);
    }
}

pub trait Render: 'static {
    #[cfg(feature = "tasks")]
    fn tasks_ready(&mut self, _cx: &mut Context<Self>)
    where
        Self: Sized,
    {
    }
    fn render(&mut self, cx: &mut Context<Self>) -> Element
    where
        Self: Sized;

    fn animation_frame(&mut self, _frame: Frame, _cx: &mut Context<Self>)
    where
        Self: Sized,
    {
    }

    fn wants_animation_frame(&self) -> bool {
        false
    }

    fn layout_changed(&mut self, _layout: &LayoutSnapshot, _cx: &mut Context<Self>)
    where
        Self: Sized,
    {
    }

    fn image_assets(&self) -> Vec<ImageAsset> {
        Vec::new()
    }

    fn vector_assets(&self) -> Vec<VectorAsset> {
        Vec::new()
    }

    fn inspector(&self) -> Option<InspectorHandle> {
        None
    }
}

impl<T: Render> Entity<T> {
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
            image_assets: Rc::new(move || image_assets.read(Render::image_assets)),
            vector_assets: Rc::new(move || vector_assets.read(Render::vector_assets)),
            inspector: Rc::new(move || inspector.read(Render::inspector)),
            take_effects: Rc::new(move || take_effects.take_effects()),
        }
    }

    #[must_use]
    pub fn render(&self) -> Element {
        let environment = self.0.presentation.environment.borrow().clone();
        self.render_in(environment)
    }

    #[must_use]
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
        let element = self
            .0
            .model
            .value
            .borrow_mut()
            .render(&mut cx)
            .semantic_scope();
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
            && (self.0.model.value.borrow().wants_animation_frame()
                || self
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
        self.0
            .model
            .value
            .borrow_mut()
            .animation_frame(frame, &mut cx);
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
        self.0
            .model
            .value
            .borrow_mut()
            .layout_changed(layout, &mut cx);
        self.0.model.signal.apply_update(cx.effects.update);
        cx.effects
    }
}

impl<T: 'static> WeakEntity<T> {
    #[must_use]
    pub fn upgrade(&self) -> Option<Entity<T>> {
        if let Some(presentation) = self.presentation.upgrade() {
            return Some(Entity(presentation));
        }
        self.model.upgrade().map(|model| {
            Entity(Rc::new(EntityCell {
                presentation: Presentation::new(model.signal.clone()),
                model,
            }))
        })
    }
}

impl AnyEntity {
    #[must_use]
    pub fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.identity, &other.identity)
    }

    pub(crate) fn render(&self, environment: WindowEnvironment) -> Element {
        (self.render)(environment)
    }

    pub(crate) fn event(&self, event: &UiEvent) {
        with_event_handler(event, &mut |handler| {
            (self.store)((self.dispatch_handler)(handler, event));
        });
    }

    pub(crate) fn animation_frame(&self, frame: Frame) {
        (self.store)((self.frame)(frame));
    }

    pub(crate) fn layout_changed(&self, layout: &LayoutSnapshot) {
        (self.store)((self.layout)(layout));
    }

    pub(crate) fn wants_frame(&self) -> bool {
        (self.wants_frame)()
    }

    pub(crate) fn image_assets(&self) -> Vec<ImageAsset> {
        (self.image_assets)()
    }

    pub(crate) fn vector_assets(&self) -> Vec<VectorAsset> {
        (self.vector_assets)()
    }

    pub(crate) fn inspector(&self) -> Option<InspectorHandle> {
        (self.inspector)()
    }

    pub(crate) fn take_effects(&self) -> ContextEffects {
        (self.take_effects)()
    }
}
