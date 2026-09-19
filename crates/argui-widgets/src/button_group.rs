use argui_paint::CornerRadii;
use argui_text::{TextStyle, TextWrap};
use argui_ui::{
    AlignItems, AlignSelf, Element, Orientation, Role, Semantics, length, percent, sides,
};

use crate::WidgetTheme;

/// A named group of independently tabbable buttons and other controls.
#[derive(Clone, Debug)]
pub struct ButtonGroup {
    pub key: String,
    pub label: String,
    pub orientation: Orientation,
    pub spacing: f32,
    pub children: Vec<Element>,
}

impl ButtonGroup {
    /// Creates a horizontal group with a semantic `label` and ordered `children`.
    ///
    /// `key` identifies the group, `label` names it to assistive technology, and `children`
    /// supplies its controls in display order.
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        children: impl IntoIterator<Item = Element>,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            orientation: Orientation::Horizontal,
            spacing: 0.0,
            children: children.into_iter().collect(),
        }
    }

    /// Sets whether children are joined horizontally or vertically.
    ///
    /// `orientation` controls both layout and the edges shared by adjacent
    /// controls.
    #[must_use]
    pub const fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Adds space between children instead of placing their joined edges flush.
    ///
    /// `spacing` is measured in logical pixels and is clamped to zero. Nested
    /// button groups can use this to form distinct action clusters.
    #[must_use]
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing.max(0.0);
        self
    }

    #[must_use]
    /// Builds the group in its configured orientation.
    pub fn build(mut self) -> Element {
        if self.spacing == 0.0 {
            let count = self.children.len();
            for (index, child) in self.children.iter_mut().enumerate() {
                join_control(child, self.orientation, index, count);
            }
        }
        let root = match self.orientation {
            Orientation::Horizontal => Element::row(self.children),
            Orientation::Vertical => Element::column(self.children),
        };
        root.keyed(self.key)
            .gap(self.spacing)
            .align_items(AlignItems::STRETCH)
            .align_self(AlignSelf::START)
            .semantics(Semantics::new(Role::Group).label(self.label))
    }
}

fn join_control(child: &mut Element, orientation: Orientation, index: usize, count: usize) {
    let Some(child) = control_surface(child) else {
        return;
    };
    let mut radii = child.paint.quad.radii;
    if count > 1 {
        match orientation {
            Orientation::Horizontal => {
                if index > 0 {
                    radii.top_left = 0.0;
                    radii.bottom_left = 0.0;
                }
                if index + 1 < count {
                    radii.top_right = 0.0;
                    radii.bottom_right = 0.0;
                }
            }
            Orientation::Vertical => {
                if index > 0 {
                    radii.top_left = 0.0;
                    radii.top_right = 0.0;
                }
                if index + 1 < count {
                    radii.bottom_left = 0.0;
                    radii.bottom_right = 0.0;
                }
            }
        }
    }
    *child = child.clone().radius(radii);
    let Some(border) = child.paint.quad.border.as_ref() else {
        return;
    };
    let mut widths = border.widths;
    if index > 0 {
        match orientation {
            Orientation::Horizontal => widths.left = 0.0,
            Orientation::Vertical => widths.top = 0.0,
        }
    }
    child.override_border_widths(widths);
}

/// Finds the first painted control inside structural overlay wrappers.
fn control_surface(element: &mut Element) -> Option<&mut Element> {
    if element.paint.quad.is_visible() {
        return Some(element);
    }
    element.children.iter_mut().find_map(control_surface)
}

/// A themed divider placed between controls in a [`ButtonGroup`].
#[derive(Clone, Debug)]
pub struct ButtonGroupSeparator {
    key: String,
    orientation: Orientation,
}

impl ButtonGroupSeparator {
    /// Creates a vertical divider suitable for a horizontal button group.
    ///
    /// `key` identifies the divider in the retained tree.
    #[must_use]
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            orientation: Orientation::Vertical,
        }
    }

    /// Sets the direction of the divider line.
    ///
    /// `orientation` is normally vertical in a horizontal group and horizontal
    /// in a vertical group.
    #[must_use]
    pub const fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Builds the decorative divider using `theme` for its color.
    #[must_use]
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let line = Element::container([])
            .keyed(self.key)
            .background(theme.border)
            .shrink(0.0)
            .semantic_hidden(true);
        match self.orientation {
            Orientation::Horizontal => line.width(percent(1.0)).height(length(1.0)),
            Orientation::Vertical => line.width(length(1.0)).align_self(AlignSelf::STRETCH),
        }
    }
}

/// Non-interactive text that shares the joined surface of a [`ButtonGroup`].
#[derive(Clone, Debug)]
pub struct ButtonGroupText {
    key: String,
    text: String,
}

impl ButtonGroupText {
    /// Creates a text segment identified by `key` and displaying `text`.
    #[must_use]
    pub fn new(key: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            text: text.into(),
        }
    }

    /// Builds the segment using the group's standard control height and `theme`.
    #[must_use]
    pub fn build(self, theme: &WidgetTheme) -> Element {
        Element::row([Element::text(self.text).text_style(TextStyle {
            font_size: 14.0,
            line_height: 20.0,
            color: theme.muted_foreground,
            wrap: TextWrap::None,
            ..TextStyle::default()
        })])
        .keyed(self.key)
        .height(length(36.0))
        .padding(sides(12.0, 0.0))
        .align_items(AlignItems::CENTER)
        .background(theme.card)
        .border(argui_paint::Border::all(1.0, theme.border))
        .radius(CornerRadii::all(7.0))
        .semantic_hidden(false)
    }
}
