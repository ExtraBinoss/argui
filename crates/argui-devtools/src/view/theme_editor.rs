use argui_core::{Color, ColorScheme};
use argui_paint::{Border, CornerRadii};
use argui_theme::ThemeValue;
use argui_ui::{
    AlignItems, Axes, Element, FlexWrap, Overflow, ScrollConfig, Sides, length, percent,
};
use argui_widgets::{Button, ColorPicker, Input, WidgetTheme};

use super::text;
use crate::host::DevtoolsHost;

pub(super) fn panel<A>(tools: &DevtoolsHost<A>, theme: &WidgetTheme) -> Element {
    let editor = &tools.theme_editing;
    let palette = editor.palette();
    let controls = Element::row([
        action(
            "__devtools-theme-system",
            "App theme",
            editor.scheme.is_none(),
            theme,
        ),
        action(
            "__devtools-theme-light",
            "Light",
            editor.scheme == Some(ColorScheme::Light),
            theme,
        ),
        action(
            "__devtools-theme-dark",
            "Dark",
            editor.scheme == Some(ColorScheme::Dark),
            theme,
        ),
        action("__devtools-theme-reset", "Reset theme", false, theme),
        action("__devtools-theme-copy", "Copy JSON", false, theme),
    ])
    .gap(6.0)
    .flex_wrap(FlexWrap::Wrap);
    let mut rows = vec![
        controls,
        Element::text("Live preview for the inspected window. Changes stay in this session; copy the resolved tokens to keep them.")
            .text_style(text(12.0, theme.muted_foreground)),
    ];
    for (name, value) in palette.tokens() {
        let modified = editor.overrides.get(name).is_some();
        let label = Element::text(name)
            .text_style(text(12.0, theme.foreground))
            .grow(1.0)
            .min_width(length(0.0));
        let reset = Button::new(
            format!("__devtools-theme-reset-{name}"),
            "Reset",
            theme.ghost_button(),
        )
        .enabled(modified)
        .build()
        .padding(Sides::length(6.0));
        let control = match value {
            ThemeValue::Color(color) => {
                let [r, g, b, a] = color.to_srgba8();
                let swatch = Element::container([])
                    .width(length(22.0))
                    .height(length(22.0))
                    .background(color)
                    .border(Border::all(1.0, theme.border))
                    .radius(CornerRadii::all(4.0));
                Button::new(
                    format!("__devtools-theme-color-{name}"),
                    format!("#{r:02X}{g:02X}{b:02X}{a:02X}"),
                    theme.outline_button(),
                )
                .leading(swatch)
                .build()
                .padding(Sides::length(6.0))
            }
            ThemeValue::Float(number) => {
                let draft = editor.draft.as_ref().filter(|draft| draft.0 == name);
                Input::new(
                    format!("__devtools-theme-number-{name}"),
                    draft.map_or_else(|| format!("{number:.1}"), |draft| draft.1.clone()),
                    "0",
                    theme.input(),
                )
                .label(name)
                .description("Blur radius in pixels, between 0 and 100")
                .invalid(draft.is_some_and(|draft| draft.2))
                .build()
                .width(length(100.0))
            }
            value => Element::text(format!("{value:?}"))
                .text_style(text(12.0, theme.muted_foreground))
                .max_width(length(360.0)),
        };
        let header = Element::row([label, control, reset])
            .gap(8.0)
            .align_items(AlignItems::CENTER)
            .flex_wrap(FlexWrap::Wrap);
        let mut children = vec![header];
        if let Some((_, picker)) = editor.color.as_ref().filter(|(token, _)| token == name) {
            children.push(
                ColorPicker::new(format!("__devtools-color-theme-{name}"), name, picker)
                    .build(theme)
                    .max_width(length(360.0)),
            );
        }
        if editor
            .draft
            .as_ref()
            .is_some_and(|draft| draft.0 == name && draft.2)
        {
            children.push(
                Element::text("Enter a value from 0 to 100")
                    .text_style(text(11.0, theme.destructive)),
            );
        }
        rows.push(
            Element::column(children)
                .gap(12.0)
                .padding(Sides::length(10.0))
                .width(percent(1.0))
                .background(if modified {
                    theme.muted
                } else {
                    Color::TRANSPARENT
                })
                .border(Border::all(1.0, theme.border))
                .radius(CornerRadii::all(6.0)),
        );
    }
    Element::column(rows)
        .keyed("__devtools-theme-tokens")
        .gap(10.0)
        .padding(Sides::length(12.0))
        .width(percent(1.0))
        .overflow(Axes {
            x: Overflow::Hidden,
            y: Overflow::Auto,
        })
        .scroll_config(ScrollConfig::default().scrollbar(theme.scrollbar.clone()))
}

fn action(key: &str, label: &str, active: bool, theme: &WidgetTheme) -> Element {
    Button::new(
        key,
        label,
        if active {
            theme.secondary_button()
        } else {
            theme.ghost_button()
        },
    )
    .build()
    .padding(Sides::length(8.0))
}
