use argui_core::{Color, Point, ScrollDelta, Size};
use argui_layout::LayoutEngine;
use argui_paint::{ImageFit, ImageId, ImageSampling, VectorId};
use argui_schema::{AssetHandle, NativeElementInput, SchemaError, SchemaValue, builtin};
use argui_text::TextEngine;
use argui_ui::{CheckedState, Element, ElementKind, Overflow, Role, UiTree, length, percent};

#[test]
fn alternative_text_names_images_and_empty_alt_hides_decoration() {
    let registry = builtin::registry().unwrap();
    let image = registry
        .construct(
            builtin::IMAGE,
            &NativeElementInput::new()
                .property(
                    builtin::SOURCE,
                    SchemaValue::Asset(AssetHandle::Image(ImageId::fresh())),
                )
                .property(
                    builtin::ALT,
                    SchemaValue::String("Saturn and its rings".into()),
                ),
        )
        .unwrap();
    let semantics = image.semantics.as_ref().unwrap();
    assert_eq!(semantics.role, Role::Image);
    assert_eq!(semantics.label.as_deref(), Some("Saturn and its rings"));

    let decorative = registry
        .construct(
            builtin::SVG,
            &NativeElementInput::new()
                .property(
                    builtin::SOURCE,
                    SchemaValue::Asset(AssetHandle::Vector(VectorId::fresh())),
                )
                .property(builtin::ALT, SchemaValue::String(String::new())),
        )
        .unwrap();
    assert!(decorative.semantic_hidden);
}

#[test]
fn image_and_svg_accept_typed_handles_and_reject_wrong_media_kinds() {
    let registry = builtin::registry().unwrap();
    let image_id = ImageId::fresh();
    let image = registry
        .construct(
            builtin::IMAGE,
            &NativeElementInput::new()
                .property(
                    builtin::SOURCE,
                    SchemaValue::Asset(AssetHandle::Image(image_id)),
                )
                .property(builtin::WIDTH, SchemaValue::Dimension(length(160.0)))
                .property(builtin::FIT, SchemaValue::String("contain".into()))
                .property(builtin::SAMPLING, SchemaValue::String("nearest".into())),
        )
        .unwrap();
    assert!(
        matches!(image.kind, ElementKind::Image { image, fit: ImageFit::Contain, .. } if image == image_id)
    );
    assert_eq!(image.style.size.width, length(160.0));

    let vector_id = VectorId::fresh();
    let svg = registry
        .construct(
            builtin::SVG,
            &NativeElementInput::new()
                .property(
                    builtin::SOURCE,
                    SchemaValue::Asset(AssetHandle::Vector(vector_id)),
                )
                .property(builtin::TEXT_COLOR, SchemaValue::Color(Color::BLACK)),
        )
        .unwrap();
    assert!(
        matches!(svg.kind, ElementKind::Vector { vector, fit: ImageFit::Contain, .. } if vector == vector_id)
    );
    assert!(
        registry
            .construct(
                builtin::SVG,
                &NativeElementInput::new().property(
                    builtin::SOURCE,
                    SchemaValue::Asset(AssetHandle::Image(image_id))
                )
            )
            .is_err()
    );
    assert!(
        registry
            .construct(
                builtin::IMAGE,
                &NativeElementInput::new().property(
                    builtin::SOURCE,
                    SchemaValue::Asset(AssetHandle::Vector(vector_id))
                )
            )
            .is_err()
    );
}

#[test]
fn scroll_view_and_focus_scope_expose_native_behavior() {
    let registry = builtin::registry().unwrap();
    let scroll = registry
        .construct(builtin::SCROLL_VIEW, &NativeElementInput::new())
        .unwrap();
    assert!(scroll.scroll.as_ref().unwrap().scrollbar.is_none());
    assert_eq!(scroll.style.overflow.y, Overflow::Auto);

    let toggle_scope = registry
        .construct(
            builtin::FOCUS_SCOPE,
            &NativeElementInput::new()
                .property(builtin::ID, SchemaValue::String("toggle".into()))
                .property(builtin::SEMANTIC_ROLE, SchemaValue::String("switch".into()))
                .property(
                    builtin::SEMANTIC_LABEL,
                    SchemaValue::String("Notifications".into()),
                )
                .property(builtin::CHECKED, SchemaValue::Bool(true)),
        )
        .unwrap();
    assert_eq!(
        toggle_scope.semantics.as_ref().unwrap().state.checked,
        Some(CheckedState::Checked)
    );
}

