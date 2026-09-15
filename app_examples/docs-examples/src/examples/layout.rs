use argui::{
    paint::CornerRadii,
    runtime::{Context, LayoutSnapshot, Render},
    text::TextStyle,
    ui::{Element, FlexWrap, Sides, length, percent},
    widgets::default_theme,
};

#[derive(Default)]
pub struct Example {
    compact: bool,
}

impl Render for Example {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = default_theme(cx.environment());
        let theme = themes.resolve(cx.environment().color_scheme);
        let width = if self.compact {
            percent(1.0)
        } else {
            length(180.0)
        };
        let card = |label| {
            Element::text(label)
                .text_style(TextStyle {
                    color: theme.primary_foreground,
                    weight: 650,
                    ..TextStyle::default()
                })
                .width(width)
                .grow(1.0)
                .padding(Sides::length(22.0))
                .background(theme.primary)
                .radius(CornerRadii::all(12.0))
        };
        Element::row([card("Flexible"), card("Responsive"), card("Retained")])
            .width(percent(1.0))
            .padding(Sides::length(24.0))
            .gap(12.0)
            .flex_wrap(FlexWrap::Wrap)
            .background(theme.background)
    }

    fn layout_changed(&mut self, layout: &LayoutSnapshot, cx: &mut Context<Self>) {
        let compact = layout.viewport_size().width < 540.0;
        if self.compact != compact {
            self.compact = compact;
            cx.notify();
        }
    }
}
