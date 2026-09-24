//! Native rendering adapter for authored vector path assets.

use argui_media::AssetHandle;
use argui_paint::ImageFit;
use argui_ui::Element;

use super::{CommonProperty, FIT, PATH, SOURCE, TEXT_COLOR, apply_common, common_property};
use crate::{
    NativeElementInput, NativeSchema, PropertySchema, SchemaError, SchemaRegistry, SchemaValue,
    ValueType,
};

/// Registers the composable Path visual primitive.
///
/// * `registry` — native schema registry receiving the path adapter.
///
/// # Errors
///
/// Returns a schema error if the declaration conflicts with another native type.
pub(super) fn register(registry: &mut SchemaRegistry) -> Result<(), SchemaError> {
    let schema = NativeSchema::new(
        PATH,
        "Path",
        "Paints an authored vector path with independent bounds, tint, and fit.",
    )
    .property(common_property(CommonProperty::Key))
    .property(common_property(CommonProperty::Tooltip))
    .property(common_property(CommonProperty::Width))
    .property(common_property(CommonProperty::Height))
    .property(common_property(CommonProperty::X))
    .property(common_property(CommonProperty::Y))
    .property(common_property(CommonProperty::Rotation))
    .property(common_property(CommonProperty::Opacity))
    .property(common_property(CommonProperty::BackdropFilter))
    .property(common_property(CommonProperty::Visible))
    .property(
        PropertySchema::new(
            SOURCE,
            "source",
            ValueType::Asset,
            "Vector asset containing authored path geometry and fill or stroke style.",
        )
        .required(),
    )
    .property(PropertySchema::new(
        FIT,
        "fit",
        ValueType::String,
        "Path fitting mode: fill, contain, or cover.",
    ))
    .property(PropertySchema::new(
        TEXT_COLOR,
        "color",
        ValueType::Color,
        "Color applied to the path's fill and stroke paint.",
    ));

    registry.register(schema, |input: &NativeElementInput| {
        let vector = match input.get(SOURCE) {
            Some(SchemaValue::Asset(AssetHandle::Vector(id))) => *id,
            Some(SchemaValue::Asset(AssetHandle::Image(_))) => {
                return Err(SchemaError::Adapter(
                    "Path source must be a vector asset, not a raster image".into(),
                ));
            }
            _ => return Err(SchemaError::Adapter("missing Path source".into())),
        };
        let fit = match input.get(FIT) {
            None => ImageFit::Contain,
            Some(SchemaValue::String(value)) => match value.as_str() {
                "fill" => ImageFit::Fill,
                "contain" => ImageFit::Contain,
                "cover" => ImageFit::Cover,
                _ => {
                    return Err(SchemaError::Adapter(format!(
                        "unknown Path fit mode `{value}`"
                    )));
                }
            },
            _ => return Err(SchemaError::Adapter("invalid Path fit mode".into())),
        };
        let mut element = Element::vector(vector).vector_fit(fit);
        if let Some(SchemaValue::Color(color)) = input.get(TEXT_COLOR) {
            element = element.vector_color(*color);
        }
        apply_common(element, input)
    })
}
