use argui_core::{Affine2D, Rect, Size};
use argui_paint::{ClipChain, DisplayList, GpuCanvasPrimitive, Quad, QuadStyle, RenderObjectId};
use std::{
    any::{Any, TypeId},
    cell::{Cell, RefCell},
    fmt,
    rc::Rc,
};

/// Space offered by the layout engine, in logical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CustomConstraints {
    pub width: Option<f32>,
    pub height: Option<f32>,
}

/// Intrinsic size and optional first baseline in logical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CustomMeasurement {
    pub size: Size,
    pub baseline: Option<f32>,
}

/// Cumulative work on one retained element; sampling never invokes a phase.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CustomPhaseStats {
    pub layouts: u64,
    pub preparations: u64,
    pub paints: u64,
    pub available: Option<CustomConstraints>,
    pub known_size: Option<CustomConstraints>,
}

/// Layout-only access to ordinary child nodes. Placements are local border boxes.
/// No model mutation, event dispatch or renderer access is available in this phase.
pub trait CustomLayoutContext {
    /// Returns the maximum width and height available to the custom element.
    fn available(&self) -> CustomConstraints;
    /// Returns the dimensions already determined by the surrounding layout.
    fn known_size(&self) -> CustomConstraints;
    /// Returns the number of ordinary child elements.
    fn child_count(&self) -> usize;
    /// Measures a child under the supplied constraints.
    ///
    /// # Arguments
    ///
    /// * `index` — zero-based child index.
    /// * `available` — maximum width and height to offer that child.
    ///
    /// # Errors
    ///
    /// Returns an error if `index` does not identify a child or measurement fails.
    fn measure_child(
        &mut self,
        index: usize,
        available: CustomConstraints,
    ) -> Result<CustomMeasurement, String>;
    /// Assigns a child a local border box within this element.
    ///
    /// # Arguments
    ///
    /// * `index` — zero-based child index.
    /// * `bounds` — child border box in the element's local coordinates.
    ///
    /// # Errors
    ///
    /// Returns an error if `index` does not identify a child or placement fails.
    fn place_child(&mut self, index: usize, bounds: Rect) -> Result<(), String>;
}

/// Extension properties are immutable. Scratch state belongs to a retained layout
/// node, not to this description or to a shared application model.
pub trait CustomElement: fmt::Debug + 'static {
    /// Scratch data retained for one custom element in a UI tree.
    type State: 'static;
    /// Creates the retained scratch state for this custom element.
    fn create_state(&self) -> Self::State;
    /// Returns a revision that changes whenever layout-affecting properties change.
    fn layout_revision(&self) -> u64;
    /// Returns a revision that changes whenever paint-affecting properties change.
    fn paint_revision(&self) -> u64;
    /// Prepares retained state for the element's current size before painting.
    fn prepare(&self, state: &mut Self::State, size: Size);
    /// Measures and places this element's children.
    ///
    /// # Errors
    ///
    /// Returns an error if custom layout cannot be completed.
    ///
    /// * `state` — retained custom state used during layout.
    /// * `context` — host-provided child measurement and placement interface.
    fn layout(
        &self,
        state: &mut Self::State,
        context: &mut dyn CustomLayoutContext,
    ) -> Result<CustomMeasurement, String>;
    /// Adds this element's custom painting to the supplied paint context.
    ///
    /// * `state` — retained custom state used during painting.
    fn paint(&self, state: &mut Self::State, context: &mut CustomPaintContext<'_>);
}

/// A clipped, renderer-independent painting surface. Coordinates are local to
/// the element. The host supplies ancestor transforms, clips and effect layers.
pub struct CustomPaintContext<'a> {
    pub bounds: Rect,
    pub transform: Affine2D,
    pub clips: &'a ClipChain,
    pub display_list: &'a mut DisplayList,
    /// Retained identity shared by primitives emitted from this custom node.
    pub object: RenderObjectId,
    /// Resolved paint opacity of the custom node.
    pub opacity: f32,
    /// Resolved rounded corners of the custom node.
    pub radii: argui_paint::CornerRadii,
}

