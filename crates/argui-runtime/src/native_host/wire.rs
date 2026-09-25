//! Sendable JSON transport for schema-typed presentation operations.

use argui_core::{Color, Insets, Name, Transform2D};
use argui_host::{CallbackId, HostId, Operation};
use argui_paint::{CornerRadii, Fill, ImageId, VectorId};
use argui_schema::{AssetHandle, EventId, NativeTypeId, PropertyId, SchemaValue};
use argui_ui::Dimension;
use serde::Deserialize;
use serde_json::Value;

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
            "Insets" => insets(value).map(SchemaValue::Insets),
            "Radii" => radii(value).map(SchemaValue::Radii),
            "Transform" => transform(value).map(SchemaValue::Transform),
            "Asset" => asset(value).map(SchemaValue::Asset),
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
    if text == "auto" || text == "fit" {
        return Some(Dimension::auto());
    }
    if text == "fill" {
        return Some(Dimension::percent(1.0));
    }
    if let Some(percent) = text.strip_suffix('%') {
        return percent
            .parse::<f32>()
            .ok()
            .map(|p| Dimension::percent(p / 100.0));
    }
    text.strip_suffix("px")?
        .parse::<f32>()
        .ok()
        .map(Dimension::length)
}

fn insets(value: &Value) -> Option<Insets> {
    if let Some(n) = number(value) {
        return Some(Insets::new(n, n, n, n));
    }
    let obj = value.as_object()?;
    Some(Insets::new(
        number(obj.get("top")?)?,
        number(obj.get("right")?)?,
        number(obj.get("bottom")?)?,
        number(obj.get("left")?)?,
    ))
}

fn radii(value: &Value) -> Option<CornerRadii> {
    if let Some(n) = number(value) {
        return Some(CornerRadii::all(n));
    }
    let obj = value.as_object()?;
    Some(CornerRadii {
        top_left: number(obj.get("topLeft")?)?,
        top_right: number(obj.get("topRight")?)?,
        bottom_right: number(obj.get("bottomRight")?)?,
        bottom_left: number(obj.get("bottomLeft")?)?,
    })
}

fn transform(value: &Value) -> Option<Transform2D> {
    let obj = value.as_object()?;
    let x = obj.get("x").and_then(number).unwrap_or(0.0);
    let y = obj.get("y").and_then(number).unwrap_or(0.0);
    let scale_x = obj.get("scaleX").and_then(number).unwrap_or(1.0);
    let scale_y = obj.get("scaleY").and_then(number).unwrap_or(1.0);
    let radians = obj.get("rotation").and_then(number).unwrap_or(0.0);
    Some(
        Transform2D::IDENTITY
            .translate(x, y)
            .scale(scale_x, scale_y)
            .rotate(radians),
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
