#[path = "element/direction.rs"]
mod direction;
use argui_text::{TextOverflow, TextStyle};
use argui_ui::{Color, Element, ElementKind, ImageFit, ImageId, ImageSampling, Insets, VectorId};

#[path = "element/overrides.rs"]
mod overrides;

#[path = "element/portal.rs"]
mod portal;

#[test]
fn media_and_text_specific_builders_only_change_matching_elements() {
    let vector = Element::vector(VectorId(3))
        .vector_fit(ImageFit::Fill)
        .vector_color(Color::srgb(0.2, 0.4, 0.6));
    assert!(matches!(
        vector.kind,
        ElementKind::Vector { fit: ImageFit::Fill, color, .. }
            if color == Color::srgb(0.2, 0.4, 0.6)
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
fn safe_area_builder_pads_all_edges_and_clamps_negative_adapter_values() {
    let element = Element::container([]).safe_area(Insets::new(10.0, 20.0, -1.0, 30.0));

    assert_eq!(element.children.len(), 1);
    assert_eq!(element.style.padding.top, argui_ui::length(10.0));
    assert_eq!(element.style.padding.right, argui_ui::length(20.0));
    assert_eq!(element.style.padding.bottom, argui_ui::length(0.0));
    assert_eq!(element.style.padding.left, argui_ui::length(30.0));
}

#[test]
fn empty_safe_area_keeps_the_original_element_identity() {
    let element = Element::container([]).keyed("content");

    assert_eq!(element.clone().safe_area(Insets::ZERO), element);
    assert_eq!(
        element
            .clone()
            .safe_area(Insets::new(-1.0, f32::NAN, -2.0, -3.0)),
        element
    );
}

#[test]
fn text_overflow_applies_only_to_textual_elements() {
    assert!(matches!(
        Element::text("truncate")
            .text_overflow(TextOverflow::Ellipsis(argui_text::EllipsisPosition::End))
            .kind,
        ElementKind::Text { ref style, .. } if style.overflow == TextOverflow::Ellipsis(argui_text::EllipsisPosition::End)
    ));
    assert!(matches!(
        Element::container([])
            .text_overflow(TextOverflow::Ellipsis(argui_text::EllipsisPosition::End))
            .kind,
        ElementKind::Container
    ));
}
