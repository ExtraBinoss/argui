//! Sendable JSON transport for schema-typed presentation operations.

use argui_core::{Color, Name, Transform2D};
use argui_host::{CallbackId, HostId, Operation};
use argui_paint::{Border, BorderWidths, CornerRadii, Fill, ImageId, Shadow, VectorId};
use argui_schema::{
    AssetHandle, ContainerRule, ContainerRuleStyle, EventId, NativeTypeId, PropertyId, SchemaValue,
};
use argui_ui::{
    AlignItems, ContainerQuery, ContainerScopeId, Dimension, GridTemplateComponent, JustifyContent,
    LayoutInsets, LengthPercentage, LengthPercentageAuto, PositionInsets, RepetitionCount,
    TrackSizingFunction, auto, fr, length, minmax, repeat,
};
use serde::Deserialize;
use serde_json::Value;

mod grid;
use grid::{container_rules, grid_tracks};

/// An ID transported between the JavaScript actor and native host.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
pub struct WireHostId {
    pub slot: u32,
    pub generation: u32,
}

impl From<WireHostId> for HostId {
    fn from(id: WireHostId) -> Self {
        Self::new(id.slot, id.generation)
    }
}

/// A schema variant tag and JSON payload transported across threads.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct WireValue {
    #[serde(rename = "type")]
    pub value_type: String,
    pub value: Value,
}

impl WireValue {
    /// Decodes a schema value after crossing the thread boundary.
    ///
    /// # Errors
    /// Returns an error for a malformed or unsupported payload.
    pub fn into_native(self) -> Result<SchemaValue, String> {
        let value = &self.value;
        match self.value_type.as_str() {
            "Bool" => value.as_bool().map(SchemaValue::Bool),
            "Int" => value.as_i64().map(SchemaValue::Int),
            "Float" => number(value).map(SchemaValue::Float),
            "String" => value.as_str().map(|s| SchemaValue::String(s.to_owned())),
            "Name" => value
                .as_str()
                .map(|s| SchemaValue::Name(Name::from_owned(s.to_owned()))),
            "Color" => value
                .as_str()
                .and_then(|s| Color::from_literal(s).ok())
                .map(SchemaValue::Color),
            "Brush" => value
                .as_str()
                .and_then(|s| Color::from_literal(s).ok())
                .map(|c| SchemaValue::Brush(Fill::Solid(c))),
            "Dimension" => dimension(value).map(SchemaValue::Dimension),
            "Constraint" => constraint(value).map(SchemaValue::Constraint),
            "Insets" => insets(value).map(SchemaValue::Insets),
            "PositionInsets" => position_insets(value).map(SchemaValue::PositionInsets),
            "Radii" => radii(value).map(SchemaValue::Radii),
            "Border" => border(value).map(SchemaValue::Border),
            "Shadow" => shadow(value).map(SchemaValue::Shadow),
            "Transform" => transform(value).map(SchemaValue::Transform),
            "Asset" => asset(value).map(SchemaValue::Asset),
            "GridTracks" => grid_tracks(value).map(SchemaValue::GridTracks),
            "ContainerRules" => container_rules(value).map(SchemaValue::ContainerRules),
            _ => None,
        }
        .ok_or_else(|| {
            format!(
                "invalid or unsupported {} wire value: {}",
                self.value_type, self.value
            )
        })
    }
}

/// A transaction operation using only data safe to send between threads.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub enum WireOperation {
    #[serde(rename = "create")]
    Create {
        id: WireHostId,
        #[serde(rename = "nativeType")]
        native_type: u32,
    },
    #[serde(rename = "setProperty")]
    SetProperty {
        id: WireHostId,
        property: u16,
        value: Option<WireValue>,
    },
    #[serde(rename = "setListener")]
    SetListener {
        id: WireHostId,
        event: u16,
        callback: Option<u32>,
    },
    #[serde(rename = "insert")]
    Insert {
        parent: WireHostId,
        child: WireHostId,
        before: Option<WireHostId>,
    },
    #[serde(rename = "remove")]
    Remove { id: WireHostId },
    #[serde(rename = "setRoot")]
    SetRoot { id: Option<WireHostId> },
}

impl WireOperation {
    /// Converts a transport operation to the schema-typed host operation.
    ///
    /// # Errors
    /// Returns an error if a property payload cannot be decoded.
    pub fn into_native(self) -> Result<Operation, String> {
        Ok(match self {
            Self::Create { id, native_type } => Operation::Create {
                id: id.into(),
                native_type: NativeTypeId::from_raw(native_type),
            },
            Self::SetProperty {
                id,
                property,
                value,
            } => Operation::SetProperty {
                id: id.into(),
                property: PropertyId::from_raw(property),
                value: value
                    .map(WireValue::into_native)
                    .transpose()
                    .map_err(|error| {
                        format!(
                            "node {}:{} property {}: {error}",
                            id.slot, id.generation, property
                        )
                    })?,
            },
            Self::SetListener {
                id,
                event,
                callback,
            } => Operation::SetListener {
                id: id.into(),
                event: EventId::from_raw(event),
                callback: callback.map(CallbackId),
            },
            Self::Insert {
                parent,
                child,
                before,
            } => Operation::Insert {
                parent: parent.into(),
                child: child.into(),
                before: before.map(Into::into),
            },
            Self::Remove { id } => Operation::Remove { id: id.into() },
            Self::SetRoot { id } => Operation::SetRoot {
                id: id.map(Into::into),
            },
        })
    }
}

