use crate::AppCommand;
use argui_animation::Frame;
use argui_core::{PointerId, Rect};
use argui_inspect::InspectorHandle;
use argui_paint::{ImageAsset, VectorAsset};
use argui_ui::{
    ClipboardRequest, Element, EventOwnerId, FocusRequest, FocusTarget, TextSelection,
    TextSelectionRequest, UiEvent,
};
use std::{
    any::Any,
    cell::{Cell, RefCell},
    marker::PhantomData,
    rc::{Rc, Weak},
};

use crate::{ThemeRequest, WindowEnvironment};

mod effects;
mod layout;
pub(crate) use effects::PointerCaptureRequest;
pub use effects::ViewUpdate;
use effects::{ContextEffects, SelectionCommandRequest, merge_effects, strongest_update};
pub use layout::{LayoutBounds, LayoutSnapshot, ScrollRequest};

/// Retained component identity. Cloning an entity never clones its state.
pub struct Entity<T: Render>(Rc<EntityCell<T>>);

struct EntityCell<T: Render> {
    value: RefCell<T>,
    cached: RefCell<Option<Element>>,
    dirty: Cell<bool>,
    pending: RefCell<ContextEffects>,
    children: RefCell<Vec<AnyEntity>>,
    observers: RefCell<std::collections::HashMap<usize, Rc<dyn Fn()>>>,
    environment: Cell<WindowEnvironment>,
    environment_used: Cell<bool>,
}

impl<T: Render> Clone for Entity<T> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

pub struct WeakEntity<T: Render>(Weak<EntityCell<T>>);

impl<T: Render> Clone for WeakEntity<T> {
    fn clone(&self) -> Self {
        Self(Weak::clone(&self.0))
    }
}

