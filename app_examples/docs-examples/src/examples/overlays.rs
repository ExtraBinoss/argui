use argui::{
    core::Color,
    paint::{Border, CornerRadii, PaintStyle, QuadStyle},
    runtime::{Context, Render},
    text::TextStyle,
    ui::{Axes, Element, FlexWrap, Overflow, ScrollConfig, Sides, auto, length, percent},
    widgets::{Button, Dialog, DialogBehavior, Popover, default_theme},
};

#[derive(Default)]
pub struct Example {
    dialog_open: bool,
    solid_open: bool,
    blurred_open: bool,
}

impl Render for Example {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = default_theme(cx.environment());
        let theme = themes.resolve(cx.environment().color_scheme);
        let text = |value: &str, size: f32, weight: u16, color: Color| {
            Element::text(value).text_style(TextStyle {
                color,
                font_size: size,
                weight,
                ..TextStyle::default()
            })
        };
        let preview = |overlay: Element| {
            let backdrop = Element::column([
                Element::container([])
                    .height(length(30.0))
                    .background(Color::from_srgb8(91, 115, 241)),
                Element::container([])
                    .height(length(30.0))
                    .background(Color::from_srgb8(216, 96, 165)),
                Element::container([])
                    .height(length(30.0))
                    .background(Color::from_srgb8(32, 168, 133)),
            ])
            .gap(8.0)
            .padding(Sides {
                top: length(68.0),
                ..Sides::length(16.0)
            });
            Element::container([
                backdrop,
                overlay.absolute(Sides {
                    top: length(18.0),
                    left: length(16.0),
                    right: auto(),
                    bottom: auto(),
                }),
            ])
            .height(length(198.0))
            .width(percent(1.0))
            .background(theme.muted)
            .border(Border::all(1.0, theme.border))
            .radius(CornerRadii::all(10.0))
        };
        let card = |title: &str, description: &str, overlay: Element| {
            Element::column([
                text(title, 16.0, 650, theme.foreground),
                text(description, 13.0, 400, theme.muted_foreground).min_height(length(38.0)),
                preview(overlay),
            ])
            .width(length(232.0))
            .max_width(percent(1.0))
            .grow(1.0)
            .gap(10.0)
        };

        let solid_content = Element::column([
            text("Solid surface", 18.0, 700, theme.foreground),
            text(
                "An opaque panel with backdrop blur disabled.",
                13.0,
                400,
                theme.muted_foreground,
            ),
            Button::new("close-solid", "Close solid popover", theme.outline_button())
                .on_click(cx.callback(|app| app.solid_open = false))
                .build(),
        ])
        .gap(12.0);
        let solid = Popover::new(
            "solid-popover",
            "Solid popover",
            self.solid_open,
            Button::new("solid-popover", "Open solid", theme.outline_button()).build(),
            solid_content,
        )
        .on_open_change(cx.value_callback(|app, open| app.solid_open = open))
        .backdrop_blur(0.0)
        .size(238.0, 220.0)
        .build(theme);

        let blurred_content = Element::column([
            text("Backdrop blur", 18.0, 700, theme.foreground),
            text(
                "Translucent paint keeps the colored backdrop visible through the blur.",
                13.0,
                400,
                theme.muted_foreground,
            ),
            Button::new(
                "close-blurred",
                "Close blurred popover",
                theme.outline_button(),
            )
            .on_click(cx.callback(|app| app.blurred_open = false))
            .build(),
        ])
        .gap(12.0);
        let blurred = Popover::new(
            "blurred-popover",
            "Blurred popover",
            self.blurred_open,
            Button::new("blurred-popover", "Open with blur", theme.outline_button()).build(),
            blurred_content,
        )
        .on_open_change(cx.value_callback(|app, open| app.blurred_open = open))
        .paint(PaintStyle::new(
            QuadStyle::solid(theme.popover.with_alpha(0.78))
                .border(Border::all(1.0, theme.popover_border))
                .radius(CornerRadii::all(8.0)),
        ))
        .backdrop_blur(12.0)
        .size(258.0, 240.0)
        .build(theme);

        let behavior = DialogBehavior::new("example-dialog", "Example dialog", self.dialog_open);
        let close_key = behavior.close_key();
        let trigger_key = behavior.trigger_key();
        let dialog_content = Element::column([
            text("A real modal portal", 24.0, 700, theme.foreground),
            text(
                "Focus stays inside until the dialog closes.",
                14.0,
                400,
                theme.muted_foreground,
            ),
            Button::new(close_key, "Close dialog", theme.outline_button()).build(),
        ])
        .gap(14.0);
        let dialog = Dialog::new(
            "example-dialog",
            "Example dialog",
            self.dialog_open,
            Button::new(trigger_key, "Open dialog", theme.button()).build(),
            dialog_content,
        )
        .on_open_change(cx.value_callback(|app, open| app.dialog_open = open))
        .build(theme);

        Element::column([
            text("Overlay surfaces", 28.0, 760, theme.foreground),
            text(
                "Compare an opaque popover, a translucent blurred popover, and a focus-trapping dialog.",
                14.0,
                400,
                theme.muted_foreground,
            ),
            Element::row([
                card(
                    "No blur",
                    "Opaque surface over the same colored stage.",
                    solid,
                ),
                card(
                    "With blur",
                    "Translucent surface using a 12 px backdrop blur.",
                    blurred,
                ),
                card(
                    "Modal dialog",
                    "Backdrop, focus trap, Escape, and focus restoration.",
                    dialog,
                ),
            ])
            .flex_wrap(FlexWrap::Wrap)
            .gap(16.0),
        ])
        .width(percent(1.0))
        .height(percent(1.0))
        .padding(Sides::length(28.0))
        .gap(18.0)
        .background(theme.background)
        .overflow(Axes {
            x: Overflow::Hidden,
            y: Overflow::Auto,
        })
        .scroll_config(ScrollConfig::default().scrollbar(theme.scrollbar.clone()))
    }
}
