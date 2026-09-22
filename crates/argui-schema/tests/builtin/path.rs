use argui_core::Color;
use argui_paint::{ImageFit, ImageId, VectorId};
use argui_schema::{AssetHandle, NativeElementInput, SchemaError, SchemaValue, builtin};
use argui_ui::{Display, ElementKind, length};

#[test]
fn path_uses_vector_asset_with_common_visual_properties() {
    let registry = builtin::registry().unwrap();
    let vector = VectorId::fresh();
    let color = Color::from_srgb8(30, 90, 180);
    let path = registry
        .construct(
            builtin::PATH,
            &NativeElementInput::new()
                .property(
                    builtin::SOURCE,
                    SchemaValue::Asset(AssetHandle::Vector(vector)),
                )
                .property(builtin::FIT, SchemaValue::String("fill".into()))
                .property(builtin::TEXT_COLOR, SchemaValue::Color(color))
                .property(builtin::WIDTH, SchemaValue::Dimension(length(80.0)))
                .property(builtin::HEIGHT, SchemaValue::Dimension(length(40.0)))
                .property(builtin::OPACITY, SchemaValue::Float(0.5))
                .property(builtin::VISIBLE, SchemaValue::Bool(false)),
        )
        .unwrap();
    assert!(matches!(
        path.kind,
        ElementKind::Vector {
            vector: id,
            fit: ImageFit::Fill,
            color: tint,
        } if id == vector && tint == color
    ));
    assert_eq!(path.style.size.width, length(80.0));
    assert_eq!(path.style.size.height, length(40.0));
    assert_eq!(path.style.display, Display::None);
    assert_eq!(path.layer.as_ref().unwrap().opacity, 0.5);
}

#[test]
fn path_rejects_missing_or_raster_sources_and_invalid_fit() {
    let registry = builtin::registry().unwrap();
    assert!(
        registry
            .construct(builtin::PATH, &NativeElementInput::new())
            .is_err()
    );
    let raster = registry.construct(
        builtin::PATH,
        &NativeElementInput::new().property(
            builtin::SOURCE,
            SchemaValue::Asset(AssetHandle::Image(ImageId::fresh())),
        ),
    );
    assert!(matches!(raster, Err(SchemaError::Adapter(_))));

    let vector = SchemaValue::Asset(AssetHandle::Vector(VectorId::fresh()));
    for (mode, fit) in [("contain", ImageFit::Contain), ("cover", ImageFit::Cover)] {
        let path = registry
            .construct(
                builtin::PATH,
                &NativeElementInput::new()
                    .property(builtin::SOURCE, vector.clone())
                    .property(builtin::FIT, SchemaValue::String(mode.into())),
            )
            .unwrap();
        assert!(matches!(path.kind, ElementKind::Vector { fit: actual, .. } if actual == fit));
    }
    let invalid = registry.construct(
        builtin::PATH,
        &NativeElementInput::new()
            .property(builtin::SOURCE, vector)
            .property(builtin::FIT, SchemaValue::String("stretch".into())),
    );
    assert!(matches!(invalid, Err(SchemaError::Adapter(_))));
}
