//! Rendering contracts and per-presentation mutation access.

use super::{
    AnyEntity, ContextEffects, Entity, LayoutSnapshot, Observer, PointerCaptureRequest,
    ScrollRequest, Subscription, WeakEntity,
    effects::{merge_effects, strongest_update},
    handler::LocalHandler,
    observation::{InteractionSnapshot, ObservationReader, ObservedInteraction},
    presentation::PresentationId,
};
use crate::{AppCommand, ThemeRequest, WindowEnvironment};
use argui_animation::Frame;
use argui_core::{PointerId, Rect};
use argui_inspect::InspectorHandle;
use argui_paint::{ImageAsset, VectorAsset};
use argui_render::EffectDefinition;
use argui_ui::{
    ClipboardRequest, Element, FocusRequest, FocusTarget, RetainedIdentity, TextSelection,
    TextSelectionRequest,
};
use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
    marker::PhantomData,
    rc::Rc,
};

/// Mutation and scheduling access scoped to one retained component.
pub struct Context<T> {
    pub(super) entity: Option<WeakEntity<T>>,
    pub(crate) effects: ContextEffects,
    pub(super) owner: Option<(PresentationId, Observer)>,
    pub(super) environment: WindowEnvironment,
    pub(super) environment_read: Cell<bool>,
    pub(super) event_target: Option<argui_ui::NodeId>,
    pub(super) handlers: Vec<LocalHandler<T>>,
    pub(super) dependencies: Vec<Subscription>,
    pub(super) observations: Rc<RefCell<InteractionSnapshot>>,
    pub(super) observed_identities: Rc<RefCell<HashSet<RetainedIdentity>>>,
    pub(super) _marker: PhantomData<fn() -> T>,
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
            observations: Rc::default(),
            observed_identities: Rc::default(),
            _marker: PhantomData,
        }
    }
}

impl<T: 'static> Context<T> {
    /// Creates a retained entity in this component's model runtime.
    ///
    /// `value` is the initial state stored by the new entity.
    #[must_use]
    pub fn new_entity<U: 'static>(&mut self, value: U) -> Entity<U> {
        match self.entity.as_ref().and_then(WeakEntity::upgrade) {
            Some(owner) => owner.0.model.runtime.entity(value),
            None => Entity::new(value),
        }
    }

    /// Marks this entity dirty so its cached subtree is rebuilt at the next frame.
    pub fn notify(&mut self) {
        self.effects.update = super::ViewUpdate::Rebuild;
    }

    /// Queues an application command for the host to process.
    ///
    /// `command` describes the requested host operation.
    pub fn command(&mut self, command: AppCommand) {
        self.effects.commands.push(command);
    }

    /// Returns the strongest view update requested so far in this context.
    #[must_use]
    pub const fn view_update(&self) -> super::ViewUpdate {
        self.effects.update
    }

    pub(crate) fn merge_effects(&mut self, child: ContextEffects) {
        merge_effects(&mut self.effects, child);
    }
}

impl<T: Render> Context<T> {
    /// Reads current interaction state for a retained source identity.
    ///
    /// `identity` identifies the source element being observed. A read during
    /// rendering subscribes this presentation to later hover, press, focus,
    /// and pointer position changes. Event handlers read the latest state
    /// without changing render dependencies. Returns the idle state if the
    /// identity is absent from the previous retained tree, including on its
    /// first render.
    #[must_use]
    pub fn observed_interaction(&self, identity: &RetainedIdentity) -> ObservedInteraction {
        self.observation_reader().get(identity)
    }

    /// Returns a cloneable reader for this context's interaction snapshot.
    ///
    /// The reader can be moved into rendering closures that cannot borrow
    /// `self`. Reads made while rendering register source identities so later
    /// interaction changes invalidate this presentation. A retained event
    /// closure reads the latest host snapshot when its handler runs.
    #[must_use]
    pub fn observation_reader(&self) -> ObservationReader {
        ObservationReader::new(self.observations.clone(), self.observed_identities.clone())
    }

    /// Returns the environment used for the current render.
    /// Reading it makes future environment changes invalidate the cached view.
    #[must_use]
    pub fn environment(&self) -> WindowEnvironment {
        self.environment_read.set(true);
        self.environment.clone()
    }

    /// Requests repainting without rebuilding the retained subtree.
    pub fn request_paint(&mut self) {
        self.effects.update = strongest_update(self.effects.update, super::ViewUpdate::Paint);
    }

    /// Requests another animation frame and a repaint.
    pub fn request_animation_frame(&mut self) {
        self.effects.animation_frame = true;
        self.request_paint();
    }

    /// Captures the pointer for the node whose handler is currently running.
    /// Returns `false` when called outside a node event handler.
    ///
    /// `pointer` identifies the pointer to capture.
    pub fn capture_pointer(&mut self, pointer: PointerId) -> bool {
        let Some(target) = self.event_target else {
            return false;
        };
        self.effects
            .pointer_capture
            .push(PointerCaptureRequest::Capture { pointer, target });
        true
    }

