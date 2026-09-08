use argui_core::{Affine2D, Rect, Size};
use argui_paint::{ClipChain, DisplayList, Quad, QuadStyle};
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
    fn available(&self) -> CustomConstraints;
    fn known_size(&self) -> CustomConstraints;
    fn child_count(&self) -> usize;
    fn measure_child(
        &mut self,
        index: usize,
        available: CustomConstraints,
    ) -> Result<CustomMeasurement, String>;
    fn place_child(&mut self, index: usize, bounds: Rect) -> Result<(), String>;
}

/// Extension properties are immutable. Scratch state belongs to a retained layout
/// node, not to this description or to a shared application model.
pub trait CustomElement: fmt::Debug + 'static {
    type State: 'static;
    fn create_state(&self) -> Self::State;
    fn layout_revision(&self) -> u64;
    fn paint_revision(&self) -> u64;
    fn prepare(&self, state: &mut Self::State, size: Size);
    fn layout(
        &self,
        state: &mut Self::State,
        context: &mut dyn CustomLayoutContext,
    ) -> Result<CustomMeasurement, String>;
    fn paint(&self, state: &mut Self::State, context: &mut CustomPaintContext<'_>);
}

/// A clipped, renderer-independent painting surface. Coordinates are local to
/// the element. The host supplies ancestor transforms, clips and effect layers.
pub struct CustomPaintContext<'a> {
    pub bounds: Rect,
    pub transform: Affine2D,
    pub clips: &'a ClipChain,
    pub display_list: &'a mut DisplayList,
}

impl CustomPaintContext<'_> {
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
    pub fn same_type(&self, other: &Self) -> bool {
        self.0.kind() == other.0.kind()
    }
    pub fn same_layout(&self, other: &Self) -> bool {
        self.same_type(other) && self.0.revisions().0 == other.0.revisions().0
    }
    pub fn type_name(&self) -> &'static str {
        self.0.name()
    }
    pub fn create_state(&self) -> CustomState {
        CustomState {
            stats: Cell::new(CustomPhaseStats::default()),
            prepared: Cell::new(None),
            kind: self.0.kind(),
            value: RefCell::new(self.0.state()),
        }
    }
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
    pub fn paint(&self, state: &CustomState, context: &mut CustomPaintContext<'_>) {
        assert_eq!(
            state.kind,
            self.0.kind(),
            "custom state belongs to another element type"
        );
        let (layout, paint) = self.0.revisions();
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
        self.0.paint(state.value.borrow_mut().as_mut(), context);
    }
}

pub struct CustomState {
    stats: Cell<CustomPhaseStats>,
    prepared: Cell<Option<(u64, u64, Size)>>,
    kind: TypeId,
    value: RefCell<Box<dyn Any>>,
}
impl CustomState {
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

    pub fn custom_container(
        properties: impl CustomElement,
        children: impl IntoIterator<Item = Self>,
    ) -> Self {
        let mut element = Self::custom(properties);
        element.children = children.into_iter().collect();
        element
    }
    pub fn custom(properties: impl CustomElement) -> Self {
        let mut element = Self::container([]);
        element.kind = crate::ElementKind::Custom(CustomDescription(Rc::new(properties)));
        element
    }
}
