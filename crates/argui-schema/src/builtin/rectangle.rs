//! Declarative painted rectangle with no control-specific interaction or styling.

use argui_core::Color;
use argui_paint::{Border, CornerRadii, Shadow};
use argui_ui::Element;

use super::{
    BACKGROUND, BORDER_COLOR, BORDER_WIDTH, CHILDREN, CLIP, CommonProperty, RADIUS, SHADOW_BLUR,
    SHADOW_COLOR, SHADOW_OFFSET_Y, apply_common, common_property,
};
use crate::{
    NativeElementInput, NativeSchema, SchemaError, SchemaRegistry, SchemaValue, SlotArity,
    SlotSchema, ValueType,
};

/// Registers the brush-painted Rectangle base element.
///
/// * `registry` — native schema registry receiving the element and its adapter.
///
/// # Errors
///
/// Returns a schema error for conflicting identifiers or invalid metadata.
pub(super) fn register(registry: &mut SchemaRegistry) -> Result<(), SchemaError> {
    let schema = NativeSchema::new(
        super::RECTANGLE,
        "Rectangle",
        "Painted rectangle with generic brush, border, radius, and clipping.",
    )
    .property(common_property(CommonProperty::Key))
    .property(common_property(CommonProperty::Tooltip))
    .property(common_property(CommonProperty::Width))
    .property(common_property(CommonProperty::Height))
    .property(common_property(CommonProperty::X))
    .property(common_property(CommonProperty::Y))
    .property(common_property(CommonProperty::MinWidth))
    .property(common_property(CommonProperty::MinHeight))
    .property(common_property(CommonProperty::Rotation))
    .property(common_property(CommonProperty::Opacity))
    .property(common_property(CommonProperty::BackdropFilter))
    .property(common_property(CommonProperty::Visible))
    .property(crate::PropertySchema::new(
        BACKGROUND,
        "background",
        ValueType::Brush,
        "Brush painted inside the rectangle.",
    ))
    .property(crate::PropertySchema::new(
        BORDER_COLOR,
        "border_color",
        ValueType::Color,
        "Border color.",
    ))
    .property(crate::PropertySchema::new(
        BORDER_WIDTH,
        "border_width",
        ValueType::Float,
        "Uniform border width in logical pixels.",
    ))
    .property(crate::PropertySchema::new(
        RADIUS,
        "radius",
        ValueType::Float,
        "Uniform corner radius in logical pixels.",
    ))
    .property(crate::PropertySchema::new(
        CLIP,
        "clip",
        ValueType::Bool,
        "Clip descendants to the rounded rectangle bounds.",
    ))
    .property(crate::PropertySchema::new(
        SHADOW_BLUR,
        "shadow_blur",
        ValueType::Float,
        "Drop shadow blur radius in logical pixels.",
    ))
    .property(crate::PropertySchema::new(
        SHADOW_OFFSET_Y,
        "shadow_offset_y",
        ValueType::Float,
        "Vertical drop shadow offset in logical pixels.",
    ))
    .property(crate::PropertySchema::new(
        SHADOW_COLOR,
        "shadow_color",
        ValueType::Color,
        "Drop shadow color.",
    ))
    .slot(SlotSchema {
        id: CHILDREN,
        name: "children".into(),
        arity: SlotArity::Many,
        documentation: "Elements painted within the rectangle.".into(),
    });
    registry.register(schema, |input: &NativeElementInput| {
        let width = finite_nonnegative(input, BORDER_WIDTH, "border_width")?.unwrap_or(0.0);
        let radius = finite_nonnegative(input, RADIUS, "radius")?.unwrap_or(0.0);
        let shadow_blur = finite_nonnegative(input, SHADOW_BLUR, "shadow_blur")?.unwrap_or(0.0);
        let shadow_offset_y = match input.get(SHADOW_OFFSET_Y) {
            Some(SchemaValue::Float(value)) if value.is_finite() => *value,
            Some(SchemaValue::Float(_)) => {
                return Err(SchemaError::Adapter(
                    "Rectangle requires finite `shadow_offset_y`".into(),
                ));
            }
            _ => 0.0,
        };
        let mut element =
            apply_common(Element::container(input.children(CHILDREN).to_vec()), input)?;
        if let Some(SchemaValue::Brush(brush)) = input.get(BACKGROUND) {
            element = element.fill(brush.clone());
        }
        if width > 0.0 {
            let color = match input.get(BORDER_COLOR) {
                Some(SchemaValue::Color(color)) => *color,
                _ => Color::BLACK,
            };
            element = element.border(Border::all(width, color));
        }
        let radii = CornerRadii::all(radius);
        element = if matches!(input.get(CLIP), Some(SchemaValue::Bool(true))) {
            element.clip(radii)
        } else {
            element.radius(radii)
        };
        if shadow_blur > 0.0 {
            let color = match input.get(SHADOW_COLOR) {
                Some(SchemaValue::Color(color)) => *color,
                _ => Color::srgba(0.0, 0.0, 0.0, 0.18),
            };
            element = element.shadow(Shadow::drop([0.0, shadow_offset_y], shadow_blur, color));
        }
        Ok(element)
    })
}

/// Reads an optional finite nonnegative scalar from validated input.
///
/// * `input` — values supplied for the rectangle.
/// * `id` — scalar property to read.
/// * `name` — property spelling included in diagnostics.
///
/// # Errors
///
/// Returns an adapter error if the scalar is negative or nonfinite.
fn finite_nonnegative(
    input: &NativeElementInput,
    id: crate::PropertyId,
    name: &str,
) -> Result<Option<f32>, SchemaError> {
    match input.get(id) {
        Some(SchemaValue::Float(value)) if value.is_finite() && *value >= 0.0 => Ok(Some(*value)),
        Some(SchemaValue::Float(_)) => Err(SchemaError::Adapter(format!(
            "Rectangle requires finite nonnegative `{name}`"
        ))),
        _ => Ok(None),
    }
}
