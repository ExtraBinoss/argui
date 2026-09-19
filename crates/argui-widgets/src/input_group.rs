use crate::WidgetTheme;
use argui_paint::{Border, BorderWidths, CornerRadii};
use argui_ui::{AlignItems, Element, Role, Semantics, length, percent};

/// One input surface with optional leading/trailing content. Controls keep their keys and focus.
#[derive(Clone, Debug)]
pub struct InputGroup {
    pub key: String,
    pub label: String,
    pub input: Element,
    pub leading: Option<Element>,
    pub trailing: Option<Element>,
    pub invalid: bool,
}

impl InputGroup {
    /// Creates an input group identified by `key`, named `label`, and containing `input`.
    #[must_use]
    pub fn new(key: impl Into<String>, label: impl Into<String>, input: Element) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            input,
            leading: None,
            trailing: None,
            invalid: false,
        }
    }

    #[must_use]
    /// Builds the group using `theme` for its label styling.
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let Self {
            key,
            label,
            mut input,
            leading,
            trailing,
            invalid,
        } = self;
        if input.paint.quad.is_visible() {
            input.override_border_widths(BorderWidths::all(0.0));
            input.override_background(None);
            input = input.radius(CornerRadii::all(0.0));
        }
        Element::row(
            leading
                .into_iter()
                .chain([input.grow(1.0).shrink(1.0).min_width(length(0.0))])
                .chain(trailing),
        )
        .keyed(key)
        .width(percent(1.0))
        .height(length(36.0))
        .gap(6.0)
        .padding(argui_ui::sides(4.0, 0.0))
        .align_items(AlignItems::CENTER)
        .background(theme.card)
        .border(Border::all(
            1.0,
            if invalid {
                theme.destructive
            } else {
                theme.input_border
            },
        ))
        .radius(CornerRadii::all(8.0))
        .semantics(Semantics::new(Role::Group).label(label))
    }
}
