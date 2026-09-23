//! Alignment and painted edges of native DSL containers.

use argui_paint::{Border, BorderWidths, Shadow};
use argui_ui::{AlignContent, AlignSelf, Element, GridAutoFlow, JustifyItems, JustifySelf};

use super::*;

/// Declares native alignment and painted-edge properties.
/// Returns properties with stable IDs for DSL checking.
pub(super) fn properties() -> Vec<PropertySchema> {
    let defs = [
        (
            ALIGN_SELF,
            "align_self",
            ValueType::String,
            "Alignment within the parent cross axis.",
        ),
        (
            JUSTIFY_ITEMS,
            "justify_items",
            ValueType::String,
            "Horizontal alignment of grid items.",
        ),
        (
            JUSTIFY_SELF,
            "justify_self",
            ValueType::String,
            "Horizontal alignment in a grid cell.",
        ),
        (
            ALIGN_CONTENT,
            "align_content",
            ValueType::String,
            "Distribution of grid or flex rows.",
        ),
        (
            GRID_AUTO_FLOW,
            "grid_auto_flow",
            ValueType::String,
            "Implicit grid placement: row, column, row_dense, or column_dense.",
        ),
        (
            RADIUS_TOP_LEFT,
            "radius_top_left",
            ValueType::Float,
            "Top-left corner radius.",
        ),
        (
            RADIUS_TOP_RIGHT,
            "radius_top_right",
            ValueType::Float,
            "Top-right corner radius.",
        ),
        (
            RADIUS_BOTTOM_RIGHT,
            "radius_bottom_right",
            ValueType::Float,
            "Bottom-right corner radius.",
        ),
        (
            RADIUS_BOTTOM_LEFT,
            "radius_bottom_left",
            ValueType::Float,
            "Bottom-left corner radius.",
        ),
        (
            BORDER_LEFT,
            "border_left",
            ValueType::Float,
            "Left border width.",
        ),
        (
            BORDER_RIGHT,
            "border_right",
            ValueType::Float,
            "Right border width.",
        ),
        (
            BORDER_TOP,
            "border_top",
            ValueType::Float,
            "Top border width.",
        ),
        (
            BORDER_BOTTOM,
            "border_bottom",
            ValueType::Float,
            "Bottom border width.",
        ),
        (
            SHADOW_OFFSET_X,
            "shadow_offset_x",
            ValueType::Float,
            "Horizontal shadow offset.",
        ),
        (
            SHADOW_SPREAD,
            "shadow_spread",
            ValueType::Float,
            "Shadow spread radius.",
        ),
        (
            CLIP,
            "clip",
            ValueType::Bool,
            "Clip descendants to rounded bounds.",
        ),
        (
            SHADOW_BLUR,
            "shadow_blur",
            ValueType::Float,
            "Drop shadow blur radius.",
        ),
        (
            SHADOW_OFFSET_Y,
            "shadow_offset_y",
            ValueType::Float,
            "Vertical shadow offset.",
        ),
        (
            SHADOW_COLOR,
            "shadow_color",
            ValueType::Color,
            "Drop shadow color.",
        ),
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
            _ => return Err(invalid("align_self", value)),
        });
    }
    if let Some(value) = string(input, JUSTIFY_ITEMS) {
        element = element.justify_items(match value.as_str() {
            "start" => JustifyItems::START,
            "center" => JustifyItems::CENTER,
            "end" => JustifyItems::END,
            "stretch" => JustifyItems::STRETCH,
            _ => return Err(invalid("justify_items", value)),
        });
    }
    if let Some(value) = string(input, JUSTIFY_SELF) {
        element = element.justify_self(match value.as_str() {
            "start" => JustifySelf::START,
            "center" => JustifySelf::CENTER,
            "end" => JustifySelf::END,
            "stretch" => JustifySelf::STRETCH,
            _ => return Err(invalid("justify_self", value)),
        });
    }
    if let Some(value) = string(input, ALIGN_CONTENT) {
        element = element.align_content(match value.as_str() {
            "start" => AlignContent::START,
            "center" => AlignContent::CENTER,
            "end" => AlignContent::END,
            "stretch" => AlignContent::STRETCH,
            "space_between" => AlignContent::SPACE_BETWEEN,
            "space_around" => AlignContent::SPACE_AROUND,
            "space_evenly" => AlignContent::SPACE_EVENLY,
            _ => return Err(invalid("align_content", value)),
        });
    }
    if let Some(value) = string(input, GRID_AUTO_FLOW) {
        element = element.grid_auto_flow(match value.as_str() {
            "row" => GridAutoFlow::Row,
            "column" => GridAutoFlow::Column,
            "row_dense" => GridAutoFlow::RowDense,
            "column_dense" => GridAutoFlow::ColumnDense,
            _ => return Err(invalid("grid_auto_flow", value)),
        });
    }
    let mut radii = element.paint.quad.radii;
    for (id, corner) in [
        (RADIUS_TOP_LEFT, &mut radii.top_left),
        (RADIUS_TOP_RIGHT, &mut radii.top_right),
        (RADIUS_BOTTOM_RIGHT, &mut radii.bottom_right),
        (RADIUS_BOTTOM_LEFT, &mut radii.bottom_left),
    ] {
        if let Some(value) = nonnegative(input, id, "corner radius")? {
            *corner = value;
        }
    }
    if matches!(input.get(CLIP), Some(SchemaValue::Bool(true))) {
        element = element.clip(radii);
    } else if [
        RADIUS_TOP_LEFT,
        RADIUS_TOP_RIGHT,
        RADIUS_BOTTOM_RIGHT,
        RADIUS_BOTTOM_LEFT,
    ]
    .iter()
    .any(|id| input.get(*id).is_some())
    {
        element = element.radius(radii);
    }
    let mut widths = element
        .paint
        .quad
        .border
        .map_or(BorderWidths::default(), |border| border.widths);
    let mut has_border = false;
    for (id, side) in [
        (BORDER_LEFT, &mut widths.left),
        (BORDER_RIGHT, &mut widths.right),
        (BORDER_TOP, &mut widths.top),
        (BORDER_BOTTOM, &mut widths.bottom),
    ] {
        if let Some(value) = nonnegative(input, id, "border width")? {
            *side = value;
            has_border = true;
        }
    }
    if has_border {
        let color = match input.get(BORDER_COLOR) {
            Some(SchemaValue::Color(color)) => *color,
            _ => argui_core::Color::BLACK,
        };
        element = element.border(Border { widths, color });
    }
    if let Some(blur) = nonnegative(input, SHADOW_BLUR, "shadow_blur")? {
        let x = scalar(input, SHADOW_OFFSET_X, "shadow_offset_x")?.unwrap_or(0.0);
        let y = scalar(input, SHADOW_OFFSET_Y, "shadow_offset_y")?.unwrap_or(0.0);
        let spread = scalar(input, SHADOW_SPREAD, "shadow_spread")?.unwrap_or(0.0);
        let color = match input.get(SHADOW_COLOR) {
            Some(SchemaValue::Color(color)) => *color,
            _ => argui_core::Color::srgba(0.0, 0.0, 0.0, 0.18),
        };
        element = element.shadow(Shadow::drop([x, y], blur, color).spread(spread));
    }
    Ok(element)
}

/// Creates an adapter error naming an unsupported string value.
fn invalid(name: &str, value: &str) -> SchemaError {
    SchemaError::Adapter(format!("unsupported {name} `{value}`"))
}
