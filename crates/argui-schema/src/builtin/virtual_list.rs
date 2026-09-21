//! Fixed-extent virtual viewport fed by a DSL repeater's mounted window.

use argui_ui::{
    EventType, ScrollbarGutter, ScrollbarPartStyle, ScrollbarStyle, ScrollbarVisibility,
    VirtualList,
};

use super::{
    CHILDREN, CommonProperty, EDGE_SHADOW_COLOR, EDGE_SHADOW_INTENSITY, EDGE_SHADOW_WIDTH, GROW,
    ITEM_COUNT, KEY, OVERSCAN, ROW_HEIGHT, SCROLL, SCROLL_OFFSET, SCROLLBAR_THUMB, VIEWPORT_HEIGHT,
    VIRTUAL_LIST, WINDOW_START, apply_common, apply_events, common_property, required_string,
};
use crate::{
    EventSchema, NativeElementInput, NativeSchema, PropertySchema, SchemaError, SchemaRegistry,
    SchemaValue, SlotArity, SlotSchema, ValueType,
};

/// Registers a fixed-row virtual list whose slot contains only mounted rows.
///
/// `registry` receives the canonical schema and its rendering adapter.
///
/// # Errors
///
/// Returns a schema error for duplicate IDs or invalid built-in metadata.
pub(super) fn register(registry: &mut SchemaRegistry) -> Result<(), SchemaError> {
    let schema = NativeSchema::new(
        VIRTUAL_LIST,
        "VList",
        "Scrollable fixed-row list with a virtualized keyed repeater.",
    )
    .property(common_property(CommonProperty::Key))
    .property(common_property(CommonProperty::Tooltip))
    .property(common_property(CommonProperty::Width))
    .property(common_property(CommonProperty::Height))
    .property(common_property(CommonProperty::Background))
    .property(common_property(CommonProperty::MinWidth))
    .property(common_property(CommonProperty::MinHeight))
    .property(common_property(CommonProperty::Rotation))
    .property(PropertySchema::new(
        GROW,
        "grow",
        ValueType::Float,
        "Flex growth inside the surrounding layout.",
    ))
    .property(common_property(CommonProperty::Opacity))
    .property(
        PropertySchema::new(
            ROW_HEIGHT,
            "row_height",
            ValueType::Float,
            "Positive fixed height of each row in logical pixels.",
        )
        .required()
        .not_animatable(),
    )
    .property(
        PropertySchema::new(
            VIEWPORT_HEIGHT,
            "viewport_height",
            ValueType::Float,
            "Optional explicit viewport height; omitted values track the laid-out height.",
        )
        .not_animatable(),
    )
    .property(
        PropertySchema::new(
            SCROLL_OFFSET,
            "offset",
            ValueType::Float,
            "Current vertical scroll offset.",
        )
        .default_value(SchemaValue::Float(0.0))
        .changed_by(SCROLL)
        .not_animatable(),
    )
    .property(
        PropertySchema::new(
            OVERSCAN,
            "overscan",
            ValueType::Int,
            "Additional rows mounted before and after the viewport.",
        )
        .default_value(SchemaValue::Int(3))
        .not_animatable(),
    )
    .property(PropertySchema::new(
        SCROLLBAR_THUMB,
        "scrollbar_thumb",
        ValueType::Color,
        "Theme color of the scrollbar thumb.",
    ))
    .property(
        PropertySchema::new(
            EDGE_SHADOW_WIDTH,
            "edge_shadow_width",
            ValueType::Float,
            "Optional scroll-reactive top/bottom shadow width in logical pixels; zero disables it.",
        )
        .default_value(SchemaValue::Float(0.0)),
    )
    .property(
        PropertySchema::new(
            EDGE_SHADOW_INTENSITY,
            "edge_shadow_intensity",
            ValueType::Float,
            "Maximum opacity of the optional scroll edge shadow.",
        )
        .default_value(SchemaValue::Float(0.15)),
    )
    .property(PropertySchema::new(
        EDGE_SHADOW_COLOR,
        "edge_shadow_color",
        ValueType::Color,
        "Theme color of the optional scroll edge shadow.",
    ))
    .property(PropertySchema::new(
        ITEM_COUNT,
        "__item_count",
        ValueType::Int,
        "Compiler-provided collection length.",
    ))
    .property(PropertySchema::new(
        WINDOW_START,
        "__window_start",
        ValueType::Int,
        "Compiler-provided first mounted row index.",
    ))
    .event(
        EventSchema::new(
            SCROLL,
            "scroll",
            EventType::Scroll,
            "Viewport scroll offset changed.",
        )
        .payload(ValueType::Float),
    )
    .slot(SlotSchema {
        id: CHILDREN,
        name: "children".into(),
        arity: SlotArity::Many,
        documentation: "Only rows in the current virtual window.".into(),
    });
    registry.register(schema, |input: &NativeElementInput| {
        let key = required_string(input, KEY, "key")?;
        let row_height = float(input, ROW_HEIGHT, "row_height")?;
        let viewport = match input.get(VIEWPORT_HEIGHT) { Some(SchemaValue::Float(value)) => *value, _ => 0.0 };
        if !row_height.is_finite() || row_height <= 0.0 || !viewport.is_finite() || viewport < 0.0 {
            return Err(SchemaError::Adapter("VList requires a positive finite row_height and nonnegative finite viewport_height".into()));
        }
        let count = nonnegative_int(input, ITEM_COUNT, "__item_count")?;
        let first = nonnegative_int(input, WINDOW_START, "__window_start")?;
        let overscan = if input.get(OVERSCAN).is_some() { nonnegative_int(input, OVERSCAN, "overscan")? } else { 3 };
        let offset = match input.get(SCROLL_OFFSET) { Some(SchemaValue::Float(value)) if value.is_finite() => *value, _ => 0.0 };
        let rows = input.children(CHILDREN);
        let window = VirtualList::fixed(count, row_height, viewport).overscan(overscan).window(offset);
        if first != window.range.start || rows.len() != window.range.len() {
            return Err(SchemaError::Adapter(format!("VList received {} mounted rows at {first}, expected {} at {}", rows.len(), window.range.len(), window.range.start)));
        }
        let thumb = match input.get(SCROLLBAR_THUMB) {
            Some(SchemaValue::Color(color)) => *color,
            _ => argui_core::Color::srgba(0.45, 0.50, 0.57, 0.75),
        };
        let scrollbar = ScrollbarStyle::new(
            ScrollbarPartStyle::new(argui_paint::QuadStyle::default()),
            ScrollbarPartStyle::new(argui_paint::QuadStyle::solid(thumb)),
        ).width(8.0).visibility(ScrollbarVisibility::Always);
        let shadow_width = optional_nonnegative_float(input, EDGE_SHADOW_WIDTH, "edge_shadow_width", 0.0)?;
        let shadow_intensity = optional_nonnegative_float(input, EDGE_SHADOW_INTENSITY, "edge_shadow_intensity", 0.15)?;
        let shadow_color = match input.get(EDGE_SHADOW_COLOR) {
            Some(SchemaValue::Color(color)) => *color,
            _ => argui_core::Color::BLACK,
        };
        let mut scroll = argui_ui::ScrollConfig::default().scrollbar(scrollbar);
        if shadow_width > 0.0 && shadow_intensity > 0.0 {
            scroll = scroll.effect(argui_effects::EdgeShadow::new(shadow_width, shadow_color)
                .intensity(shadow_intensity)
                .strengths([0.0, 1.0, 0.0, 1.0])
                .scroll());
        }
        let mut list = VirtualList::fixed(count, row_height, viewport)
            .overscan(overscan)
            .scroll_config(scroll)
            .build(key.clone(), offset, |index| rows[index - first].clone())
            .scrollbar_gutter(ScrollbarGutter::Stable);
        if let Some(SchemaValue::Float(grow)) = input.get(GROW) { list = list.grow(*grow); }
        Ok(apply_events(apply_common(list, input), input))
    })
}

