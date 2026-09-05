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

mod effects;
mod handler;
mod layout;
pub(crate) use effects::PointerCaptureRequest;
pub use effects::ViewUpdate;
use effects::{ContextEffects, SelectionCommandRequest, merge_effects, strongest_update};
use handler::{HandlerRegistry, LocalHandler};
pub use layout::{LayoutBounds, LayoutSnapshot, ScrollRequest};

type HandlerDispatch = dyn Fn(EventHandlerId, &UiEvent) -> ContextEffects;
type Observer = Rc<dyn Fn()>;

struct RenderCache {
    element: RefCell<Option<Element>>,
    dirty: Rc<Cell<bool>>,
    observers: RefCell<std::collections::HashMap<usize, Observer>>,
}

impl RenderCache {
    fn new() -> Self {
        Self {
            element: RefCell::new(None),
            dirty: Rc::new(Cell::new(true)),
            observers: RefCell::new(std::collections::HashMap::new()),
        }
    }

    fn observer(&self) -> Observer {
        let dirty = Rc::downgrade(&self.dirty);
        Rc::new(move || {
            if let Some(dirty) = dirty.upgrade() {
                dirty.set(true);
            }
        })
    }

    fn register(&self, owner: &Option<(usize, Observer)>) {
        if let Some((identity, observer)) = owner {
            self.observers
                .borrow_mut()
                .insert(*identity, observer.clone());
        }
    }

    fn reusable(&self, environment_changed: bool) -> Option<Element> {
        if !self.dirty.get()
            && !environment_changed
            && let Some(element) = self.element.borrow().as_ref()
        {
            return Some(element.clone());
        }
        None
    }

    fn store(&self, element: &Element) {
        *self.element.borrow_mut() = Some(element.clone());
        self.dirty.set(false);
    }

    fn mark_dirty(&self) {
        if !self.dirty.replace(true) {
            for observer in self.observers.borrow().values() {
                observer();
            }
        }
    }

    fn apply_update(&self, update: ViewUpdate) {
        if update == ViewUpdate::Rebuild {
            self.mark_dirty();
        }
    }
}

/// Retained component identity. Cloning an entity never clones its state.
pub struct Entity<T>(Rc<EntityCell<T>>);

struct EntityCell<T> {
    value: RefCell<T>,
    cache: RenderCache,
    pending: RefCell<ContextEffects>,
    children: RefCell<Vec<AnyEntity>>,
    event_routes: RefCell<Vec<AnyEntity>>,
    environment: Cell<WindowEnvironment>,
    environment_used: Cell<bool>,
    handlers: RefCell<HandlerRegistry<T>>,
}

impl<T> Clone for Entity<T> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

pub struct WeakEntity<T>(Weak<EntityCell<T>>);

impl<T> Clone for WeakEntity<T> {
    fn clone(&self) -> Self {
        Self(Weak::clone(&self.0))
    }
}

#[derive(Clone)]
pub struct AnyEntity {
    identity: Rc<dyn Any>,
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
    pub(crate) effects: ContextEffects,
    owner: Option<(usize, Observer)>,
    environment: WindowEnvironment,
    environment_read: Cell<bool>,
    event_target: Option<argui_ui::NodeId>,
    handlers: Vec<LocalHandler<T>>,
    _marker: PhantomData<fn() -> T>,
}

impl<T> Default for Context<T> {
    fn default() -> Self {
        Self {
            effects: ContextEffects::default(),
            owner: None,
            environment: WindowEnvironment::default(),
            environment_read: Cell::new(false),
            event_target: None,
            handlers: Vec::new(),
            _marker: PhantomData,
        }
    }
}

impl<T: Render> Context<T> {
    #[must_use]
    pub fn environment(&self) -> WindowEnvironment {
        self.environment_read.set(true);
        self.environment
    }

    #[must_use]
    pub fn new_entity<U: Render>(&mut self, value: U) -> Entity<U> {
        Entity::new(value)
    }

    /// Marks this entity dirty. Its cached subtree is rebuilt once at the next frame.
    pub fn notify(&mut self) {
        self.effects.update = ViewUpdate::Rebuild;
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

    pub fn selection_command(&mut self, command: argui_ui::SelectionCommand) {
        self.effects.selection_command = Some(SelectionCommandRequest {
            target: self.event_target,
            command,
        });
    }

    pub fn selection_command_for(
        &mut self,
        target: argui_ui::NodeId,
        command: argui_ui::SelectionCommand,
    ) {
        self.effects.selection_command = Some(SelectionCommandRequest {
            target: Some(target),
            command,
        });
    }

    pub fn set_theme(&mut self, request: ThemeRequest) {
        self.effects.theme = Some(request);
        self.notify();
    }

    pub fn command(&mut self, command: AppCommand) {
        self.effects.commands.push(command);
    }

    #[must_use]
    pub const fn view_update(&self) -> ViewUpdate {
        self.effects.update
    }

    #[must_use]
    pub fn observe_bounds(&self, layout: &LayoutSnapshot, key: &str) -> Option<Rect> {
        layout.bounds(key)
    }

    /// Renders a child entity and reuses its exact subtree while it is clean.
    pub fn entity<U: Render>(&mut self, entity: &Entity<U>) -> Element {
        let element = entity.render_in(self.environment);
        inherit_environment_use(&self.environment_read, &entity.0.environment_used);
        entity.0.cache.register(&self.owner);
        self.effects.children.push(entity.erase());
        element
    }

    /// Connects handlers from an already-rendered retained subtree to this entity.
    pub fn route_events_to(&mut self, entity: AnyEntity) {
        self.effects.event_routes.push(entity);
    }

    pub(crate) fn merge_effects(&mut self, child: ContextEffects) {
        merge_effects(&mut self.effects, child);
    }

    /// Delivers a layout snapshot to a retained child with a component-specific viewport.
    pub fn layout_entity<U: Render>(&mut self, entity: &Entity<U>, layout: &LayoutSnapshot) {
        self.merge_effects(entity.dispatch_layout(layout));
    }
}

fn inherit_environment_use(parent: &Cell<bool>, child: &Cell<bool>) {
    if child.get() {
        parent.set(true);
    }
}

fn update_environment(
    current: &Cell<WindowEnvironment>,
    used: &Cell<bool>,
    next: WindowEnvironment,
) -> bool {
    current.replace(next) != next && used.get()
}

fn with_event_handler(event: &UiEvent, dispatch: &mut dyn FnMut(EventHandlerId)) {
    if let Some(handler) = event.current_handler() {
        dispatch(handler);
    }
}

pub trait Render: 'static {
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
    pub fn new(value: T) -> Self {
        Self(Rc::new(EntityCell {
            value: RefCell::new(value),
            cache: RenderCache::new(),
            pending: RefCell::new(ContextEffects::default()),
            children: RefCell::new(Vec::new()),
            event_routes: RefCell::new(Vec::new()),
            environment: Cell::new(WindowEnvironment::default()),
            environment_used: Cell::new(false),
            handlers: RefCell::new(HandlerRegistry::default()),
        }))
    }

