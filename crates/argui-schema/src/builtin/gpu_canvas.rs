//! Native GPU viewport backed by an application-owned WGPU canvas factory.

use argui_paint::{GpuCanvasId, ImageSampling};
use argui_ui::{Element, GpuCanvasSpec, Role, Semantics};

use super::{
    ALT, CANVAS_ID, CANVAS_REVISION, CommonProperty, GPU_CANVAS, RESOLUTION_SCALE, SAMPLING,
    apply_common, common_property,
};
use crate::{
    NativeElementInput, NativeSchema, PropertySchema, SchemaError, SchemaRegistry, SchemaValue,
    ValueType,
};

/// Registers the application-owned native viewport primitive.
///
/// `registry` receives the stable schema and its Rust adapter.
///
/// # Errors
///
/// Returns a schema error when its name or ID collides with another primitive.
pub(super) fn register(registry: &mut SchemaRegistry) -> Result<(), SchemaError> {
    let schema = NativeSchema::new(
        GPU_CANVAS,
        "GpuCanvas",
        "Displays a native WGPU texture produced by an application-registered callback.",
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
            CANVAS_ID,
            "canvas_id",
            ValueType::Int,
            "Numeric ID returned by the application's GPU canvas registration.",
        )
        .required(),
    )
    .property(PropertySchema::new(
        CANVAS_REVISION,
        "revision",
        ValueType::Int,
        "Optional content revision for changes driven by TSX controls.",
    ))
    .property(PropertySchema::new(
        RESOLUTION_SCALE,
        "resolution_scale",
        ValueType::Float,
        "Physical texture resolution multiplier from 0.125 through 4.",
    ))
    .property(PropertySchema::new(
        SAMPLING,
        "sampling",
        ValueType::String,
        "Compositor sampling mode: linear or nearest.",
    ))
    .property(PropertySchema::new(
        ALT,
        "alt",
        ValueType::String,
        "Accessible description; an empty value marks the viewport decorative.",
    ));
    registry.register(schema, |input: &NativeElementInput| {
        let raw = match input.get(CANVAS_ID) {
            Some(SchemaValue::Int(raw)) if *raw > 0 => *raw as u64,
            _ => return Err(SchemaError::Adapter("invalid GpuCanvas canvas_id".into())),
        };
        let canvas = GpuCanvasId::from_raw(raw)
            .ok_or_else(|| SchemaError::Adapter("invalid GpuCanvas canvas_id".into()))?;
        let revision = match input.get(CANVAS_REVISION) {
            None => 0,
            Some(SchemaValue::Int(revision)) if *revision >= 0 => *revision as u64,
            _ => return Err(SchemaError::Adapter("invalid GpuCanvas revision".into())),
        };
        let scale = match input.get(RESOLUTION_SCALE) {
            None => 1.0,
            Some(SchemaValue::Float(scale))
                if scale.is_finite()
                    && *scale >= GpuCanvasSpec::MIN_RESOLUTION_SCALE
                    && *scale <= GpuCanvasSpec::MAX_RESOLUTION_SCALE =>
            {
                *scale
            }
            _ => {
                return Err(SchemaError::Adapter(
                    "invalid GpuCanvas resolution_scale".into(),
                ));
            }
        };
        let sampling = match input.get(SAMPLING) {
            None => ImageSampling::Linear,
            Some(SchemaValue::String(value)) if value == "linear" => ImageSampling::Linear,
            Some(SchemaValue::String(value)) if value == "nearest" => ImageSampling::Nearest,
            _ => return Err(SchemaError::Adapter("invalid GpuCanvas sampling".into())),
        };
        let spec = GpuCanvasSpec::new(canvas)
            .content_revision(revision)
            .resolution_scale(scale)
            .sampling(sampling);
        let mut element = Element::gpu_canvas(spec);
        match input.get(ALT) {
            Some(SchemaValue::String(alt)) if alt.is_empty() => {
                element = element.semantic_hidden(true);
            }
            Some(SchemaValue::String(alt)) => {
                element = element.semantics(Semantics::new(Role::Image).label(alt));
            }
            _ => {}
        }
        apply_common(element, input)
    })
}
