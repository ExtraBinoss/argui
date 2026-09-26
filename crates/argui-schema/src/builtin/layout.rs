//! Layout properties shared by native containers, including grid tracks.

use argui_ui::{
    ContainerScopeId, Dimensions, Element, GridPlacement, LengthPercentageAuto, Line,
    StyleCondition, StylePatch, length, line,
};

use super::*;

mod visual;

/// Declares the layout properties accepted by every native container.
/// Returns the stable property IDs and their public types.
pub(super) fn properties() -> Vec<PropertySchema> {
    let defs = [
        (
            GRID_ROWS,
            "gridRows",
            ValueType::GridTracks,
            "Typed grid row tracks, including minmax and adaptive repetition.",
        ),
        (
            GRID_COLUMNS,
            "gridColumns",
            ValueType::GridTracks,
            "Typed grid column tracks, including minmax and adaptive repetition.",
        ),
        (
            GRID_ROW_START,
            "gridRowStart",
            ValueType::Int,
            "One-based grid row line.",
        ),
        (
            GRID_ROW_SPAN,
            "gridRowSpan",
            ValueType::Int,
            "Number of grid rows occupied.",
        ),
        (
            GRID_COLUMN_START,
            "gridColumnStart",
            ValueType::Int,
            "One-based grid column line.",
        ),
        (
            GRID_COLUMN_SPAN,
            "gridColumnSpan",
            ValueType::Int,
            "Number of grid columns occupied.",
        ),
        (
            MAX_WIDTH,
            "maxWidth",
            ValueType::Constraint,
            "Maximum width as pixels, a percentage, or auto.",
        ),
        (
            MAX_HEIGHT,
            "maxHeight",
            ValueType::Constraint,
            "Maximum height as pixels, a percentage, or auto.",
        ),
        (
            ASPECT_RATIO,
            "aspectRatio",
            ValueType::Float,
            "Preferred width divided by height.",
        ),
        (
            FLEX_BASIS,
            "flexBasis",
            ValueType::Dimension,
            "Initial flex size.",
        ),
        (
            ROW_GAP,
            "rowGap",
            ValueType::Float,
            "Space between grid or flex rows.",
        ),
        (
            COLUMN_GAP,
            "columnGap",
            ValueType::Float,
            "Space between grid or flex columns.",
        ),
        (
            Z_INDEX,
            "zIndex",
            ValueType::Int,
            "Stacking order within the window layer.",
        ),
        (
            QUERY_SCOPE,
            "containerScope",
            ValueType::String,
            "Name of this element's container-query scope.",
        ),
        (
            MARGIN,
            "margin",
            ValueType::Insets,
            "Outer spacing; start/end follow writing direction.",
        ),
        (
            TRANSFORM,
            "transform",
            ValueType::Transform,
            "Visual transform; does not reserve layout space.",
        ),
        (
            CONTAINER_RULES,
            "containerRules",
            ValueType::ContainerRules,
            "Typed layout overrides selected by an ancestor container's size.",
        ),
    ];
    let mut properties = defs
        .into_iter()
        .map(|(id, name, ty, docs)| PropertySchema::new(id, name, ty, docs))
        .collect::<Vec<_>>();
    properties.extend(visual::properties());
    properties
}

