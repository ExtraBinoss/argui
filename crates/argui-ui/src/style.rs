#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Length {
    #[default]
    Auto,
    Px(f32),
    Percent(f32),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Direction {
    Row,
    #[default]
    Column,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Wrap {
    #[default]
    NoWrap,
    Wrap,
    Reverse,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Align {
    Start,
    Center,
    End,
    #[default]
    Stretch,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Justify {
    #[default]
    Start,
    Center,
    End,
    SpaceBetween,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Edges {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

impl Edges {
    #[must_use]
    pub const fn all(value: f32) -> Self {
        Self {
            left: value,
            right: value,
            top: value,
            bottom: value,
        }
    }

    #[must_use]
    pub const fn symmetric(horizontal: f32, vertical: f32) -> Self {
        Self {
            left: horizontal,
            right: horizontal,
            top: vertical,
            bottom: vertical,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayoutStyle {
    pub width: Length,
    pub height: Length,
    pub min_width: Length,
    pub min_height: Length,
    pub max_width: Length,
    pub max_height: Length,
    pub direction: Direction,
    pub wrap: Wrap,
    pub align: Align,
    pub justify: Justify,
    pub padding: Edges,
    pub gap: f32,
    pub grow: f32,
    pub shrink: f32,
}

impl Default for LayoutStyle {
    fn default() -> Self {
        Self {
            width: Length::Auto,
            height: Length::Auto,
            min_width: Length::Auto,
            min_height: Length::Auto,
            max_width: Length::Auto,
            max_height: Length::Auto,
            direction: Direction::Column,
            wrap: Wrap::NoWrap,
            align: Align::Stretch,
            justify: Justify::Start,
            padding: Edges::default(),
            gap: 0.0,
            grow: 0.0,
            shrink: 1.0,
        }
    }
}