impl CustomPaintContext<'_> {
    /// Adds a quad in element-local coordinates to the display list.
    ///
    /// * `bounds` — local rectangle for the quad.
    /// * `style` — paint style applied to the rectangle.
    pub fn quad(&mut self, mut bounds: Rect, style: QuadStyle) {
        bounds.origin.x += self.bounds.origin.x;
        bounds.origin.y += self.bounds.origin.y;
        self.display_list.push_quad(Quad {
            bounds,
            background: style.background,
            border: style.border.unwrap_or(argui_paint::Border::all(
                0.0,
                argui_paint::Color::TRANSPARENT,
            )),
            radii: style.radii,
            opacity: style.opacity,
            transform: self.transform,
            clips: self.clips.clone(),
        });
    }

    /// Adds a retained GPU canvas in element-local coordinates.
    ///
    /// `slot` distinguishes multiple canvases emitted by the same retained
    /// custom node. Reusing a slot in one frame is diagnosed by the renderer.
    /// `bounds` is translated by the custom element's content-box origin.
    pub fn gpu_canvas(&mut self, slot: u32, mut bounds: Rect, spec: crate::GpuCanvasSpec) {
        bounds.origin.x += self.bounds.origin.x;
        bounds.origin.y += self.bounds.origin.y;
        self.display_list.push_gpu_canvas(GpuCanvasPrimitive {
            canvas: spec.canvas(),
            object: self.object,
            slot,
            bounds,
            content_revision: spec.revision(),
            resolution_scale: spec.scale(),
            sampling: spec.image_sampling(),
            opacity: self.opacity,
            radii: self.radii,
            transform: self.transform,
            clips: self.clips.clone(),
        });
    }
}

trait Erased: fmt::Debug {
    fn kind(&self) -> TypeId;
    fn name(&self) -> &'static str;
    fn revisions(&self) -> (u64, u64);
    fn state(&self) -> Box<dyn Any>;
    fn prepare(&self, state: &mut dyn Any, size: Size);
    fn layout(
        &self,
        state: &mut dyn Any,
        context: &mut dyn CustomLayoutContext,
    ) -> Result<CustomMeasurement, String>;
    fn paint(&self, state: &mut dyn Any, context: &mut CustomPaintContext<'_>);
}

impl<T: CustomElement> Erased for T {
    fn kind(&self) -> TypeId {
        TypeId::of::<T>()
    }
    fn name(&self) -> &'static str {
        std::any::type_name::<T>()
    }
    fn revisions(&self) -> (u64, u64) {
        (self.layout_revision(), self.paint_revision())
    }
    fn state(&self) -> Box<dyn Any> {
        Box::new(self.create_state())
    }
    fn prepare(&self, state: &mut dyn Any, size: Size) {
        CustomElement::prepare(
            self,
            state.downcast_mut::<T::State>().expect("custom state type"),
            size,
        );
    }
    fn layout(
        &self,
        state: &mut dyn Any,
        context: &mut dyn CustomLayoutContext,
    ) -> Result<CustomMeasurement, String> {
        CustomElement::layout(
            self,
            state.downcast_mut::<T::State>().expect("custom state type"),
            context,
        )
    }
    fn paint(&self, state: &mut dyn Any, context: &mut CustomPaintContext<'_>) {
        CustomElement::paint(
            self,
            state.downcast_mut::<T::State>().expect("custom state type"),
            context,
        );
    }
}

#[derive(Clone, Debug)]
pub struct CustomDescription(Rc<dyn Erased>);

impl PartialEq for CustomDescription {
    fn eq(&self, other: &Self) -> bool {
        self.same_type(other) && self.0.revisions() == other.0.revisions()
    }
}

