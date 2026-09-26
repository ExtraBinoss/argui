//! Alignment and painted edges of native containers.

use argui_ui::{AlignContent, AlignSelf, Element, GridAutoFlow, JustifyItems, JustifySelf};

use super::*;

/// Declares native alignment and painted-edge properties.
/// Returns properties with stable IDs for host validation.
pub(super) fn properties() -> Vec<PropertySchema> {
    let defs = [
        (
            ALIGN_SELF,
            "alignSelf",
            ValueType::String,
            "Alignment within the parent cross axis.",
        ),
        (
            JUSTIFY_ITEMS,
            "justifyItems",
            ValueType::String,
            "Horizontal alignment of grid items.",
        ),
        (
            JUSTIFY_SELF,
            "justifySelf",
            ValueType::String,
            "Horizontal alignment in a grid cell.",
        ),
        (
            ALIGN_CONTENT,
            "alignContent",
            ValueType::String,
            "Distribution of grid or flex rows.",
        ),
        (
            GRID_AUTO_FLOW,
            "gridAutoFlow",
            ValueType::String,
            "Implicit grid placement order.",
        ),
        (
            CLIP,
            "clip",
            ValueType::Bool,
            "Clip descendants to rounded bounds.",
        ),
        (
            RADII,
            "radii",
            ValueType::Radii,
            "Corner radii in logical pixels.",
        ),
        (
            BORDER,
            "border",
            ValueType::Border,
            "Solid border with uniform or per-edge widths.",
        ),
        (SHADOW, "shadow", ValueType::Shadow, "Drop or inset shadow."),
    ];
    defs.into_iter()
        .map(|(id, name, ty, docs)| PropertySchema::new(id, name, ty, docs))
        .collect()
}

/// Applies alignment, rounded clips, per-edge borders, and shadows.
/// Returns an adapter error for an unsupported alignment or nonfinite geometry.
pub(super) fn apply(
    mut element: Element,
    input: &NativeElementInput,
) -> Result<Element, SchemaError> {
    if let Some(value) = string(input, ALIGN_SELF) {
        element = element.align_self(match value.as_str() {
            "start" => AlignSelf::FLEX_START,
            "center" => AlignSelf::CENTER,
            "end" => AlignSelf::FLEX_END,
            "stretch" => AlignSelf::STRETCH,
            _ => return Err(invalid("alignSelf", value)),
        });
    }
    if let Some(value) = string(input, JUSTIFY_ITEMS) {
        element = element.justify_items(match value.as_str() {
            "start" => JustifyItems::START,
            "center" => JustifyItems::CENTER,
            "end" => JustifyItems::END,
            "stretch" => JustifyItems::STRETCH,
            _ => return Err(invalid("justifyItems", value)),
        });
    }
    if let Some(value) = string(input, JUSTIFY_SELF) {
        element = element.justify_self(match value.as_str() {
            "start" => JustifySelf::START,
            "center" => JustifySelf::CENTER,
            "end" => JustifySelf::END,
            "stretch" => JustifySelf::STRETCH,
            _ => return Err(invalid("justifySelf", value)),
        });
    }
    if let Some(value) = string(input, ALIGN_CONTENT) {
        element = element.align_content(match value.as_str() {
            "start" => AlignContent::START,
            "center" => AlignContent::CENTER,
            "end" => AlignContent::END,
            "stretch" => AlignContent::STRETCH,
            "spaceBetween" => AlignContent::SPACE_BETWEEN,
            "spaceAround" => AlignContent::SPACE_AROUND,
            "spaceEvenly" => AlignContent::SPACE_EVENLY,
            _ => return Err(invalid("alignContent", value)),
        });
    }
    if let Some(value) = string(input, GRID_AUTO_FLOW) {
        element = element.grid_auto_flow(match value.as_str() {
            "row" => GridAutoFlow::Row,
            "column" => GridAutoFlow::Column,
            "rowDense" => GridAutoFlow::RowDense,
            "columnDense" => GridAutoFlow::ColumnDense,
            _ => return Err(invalid("gridAutoFlow", value)),
        });
    }
    let radii = match input.get(RADII) {
        Some(SchemaValue::Radii(value)) => *value,
        _ => element.paint.quad.radii,
    };
    if radii
        .as_array()
        .iter()
        .any(|radius| !radius.is_finite() || *radius < 0.0)
    {
        return Err(SchemaError::Adapter(
            "radii must be finite and nonnegative".into(),
        ));
    }
    if matches!(input.get(CLIP), Some(SchemaValue::Bool(true))) {
        element = element.clip(radii);
    } else if input.get(RADII).is_some() {
        element = element.radius(radii);
    }
    if let Some(SchemaValue::Border(value)) = input.get(BORDER) {
        element = element.border(*value);
    }
    if let Some(SchemaValue::Shadow(value)) = input.get(SHADOW) {
        element = element.shadow(*value);
    }
    Ok(element)
}

/// Creates an adapter error naming an unsupported string value.
fn invalid(name: &str, value: &str) -> SchemaError {
    SchemaError::Adapter(format!("unsupported {name} `{value}`"))
}
