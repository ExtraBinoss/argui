use argui_text::TextStyle;
use argui_ui::{Element, ElementKind, ImageFit, ImageId, ImageSampling, VectorId};

#[test]
fn media_and_text_specific_builders_only_change_matching_elements() {
    let vector = Element::vector(VectorId(3)).vector_progress(0.75);
    assert!(matches!(
        vector.kind,
        ElementKind::Vector { progress, .. } if progress == 0.75
    ));
    assert!(matches!(
        Element::container([]).vector_progress(0.75).kind,
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
