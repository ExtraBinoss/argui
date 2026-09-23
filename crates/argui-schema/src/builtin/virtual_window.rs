//! Visually neutral fixed-row virtual viewport fed by a keyed repeater.

use argui_ui::{
    EventType, ScrollConfig, ScrollbarGutter, ScrollbarPartStyle, ScrollbarStyle,
    ScrollbarVisibility, VirtualList,
};

use super::{
    CHILDREN, CONTENT_HEIGHT, CommonProperty, GROW, ITEM_COUNT, KEY, OFFSET_Y, OVERSCAN,
    ROW_HEIGHT, SCROLL, SCROLL_OFFSET, SCROLLBAR_THUMB, VARIABLE_HEIGHT, VIEWPORT_HEIGHT,
    VIRTUAL_WINDOW, VISIBLE_HEIGHT, WINDOW_START, apply_common, apply_events, common_property,
    required_string,
};
use crate::{
    EventSchema, NativeElementInput, NativeSchema, ObservationKind, PropertyId, PropertySchema,
    SchemaError, SchemaRegistry, SchemaValue, SlotArity, SlotSchema, ValueType,
};

/// Registers a virtual viewport without a prescribed scrollbar or visual surface.
///
/// * `registry` — native registry receiving the virtual window schema and adapter.
///
/// # Errors
///
/// Returns a schema error for duplicate identifiers or invalid built-in metadata.
pub(super) fn register(registry: &mut SchemaRegistry) -> Result<(), SchemaError> {
    let schema = NativeSchema::new(
        VIRTUAL_WINDOW,
        "VirtualWindow",
        "Measured or fixed-row window that mounts only keyed rows near its viewport.",
    )
    .virtual_window()
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
    .property(PropertySchema::new(
        GROW,
        "grow",
        ValueType::Float,
        "Flex growth inside the surrounding layout.",
    ))
    .property(
        PropertySchema::new(
            ROW_HEIGHT,
            "row_height",
            ValueType::Float,
            "Positive row height or estimate before variable rows are measured.",
        )
        .required()
        .not_animatable(),
    )
    .property(
        PropertySchema::new(
            VARIABLE_HEIGHT,
            "variable_height",
            ValueType::Bool,
            "Measure rich row heights and retain their scroll anchor.",
        )
        .default_value(SchemaValue::Bool(false))
        .not_animatable(),
    )
    .property(
        PropertySchema::new(
            VIEWPORT_HEIGHT,
            "viewport_height",
            ValueType::Float,
            "Optional viewport height; omitted values track the laid-out height.",
        )
        .not_animatable(),
    )
    .property(
        PropertySchema::new(
            SCROLL_OFFSET,
            "offset",
            ValueType::Float,
            "Current vertical scroll offset in logical pixels.",
        )
        .default_value(SchemaValue::Float(0.0))
        .changed_by(SCROLL)
        .not_animatable(),
    )
    .property(PropertySchema::new(
        SCROLLBAR_THUMB,
        "scrollbar_thumb",
        ValueType::Color,
        "Optional thumb color; omitted windows remain visually neutral.",
    ))
    .property(
        PropertySchema::new(
            OFFSET_Y,
            "offset_y",
            ValueType::Dimension,
            "Laid-out vertical scroll position in logical pixels.",
        )
        .observed(ObservationKind::ScrollY),
    )
    .property(
        PropertySchema::new(
            VISIBLE_HEIGHT,
            "visible_height",
            ValueType::Dimension,
            "Laid-out visible viewport height in logical pixels.",
        )
        .observed(ObservationKind::ViewportHeight),
    )
    .property(
        PropertySchema::new(
            CONTENT_HEIGHT,
            "content_height",
            ValueType::Dimension,
            "Laid-out full content height in logical pixels.",
        )
        .observed(ObservationKind::ContentHeight),
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
        documentation: "Only keyed rows in the current virtual window.".into(),
    });
    registry.register(schema, |input: &NativeElementInput| {
        let key = required_string(input, KEY, "key")?;
        let row_height = float(input, ROW_HEIGHT, "row_height", 0.0)?;
        let viewport = float(input, VIEWPORT_HEIGHT, "viewport_height", 0.0)?;
        let offset = float(input, SCROLL_OFFSET, "offset", 0.0)?;
        if row_height <= 0.0 || viewport < 0.0 {
            return Err(SchemaError::Adapter(
                "VirtualWindow requires positive row_height and nonnegative viewport_height".into(),
            ));
        }
        let count = nonnegative_int(input, ITEM_COUNT, "__item_count", None)?;
        let first = nonnegative_int(input, WINDOW_START, "__window_start", None)?;
        let overscan = nonnegative_int(input, OVERSCAN, "overscan", Some(3))?;
        let rows = input.children(CHILDREN);
        let list = input
            .virtual_list
            .clone()
            .unwrap_or_else(|| VirtualList::fixed(count, row_height, viewport).overscan(overscan));
        let window = list.window(offset);
        if first != window.range.start || rows.len() != window.range.len() {
            return Err(SchemaError::Adapter(format!(
                "VirtualWindow received {} rows at {first}, expected {} at {}",
                rows.len(),
                window.range.len(),
                window.range.start
            )));
        }
        let mut element = list.build(key.clone(), offset, |index| rows[index - first].clone());
        if let Some(SchemaValue::Color(thumb)) = input.get(SCROLLBAR_THUMB) {
            let scrollbar = ScrollbarStyle::new(
                ScrollbarPartStyle::new(argui_paint::QuadStyle::default()),
                ScrollbarPartStyle::new(argui_paint::QuadStyle::solid(*thumb)),
            )
            .width(8.0)
            .visibility(ScrollbarVisibility::Always);
            element = element
                .scroll_config(
                    ScrollConfig::default()
                        .line_size(row_height)
                        .scrollbar(scrollbar),
                )
                .scrollbar_gutter(ScrollbarGutter::Stable);
        }
        if let Some(SchemaValue::Float(grow)) = input.get(GROW) {
            element = element.grow(*grow);
        }
        Ok(apply_events(apply_common(element, input)?, input))
    })
}