#[test]
fn native_scroll_view_tracks_viewport_resize() {
    let registry = builtin::registry().unwrap();
    let scroll = registry
        .construct(
            builtin::SCROLL_VIEW,
            &NativeElementInput::new()
                .property(builtin::HEIGHT, SchemaValue::Dimension(percent(1.0)))
                .property(builtin::WIDTH, SchemaValue::Dimension(percent(1.0)))
                .property(
                    builtin::MIN_HEIGHT,
                    SchemaValue::Constraint(argui_ui::LengthPercentageAuto::length(0.0)),
                )
                .slot(argui_schema::NativeSlotValue::new(
                    builtin::CHILDREN,
                    [Element::container([]).height(length(240.0)).shrink(0.0)],
                )),
        )
        .unwrap();
    assert_eq!(scroll.style.min_size.height, length(0.0));
    let mut tree = UiTree::new(scroll);
    let mut layout = LayoutEngine::new();
    let mut text = TextEngine::new();
    let mut short = layout
        .compute(&mut tree, &mut text, Size::new(200.0, 100.0))
        .unwrap();
    assert_eq!(short.scroll_regions.len(), 1);
    assert!(short.scroll_regions[0].max_offset.y > 100.0);
    assert!(short.scroll_regions[0].scrollbar.is_none());
    tree.scroll(
        Point::new(10.0, 10.0),
        ScrollDelta::Pixels(Point::new(0.0, -50.0)),
        &short.scroll_regions,
    );
    layout.apply_scroll(&tree, &mut short).unwrap();
    assert!(tree.scroll_offset(short.scroll_regions[0].node).y > 0.0);

    let tall = layout
        .compute(&mut tree, &mut text, Size::new(200.0, 160.0))
        .unwrap();
    assert_eq!(tall.scroll_regions.len(), 1);
    assert!(tall.scroll_regions[0].max_offset.y < short.scroll_regions[0].max_offset.y);
    assert!(tall.scroll_regions[0].scrollbar.is_none());
}
#[test]
fn media_fit_modes_and_image_sampling_are_validated() {
    let registry = builtin::registry().unwrap();
    let image = AssetHandle::Image(ImageId::fresh());
    for (name, expected) in [
        ("fill", ImageFit::Fill),
        ("contain", ImageFit::Contain),
        ("cover", ImageFit::Cover),
    ] {
        let element = registry
            .construct(
                builtin::IMAGE,
                &NativeElementInput::new()
                    .property(builtin::SOURCE, SchemaValue::Asset(image))
                    .property(builtin::FIT, SchemaValue::String(name.into())),
            )
            .unwrap();
        assert!(matches!(element.kind, ElementKind::Image { fit, .. } if fit == expected));
    }
    let default = registry
        .construct(
            builtin::IMAGE,
            &NativeElementInput::new().property(builtin::SOURCE, SchemaValue::Asset(image)),
        )
        .unwrap();
    assert!(matches!(
        default.kind,
        ElementKind::Image {
            fit: ImageFit::Cover,
            sampling: ImageSampling::Linear,
            ..
        }
    ));
    let explicit_linear = registry
        .construct(
            builtin::IMAGE,
            &NativeElementInput::new()
                .property(builtin::SOURCE, SchemaValue::Asset(image))
                .property(builtin::SAMPLING, SchemaValue::String("linear".into())),
        )
        .unwrap();
    assert!(matches!(
        explicit_linear.kind,
        ElementKind::Image {
            sampling: ImageSampling::Linear,
            ..
        }
    ));

    for property in [(builtin::FIT, "stretch"), (builtin::SAMPLING, "cubic")] {
        let result = registry.construct(
            builtin::IMAGE,
            &NativeElementInput::new()
                .property(builtin::SOURCE, SchemaValue::Asset(image))
                .property(property.0, SchemaValue::String(property.1.into())),
        );
        assert!(matches!(
            result,
            Err(SchemaError::InvalidPropertyValue { .. })
        ));
    }
}

#[test]
fn svg_fit_modes_are_independent_of_image_defaults() {
    let registry = builtin::registry().unwrap();
    let vector = AssetHandle::Vector(VectorId::fresh());
    let default = registry
        .construct(
            builtin::SVG,
            &NativeElementInput::new().property(builtin::SOURCE, SchemaValue::Asset(vector)),
        )
        .unwrap();
    assert!(matches!(
        default.kind,
        ElementKind::Vector {
            fit: ImageFit::Contain,
            ..
        }
    ));
    let fill = registry
        .construct(
            builtin::SVG,
            &NativeElementInput::new()
                .property(builtin::SOURCE, SchemaValue::Asset(vector))
                .property(builtin::FIT, SchemaValue::String("fill".into())),
        )
        .unwrap();
    assert!(matches!(
        fill.kind,
        ElementKind::Vector {
            fit: ImageFit::Fill,
            ..
        }
    ));
    let invalid = registry.construct(
        builtin::SVG,
        &NativeElementInput::new()
            .property(builtin::SOURCE, SchemaValue::Asset(vector))
            .property(builtin::FIT, SchemaValue::String("stretch".into())),
    );
    assert!(matches!(
        invalid,
        Err(SchemaError::InvalidPropertyValue { .. })
    ));
}
