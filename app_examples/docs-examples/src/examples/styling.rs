use std::sync::Arc;

use argui::{
    core::{Color, ColorScheme},
    paint::{Border, CornerRadii},
    runtime::{Context, Render},
    text::TextStyle,
    theme::{ThemeOverrides, ThemeValue},
    ui::{Element, EventType, FlexWrap, Sides, UiEventKind, percent},
    widgets::{Button, WidgetTheme, default_theme},
};

#[derive(Default)]
pub struct Example {
    scheme: Option<ColorScheme>,
    accent: usize,
    custom_tokens: bool,
}

fn accent(index: usize, fallback: Color) -> Color {
    match index {
        1 => Color::from_srgb8(124, 58, 237),
        2 => Color::from_srgb8(217, 119, 6),
        _ => fallback,
    }
}

fn custom_overrides(scheme: ColorScheme) -> Arc<ThemeOverrides> {
    let mut tokens = ThemeOverrides::default();
    let (background, card, muted, border) = match scheme {
        ColorScheme::Light => (
            Color::from_srgb8(245, 243, 255),
            Color::from_srgb8(255, 255, 255),
            Color::from_srgb8(237, 233, 254),
            Color::from_srgb8(196, 181, 253),
        ),
        ColorScheme::Dark => (
            Color::from_srgb8(24, 20, 38),
            Color::from_srgb8(34, 28, 53),
            Color::from_srgb8(49, 40, 74),
            Color::from_srgb8(109, 88, 164),
        ),
    };
    for (name, color) in [
        ("background", background),
        ("card", card),
        ("muted", muted),
        ("secondary", muted),
        ("border", border),
        ("input-border", border),
    ] {
        tokens.set(name, ThemeValue::Color(color));
    }
    tokens.set("overlay-blur", ThemeValue::Number(10.0));
    Arc::new(tokens)
}

fn choice(key: &'static str, label: &'static str, selected: bool, theme: &WidgetTheme) -> Element {
    let style = if selected {
        theme.button()
    } else {
        theme.outline_button()
    };
    Button::new(key, label, style).build()
}

impl Render for Example {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let mut source = cx.environment().clone();
        source.color_scheme = self.scheme.unwrap_or(source.color_scheme);
        source.primary = accent(self.accent, source.primary);
        if self.custom_tokens {
            source.theme_overrides = Some(custom_overrides(source.color_scheme));
        }
        let themes = default_theme(&source);
        let theme = themes.resolve(source.color_scheme);
        let text = |value, size, weight, color| {
            Element::text(value).text_style(TextStyle {
                color,
                font_size: size,
                line_height: size * 1.35,
                weight,
                ..TextStyle::default()
            })
        };

        let schemes = Element::row([
            choice("system", "System", self.scheme.is_none(), theme),
            choice(
                "light",
                "Light",
                self.scheme == Some(ColorScheme::Light),
                theme,
            ),
            choice(
                "dark",
                "Dark",
                self.scheme == Some(ColorScheme::Dark),
                theme,
            ),
        ])
        .gap(8.0)
        .flex_wrap(FlexWrap::Wrap);
        let accents = Element::row([
            choice("blue", "Blue", self.accent == 0, theme),
            choice("violet", "Violet", self.accent == 1, theme),
            choice("amber", "Amber", self.accent == 2, theme),
            choice(
                "tokens",
                if self.custom_tokens {
                    "Reset tokens"
                } else {
                    "Override tokens"
                },
                self.custom_tokens,
                theme,
            ),
        ])
        .gap(8.0)
        .flex_wrap(FlexWrap::Wrap);
        let preview = Element::column([
            text("Live theme preview", 24.0, 720, theme.foreground),
            text(
                if self.custom_tokens {
                    "Background, card, muted, borders, inputs and blur are overridden."
                } else {
                    "Every widget derives its states from the resolved default theme."
                },
                14.0,
                450,
                theme.muted_foreground,
            ),
            Element::row([
                Button::new("preview-primary", "Primary", theme.button()).build(),
                Button::new("preview-secondary", "Secondary", theme.secondary_button()).build(),
                Button::new("preview-delete", "Delete", theme.destructive_button()).build(),
            ])
            .gap(8.0)
            .flex_wrap(FlexWrap::Wrap),
        ])
        .padding(Sides::length(18.0))
        .gap(12.0)
        .background(theme.card)
        .border(Border::all(1.0, theme.border))
        .radius(CornerRadii::all(if self.custom_tokens {
            24.0
        } else {
            12.0
        }));

        Element::column([
            text("Theme configurator", 28.0, 760, theme.foreground),
            schemes,
            accents,
            preview,
        ])
        .width(percent(1.0))
        .height(percent(1.0))
        .padding(Sides::length(20.0))
        .gap(14.0)
        .background(theme.background)
        .on(cx.listener(EventType::Click, |app, event, cx| {
            if !matches!(event.kind, UiEventKind::Click(_)) {
                return;
            }
            match event.target_key() {
                Some("system") => app.scheme = None,
                Some("light") => app.scheme = Some(ColorScheme::Light),
                Some("dark") => app.scheme = Some(ColorScheme::Dark),
                Some("blue") => app.accent = 0,
                Some("violet") => app.accent = 1,
                Some("amber") => app.accent = 2,
                Some("tokens") => app.custom_tokens = !app.custom_tokens,
                _ => return,
            }
            cx.notify();
        }))
    }
}
