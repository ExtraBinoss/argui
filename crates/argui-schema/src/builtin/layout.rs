//! Layout properties shared by native containers, including grid tracks.

use argui_core::{Transform2D, TransformOrigin};
use argui_ui::{
    ContainerQuery, ContainerScopeId, Dimensions, Element, GridPlacement, GridTemplateComponent,
    LengthPercentage, LengthPercentageAuto, Line, Position, StylePatch, TrackSizingFunction, auto,
    fr, length, line, minmax,
};

use super::*;

mod visual;

/// Declares the layout properties accepted by every native container.
/// Returns the stable property IDs and their public types.
pub(super) fn properties() -> Vec<PropertySchema> {
    let defs = [
        (
            GRID_ROWS,
            "grid_rows",
            ValueType::String,
            "Whitespace-separated grid row tracks: px, %, fr, auto, or minmax(px,fr).",
        ),
        (
            GRID_COLUMNS,
            "grid_columns",
            ValueType::String,
            "Whitespace-separated grid column tracks.",
        ),
        (
            GRID_ROW_START,
            "grid_row_start",
            ValueType::Int,
            "One-based grid row line.",
        ),
        (
            GRID_ROW_SPAN,
            "grid_row_span",
            ValueType::Int,
            "Number of grid rows occupied.",
        ),
        (
            GRID_COLUMN_START,
            "grid_column_start",
            ValueType::Int,
            "One-based grid column line.",
        ),
        (
            GRID_COLUMN_SPAN,
            "grid_column_span",
            ValueType::Int,
            "Number of grid columns occupied.",
        ),
        (
            MAX_WIDTH,
            "max_width",
            ValueType::Float,
            "Maximum width in logical pixels.",
        ),
        (
            MAX_HEIGHT,
            "max_height",
            ValueType::Float,
            "Maximum height in logical pixels.",
        ),
        (
            ASPECT_RATIO,
            "aspect_ratio",
            ValueType::Float,
            "Preferred width divided by height.",
        ),
        (
            FLEX_BASIS,
            "flex_basis",
            ValueType::Dimension,
            "Initial flex size.",
        ),
        (
            ROW_GAP,
            "row_gap",
            ValueType::Float,
            "Space between grid or flex rows.",
        ),
        (
            COLUMN_GAP,
            "column_gap",
            ValueType::Float,
            "Space between grid or flex columns.",
        ),
        (
            PADDING_LEFT,
            "padding_left",
            ValueType::Float,
            "Left inner spacing.",
        ),
        (
            PADDING_RIGHT,
            "padding_right",
            ValueType::Float,
            "Right inner spacing.",
        ),
        (
            PADDING_TOP,
            "padding_top",
            ValueType::Float,
            "Top inner spacing.",
        ),
        (
            PADDING_BOTTOM,
            "padding_bottom",
            ValueType::Float,
            "Bottom inner spacing.",
        ),
        (
            MARGIN_LEFT,
            "margin_left",
            ValueType::Float,
            "Left outer spacing.",
        ),
        (
            MARGIN_RIGHT,
            "margin_right",
            ValueType::Float,
            "Right outer spacing.",
        ),
        (
            MARGIN_TOP,
            "margin_top",
            ValueType::Float,
            "Top outer spacing.",
        ),
        (
            MARGIN_BOTTOM,
            "margin_bottom",
            ValueType::Float,
            "Bottom outer spacing.",
        ),
        (
            SCALE_X,
            "scale_x",
            ValueType::Float,
            "Horizontal visual scale.",
        ),
        (
            SCALE_Y,
            "scale_y",
            ValueType::Float,
            "Vertical visual scale.",
        ),
        (
            TRANSLATE_X,
            "translate_x",
            ValueType::Float,
            "Horizontal visual translation.",
        ),
        (
            TRANSLATE_Y,
            "translate_y",
            ValueType::Float,
            "Vertical visual translation.",
        ),
        (
            ORIGIN_X,
            "origin_x",
            ValueType::Float,
            "Horizontal transform pivot fraction.",
        ),
        (
            ORIGIN_Y,
            "origin_y",
            ValueType::Float,
            "Vertical transform pivot fraction.",
        ),
        (
            POSITION,
            "position",
            ValueType::String,
            "relative, absolute, or sticky positioning.",
        ),
        (
            INSET_LEFT,
            "inset_left",
            ValueType::Float,
            "Left positioned inset in logical pixels.",
        ),
        (
            INSET_RIGHT,
            "inset_right",
            ValueType::Float,
            "Right positioned inset in logical pixels.",
        ),
        (
            INSET_TOP,
            "inset_top",
            ValueType::Float,
            "Top positioned inset in logical pixels.",
        ),
        (
            INSET_BOTTOM,
            "inset_bottom",
            ValueType::Float,
            "Bottom positioned inset in logical pixels.",
        ),
        (
            Z_INDEX,
            "z_index",
            ValueType::Int,
            "Stacking order within the window layer.",
        ),
        (
            QUERY_SCOPE,
            "query_scope",
            ValueType::String,
            "Name of this element's container-query scope.",
        ),
        (
            QUERY_MIN_WIDTH,
            "query_min_width",
            ValueType::Float,
            "Minimum named container width for alternate grid columns.",
        ),
        (
            QUERY_COLUMNS,
            "query_columns",
            ValueType::String,
            "Grid columns used at the named container breakpoint.",
        ),
    ];
    let mut properties: Vec<_> = defs
        .into_iter()
        .map(|(id, name, ty, docs)| PropertySchema::new(id, name, ty, docs))
        .collect();
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
    if let Some(columns) = string(input, GRID_COLUMNS) {
        element = element.grid_template_columns(parse_tracks(columns)?);
    }
    if let Some(rows) = string(input, GRID_ROWS) {
        element = element.grid_template_rows(parse_tracks(rows)?);
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
    if let Some(value) = nonnegative(input, MAX_WIDTH, "max_width")? {
        max.width = length(value);
    }
    if let Some(value) = nonnegative(input, MAX_HEIGHT, "max_height")? {
        max.height = length(value);
    }
    if input.get(MAX_WIDTH).is_some() || input.get(MAX_HEIGHT).is_some() {
        element = element.max_size(max);
    }
    if let Some(value) = positive(input, ASPECT_RATIO, "aspect_ratio")? {
        element = element.aspect_ratio(value);
    }
    if let Some(SchemaValue::Dimension(value)) = input.get(FLEX_BASIS) {
        element = element.flex_basis(*value);
    }
    if let Some(value) = nonnegative(input, ROW_GAP, "row_gap")? {
        element = element.row_gap(value);
    }
    if let Some(value) = nonnegative(input, COLUMN_GAP, "column_gap")? {
        element = element.column_gap(value);
    }
    let mut padding = element.style.padding;
    let mut margin = element.style.margin;
    for (id, side) in [
        (PADDING_LEFT, &mut padding.left),
        (PADDING_RIGHT, &mut padding.right),
        (PADDING_TOP, &mut padding.top),
        (PADDING_BOTTOM, &mut padding.bottom),
    ] {
        if let Some(value) = nonnegative(input, id, "padding side")? {
            *side = length(value);
        }
    }
    for (id, side) in [
        (MARGIN_LEFT, &mut margin.left),
        (MARGIN_RIGHT, &mut margin.right),
        (MARGIN_TOP, &mut margin.top),
        (MARGIN_BOTTOM, &mut margin.bottom),
    ] {
        if let Some(value) = scalar(input, id, "margin side")? {
            *side = length(value);
        }
    }
    element = element.padding(padding).margin(margin);
    if let Some(mode) = string(input, POSITION) {
        element = element.position(match mode.as_str() {
            "relative" => Position::Relative,
            "absolute" => Position::Absolute,
            "sticky" => Position::Sticky,
            _ => {
                return Err(SchemaError::Adapter(format!(
                    "unsupported position `{mode}`"
                )));
            }
        });
    }
    let mut inset = element.style.inset;
    for (id, side) in [
        (INSET_LEFT, &mut inset.left),
        (INSET_RIGHT, &mut inset.right),
        (INSET_TOP, &mut inset.top),
        (INSET_BOTTOM, &mut inset.bottom),
    ] {
        if let Some(value) = scalar(input, id, "inset side")? {
            *side = length(value);
        }
    }
    element = element.inset(inset);
    if let Some(SchemaValue::Int(value)) = input.get(Z_INDEX) {
        let index = i32::try_from(*value)
            .map_err(|_| SchemaError::Adapter("z_index exceeds i32 range".into()))?;
        element = element.z_index(index);
    }
    let transform = Transform2D::IDENTITY
        .translate(
            scalar(input, TRANSLATE_X, "translate_x")?.unwrap_or(0.0),
            scalar(input, TRANSLATE_Y, "translate_y")?.unwrap_or(0.0),
        )
        .scale(
            scalar(input, SCALE_X, "scale_x")?.unwrap_or(1.0),
            scalar(input, SCALE_Y, "scale_y")?.unwrap_or(1.0),
        )
        .rotate(
            scalar(input, ROTATION, "rotation")?
                .unwrap_or(0.0)
                .to_radians(),
        );
    if [TRANSLATE_X, TRANSLATE_Y, SCALE_X, SCALE_Y, ROTATION]
        .iter()
        .any(|id| input.get(*id).is_some())
    {
        element = element.transform(transform);
    }
    if input.get(ORIGIN_X).is_some() || input.get(ORIGIN_Y).is_some() {
        element = element.transform_origin(TransformOrigin::new(
            scalar(input, ORIGIN_X, "origin_x")?.unwrap_or(0.5),
            scalar(input, ORIGIN_Y, "origin_y")?.unwrap_or(0.5),
        ));
    }
    if let Some(scope) = string(input, QUERY_SCOPE) {
        element = element.container_scope(ContainerScopeId::from_owned(scope.clone()));
    }
    if input.get(QUERY_MIN_WIDTH).is_some() || input.get(QUERY_COLUMNS).is_some() {
        let width = nonnegative(input, QUERY_MIN_WIDTH, "query_min_width")?.ok_or_else(|| {
            SchemaError::Adapter(format!(
                "{name} requires query_min_width with query_columns"
            ))
        })?;
        let columns = string(input, QUERY_COLUMNS).ok_or_else(|| {
            SchemaError::Adapter(format!(
                "{name} requires query_columns with query_min_width"
            ))
        })?;
        let scope = string(input, QUERY_SCOPE).ok_or_else(|| {
            SchemaError::Adapter(format!(
                "{name} requires query_scope for responsive columns"
            ))
        })?;
        let mut style = element.style.clone();
        style.grid_template_columns = parse_tracks(columns)?;
        element = element.when(
            ContainerQuery::min_width(ContainerScopeId::from_owned(scope.clone()), width),
            StylePatch::new().layout(style),
        );
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

/// Parses a whitespace-separated sequence of grid tracks.
/// Supports `auto`, logical `px`, percentages, fractions, and `minmax(px,fr)`.
/// Returns an adapter error for malformed or unsupported tracks.
fn parse_tracks(value: &str) -> Result<Vec<GridTemplateComponent<String>>, SchemaError> {
    value
        .split_whitespace()
        .map(|token| {
            let track: TrackSizingFunction = if token == "auto" {
                auto()
            } else if let Some(raw) = token.strip_suffix("fr") {
                fr(parse_positive(raw, token)?)
            } else if let Some(raw) = token.strip_suffix("px") {
                TrackSizingFunction::from(LengthPercentage::length(parse_nonnegative(raw, token)?))
            } else if let Some(raw) = token.strip_suffix('%') {
                TrackSizingFunction::from(LengthPercentage::percent(
                    parse_nonnegative(raw, token)? / 100.0,
                ))
            } else if let Some(raw) = token
                .strip_prefix("minmax(")
                .and_then(|raw| raw.strip_suffix(')'))
            {
                let (min, max) = raw
                    .split_once(',')
                    .ok_or_else(|| SchemaError::Adapter(format!("invalid grid track `{token}`")))?;
                let min = min
                    .strip_suffix("px")
                    .ok_or_else(|| SchemaError::Adapter(format!("invalid grid track `{token}`")))?;
                let max = max
                    .strip_suffix("fr")
                    .ok_or_else(|| SchemaError::Adapter(format!("invalid grid track `{token}`")))?;
                minmax(
                    length(parse_nonnegative(min, token)?),
                    fr(parse_positive(max, token)?),
                )
            } else {
                return Err(SchemaError::Adapter(format!(
                    "invalid grid track `{token}`"
                )));
            };
            Ok(GridTemplateComponent::Single(track))
        })
        .collect()
}

/// Parses a finite nonnegative track amount. Returns an adapter error for invalid input.
fn parse_nonnegative(raw: &str, token: &str) -> Result<f32, SchemaError> {
    let value = raw
        .parse::<f32>()
        .map_err(|_| SchemaError::Adapter(format!("invalid grid track `{token}`")))?;
    if !value.is_finite() || value < 0.0 {
        return Err(SchemaError::Adapter(format!(
            "invalid grid track `{token}`"
        )));
    }
    Ok(value)
}

/// Parses a finite positive track amount. Returns an adapter error for invalid input.
fn parse_positive(raw: &str, token: &str) -> Result<f32, SchemaError> {
    let value = parse_nonnegative(raw, token)?;
    if value == 0.0 {
        return Err(SchemaError::Adapter(format!(
            "invalid grid track `{token}`"
        )));
    }
    Ok(value)
}
