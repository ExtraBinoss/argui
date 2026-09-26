//! Typed grid tracks and container query rule decoding.

use super::*;

/// Decodes a bounded array of typed grid tracks.
///
/// * `value` — array of fixed, fractional, minmax, or repeated tracks.
///
/// Returns `None` for unsupported shapes or unbounded repetition definitions.
pub(super) fn grid_tracks(value: &Value) -> Option<Vec<GridTemplateComponent<String>>> {
    let values = value.as_array()?;
    if values.len() > 256 {
        return None;
    }
    values.iter().map(grid_track).collect()
}

/// Decodes a single grid template component.
///
/// * `value` — one track or a repeat object.
///
/// Returns `None` for nested repeats and malformed repetition counts.
fn grid_track(value: &Value) -> Option<GridTemplateComponent<String>> {
    if let Some(track) = grid_track_single(value) {
        return Some(GridTemplateComponent::Single(track));
    }
    let object = value.as_object()?;
    if object.len() != 1 {
        return None;
    }
    let repeated = object.get("repeat")?.as_object()?;
    if repeated.len() != 2 {
        return None;
    }
    let count = match repeated.get("count")? {
        Value::String(name) if name == "autoFit" => RepetitionCount::AutoFit,
        Value::String(name) if name == "autoFill" => RepetitionCount::AutoFill,
        value => RepetitionCount::Count(
            u16::try_from(value.as_u64()?)
                .ok()
                .filter(|count| *count > 0)?,
        ),
    };
    let values = repeated.get("tracks")?.as_array()?;
    if values.is_empty() || values.len() > 256 {
        return None;
    }
    let tracks = values
        .iter()
        .map(grid_track_single)
        .collect::<Option<Vec<_>>>()?;
    Some(repeat(count, tracks))
}

/// Decodes one non-repeated track sizing function.
///
/// * `value` — logical-pixel length, percentage, auto, fraction, or minmax.
///
/// Returns `None` for invalid lengths and object keys.
fn grid_track_single(value: &Value) -> Option<TrackSizingFunction> {
    if let Some(pixels) = number(value) {
        return Some(TrackSizingFunction::from(LengthPercentage::length(
            nonnegative(pixels)?,
        )));
    }
    if value.as_str() == Some("auto") {
        return Some(auto());
    }
    if let Some(percent) = value.as_str().and_then(|text| text.strip_suffix('%')) {
        let percent = percent
            .parse::<f32>()
            .ok()
            .filter(|value| value.is_finite() && *value >= 0.0)?;
        return Some(TrackSizingFunction::from(LengthPercentage::percent(
            percent / 100.0,
        )));
    }
    let object = value.as_object()?;
    if object.len() != 1 {
        return None;
    }
    if let Some(value) = object.get("fr") {
        return Some(fr(number(value).filter(|value| *value > 0.0)?));
    }
    let minmax_value = object.get("minmax")?.as_object()?;
    if minmax_value.len() != 2 {
        return None;
    }
    let min = length(nonnegative(number(minmax_value.get("min")?)?)?);
    let max = if let Some(pixels) = number(minmax_value.get("max")?) {
        length(nonnegative(pixels)?)
    } else {
        let fraction = minmax_value.get("max")?.as_object()?;
        if fraction.len() != 1 {
            return None;
        }
        fr(number(fraction.get("fr")?).filter(|value| *value > 0.0)?)
    };
    Some(minmax(min, max))
}

/// Decodes bounded container size rules with typed layout overrides.
///
/// * `value` — array of `{scope, when, style}` objects.
///
/// Returns `None` for invalid conditions, styles, or contradictory bounds.
pub(super) fn container_rules(value: &Value) -> Option<Vec<ContainerRule>> {
    let rules = value.as_array()?;
    if rules.len() > 64 {
        return None;
    }
    rules.iter().map(container_rule).collect()
}