    #[must_use]
    pub fn downgrade(&self) -> WeakEntity<T> {
        WeakEntity(Rc::downgrade(&self.0))
    }

    #[must_use]
    pub fn erase(&self) -> AnyEntity {
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
        AnyEntity {
            identity: self.0.clone(),
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

    pub fn update(&self, update: impl FnOnce(&mut T, &mut Context<T>)) {
        let observer = self.0.cache.observer();
        let mut cx = Context {
            owner: Some((Rc::as_ptr(&self.0) as usize, observer)),
            environment: self.0.environment.get(),
            ..Context::default()
        };
        update(&mut self.0.value.borrow_mut(), &mut cx);
        self.finish(cx);
    }

    #[must_use]
    pub fn render(&self) -> Element {
        self.render_in(self.0.environment.get())
    }

    #[must_use]
    pub fn render_in(&self, environment: WindowEnvironment) -> Element {
        let environment_changed =
            update_environment(&self.0.environment, &self.0.environment_used, environment);
        if let Some(element) = self.0.cache.reusable(environment_changed) {
            return element;
        }
        let observer = self.0.cache.observer();
        let mut cx = Context {
            owner: Some((Rc::as_ptr(&self.0) as usize, observer)),
            environment,
            ..Context::default()
        };
        let element = self.0.value.borrow_mut().render(&mut cx);
        self.0
            .handlers
            .borrow_mut()
            .replace(std::mem::take(&mut cx.handlers));
        *self.0.children.borrow_mut() = std::mem::take(&mut cx.effects.children);
        *self.0.event_routes.borrow_mut() = std::mem::take(&mut cx.effects.event_routes);
        self.0.environment_used.set(cx.environment_read.get());
        self.0.cache.store(&element);
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

    fn finish(&self, cx: Context<T>) {
        self.store_effects(cx.effects);
    }

    fn store_effects(&self, effects: ContextEffects) {
        self.0.cache.apply_update(effects.update);
        let mut pending = self.0.pending.borrow_mut();
        merge_effects(&mut pending, effects);
    }

    fn owner_id(&self) -> EventOwnerId {
        EventOwnerId(Rc::as_ptr(&self.0) as usize)
    }

    fn owns(&self, owner: EventOwnerId) -> bool {
        owner == self.owner_id()
            || self
                .0
                .children
                .borrow()
                .iter()
                .any(|child| (child.owns)(owner))
            || self
                .0
                .event_routes
                .borrow()
                .iter()
                .any(|route| (route.owns)(owner))
    }

    pub(crate) fn wants_frame(&self) -> bool {
        self.0.value.borrow().wants_animation_frame()
            || self
                .0
                .children
                .borrow()
                .iter()
                .any(|child| (child.wants_frame)())
    }

    fn dispatch_frame(&self, frame: Frame) -> ContextEffects {
        let mut effects = ContextEffects::default();
        for child in self.0.children.borrow().iter() {
            if (child.wants_frame)() {
                merge_effects(&mut effects, (child.frame)(frame));
            }
        }
        let mut cx = Context {
            environment: self.0.environment.get(),
            ..Context::default()
        };
        self.0.value.borrow_mut().animation_frame(frame, &mut cx);
        merge_effects(&mut effects, cx.effects);
        self.0.cache.apply_update(effects.update);
        effects
    }

    fn dispatch_layout(&self, layout: &LayoutSnapshot) -> ContextEffects {
        let mut cx = Context {
            environment: self.0.environment.get(),
            ..Context::default()
        };
        self.0.value.borrow_mut().layout_changed(layout, &mut cx);
        self.0.cache.apply_update(cx.effects.update);
        cx.effects
    }

    pub(crate) fn take_effects(&self) -> ContextEffects {
        std::mem::take(&mut *self.0.pending.borrow_mut())
    }

    pub fn read<R>(&self, read: impl FnOnce(&T) -> R) -> R {
        read(&self.0.value.borrow())
    }
}

impl<T: Render> WeakEntity<T> {
    #[must_use]
    pub fn upgrade(&self) -> Option<Entity<T>> {
        self.0.upgrade().map(Entity)
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