/// Applies validated layout input to a native container.
/// `name` appears in adapter diagnostics. Returns an error for malformed
/// track definitions, grid placement, or incomplete responsive declarations.
pub(super) fn apply(
    mut element: Element,
    input: &NativeElementInput,
    name: &str,
) -> Result<Element, SchemaError> {
    if let Some(SchemaValue::GridTracks(columns)) = input.get(GRID_COLUMNS) {
        element = element.grid_template_columns(columns.clone());
    }
    if let Some(SchemaValue::GridTracks(rows)) = input.get(GRID_ROWS) {
        element = element.grid_template_rows(rows.clone());
    }
    if let Some(line) = placement(input, GRID_ROW_START, GRID_ROW_SPAN, name)? {
        element = element.grid_row(line);
    }
    if let Some(line) = placement(input, GRID_COLUMN_START, GRID_COLUMN_SPAN, name)? {
        element = element.grid_column(line);
    }
    let mut max = Dimensions {
        width: LengthPercentageAuto::auto(),
        height: LengthPercentageAuto::auto(),
    };
    if let Some(SchemaValue::Constraint(value)) = input.get(MAX_WIDTH) {
        validate_constraint(*value, "maxWidth")?;
        max.width = *value;
    }
    if let Some(SchemaValue::Constraint(value)) = input.get(MAX_HEIGHT) {
        validate_constraint(*value, "maxHeight")?;
        max.height = *value;
    }
    if input.get(MAX_WIDTH).is_some() || input.get(MAX_HEIGHT).is_some() {
        element = element.max_size(max);
    }
    if let Some(value) = positive(input, ASPECT_RATIO, "aspectRatio")? {
        element = element.aspect_ratio(value);
    }
    if let Some(SchemaValue::Dimension(value)) = input.get(FLEX_BASIS) {
        element = element.flex_basis(*value);
    }
    if let Some(value) = nonnegative(input, ROW_GAP, "rowGap")? {
        element = element.row_gap(value);
    }
    if let Some(value) = nonnegative(input, COLUMN_GAP, "columnGap")? {
        element = element.column_gap(value);
    }
    if let Some(SchemaValue::Insets(value)) = input.get(MARGIN) {
        element = element.layout_margin(*value);
    }
    if let Some(SchemaValue::Int(value)) = input.get(Z_INDEX) {
        let index = i32::try_from(*value)
            .map_err(|_| SchemaError::Adapter("z_index exceeds i32 range".into()))?;
        element = element.z_index(index);
    }
    if let Some(SchemaValue::Transform(value)) = input.get(TRANSFORM) {
        element = element.transform(*value);
    }
    if let Some(scope) = string(input, QUERY_SCOPE) {
        element = element.container_scope(ContainerScopeId::from_owned(scope.clone()));
    }
    if let Some(SchemaValue::ContainerRules(rules)) = input.get(CONTAINER_RULES) {
        for rule in rules {
            let mut style = element.style.clone();
            let override_style = &rule.style;
            if let Some(rows) = &override_style.grid_rows {
                style.grid_template_rows = rows.clone();
            }
            if let Some(columns) = &override_style.grid_columns {
                style.grid_template_columns = columns.clone();
            }
            if let Some(width) = override_style.width {
                style.size.width = width;
            }
            if let Some(height) = override_style.height {
                style.size.height = height;
            }
            if let Some(gap) = override_style.gap {
                style.gap.width = length(gap);
                style.gap.height = length(gap);
            }
            if let Some(grow) = override_style.grow {
                style.flex_grow = grow;
            }
            if let Some(shrink) = override_style.shrink {
                style.flex_shrink = shrink;
            }
            if let Some(align) = override_style.align_items {
                style.align_items = Some(align);
            }
            if let Some(justify) = override_style.justify_content {
                style.justify_content = Some(justify);
            }
            let condition = StyleCondition::all(
                rule.conditions
                    .iter()
                    .cloned()
                    .map(StyleCondition::container),
            );
            element = element.when(condition, StylePatch::new().layout(style));
        }
    }
    visual::apply(element, input)
}

/// Reads a validated string property, if supplied.
fn string(input: &NativeElementInput, id: PropertyId) -> Option<&String> {
    match input.get(id) {
        Some(SchemaValue::String(value)) => Some(value),
        _ => None,
    }
}

/// Reads a finite scalar, preserving an omitted property as `None`.
/// Returns an adapter error for nonfinite values.
fn scalar(
    input: &NativeElementInput,
    id: PropertyId,
    name: &str,
) -> Result<Option<f32>, SchemaError> {
    match input.get(id) {
        Some(SchemaValue::Float(value)) if value.is_finite() => Ok(Some(*value)),
        Some(SchemaValue::Float(_)) => Err(SchemaError::Adapter(format!("{name} must be finite"))),
        _ => Ok(None),
    }
}

/// Reads a finite nonnegative scalar. Returns an error for negative values.
fn nonnegative(
    input: &NativeElementInput,
    id: PropertyId,
    name: &str,
) -> Result<Option<f32>, SchemaError> {
    let value = scalar(input, id, name)?;
    if value.is_some_and(|value| value < 0.0) {
        return Err(SchemaError::Adapter(format!("{name} must be nonnegative")));
    }
    Ok(value)
}

/// Reads a finite positive scalar. Returns an error for zero or negative values.
fn positive(
    input: &NativeElementInput,
    id: PropertyId,
    name: &str,
) -> Result<Option<f32>, SchemaError> {
    let value = scalar(input, id, name)?;
    if value.is_some_and(|value| value <= 0.0) {
        return Err(SchemaError::Adapter(format!("{name} must be positive")));
    }
    Ok(value)
}

/// Parses one-based grid start and span properties into a placement line.
/// Returns an adapter error for indices or spans outside Taffy's range.
fn placement(
    input: &NativeElementInput,
    start: PropertyId,
    span: PropertyId,
    name: &str,
) -> Result<Option<Line<GridPlacement<String>>>, SchemaError> {
    let start = match input.get(start) {
        Some(SchemaValue::Int(value)) => Some(*value),
        _ => None,
    };
    let span = match input.get(span) {
        Some(SchemaValue::Int(value)) => Some(*value),
        _ => None,
    };
    if start.is_none() && span.is_none() {
        return Ok(None);
    }
    let start = start
        .map(|value| {
            i16::try_from(value)
                .ok()
                .filter(|value| *value != 0)
                .map(line)
                .ok_or_else(|| {
                    SchemaError::Adapter(format!("{name} grid start must be a nonzero 16-bit line"))
                })
        })
        .transpose()?
        .unwrap_or(GridPlacement::Auto);
    let end = span
        .map(|value| {
            u16::try_from(value)
                .ok()
                .filter(|value| *value != 0)
                .map(GridPlacement::Span)
                .ok_or_else(|| {
                    SchemaError::Adapter(format!(
                        "{name} grid span must be a positive 16-bit count"
                    ))
                })
        })
        .transpose()?
        .unwrap_or(GridPlacement::Auto);
    Ok(Some(Line { start, end }))
}