/// Reads a finite float field, using `fallback` when omitted.
///
/// * `input` — validated native input.
/// * `id` — property identifier to read.
/// * `name` — field name used in an error.
/// * `fallback` — value used when the field is absent.
///
/// # Errors
///
/// Returns an adapter error for a non-finite or incompatible value.
fn float(
    input: &NativeElementInput,
    id: PropertyId,
    name: &str,
    fallback: f32,
) -> Result<f32, SchemaError> {
    match input.get(id) {
        None => Ok(fallback),
        Some(SchemaValue::Float(value)) if value.is_finite() => Ok(*value),
        _ => Err(SchemaError::Adapter(format!(
            "VirtualWindow requires finite float `{name}`"
        ))),
    }
}

/// Reads a nonnegative collection length or row index.
///
/// * `input` — validated native input.
/// * `id` — property identifier to read.
/// * `name` — field name used in an error.
/// * `fallback` — optional value used when the field is absent.
///
/// # Errors
///
/// Returns an adapter error for a missing, negative, or incompatible value.
fn nonnegative_int(
    input: &NativeElementInput,
    id: PropertyId,
    name: &str,
    fallback: Option<usize>,
) -> Result<usize, SchemaError> {
    match input.get(id) {
        None => fallback.ok_or_else(|| {
            SchemaError::Adapter(format!("VirtualWindow requires integer `{name}`"))
        }),
        Some(SchemaValue::Int(value)) => usize::try_from(*value).map_err(|_| {
            SchemaError::Adapter(format!("VirtualWindow requires nonnegative `{name}`"))
        }),
        _ => Err(SchemaError::Adapter(format!(
            "VirtualWindow requires integer `{name}`"
        ))),
    }
}
