//! CSS-shaped layout values consumed by the Taffy adapter.

pub use taffy::style_helpers::{
    auto, evenly_sized_tracks, flex, fr, length, line, minmax, percent, repeat, span, zero,
};
pub use taffy::{
    AlignContent, AlignItems, AlignSelf, AlignmentSafety, BoxSizing, Dimension, Display,
    ExpandedDimension, ExpandedLengthPercentage, ExpandedLengthPercentageAuto, FlexDirection,
    FlexWrap, GridAutoFlow, GridPlacement, GridTemplateArea, GridTemplateAreas,
    GridTemplateComponent, GridTemplateRepetition, JustifyContent, JustifyItems, JustifySelf,
    LengthPercentage, LengthPercentageAuto, MaxTrackSizingFunction, MinTrackSizingFunction,
    RepetitionCount, TrackSizingFunction, geometry::Line, style::Direction as WritingDirection,
};

pub type Sides<T> = taffy::geometry::Rect<T>;
pub type Dimensions<T> = taffy::geometry::Size<T>;
pub type Axes<T> = taffy::geometry::Point<T>;

/// Logical pixel insets resolved against the effective writing direction.
///
/// Physical sides remain available for ordinary layout. In Rust builders,
/// `start` and `end` replace the corresponding physical side after direction
/// inheritance. The TSX wire rejects a value that mixes logical and physical
/// horizontal sides, so one authored object has one unambiguous convention.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LayoutInsets {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
    pub start: Option<f32>,
    pub end: Option<f32>,
}

impl LayoutInsets {
    /// Creates equal physical insets on all four sides in logical pixels.
    #[must_use]
    pub const fn all(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
            start: None,
            end: None,
        }
    }

    /// Returns logical horizontal sides mapped to physical left and right.
    ///
    /// `direction` is the inherited writing direction of the target element.
    /// `None` leaves the corresponding physical side unchanged.
    #[must_use]
    pub(crate) const fn horizontal(
        self,
        direction: WritingDirection,
    ) -> (Option<f32>, Option<f32>) {
        match direction {
            WritingDirection::Ltr => (self.start, self.end),
            WritingDirection::Rtl => (self.end, self.start),
        }
    }
}

/// Positioned edges in logical pixels, preserving absent sides as `auto`.
///
/// Unlike padding and margin, an unspecified inset must not pin an absolute
/// element to that side of its containing box.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PositionInsets {
    pub top: Option<f32>,
    pub right: Option<f32>,
    pub bottom: Option<f32>,
    pub left: Option<f32>,
    pub start: Option<f32>,
    pub end: Option<f32>,
}

