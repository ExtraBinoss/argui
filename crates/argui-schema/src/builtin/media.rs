//! Native image and SVG surfaces backed by typed asset handles.

use argui_media::AssetHandle;
use argui_paint::{ImageFit, ImageSampling};
use argui_ui::{Element, Role, Semantics};

use super::{
    ALT, CommonProperty, FIT, IMAGE, SAMPLING, SOURCE, SVG, TEXT_COLOR, apply_common,
    common_property,
};
use crate::{
    NativeElementInput, NativeSchema, PropertySchema, SchemaError, SchemaRegistry, SchemaValue,
    ValueType,
};

/// Registers native raster and SVG primitives in the canonical schema.
///
/// * `registry` — registry receiving both media adapters.
///
/// # Errors
///
/// Returns a schema error if a built-in declaration collides or is invalid.
pub(super) fn register(registry: &mut SchemaRegistry) -> Result<(), SchemaError> {
    let image = NativeSchema::new(IMAGE, "Image", "Displays an imported raster image.")
        .property(common_property(CommonProperty::Key))
        .property(common_property(CommonProperty::Tooltip))
        .property(common_property(CommonProperty::Width))
        .property(common_property(CommonProperty::Height))
        .property(common_property(CommonProperty::X))
        .property(common_property(CommonProperty::Y))
        .property(common_property(CommonProperty::Rotation))
        .property(common_property(CommonProperty::Opacity))
        .property(common_property(CommonProperty::BackdropFilter))
        .property(common_property(CommonProperty::DesktopBackdropTint))
        .property(common_property(CommonProperty::DesktopBackdropFallback))
        .property(common_property(CommonProperty::Visible))
        .property(PropertySchema::new(
            ALT,
            "alt",
            ValueType::String,
            "Alternative text; an empty value marks the image decorative.",
        ))
        .property(
            PropertySchema::new(SOURCE, "source", ValueType::Asset, "Imported image asset.")
                .required(),
        )
        .property(PropertySchema::new(
            FIT,
            "fit",
            ValueType::String,
            "Image fitting mode: fill, contain, or cover.",
        ))
        .property(PropertySchema::new(
            SAMPLING,
            "sampling",
            ValueType::String,
            "Pixel sampling mode: linear or nearest.",
        ));
    registry.register(image, |input: &NativeElementInput| {
        let AssetHandle::Image(id) = source(input)? else {
            return Err(SchemaError::Adapter(
                "Image source must be a raster image, not SVG".into(),
            ));
        };
        let fit = fit(input, ImageFit::Cover)?;
        let sampling = match input.get(SAMPLING) {
            None => ImageSampling::Linear,
            Some(SchemaValue::String(value)) if value == "linear" => ImageSampling::Linear,
            Some(SchemaValue::String(value)) if value == "nearest" => ImageSampling::Nearest,
            Some(SchemaValue::String(value)) => {
                return Err(SchemaError::Adapter(format!(
                    "unknown image sampling mode `{value}`"
                )));
            }
            _ => ImageSampling::Linear,
        };
        let element = apply_common(
            Element::image(id).image_fit(fit).image_sampling(sampling),
            input,
        )?;
        Ok(apply_alt(element, input))
    })?;

    let svg = NativeSchema::new(SVG, "Svg", "Displays an imported SVG vector.")
        .property(common_property(CommonProperty::Key))
        .property(common_property(CommonProperty::Tooltip))
        .property(common_property(CommonProperty::Width))
        .property(common_property(CommonProperty::Height))
        .property(common_property(CommonProperty::X))
        .property(common_property(CommonProperty::Y))
        .property(common_property(CommonProperty::Rotation))
        .property(common_property(CommonProperty::Opacity))
        .property(common_property(CommonProperty::BackdropFilter))
        .property(common_property(CommonProperty::DesktopBackdropTint))
        .property(common_property(CommonProperty::DesktopBackdropFallback))
        .property(common_property(CommonProperty::Visible))
        .property(PropertySchema::new(
            ALT,
            "alt",
            ValueType::String,
            "Alternative text; an empty value marks the SVG decorative.",
        ))
        .property(
            PropertySchema::new(SOURCE, "source", ValueType::Asset, "Imported SVG asset.")
                .required(),
        )
        .property(PropertySchema::new(
            FIT,
            "fit",
            ValueType::String,
            "SVG fitting mode: fill, contain, or cover.",
        ))
        .property(PropertySchema::new(
            TEXT_COLOR,
            "color",
            ValueType::Color,
            "Tint for monochrome SVGs; multicolor SVGs retain their own colors.",
        ));
    registry.register(svg, |input: &NativeElementInput| {
        let AssetHandle::Vector(id) = source(input)? else {
            return Err(SchemaError::Adapter(
                "Svg source must be an SVG vector, not a raster image".into(),
            ));
        };
        let mut element = Element::vector(id).vector_fit(fit(input, ImageFit::Contain)?);
        if let Some(SchemaValue::Color(color)) = input.get(TEXT_COLOR) {
            element = element.vector_color(*color);
        }
        Ok(apply_alt(apply_common(element, input)?, input))
    })
}

/// Gives media `element` image semantics from the optional `alt` property in `input`.
/// An empty alternative hides the decorative image from assistive technology.
fn apply_alt(element: Element, input: &NativeElementInput) -> Element {
    match input.get(ALT) {
        Some(SchemaValue::String(alt)) if alt.is_empty() => element.semantic_hidden(true),
        Some(SchemaValue::String(alt)) => element.semantics(Semantics::new(Role::Image).label(alt)),
        _ => element,
    }
}

/// Reads the required, schema-checked asset handle.
///
/// * `input` — native property collection containing `source`.
///
/// # Errors
///
/// Returns an adapter error if the required source is absent after validation.
fn source(input: &NativeElementInput) -> Result<AssetHandle, SchemaError> {
    match input.get(SOURCE) {
        Some(SchemaValue::Asset(handle)) => Ok(*handle),
        _ => Err(SchemaError::Adapter("missing media source".into())),
    }
}

/// Resolves an optional fit mode for either media primitive.
///
/// * `input` — native property collection.
/// * `default` — fit mode used when no explicit property is supplied.
///
/// # Errors
///
/// Returns an adapter error for an unknown fitting mode.
fn fit(input: &NativeElementInput, default: ImageFit) -> Result<ImageFit, SchemaError> {
    match input.get(FIT) {
        None => Ok(default),
        Some(SchemaValue::String(value)) => match value.as_str() {
            "fill" => Ok(ImageFit::Fill),
            "contain" => Ok(ImageFit::Contain),
            "cover" => Ok(ImageFit::Cover),
            _ => Err(SchemaError::Adapter(format!(
                "unknown media fit mode `{value}`"
            ))),
        },
        _ => Err(SchemaError::Adapter("invalid media fit mode".into())),
    }
}