/// Decodes one query and its nonempty layout override.
///
/// * `value` — query object using one named ancestor scope.
///
/// Returns `None` for unsupported keys or empty conditions.
fn container_rule(value: &Value) -> Option<ContainerRule> {
    let object = value.as_object()?;
    if object.len() != 3 {
        return None;
    }
    let scope = object.get("scope")?.as_str()?;
    if scope.is_empty() {
        return None;
    }
    let scope = ContainerScopeId::from_owned(scope.to_owned());
    let when = object.get("when")?.as_object()?;
    if when.is_empty()
        || when.keys().any(|key| {
            !matches!(
                key.as_str(),
                "minWidth" | "maxWidth" | "minHeight" | "maxHeight" | "orientation"
            )
        })
    {
        return None;
    }
    for (minimum, maximum) in [("minWidth", "maxWidth"), ("minHeight", "maxHeight")] {
        if let (Some(min), Some(max)) = (when.get(minimum), when.get(maximum))
            && nonnegative(number(min)?)? >= nonnegative(number(max)?)?
        {
            return None;
        }
    }
    let mut conditions = Vec::with_capacity(when.len());
    for (key, value) in when {
        let query = match key.as_str() {
            "minWidth" => ContainerQuery::min_width(scope.clone(), nonnegative(number(value)?)?),
            "maxWidth" => ContainerQuery::max_width(scope.clone(), nonnegative(number(value)?)?),
            "minHeight" => ContainerQuery::min_height(scope.clone(), nonnegative(number(value)?)?),
            "maxHeight" => ContainerQuery::max_height(scope.clone(), nonnegative(number(value)?)?),
            "orientation" => match value.as_str()? {
                "landscape" => ContainerQuery::landscape(scope.clone()),
                "portrait" => ContainerQuery::portrait(scope.clone()),
                _ => return None,
            },
            _ => return None,
        };
        conditions.push(query);
    }
    let style = container_rule_style(object.get("style")?)?;
    Some(ContainerRule { conditions, style })
}

/// Decodes layout fields supported by a container rule.
///
/// * `value` — nonempty style object with only supported property names.
///
/// Returns `None` for invalid dimensions, factors, alignments, or tracks.
fn container_rule_style(value: &Value) -> Option<ContainerRuleStyle> {
    let object = value.as_object()?;
    if object.is_empty()
        || object.keys().any(|key| {
            !matches!(
                key.as_str(),
                "gridRows"
                    | "gridColumns"
                    | "width"
                    | "height"
                    | "gap"
                    | "grow"
                    | "shrink"
                    | "alignItems"
                    | "justifyContent"
            )
        })
    {
        return None;
    }
    let align_items = match object.get("alignItems") {
        None => None,
        Some(value) => Some(match value.as_str()? {
            "start" => AlignItems::START,
            "center" => AlignItems::CENTER,
            "end" => AlignItems::END,
            "stretch" => AlignItems::STRETCH,
            _ => return None,
        }),
    };
    let justify_content = match object.get("justifyContent") {
        None => None,
        Some(value) => Some(match value.as_str()? {
            "start" => JustifyContent::START,
            "center" => JustifyContent::CENTER,
            "end" => JustifyContent::END,
            "spaceBetween" => JustifyContent::SPACE_BETWEEN,
            "spaceAround" => JustifyContent::SPACE_AROUND,
            "spaceEvenly" => JustifyContent::SPACE_EVENLY,
            _ => return None,
        }),
    };
    Some(ContainerRuleStyle {
        grid_rows: optional(object, "gridRows", grid_tracks)?,
        grid_columns: optional(object, "gridColumns", grid_tracks)?,
        width: optional(object, "width", dimension)?,
        height: optional(object, "height", dimension)?,
        gap: optional(object, "gap", |value| nonnegative(number(value)?))?,
        grow: optional(object, "grow", |value| nonnegative(number(value)?))?,
        shrink: optional(object, "shrink", |value| nonnegative(number(value)?))?,
        align_items,
        justify_content,
    })
}