fn number(value: &Value) -> Option<f32> {
    let result = value.as_f64()? as f32;
    result.is_finite().then_some(result)
}

fn dimension(value: &Value) -> Option<Dimension> {
    if let Some(number) = number(value) {
        return Some(Dimension::length(number));
    }
    let text = value.as_str()?;
    if text == "auto" {
        return Some(Dimension::auto());
    }
    if text == "minContent" {
        return Some(Dimension::min_content());
    }
    if text == "maxContent" {
        return Some(Dimension::max_content());
    }
    if text == "fitContent" {
        return Some(Dimension::fit_content());
    }
    if let Some(percent) = text.strip_suffix('%') {
        return percent
            .parse::<f32>()
            .ok()
            .filter(|percent| percent.is_finite())
            .map(|percent| Dimension::percent(percent / 100.0));
    }
    text.strip_suffix("px")?
        .parse::<f32>()
        .ok()
        .filter(|pixels| pixels.is_finite())
        .map(Dimension::length)
}

/// Decodes a nonnegative min/max constraint with pixel, percent, or auto semantics.
///
/// `value` is a JSON number or supported string. Returns `None` for invalid
/// geometry or unsupported keywords.
fn constraint(value: &Value) -> Option<LengthPercentageAuto> {
    if let Some(pixels) = number(value).filter(|value| *value >= 0.0) {
        return Some(LengthPercentageAuto::length(pixels));
    }
    let text = value.as_str()?;
    if text == "auto" {
        return Some(LengthPercentageAuto::auto());
    }
    let percent = text.strip_suffix('%')?.parse::<f32>().ok()?;
    (percent.is_finite() && percent >= 0.0)
        .then_some(LengthPercentageAuto::percent(percent / 100.0))
}

fn insets(value: &Value) -> Option<LayoutInsets> {
    if let Some(n) = number(value) {
        return Some(LayoutInsets {
            top: n,
            right: n,
            bottom: n,
            left: n,
            start: None,
            end: None,
        });
    }
    let obj = value.as_object()?;
    if obj.keys().any(|key| {
        !matches!(
            key.as_str(),
            "top" | "right" | "bottom" | "left" | "start" | "end"
        )
    }) {
        return None;
    }
    if (obj.contains_key("start") || obj.contains_key("end"))
        && (obj.contains_key("left") || obj.contains_key("right"))
    {
        return None;
    }
    Some(LayoutInsets {
        top: obj.get("top").map(number).unwrap_or(Some(0.0))?,
        right: obj.get("right").map(number).unwrap_or(Some(0.0))?,
        bottom: obj.get("bottom").map(number).unwrap_or(Some(0.0))?,
        left: obj.get("left").map(number).unwrap_or(Some(0.0))?,
        start: if let Some(value) = obj.get("start") {
            Some(number(value)?)
        } else {
            None
        },
        end: if let Some(value) = obj.get("end") {
            Some(number(value)?)
        } else {
            None
        },
    })
}

/// Decodes positioned edges while preserving omitted sides as `auto`.
///
/// `value` is a number for all four physical sides or a partial edge object.
/// Returns `None` for an invalid edge or a logical/physical horizontal mix.
fn position_insets(value: &Value) -> Option<PositionInsets> {
    if let Some(n) = number(value) {
        return Some(PositionInsets {
            top: Some(n),
            right: Some(n),
            bottom: Some(n),
            left: Some(n),
            start: None,
            end: None,
        });
    }
    let object = value.as_object()?;
    if object.keys().any(|key| {
        !matches!(
            key.as_str(),
            "top" | "right" | "bottom" | "left" | "start" | "end"
        )
    }) {
        return None;
    }
    if (object.contains_key("start") || object.contains_key("end"))
        && (object.contains_key("left") || object.contains_key("right"))
    {
        return None;
    }
    Some(PositionInsets {
        top: optional(object, "top", number)?,
        right: optional(object, "right", number)?,
        bottom: optional(object, "bottom", number)?,
        left: optional(object, "left", number)?,
        start: optional(object, "start", number)?,
        end: optional(object, "end", number)?,
    })
}

/// Decodes an optional object field without losing a malformed present value.
///
/// `object` is the inspected JSON map, `key` names the field, and `decode`
/// converts a present value. Returns `None` when conversion fails.
fn optional<T>(
    object: &serde_json::Map<String, Value>,
    key: &str,
    decode: impl FnOnce(&Value) -> Option<T>,
) -> Option<Option<T>> {
    match object.get(key) {
        Some(value) => decode(value).map(Some),
        None => Some(None),
    }
}