impl CustomDescription {
    /// Returns whether both descriptions wrap the same concrete element type.
    /// * `other` — description to compare with this one.
    pub fn same_type(&self, other: &Self) -> bool {
        self.0.kind() == other.0.kind()
    }
    /// Returns whether these descriptions have identical layout revisions.
    /// * `other` — description to compare with this one.
    pub fn same_layout(&self, other: &Self) -> bool {
        self.same_type(other) && self.0.revisions().0 == other.0.revisions().0
    }
    /// Returns the concrete Rust type name of the custom element.
    pub fn type_name(&self) -> &'static str {
        self.0.name()
    }
    /// Creates fresh retained scratch state for this custom element.
    pub fn create_state(&self) -> CustomState {
        CustomState {
            stats: Cell::new(CustomPhaseStats::default()),
            paint_cache: RefCell::new(None),
            prepared: Cell::new(None),
            kind: self.0.kind(),
            value: RefCell::new(self.0.state()),
        }
    }
    /// Measures and places this custom element's children.
    ///
    /// # Arguments
    ///
    /// * `state` — retained state belonging to this custom element.
    /// * `context` — access to child measurement and placement operations.
    ///
    /// # Errors
    ///
    /// Returns the error produced by the custom element's layout implementation.
    ///
    /// # Panics
    ///
    /// Panics if `state` was created for a different custom element type.
    pub fn layout(
        &self,
        state: &CustomState,
        context: &mut dyn CustomLayoutContext,
    ) -> Result<CustomMeasurement, String> {
        assert_eq!(
            state.kind,
            self.0.kind(),
            "custom state belongs to another element type"
        );
        let mut stats = state.stats.get();
        stats.layouts += 1;
        stats.available = Some(context.available());
        stats.known_size = Some(context.known_size());
        state.stats.set(stats);
        self.0.layout(state.value.borrow_mut().as_mut(), context)
    }
    /// Paints the custom element, reusing cached commands when its inputs are unchanged.
    ///
    /// # Arguments
    ///
    /// * `state` — retained state belonging to this custom element.
    /// * `context` — clipped, renderer-independent painting surface.
    ///
    /// # Panics
    ///
    /// Panics if `state` was created for a different custom element type.
    pub fn paint(&self, state: &CustomState, context: &mut CustomPaintContext<'_>) {
        assert_eq!(
            state.kind,
            self.0.kind(),
            "custom state belongs to another element type"
        );
        let (layout, paint) = self.0.revisions();
        let mut cache = state.paint_cache.borrow_mut();
        if let Some(cached) = cache.as_ref()
            && cached.revisions == (layout, paint)
            && cached.bounds == context.bounds
            && cached.transform == context.transform
            && cached.clips == *context.clips
            && cached.object == context.object
            && cached.opacity == context.opacity
            && cached.radii == context.radii
        {
            context
                .display_list
                .extend(cached.commands.commands().iter().cloned());
            return;
        }
        let key = (layout, paint, context.bounds.size);
        if state.prepared.get() != Some(key) {
            self.0
                .prepare(state.value.borrow_mut().as_mut(), context.bounds.size);
            state.prepared.set(Some(key));
            let mut stats = state.stats.get();
            stats.preparations += 1;
            state.stats.set(stats);
        }
        let mut stats = state.stats.get();
        stats.paints += 1;
        state.stats.set(stats);
        let mut commands = cache
            .take()
            .map_or_else(DisplayList::new, |cached| cached.commands);
        commands.clear();
        self.0.paint(
            state.value.borrow_mut().as_mut(),
            &mut CustomPaintContext {
                bounds: context.bounds,
                transform: context.transform,
                clips: context.clips,
                display_list: &mut commands,
                object: context.object,
                opacity: context.opacity,
                radii: context.radii,
            },
        );
        context
            .display_list
            .extend(commands.commands().iter().cloned());
        *cache = Some(CachedCustomPaint {
            revisions: (layout, paint),
            bounds: context.bounds,
            transform: context.transform,
            clips: context.clips.clone(),
            object: context.object,
            opacity: context.opacity,
            radii: context.radii,
            commands,
        });
    }
}

// One bounded fragment per custom node survives a parent fragment's invalidation.
struct CachedCustomPaint {
    revisions: (u64, u64),
    bounds: Rect,
    transform: Affine2D,
    clips: ClipChain,
    object: RenderObjectId,
    opacity: f32,
    radii: argui_paint::CornerRadii,
    commands: DisplayList,
}

pub struct CustomState {
    paint_cache: RefCell<Option<CachedCustomPaint>>,
    stats: Cell<CustomPhaseStats>,
    prepared: Cell<Option<(u64, u64, Size)>>,
    kind: TypeId,
    value: RefCell<Box<dyn Any>>,
}
impl CustomState {
    /// Returns layout, preparation, and paint counts recorded for this state.
    pub fn phase_stats(&self) -> CustomPhaseStats {
        self.stats.get()
    }
}
impl fmt::Debug for CustomState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CustomState")
            .field("type", &self.kind)
            .finish_non_exhaustive()
    }
}

impl crate::Element {
    /// Declares a retained sub-region for a custom container to place. Regions
    /// use ordinary UI identities, listeners, focus, gestures and semantics;
    /// they do not create a second hit-test or accessibility tree. Paint is
    /// transparent until supplied by the caller or by the enclosing element.
    ///
    /// * `key` — stable key for this retained sub-region.
    /// * `interaction` — pointer, focus, keyboard, and gesture behavior.
    pub fn custom_region(
        key: impl Into<String>,
        interaction: crate::Interaction,
        semantics: argui_accessibility::Semantics,
    ) -> Self {
        Self::container([])
            .keyed(key)
            .interaction(interaction)
            .semantics(semantics)
            .user_select(crate::UserSelect::None)
    }

    /// Creates a custom container with ordinary child elements.
    ///
    /// * `properties` — immutable custom behavior and properties.
    /// * `children` — ordinary child elements placed by the custom layout.
    pub fn custom_container(
        properties: impl CustomElement,
        children: impl IntoIterator<Item = Self>,
    ) -> Self {
        let mut element = Self::custom(properties);
        element.children = children.into_iter().collect();
        element
    }
    /// Creates a custom element without ordinary child elements.
    ///
    /// * `properties` — immutable custom behavior and properties.
    pub fn custom(properties: impl CustomElement) -> Self {
        let mut element = Self::container([]);
        element.kind = crate::ElementKind::Custom(CustomDescription(Rc::new(properties)));
        element
    }
}
