//! One showcase shared by native and WebAssembly launchers.

use argui_paint::{Border, ClipBehavior, Color, CornerRadii, PaintStyle, QuadStyle};
use argui_runtime::{UiApp, ViewUpdate};
use argui_text::{TextColor, TextEngine, TextStyle, TextWrap};
use argui_ui::{Align, Button, ButtonStyle, Edges, Element, Length, UiEvent, UiEventKind, Wrap};

#[derive(Default)]
pub struct StateShowcase {
    count: u32,
    warm: bool,
    reversed: bool,
}

impl UiApp for StateShowcase {
    fn view(&self) -> Element {
        let accent = if self.warm {
            Color::rgb(0.96, 0.52, 0.26)
        } else {
            Color::rgb(0.20, 0.68, 0.94)
        };
        let items = if self.reversed {
            vec![chip("beta", "Stable beta"), chip("alpha", "Stable alpha")]
        } else {
            vec![chip("alpha", "Stable alpha"), chip("beta", "Stable beta")]
        };
        Element::column([Element::column([
            Element::text(format!("State updates: {}", self.count)).text_style(text_style(
                38.0,
                TextColor::WHITE,
                700,
                TextWrap::Word,
            )),
            Element::text(
                "Clicks mutate plain Rust state. The rebuilt tree decides whether it needs no work, a repaint, or a new layout.",
            )
            .text_style(text_style(
                19.0,
                TextColor::rgb(0.78, 0.84, 0.92),
                400,
                TextWrap::Word,
            )),
            Element::row([
                button("increment", "Increment", accent),
                button("theme", "Toggle paint", accent),
                button("reorder", "Reorder keys", accent),
            ])
            .wrap(Wrap::Wrap)
            .gap(12.0)
            .align(Align::Center),
            Element::row(items).wrap(Wrap::Wrap).gap(10.0),
        ])
        .padding(Edges::all(30.0))
        .gap(22.0)
        .background(if self.warm {
            Color::rgb(0.16, 0.09, 0.07)
        } else {
            Color::rgb(0.06, 0.08, 0.12)
        })
        .border(Border::all(1.5, accent))
        .radius(CornerRadii::all(22.0))
        .clip(ClipBehavior::Bounds)])
        .width(Length::Percent(1.0))
        .height(Length::Percent(1.0))
        .padding(Edges::symmetric(20.0, 24.0))
    }

    fn update(&mut self, event: &UiEvent) -> ViewUpdate {
        if event.kind != UiEventKind::Clicked {
            return ViewUpdate::None;
        }
        match event.key.as_deref() {
            Some("increment") => self.count += 1,
            Some("theme") => self.warm = !self.warm,
            Some("reorder") => self.reversed = !self.reversed,
            _ => return ViewUpdate::None,
        }
        ViewUpdate::Rebuild
    }
}

#[must_use]
pub fn text_engine() -> TextEngine {
    TextEngine::from_embedded_fonts(
        [
            include_bytes!("../../argui-web-demo/assets/fonts/NotoSans-Regular.ttf").as_slice(),
            include_bytes!("../../argui-web-demo/assets/fonts/NotoSansArabic.ttf").as_slice(),
            include_bytes!("../../argui-web-demo/assets/fonts/NotoSansHebrew.ttf").as_slice(),
            include_bytes!("../../argui-web-demo/assets/fonts/FiraMono-Medium.ttf").as_slice(),
        ],
        "Noto Sans",
        "Noto Sans",
        "Fira Mono",
    )
}

fn text_style(size: f32, color: TextColor, weight: u16, wrap: TextWrap) -> TextStyle {
    TextStyle {
        font_size: size,
        line_height: size * 1.25,
        color,
        weight,
        wrap,
        ..TextStyle::default()
    }
}

fn button(key: &str, label: &str, accent: Color) -> Element {
    let radius = CornerRadii::all(12.0);
    let rest = QuadStyle::solid(Color::rgb(0.10, 0.14, 0.20))
        .border(Border::all(1.0, Color::rgb(0.28, 0.36, 0.48)))
        .radius(radius);
    let active = QuadStyle::solid(Color::rgb(0.14, 0.22, 0.30))
        .border(Border::all(1.0, accent))
        .radius(radius);
    Button::new(
        key,
        label,
        ButtonStyle::new(
            PaintStyle::new(rest),
            text_style(17.0, TextColor::WHITE, 600, TextWrap::None),
        )
        .hovered(active)
        .pressed(active.opacity(0.72))
        .focused(rest.border(Border::all(2.0, accent))),
    )
    .build()
}

fn chip(key: &str, label: &str) -> Element {
    Element::text(label)
        .keyed(key)
        .text_style(text_style(
            15.0,
            TextColor::rgb(0.66, 0.88, 0.72),
            600,
            TextWrap::None,
        ))
        .padding(Edges::symmetric(12.0, 7.0))
        .shrink(0.0)
        .background(Color::rgb(0.08, 0.20, 0.14))
        .radius(CornerRadii::all(9.0))
}