impl PositionInsets {
    /// Maps logical horizontal edges to physical sides in `direction`.
    /// Missing edges remain unset, so the layout engine can retain `auto`.
    #[must_use]
    pub(crate) const fn horizontal(
        self,
        direction: WritingDirection,
    ) -> (Option<f32>, Option<f32>) {
        match direction {
            WritingDirection::Ltr => (self.start, self.end),
            WritingDirection::Rtl => (self.end, self.start),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Position {
    #[default]
    Relative,
    Absolute,
    Sticky,
}

/// Creates equal horizontal and vertical inset sides from logical lengths.
///
/// * `horizontal` — length used for the left and right sides.
/// * `vertical` — length used for the top and bottom sides.
#[must_use]
pub fn sides<T>(horizontal: f32, vertical: f32) -> Sides<T>
where
    T: taffy::style_helpers::FromLength,
{
    Sides {
        left: length(horizontal),
        right: length(horizontal),
        top: length(vertical),
        bottom: length(vertical),
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Overflow {
    #[default]
    Visible,
    Clip,
    Hidden,
    Auto,
    Scroll,
}

impl Overflow {
    /// Returns whether this overflow mode clips content to its bounds.
    #[must_use]
    pub const fn clips(self) -> bool {
        !matches!(self, Self::Visible)
    }

    /// Returns whether this overflow mode enables scroll input.
    #[must_use]
    pub const fn scrolls(self) -> bool {
        matches!(self, Self::Auto | Self::Scroll)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ScrollbarGutter {
    #[default]
    Auto,
    Stable,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayoutStyle {
    pub display: Display,
    pub box_sizing: BoxSizing,
    pub writing_direction: WritingDirection,
    pub overflow: Axes<Overflow>,
    pub scrollbar_gutter: ScrollbarGutter,
    pub scrollbar_width: f32,
    pub position: Position,
    pub inset: Sides<LengthPercentageAuto>,
    pub size: Dimensions<Dimension>,
    pub min_size: Dimensions<LengthPercentageAuto>,
    pub max_size: Dimensions<LengthPercentageAuto>,
    pub aspect_ratio: Option<f32>,
    pub margin: Sides<LengthPercentageAuto>,
    pub padding: Sides<LengthPercentage>,
    /// Logical horizontal padding awaiting inherited direction resolution.
    pub logical_padding: Option<LayoutInsets>,
    /// Logical horizontal margin awaiting inherited direction resolution.
    pub logical_margin: Option<LayoutInsets>,
    /// Logical horizontal position insets awaiting inherited direction resolution.
    pub logical_inset: Option<PositionInsets>,
    pub border: Sides<LengthPercentage>,
    pub align_items: Option<AlignItems>,
    pub align_self: Option<AlignSelf>,
    pub justify_items: Option<JustifyItems>,
    pub justify_self: Option<JustifySelf>,
    pub align_content: Option<AlignContent>,
    pub justify_content: Option<JustifyContent>,
    pub gap: Dimensions<LengthPercentage>,
    pub flex_direction: FlexDirection,
    pub flex_wrap: FlexWrap,
    pub flex_basis: Dimension,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub grid_template_rows: Vec<GridTemplateComponent<String>>,
    pub grid_template_columns: Vec<GridTemplateComponent<String>>,
    pub grid_auto_rows: Vec<TrackSizingFunction>,
    pub grid_auto_columns: Vec<TrackSizingFunction>,
    pub grid_auto_flow: GridAutoFlow,
    pub grid_template_areas: Option<GridTemplateAreas<String>>,
    pub grid_template_column_names: Vec<Vec<String>>,
    pub grid_template_row_names: Vec<Vec<String>>,
    pub grid_row: Line<GridPlacement<String>>,
    pub grid_column: Line<GridPlacement<String>>,
}

impl Default for LayoutStyle {
    fn default() -> Self {
        Self {
            display: Display::Block,
            box_sizing: BoxSizing::BorderBox,
            writing_direction: WritingDirection::Ltr,
            overflow: Axes {
                x: Overflow::Visible,
                y: Overflow::Visible,
            },
            scrollbar_gutter: ScrollbarGutter::Auto,
            scrollbar_width: 12.0,
            position: Position::Relative,
            inset: auto_sides(),
            size: auto_dimensions(),
            min_size: auto_dimensions(),
            max_size: auto_dimensions(),
            aspect_ratio: None,
            margin: zero_auto_sides(),
            padding: zero_sides(),
            logical_padding: None,
            logical_margin: None,
            logical_inset: None,
            border: zero_sides(),
            align_items: None,
            align_self: None,
            justify_items: None,
            justify_self: None,
            align_content: None,
            justify_content: None,
            gap: zero_dimensions(),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::NoWrap,
            flex_basis: Dimension::auto(),
            flex_grow: 0.0,
            flex_shrink: 1.0,
            grid_template_rows: Vec::new(),
            grid_template_columns: Vec::new(),
            grid_auto_rows: Vec::new(),
            grid_auto_columns: Vec::new(),
            grid_auto_flow: GridAutoFlow::Row,
            grid_template_areas: None,
            grid_template_column_names: Vec::new(),
            grid_template_row_names: Vec::new(),
            grid_row: Line {
                start: GridPlacement::Auto,
                end: GridPlacement::Auto,
            },
            grid_column: Line {
                start: GridPlacement::Auto,
                end: GridPlacement::Auto,
            },
        }
    }
}

fn auto_sides() -> Sides<LengthPercentageAuto> {
    Sides {
        left: LengthPercentageAuto::auto(),
        right: LengthPercentageAuto::auto(),
        top: LengthPercentageAuto::auto(),
        bottom: LengthPercentageAuto::auto(),
    }
}

fn zero_sides() -> Sides<LengthPercentage> {
    Sides {
        left: LengthPercentage::length(0.0),
        right: LengthPercentage::length(0.0),
        top: LengthPercentage::length(0.0),
        bottom: LengthPercentage::length(0.0),
    }
}

fn zero_auto_sides() -> Sides<LengthPercentageAuto> {
    Sides {
        left: LengthPercentageAuto::length(0.0),
        right: LengthPercentageAuto::length(0.0),
        top: LengthPercentageAuto::length(0.0),
        bottom: LengthPercentageAuto::length(0.0),
    }
}

fn auto_dimensions<T>() -> Dimensions<T>
where
    T: AutoValue,
{
    Dimensions {
        width: T::auto(),
        height: T::auto(),
    }
}

fn zero_dimensions() -> Dimensions<LengthPercentage> {
    Dimensions {
        width: LengthPercentage::length(0.0),
        height: LengthPercentage::length(0.0),
    }
}

trait AutoValue {
    fn auto() -> Self;
}

impl AutoValue for Dimension {
    fn auto() -> Self {
        Self::auto()
    }
}

impl AutoValue for LengthPercentageAuto {
    fn auto() -> Self {
        Self::auto()
    }
}