fn radii(value: &Value) -> Option<CornerRadii> {
    if let Some(n) = number(value) {
        return Some(CornerRadii::all(n));
    }
    let obj = value.as_object()?;
    if obj.keys().any(|key| {
        !matches!(
            key.as_str(),
            "topLeft" | "topRight" | "bottomRight" | "bottomLeft"
        )
    }) {
        return None;
    }
    Some(CornerRadii {
        top_left: obj.get("topLeft").map(number).unwrap_or(Some(0.0))?,
        top_right: obj.get("topRight").map(number).unwrap_or(Some(0.0))?,
        bottom_right: obj.get("bottomRight").map(number).unwrap_or(Some(0.0))?,
        bottom_left: obj.get("bottomLeft").map(number).unwrap_or(Some(0.0))?,
    })
}

/// Decodes a solid-colored border with a uniform or per-edge width.
///
/// * `value` — object containing `width` and `color`.
///
/// Returns `None` when a required field, color, or width is invalid.
fn border(value: &Value) -> Option<Border> {
    let obj = value.as_object()?;
    if obj
        .keys()
        .any(|key| !matches!(key.as_str(), "width" | "color"))
    {
        return None;
    }
    let color = Color::from_literal(obj.get("color")?.as_str()?).ok()?;
    let width = obj.get("width")?;
    let widths = if let Some(width) = number(width) {
        let width = nonnegative(width)?;
        BorderWidths::all(width)
    } else {
        let edges = width.as_object()?;
        if edges
            .keys()
            .any(|key| !matches!(key.as_str(), "top" | "right" | "bottom" | "left"))
        {
            return None;
        }
        BorderWidths {
            top: nonnegative(number(edges.get("top")?)?)?,
            right: nonnegative(number(edges.get("right")?)?)?,
            bottom: nonnegative(number(edges.get("bottom")?)?)?,
            left: nonnegative(number(edges.get("left")?)?)?,
        }
    };
    Some(Border { widths, color })
}

/// Decodes a typed shadow using logical pixels and a solid color.
///
/// * `value` — object with required `blur` and `color`, plus optional offsets, spread, and inset.
///
/// Returns `None` for unknown fields or invalid geometry.
fn shadow(value: &Value) -> Option<Shadow> {
    let obj = value.as_object()?;
    if obj.keys().any(|key| {
        !matches!(
            key.as_str(),
            "offsetX" | "offsetY" | "blur" | "spread" | "color" | "inset"
        )
    }) {
        return None;
    }
    let blur = nonnegative(number(obj.get("blur")?)?)?;
    let color = Color::from_literal(obj.get("color")?.as_str()?).ok()?;
    let x = obj.get("offsetX").map(number).unwrap_or(Some(0.0))?;
    let y = obj.get("offsetY").map(number).unwrap_or(Some(0.0))?;
    let spread = obj.get("spread").map(number).unwrap_or(Some(0.0))?;
    let inset = obj
        .get("inset")
        .map(Value::as_bool)
        .unwrap_or(Some(false))?;
    Some(
        Shadow::drop([x, y], blur, color)
            .spread(spread)
            .inset(inset),
    )
}

/// Accepts finite nonnegative geometry, preserving zero.
///
/// * `value` — decoded scalar.
///
/// Returns the scalar when nonnegative.
fn nonnegative(value: f32) -> Option<f32> {
    (value >= 0.0).then_some(value)
}

fn transform(value: &Value) -> Option<Transform2D> {
    let obj = value.as_object()?;
    if obj.keys().any(|key| {
        !matches!(
            key.as_str(),
            "translateX" | "translateY" | "scaleX" | "scaleY" | "rotation"
        )
    }) {
        return None;
    }
    let x = obj.get("translateX").map(number).unwrap_or(Some(0.0))?;
    let y = obj.get("translateY").map(number).unwrap_or(Some(0.0))?;
    let scale_x = obj.get("scaleX").map(number).unwrap_or(Some(1.0))?;
    let scale_y = obj.get("scaleY").map(number).unwrap_or(Some(1.0))?;
    let degrees = obj.get("rotation").map(number).unwrap_or(Some(0.0))?;
    Some(
        Transform2D::IDENTITY
            .translate(x, y)
            .scale(scale_x, scale_y)
            .rotate(degrees.to_radians()),
    )
}

/// Decodes an image or SVG handle restricted to JavaScript's exact integer range.
///
/// * `value` — object with a media kind and positive numeric asset ID.
///
/// Returns `None` for an unknown kind or an ID that cannot round-trip through JS.
fn asset(value: &Value) -> Option<AssetHandle> {
    let source = value.as_object()?;
    let id = source.get("id")?.as_u64()?;
    if id == 0 || id > 9_007_199_254_740_991 {
        return None;
    }
    match source.get("kind")?.as_str()? {
        "image" => Some(AssetHandle::Image(ImageId(id))),
        "svg" => Some(AssetHandle::Vector(VectorId(id))),
        _ => None,
    }
}
