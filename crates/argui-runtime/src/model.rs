use crate::AppCommand;
use argui_animation::Frame;
use argui_core::{Point, Rect, Size};
use argui_inspect::InspectorHandle;
use argui_paint::{ImageAsset, VectorAsset};
use argui_ui::{ClipboardRequest, Element, FocusRequest, FocusTarget, NodeId, UiEvent};
use std::{
    any::Any,
    cell::{Cell, RefCell},
    collections::HashSet,
    marker::PhantomData,
    rc::{Rc, Weak},
};

/// Retained component identity. Cloning an entity never clones its state.
pub struct Entity<T: Render>(Rc<EntityCell<T>>);

struct EntityCell<T: Render> {
    value: RefCell<T>,
    cached: RefCell<Option<Element>>,
    dirty: Cell<bool>,
    pending: RefCell<ContextEffects>,
    children: RefCell<Vec<AnyEntity>>,
    keys: RefCell<HashSet<String>>,
    observers: RefCell<std::collections::HashMap<usize, Rc<dyn Fn()>>>,
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
    render: Rc<dyn Fn() -> Element>,
    dispatch: Rc<dyn Fn(&UiEvent) -> ContextEffects>,
    store: Rc<dyn Fn(ContextEffects)>,
    wants_frame: Rc<dyn Fn() -> bool>,
    frame: Rc<dyn Fn(Frame) -> ContextEffects>,
    layout: Rc<dyn Fn(&LayoutSnapshot) -> ContextEffects>,
    keys: Rc<dyn Fn(&str) -> bool>,
    image_assets: Rc<dyn Fn() -> Vec<ImageAsset>>,
    vector_assets: Rc<dyn Fn() -> Vec<VectorAsset>>,
    inspector: Rc<dyn Fn() -> Option<InspectorHandle>>,
    take_effects: Rc<dyn Fn() -> ContextEffects>,
}

#[derive(Default)]
pub(crate) struct ContextEffects {
    pub(crate) update: ViewUpdate,
    pub(crate) animation_frame: bool,
    pub(crate) propagation_stopped: bool,
    pub(crate) clipboard: Option<ClipboardRequest>,
    pub(crate) scroll: Option<ScrollRequest>,
    pub(crate) focus: Option<FocusRequest>,
    pub(crate) commands: Vec<AppCommand>,
    children: Vec<AnyEntity>,
}

/// Mutation and scheduling access scoped to one retained component.
pub struct Context<T: Render> {
    pub(crate) effects: ContextEffects,
    owner: Option<(usize, Rc<dyn Fn()>)>,
    _marker: PhantomData<fn() -> T>,
}

impl<T: Render> Default for Context<T> {
    fn default() -> Self {
        Self {
            effects: ContextEffects::default(),
            owner: None,
            _marker: PhantomData,
        }
    }
}

impl<T: Render> Context<T> {
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

    pub fn stop_propagation(&mut self) {
        self.effects.propagation_stopped = true;
    }

    pub fn write_clipboard(&mut self, request: ClipboardRequest) {
        self.effects.clipboard = Some(request);
    }

    pub fn scroll_to(&mut self, key: impl Into<String>, offset: Point) {
        self.effects.scroll = Some(ScrollRequest {
            key: key.into(),
            offset,
        });
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
        let element = entity.render();
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
        self.effects.update = strongest_update(self.effects.update, child.effects.update);
        self.effects.animation_frame |= child.effects.animation_frame;
        self.effects.propagation_stopped |= child.effects.propagation_stopped;
        if child.effects.clipboard.is_some() {
            self.effects.clipboard = child.effects.clipboard;
        }
        if child.effects.scroll.is_some() {
            self.effects.scroll = child.effects.scroll;
        }
        if child.effects.focus.is_some() {
            self.effects.focus = child.effects.focus;
        }
        self.effects.commands.append(&mut child.effects.commands);
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
            keys: RefCell::new(HashSet::new()),
            observers: RefCell::new(std::collections::HashMap::new()),
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
        let keys = self.clone();
        let image_assets = self.clone();
        let vector_assets = self.clone();
        let inspector = self.clone();
        let take_effects = self.clone();
        AnyEntity {
            identity: self.0.clone(),
            render: Rc::new(move || render.render()),
            dispatch: Rc::new(move |event| dispatch.dispatch(event)),
            store: Rc::new(move |effects| store.store_effects(effects)),
            wants_frame: Rc::new(move || wants_frame.wants_frame()),
            frame: Rc::new(move |value| frame.dispatch_frame(value)),
            layout: Rc::new(move |snapshot| layout.dispatch_layout(snapshot)),
            keys: Rc::new(move |key| keys.0.keys.borrow().contains(key)),
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
            ..Context::default()
        };
        update(&mut self.0.value.borrow_mut(), &mut cx);
        self.finish(cx);
    }

