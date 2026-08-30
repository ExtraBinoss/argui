use argui_core::Size;
use argui_paint::{Border, CornerRadii};
use argui_text::{TextStyle, TextWrap};
use argui_theme::{ThemeMode, WidgetAssets, WidgetTheme};
use argui_ui::{Edges, Element, Length, Resizable, TextArea, TextInput};

pub(super) fn text_input(
    key: &str,
    value: &str,
    placeholder: &str,
    widgets: &WidgetTheme,
) -> Element {
    TextInput::new(key, value, placeholder, widgets.text_input.clone()).build()
}

pub(super) fn editor(
    widgets: &WidgetTheme,
    assets: &WidgetAssets,
    size: Size,
    value: &str,
) -> Element {
    let mut style = widgets.text_input.clone();
    style.layout.height = Length::Percent(1.0);
    style.layout.padding = Edges::all(14.0);
    let scrollbar = widgets.scrollbar.clone().insets(Edges {
        bottom: 22.0,
        ..Edges::all(4.0)
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
        Resizable::new(size, area, "notes-resize", assets.resize_handle()).build(),
    ])
    .gap(10.0)
    .padding(Edges::all(16.0))
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
