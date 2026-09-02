use argui_text::{TextOverflow, TextStyle};
use argui_ui::{Color, Element, ElementKind, ImageFit, ImageId, ImageSampling, VectorId};

#[test]
fn media_and_text_specific_builders_only_change_matching_elements() {
    let vector = Element::vector(VectorId(3))
        .vector_fit(ImageFit::Fill)
        .vector_color(Color::rgb(0.2, 0.4, 0.6));
    assert!(matches!(
        vector.kind,
        ElementKind::Vector { fit: ImageFit::Fill, color, .. }
            if color == Color::rgb(0.2, 0.4, 0.6)
    ));
    assert!(matches!(
        Element::container([]).vector_fit(ImageFit::Fill).kind,
        ElementKind::Container
    ));

    let image = Element::image(ImageId(4))
        .image_fit(ImageFit::Cover)
        .image_sampling(ImageSampling::Nearest);
    assert!(matches!(
        image.kind,
        ElementKind::Image {
            fit: ImageFit::Cover,
            sampling: ImageSampling::Nearest,
            ..
        }
    ));
    assert!(matches!(
        Element::text("plain")
            .image_fit(ImageFit::Cover)
            .image_sampling(ImageSampling::Nearest)
            .kind,
        ElementKind::Text { .. }
    ));

    let style = TextStyle {
        font_size: 27.0,
        ..TextStyle::default()
    };
    assert!(matches!(
        &Element::text("styled").text_style(style.clone()).kind,
        ElementKind::Text { style: value, .. } if *value == style
    ));
    assert!(matches!(
        Element::container([]).text_style(style).kind,
        ElementKind::Container
    ));
}

#[test]
fn text_overflow_applies_only_to_textual_elements() {
    assert!(matches!(
        Element::text("truncate")
            .text_overflow(TextOverflow::Ellipsis)
            .kind,
        ElementKind::Text { ref style, .. } if style.overflow == TextOverflow::Ellipsis
    ));
    assert!(matches!(
        Element::container([])
            .text_overflow(TextOverflow::Ellipsis)
            .kind,
        ElementKind::Container
    ));
}