    #[must_use]
    pub fn render(&self) -> Element {
        if !self.0.dirty.get()
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
            ..Context::default()
        };
        let element = self.0.value.borrow_mut().render(&mut cx);
        self.0.keys.borrow_mut().clear();
        collect_keys(&element, &mut self.0.keys.borrow_mut());
        *self.0.children.borrow_mut() = std::mem::take(&mut cx.effects.children);
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
        let mut effects = ContextEffects::default();
        if let Some(key) = event.key.as_deref()
            && let Some(child) = self
                .0
                .children
                .borrow()
                .iter()
                .rev()
                .find(|child| (child.keys)(key))
        {
            merge_effects(&mut effects, (child.dispatch)(event));
        }
        if !effects.propagation_stopped {
            let mut cx = Context::default();
            self.0.value.borrow_mut().event(event, &mut cx);
            merge_effects(&mut effects, cx.effects);
        }
        if effects.update == ViewUpdate::Rebuild {
            self.mark_dirty();
        }
        effects
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
        let mut cx = Context::default();
        self.0.value.borrow_mut().animation_frame(frame, &mut cx);
        merge_effects(&mut effects, cx.effects);
        if effects.update == ViewUpdate::Rebuild {
            self.mark_dirty();
        }
        effects
    }

    fn dispatch_layout(&self, layout: &LayoutSnapshot) -> ContextEffects {
        let mut cx = Context::default();
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

    pub(crate) fn render(&self) -> Element {
        (self.render)()
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

fn strongest_update(left: ViewUpdate, right: ViewUpdate) -> ViewUpdate {
    match (left, right) {
        (ViewUpdate::Rebuild, _) | (_, ViewUpdate::Rebuild) => ViewUpdate::Rebuild,
        (ViewUpdate::Paint, _) | (_, ViewUpdate::Paint) => ViewUpdate::Paint,
        _ => ViewUpdate::None,
    }
}

fn merge_effects(target: &mut ContextEffects, mut source: ContextEffects) {
    target.update = strongest_update(target.update, source.update);
    target.animation_frame |= source.animation_frame;
    target.propagation_stopped |= source.propagation_stopped;
    if source.clipboard.is_some() {
        target.clipboard = source.clipboard.take();
    }
    if source.scroll.is_some() {
        target.scroll = source.scroll.take();
    }
    if source.focus.is_some() {
        target.focus = source.focus.take();
    }
    target.commands.append(&mut source.commands);
}

fn collect_keys(element: &Element, keys: &mut HashSet<String>) {
    if let Some(key) = &element.key {
        keys.insert(key.clone());
    }
    for child in &element.children {
        collect_keys(child, keys);
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ViewUpdate {
    #[default]
    None,
    Paint,
    Rebuild,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayoutBounds {
    pub node: NodeId,
    pub key: Option<String>,
    pub bounds: Rect,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LayoutSnapshot {
    pub viewport: Rect,
    pub nodes: Vec<LayoutBounds>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScrollRequest {
    pub key: String,
    pub offset: Point,
}

impl LayoutSnapshot {
    #[must_use]
    pub fn bounds(&self, key: &str) -> Option<Rect> {
        self.nodes
            .iter()
            .find(|node| node.key.as_deref() == Some(key))
            .map(|node| node.bounds)
    }

    #[must_use]
    pub const fn viewport_size(&self) -> Size {
        self.viewport.size
    }
}
