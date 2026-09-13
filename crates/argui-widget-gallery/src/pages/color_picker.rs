use argui::{
    core::Color,
    runtime::{Context, LayoutSnapshot, Render},
    ui::{Element, EventType, FlexWrap, UiEventKind, length},
    widgets::{Button, ColorPicker, ColorPickerState, shadcn},
};

pub(crate) struct ColorPickerDemo {
    color: ColorPickerState,
    enabled: bool,
}

impl Default for ColorPickerDemo {
    fn default() -> Self {
        Self {
            color: ColorPickerState::new(Color::srgba(0.45, 0.25, 0.95, 0.8)),
            enabled: true,
        }
    }
}

impl Render for ColorPickerDemo {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let environment = cx.environment();
        let themes = shadcn(&environment);
        let theme = themes.resolve(environment.color_scheme);
        let mut preview_theme = theme.clone();
        preview_theme.primary = self.color.color();
        let picker = ColorPicker::new("gallery-color", "Accent color", &self.color)
            .build(theme)
            .width(length(300.0))
            .max_width(argui::ui::percent(1.0));
        let [r, g, b, a] = self.color.color().to_srgba8();
        let preview = Element::column([
            crate::app::text(
                format!("#{r:02X}{g:02X}{b:02X}{a:02X}"),
                16.0,
                theme.foreground,
                600,
            ),
            Button::new(
                "color-preview",
                "Live button preview",
                preview_theme.button(),
            )
            .build(),
            Element::container([])
                .keyed("color-preview-swatch")
                .width(length(180.0))
                .height(length(100.0))
                .background(self.color.color())
                .radius(argui::paint::CornerRadii::all(12.0)),
            Button::new(
                "color-enabled",
                if self.enabled {
                    "Disable editor"
                } else {
                    "Enable editor"
                },
                theme.outline_button(),
            )
            .build(),
        ])
        .gap(16.0);
        let mut root = super::preview(
            "Choose a color",
            "Drag the pad, adjust hue and opacity, or switch between HEX, RGB, HSL and HSV. Arrow keys work on the pad and both sliders.",
            Element::row([picker, preview])
                .gap(32.0)
                .flex_wrap(FlexWrap::Wrap),
            theme,
        );
        for event_type in EventType::ALL {
            root = root.on(cx.listener(event_type, |demo, event, cx| {
                if demo.color.update("gallery-color", event) {
                    cx.notify();
                }
                if event.target_key() == Some("color-enabled")
                    && matches!(event.kind, UiEventKind::Click(_))
                {
                    demo.enabled = !demo.enabled;
                    demo.color.set_enabled(demo.enabled);
                    cx.notify();
                }
            }));
        }
        root
    }

    fn layout_changed(&mut self, layout: &LayoutSnapshot, _: &mut Context<Self>) {
        self.color.layout_changed("gallery-color", layout);
    }
}