/// Reads one optional finite nonnegative float without changing the default.
///
/// `input` supplies the native properties, `id` and `name` identify the field,
/// and `fallback` is returned when it is omitted.
///
/// # Errors
///
/// Returns an adapter error for a negative, non-finite, or non-float value.
fn optional_nonnegative_float(
    input: &NativeElementInput,
    id: crate::PropertyId,
    name: &str,
    fallback: f32,
) -> Result<f32, SchemaError> {
    match input.get(id) {
        None => Ok(fallback),
        Some(SchemaValue::Float(value)) if value.is_finite() && *value >= 0.0 => Ok(*value),
        _ => Err(SchemaError::Adapter(format!(
            "VList requires nonnegative finite `{name}`"
        ))),
    }
}

/// Reads a required float value supplied by the compiler or author.
///
/// `input` is the validated native input; `id` and `name` identify the property.
///
/// # Errors
///
/// Returns an adapter error when the value is absent or has an unexpected type.
fn float(
    input: &NativeElementInput,
    id: crate::PropertyId,
    name: &str,
) -> Result<f32, SchemaError> {
    match input.get(id) {
        Some(SchemaValue::Float(value)) => Ok(*value),
        _ => Err(SchemaError::Adapter(format!(
            "VList requires float `{name}`"
        ))),
    }
}

/// Reads a nonnegative integer count or index.
///
/// `input` is the validated native input; `id` and `name` identify the property.
///
/// # Errors
///
/// Returns an adapter error for a missing or negative value.
fn nonnegative_int(
    input: &NativeElementInput,
    id: crate::PropertyId,
    name: &str,
) -> Result<usize, SchemaError> {
    match input.get(id) {
        Some(SchemaValue::Int(value)) => usize::try_from(*value)
            .map_err(|_| SchemaError::Adapter(format!("VList requires nonnegative `{name}`"))),
        _ => Err(SchemaError::Adapter(format!(
            "VList requires integer `{name}`"
        ))),
    }
}