    /// Releases the pointer capture held by the current event-handler node.
    /// Returns `false` when called outside a node event handler.
    ///
    /// `pointer` identifies the pointer to release.
    pub fn release_pointer(&mut self, pointer: PointerId) -> bool {
        let Some(target) = self.event_target else {
            return false;
        };
        self.effects
            .pointer_capture
            .push(PointerCaptureRequest::Release { pointer, target });
        true
    }

    /// Queues a clipboard operation for the current window.
    ///
    /// `request` describes the read or write to perform.
    pub fn write_clipboard(&mut self, request: ClipboardRequest) {
        self.effects.clipboard = Some(request);
    }

    /// Queues a scroll request and requests a repaint.
    ///
    /// `request` describes the target and desired scroll position.
    pub fn scroll(&mut self, request: ScrollRequest) {
        self.effects.scroll = Some(request);
        self.request_paint();
    }

    /// Requests focus for a UI target and schedules a repaint.
    ///
    /// `target` identifies the target to focus.
    pub fn request_focus(&mut self, target: impl Into<FocusTarget>) {
        self.effects.focus = Some(FocusRequest::Focus(target.into()));
        self.request_paint();
    }

    /// Requests that the current UI focus be cleared and schedules a repaint.
    pub fn clear_focus(&mut self) {
        self.effects.focus = Some(FocusRequest::Clear);
        self.request_paint();
    }

    /// Requests focus on the next enabled Tab stop within the active focus scope.
    pub fn focus_next(&mut self) {
        self.effects.focus = Some(FocusRequest::Next);
        self.request_paint();
    }

    /// Requests focus on the previous enabled Tab stop within the active focus scope.
    pub fn focus_previous(&mut self) {
        self.effects.focus = Some(FocusRequest::Previous);
        self.request_paint();
    }

    /// Queues a text selection for a target and schedules a repaint.
    ///
    /// `target` identifies the text input; `selection` is the desired selection.
    pub fn select_text(&mut self, target: impl Into<FocusTarget>, selection: TextSelection) {
        self.effects.text_selection = Some(TextSelectionRequest::new(target, selection));
        self.request_paint();
    }

    /// Applies application-selected theme preferences to the current window.
    ///
    /// `request` contains the color-scheme and primary-color overrides.
    pub fn set_theme(&mut self, request: ThemeRequest) {
        self.effects.theme = Some(request);
        self.notify();
    }

    /// Looks up the bounds of a named layout element.
    ///
    /// `layout` is the snapshot to query; `key` is the element's layout key.
    #[must_use]
    pub fn observe_bounds(&self, layout: &LayoutSnapshot, key: &str) -> Option<Rect> {
        layout.bounds(key)
    }

    /// Connects handlers from an already-rendered retained subtree to this entity.
    ///
    /// `entity` is the retained subtree whose event handlers should be routed here.
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

/// Stateful component contract used by the retained model runtime.
pub trait Render: 'static {
    /// Handles asynchronous task completions for this component.
    ///
    /// `_cx` collects any effects requested while processing those completions.
    #[cfg(feature = "tasks")]
    fn tasks_ready(&mut self, _cx: &mut Context<Self>)
    where
        Self: Sized,
    {
    }

    /// Builds this component's retained UI subtree.
    ///
    /// `cx` provides model access and records the effects requested during rendering.
    fn render(&mut self, cx: &mut Context<Self>) -> Element
    where
        Self: Sized;

    /// Advances component state for an animation `frame`.
    ///
    /// `_frame` is the current animation timing information; `_cx` records any
    /// effects requested while handling it.
    fn animation_frame(&mut self, _frame: Frame, _cx: &mut Context<Self>)
    where
        Self: Sized,
    {
    }

    /// Reports whether this component needs animation frames.
    fn wants_animation_frame(&self) -> bool {
        false
    }

    /// Notifies the component that its layout has changed.
    ///
    /// `_layout` is the latest geometry snapshot; `_cx` records resulting effects.
    fn layout_changed(&mut self, _layout: &LayoutSnapshot, _cx: &mut Context<Self>)
    where
        Self: Sized,
    {
    }

    /// Returns image assets used by this component.
    fn image_assets(&self) -> Vec<ImageAsset> {
        Vec::new()
    }

    /// Returns vector assets used by this component.
    fn vector_assets(&self) -> Vec<VectorAsset> {
        Vec::new()
    }

    /// Returns custom GPU effect definitions used by this component.
    ///
    /// The host registers these definitions before drawing the returned UI
    /// tree and refreshes them when the component requests a rebuild.
    fn effect_definitions(&self) -> Vec<EffectDefinition> {
        Vec::new()
    }

    /// Returns the component's inspection handle, if available.
    fn inspector(&self) -> Option<InspectorHandle> {
        None
    }
}
