use argui_paint::{Border, CornerRadii, PaintStyle, QuadStyle};
use argui_ui::{AlignItems, Display, Element, JustifyContent, Sides, length, percent};

use crate::{DialogBehavior, DialogPart, WidgetTheme};

#[derive(Clone, Debug)]
pub struct Dialog {
    key: String,
    label: String,
    open: bool,
    trigger: Element,
    content: Element,
}

impl Dialog {
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        open: bool,
        trigger: Element,
        content: Element,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            open,
            trigger,
            content,
        }
    }

    #[must_use]
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let behavior = DialogBehavior::new(&self.key, &self.label, self.open);
        let trigger = behavior.decorate(DialogPart::Trigger, self.trigger);
        let overlay = self.open.then(|| {
            let backdrop = behavior.decorate(
                DialogPart::Backdrop,
                Element::container([])
                    .absolute(Sides::length(0.0))
                    .width(percent(1.0))
                    .height(percent(1.0))
                    .background(argui_core::Color::rgba(0.0, 0.0, 0.0, 0.52)),
            );
            let panel = behavior.decorate(
                DialogPart::Panel,
                Element::column([self.content])
                    .width(length(480.0))
                    .max_width(percent(0.90))
                    .padding(Sides::length(24.0))
                    .gap(18.0)
                    .paint_style(PaintStyle::new(
                        QuadStyle::solid(theme.popover)
                            .border(Border::all(1.0, theme.border))
                            .radius(CornerRadii::all(12.0)),
                    )),
            );
            behavior.decorate(
                DialogPart::Overlay,
                Element::container([backdrop, panel])
                    .absolute(Sides::length(0.0))
                    .width(percent(1.0))
                    .height(percent(1.0))
                    .display(Display::Flex)
                    .align_items(AlignItems::CENTER)
                    .justify_content(JustifyContent::CENTER)
                    .z_index(2_000),
            )
        });
        behavior.decorate(
            DialogPart::Root,
            Element::container(std::iter::once(trigger).chain(overlay)),
        )
    }
}
