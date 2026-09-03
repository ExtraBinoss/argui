use argui_core::{Color, Size};
use argui_paint::{Border, CornerRadii};
use argui_text::{TextStyle, TextWrap};
use argui_theme::ThemeMode;
use argui_ui::{Element, LengthPercentageAuto, Resizable, Sides, auto, length, percent};
use argui_widgets::{Input, TablerIcon, TextArea, WidgetAssets, WidgetTheme};

pub(super) static PRIMARIES: LazyLock<[(&str, Color); 5]> = LazyLock::new(|| {
    [
        ("Primary: blue", Color::srgb(0.10, 0.45, 0.91)),
        ("Primary: violet", Color::srgb(0.49, 0.23, 0.93)),
        ("Primary: rose", Color::srgb(0.88, 0.11, 0.28)),
        ("Primary: orange", Color::srgb(0.92, 0.35, 0.05)),
        ("Primary: emerald", Color::srgb(0.02, 0.59, 0.41)),
    ]
});

pub(super) fn primary_label(index: usize) -> &'static str {
    PRIMARIES[index].0
}

pub(super) fn top_right(top: f32, right: f32) -> Sides<LengthPercentageAuto> {
    Sides {
        left: auto(),
        right: length(right),
        top: length(top),
        bottom: auto(),
    }
}

pub(super) fn text_input(
    key: &str,
    value: &str,
    placeholder: &str,
    widgets: &WidgetTheme,
) -> Element {
    Input::new(key, value, placeholder, widgets.input.clone()).build()
}

pub(super) fn editor(
    widgets: &WidgetTheme,
    assets: &WidgetAssets,
    size: Size,
    value: &str,
) -> Element {
    let mut style = widgets.input.clone();
    style.layout.size.height = percent(1.0);
    style.layout.padding = Sides::length(14.0);
    let scrollbar = widgets.scrollbar.clone().insets(Sides {
        bottom: 22.0,
        ..Sides::length(4.0)
    });
    let area = TextArea::new("notes", value, "Write notes…", style)
        .scrollbar(scrollbar)
        .build();
    Element::column([
        Element::text("Resizable textarea").text_style(TextStyle {
            font_size: 16.0,
            line_height: 20.0,
            color: widgets.foreground,
            weight: 600,
            wrap: TextWrap::None,
            ..TextStyle::default()
        }),
        Resizable::new(
            size,
            area,
            "notes-resize",
            assets.icon(TablerIcon::Resize, 18.0),
        )
        .build(),
    ])
    .gap(10.0)
    .padding(Sides::length(16.0))
    .background(widgets.card)
    .border(Border::all(1.0, widgets.border))
    .radius(CornerRadii::all(10.0))
}

pub(super) const fn label(mode: ThemeMode) -> &'static str {
    match mode {
        ThemeMode::Light => "Theme: light",
        ThemeMode::Dark => "Theme: dark",
        ThemeMode::System => "Theme: system",
    }
}
use std::sync::LazyLock;
