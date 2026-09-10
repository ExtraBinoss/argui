use argui::{
    paint::{CornerRadii, ImageFit, ImageId},
    text::{TextStyle, TextWrap},
    ui::{AlignItems, Element, JustifyContent, evenly_sized_tracks, length},
    widgets::{AspectRatio, WidgetTheme},
};

pub(super) fn render(theme: &WidgetTheme, image: ImageId) -> Element {
    super::preview(
        "Consistent proportions",
        "The content follows the available width while keeping its ratio.",
        Element::column([
            AspectRatio::new(
                "ratio-wide",
                16.0 / 9.0,
                Element::image(image)
                    .image_fit(ImageFit::Contain)
                    .background(theme.muted),
            )
            .build()
            .clip(CornerRadii::all(12.0)),
            Element::grid([
                sample("ratio-square", "1 : 1", 1.0, theme),
                sample("ratio-photo", "4 : 3", 4.0 / 3.0, theme),
                sample("ratio-video", "16 : 9", 16.0 / 9.0, theme),
            ])
            .grid_template_columns(evenly_sized_tracks::<String>(3))
            .gap(16.0),
        ])
        .gap(20.0)
        .max_width(length(520.0)),
        theme,
    )
}

fn sample(key: &str, label: &str, ratio: f32, theme: &WidgetTheme) -> Element {
    AspectRatio::new(
        key,
        ratio,
        Element::row([Element::text(label).text_style(TextStyle {
            font_size: 16.0,
            line_height: 22.0,
            weight: 600,
            color: theme.foreground,
            wrap: TextWrap::None,
            ..TextStyle::default()
        })])
        .align_items(AlignItems::CENTER)
        .justify_content(JustifyContent::CENTER)
        .background(theme.muted),
    )
    .build()
    .clip(CornerRadii::all(10.0))
}