#[derive(Clone)]
pub struct AnyEntity {
    identity: Rc<dyn Any>,
    render: Rc<dyn Fn(WindowEnvironment) -> Element>,
    dispatch: Rc<dyn Fn(&UiEvent) -> ContextEffects>,
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
pub struct Context<T: Render> {
    pub(crate) effects: ContextEffects,
    owner: Option<(usize, Rc<dyn Fn()>)>,
    environment: WindowEnvironment,
    environment_read: Cell<bool>,
    event_target: Option<argui_ui::NodeId>,
    _marker: PhantomData<fn() -> T>,
}

impl<T: Render> Default for Context<T> {
    fn default() -> Self {
        Self {
            effects: ContextEffects::default(),
            owner: None,
            environment: WindowEnvironment::default(),
            environment_read: Cell::new(false),
            event_target: None,
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

    /// Creates a context for a directly rendered child while preserving its window environment.
    #[must_use]
    pub fn child_context<U: Render>(&self) -> Context<U> {
        Context {
            environment: self.environment,
            ..Context::default()
        }
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
        self.environment_read
            .set(self.environment_read.get() || entity.0.environment_used.get());
        if let Some((identity, observer)) = &self.owner {
            entity
                .0
                .observers
                .borrow_mut()
                .insert(*identity, observer.clone());
        }
        self.effects.children.push(entity.erase());
        element
    }

    /// Bubbles scheduling effects produced while rendering or dispatching to a child component.
    pub fn propagate<U: Render>(&mut self, mut child: Context<U>) {
        self.environment_read
            .set(self.environment_read.get() || child.environment_read.get());
        self.effects.update = strongest_update(self.effects.update, child.effects.update);
        self.effects.animation_frame |= child.effects.animation_frame;
        if child.effects.clipboard.is_some() {
            self.effects.clipboard = child.effects.clipboard;
        }
        if child.effects.scroll.is_some() {
            self.effects.scroll = child.effects.scroll;
        }
        if child.effects.focus.is_some() {
            self.effects.focus = child.effects.focus;
        }
        if child.effects.text_selection.is_some() {
            self.effects.text_selection = child.effects.text_selection;
        }
        if child.effects.theme.is_some() {
            self.effects.theme = child.effects.theme;
        }
        if child.effects.selection_command.is_some() {
            self.effects.selection_command = child.effects.selection_command;
        }
        self.effects.commands.append(&mut child.effects.commands);
        self.effects
            .pointer_capture
            .append(&mut child.effects.pointer_capture);
        self.effects.children.append(&mut child.effects.children);
    }
}

pub trait Render: 'static {
    fn render(&mut self, cx: &mut Context<Self>) -> Element
    where
        Self: Sized;

    fn event(&mut self, _event: &UiEvent, _cx: &mut Context<Self>)
    where
        Self: Sized,
    {
    }

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
            cached: RefCell::new(None),
            dirty: Cell::new(true),
            pending: RefCell::new(ContextEffects::default()),
            children: RefCell::new(Vec::new()),
            observers: RefCell::new(std::collections::HashMap::new()),
            environment: Cell::new(WindowEnvironment::default()),
            environment_used: Cell::new(false),
        }))
    }

    #[must_use]
    pub fn downgrade(&self) -> WeakEntity<T> {
        WeakEntity(Rc::downgrade(&self.0))
    }

    #[must_use]
    pub fn erase(&self) -> AnyEntity {
        let render = self.clone();
        let dispatch = self.clone();
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
            dispatch: Rc::new(move |event| dispatch.dispatch(event)),
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
        let weak = self.downgrade();
        let observer = Rc::new(move || {
            if let Some(entity) = weak.upgrade() {
                entity.mark_dirty();
            }
        });
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
            self.0.environment.replace(environment) != environment && self.0.environment_used.get();
        if !self.0.dirty.get()
            && !environment_changed
            && let Some(cached) = self.0.cached.borrow().as_ref()
        {
            return cached.clone();
        }
        let weak = self.downgrade();
        let observer = Rc::new(move || {
            if let Some(entity) = weak.upgrade() {
                entity.mark_dirty();
            }
        });
        let mut cx = Context {
            owner: Some((Rc::as_ptr(&self.0) as usize, observer)),
            environment,
            ..Context::default()
        };
        let mut element = self.0.value.borrow_mut().render(&mut cx);
        element.assign_event_owner(self.owner_id());
        *self.0.children.borrow_mut() = std::mem::take(&mut cx.effects.children);
        self.0.environment_used.set(cx.environment_read.get());
        *self.0.cached.borrow_mut() = Some(element.clone());
        self.0.dirty.set(false);
        self.finish(cx);
        element
    }

    pub(crate) fn event(&self, event: &UiEvent) {
        let effects = self.dispatch(event);
        self.store_effects(effects);
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
        if effects.update == ViewUpdate::Rebuild {
            self.mark_dirty();
        }
        let mut pending = self.0.pending.borrow_mut();
        merge_effects(&mut pending, effects);
    }

    fn dispatch(&self, event: &UiEvent) -> ContextEffects {
        if let Some(owner) = event.current_owner()
            && owner != self.owner_id()
            && let Some(child) = self
                .0
                .children
                .borrow()
                .iter()
                .find(|child| (child.owns)(owner))
        {
            return (child.dispatch)(event);
        }
        let mut cx = Context {
            environment: self.0.environment.get(),
            event_target: Some(event.current_target()),
            ..Context::default()
        };
        self.0.value.borrow_mut().event(event, &mut cx);
        let effects = cx.effects;
        if effects.update == ViewUpdate::Rebuild {
            self.mark_dirty();
        }
        effects
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
        if effects.update == ViewUpdate::Rebuild {
            self.mark_dirty();
        }
        effects
    }

    fn dispatch_layout(&self, layout: &LayoutSnapshot) -> ContextEffects {
        let mut cx = Context {
            environment: self.0.environment.get(),
            ..Context::default()
        };
        self.0.value.borrow_mut().layout_changed(layout, &mut cx);
        if cx.effects.update == ViewUpdate::Rebuild {
            self.mark_dirty();
        }
        cx.effects
    }

    fn mark_dirty(&self) {
        let was_dirty = self.0.dirty.replace(true);
        if !was_dirty {
            for observer in self.0.observers.borrow().values() {
                observer();
            }
        }
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
        (self.store)((self.dispatch)(event));
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
