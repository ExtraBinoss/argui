//! Native vertical or horizontal virtual viewport fed by a keyed presenter.

use argui_effects::{EdgeFade, EdgeShadow};
use argui_ui::{
    EventType, ScrollbarGutter, ScrollbarPartStyle, ScrollbarStyle, ScrollbarVisibility,
    VirtualList,
};

use super::{
    CHILDREN, CONTENT_HEIGHT, CONTENT_WIDTH, CommonProperty, GROW, ID, ITEM_COUNT, OFFSET_X,
    OFFSET_Y, OVERSCAN, ROW_HEIGHT, SCROLL, SCROLL_OFFSET, SCROLLBAR_THUMB, VARIABLE_HEIGHT,
    VIEWPORT_HEIGHT, VIRTUAL_DATA_VERSION, VIRTUAL_HORIZONTAL, VIRTUAL_MEASURE,
    VIRTUAL_SCROLLBAR_VISIBLE, VIRTUAL_SCROLLBAR_WIDTH, VIRTUAL_SHADOW_COLOR, VIRTUAL_SHADOW_END,
    VIRTUAL_SHADOW_INTENSITY, VIRTUAL_SHADOW_START, VIRTUAL_SHADOW_WIDTH, VIRTUAL_VIEWPORT_WIDTH,
    VIRTUAL_VISIBLE_WIDTH, VIRTUAL_WINDOW, VIRTUAL_WINDOW_CHANGE, VISIBLE_HEIGHT, WINDOW_START,
    apply_common, apply_events, common_property, optional_bool, required_string,
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
    .property(common_property(CommonProperty::Position))
    .property(common_property(CommonProperty::Inset))
    .property(common_property(CommonProperty::MinWidth))
    .property(common_property(CommonProperty::MinHeight))
    .property(common_property(CommonProperty::MaxWidth))
    .property(common_property(CommonProperty::MaxHeight))
    .property(common_property(CommonProperty::Shrink))
    .property(common_property(CommonProperty::AlignSelf))
    .property(common_property(CommonProperty::Margin))
    .property(common_property(CommonProperty::Rotation))
    .property(common_property(CommonProperty::Opacity))
    .property(common_property(CommonProperty::BackdropFilter))
    .property(common_property(CommonProperty::DesktopBackdropTint))
    .property(common_property(CommonProperty::DesktopBackdropFallback))
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
            "rowHeight",
            ValueType::Float,
            "Positive row height or estimate before variable rows are measured.",
        )
        .required()
        .not_animatable(),
    )
    .property(
        PropertySchema::new(
            VARIABLE_HEIGHT,
            "variableHeight",
            ValueType::Bool,
            "Measure rich row heights and retain their scroll anchor.",
        )
        .default_value(SchemaValue::Bool(false))
        .not_animatable(),
    )
    .property(
        PropertySchema::new(
            VIRTUAL_HORIZONTAL,
            "horizontal",
            ValueType::Bool,
            "Scroll and measure widths along the horizontal axis.",
        )
        .default_value(SchemaValue::Bool(false))
        .not_animatable(),
    )
    .property(
        PropertySchema::new(
            VIRTUAL_VIEWPORT_WIDTH,
            "viewportWidth",
            ValueType::Float,
            "Optional measured horizontal viewport width.",
        )
        .not_animatable(),
    )
    .property(
        PropertySchema::new(
            VIEWPORT_HEIGHT,
            "viewportHeight",
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
            "Current scroll offset along the active axis in logical pixels.",
        )
        .default_value(SchemaValue::Float(0.0))
        .changed_by(SCROLL)
        .not_animatable(),
    )
    .property(PropertySchema::new(
        SCROLLBAR_THUMB,
        "scrollbarThumb",
        ValueType::Color,
        "Optional thumb color; omitted windows remain visually neutral.",
    ))
    .property(PropertySchema::new(
        VIRTUAL_SCROLLBAR_VISIBLE,
        "scrollbarVisible",
        ValueType::Bool,
        "Show a native scrollbar for this window.",
    ))
    .property(PropertySchema::new(
        VIRTUAL_SCROLLBAR_WIDTH,
        "scrollbarWidth",
        ValueType::Float,
        "Native scrollbar width in logical pixels.",
    ))
    .property(PropertySchema::new(
        VIRTUAL_SHADOW_COLOR,
        "shadowColor",
        ValueType::Color,
        "Optional scroll edge shadow color; omitted color fades into the backdrop.",
    ))
    .property(PropertySchema::new(
        VIRTUAL_SHADOW_INTENSITY,
        "shadowIntensity",
        ValueType::Float,
        "Scroll edge shadow opacity strength.",
    ))
    .property(PropertySchema::new(
        VIRTUAL_SHADOW_WIDTH,
        "shadowWidth",
        ValueType::Float,
        "Scroll edge shadow width in logical pixels.",
    ))
    .property(PropertySchema::new(
        VIRTUAL_SHADOW_START,
        "shadowStart",
        ValueType::Bool,
        "Enable the leading scroll edge shadow.",
    ))
    .property(PropertySchema::new(
        VIRTUAL_SHADOW_END,
        "shadowEnd",
        ValueType::Bool,
        "Enable the trailing scroll edge shadow.",
    ))
    .property(
        PropertySchema::new(
            OFFSET_X,
            "offsetX",
            ValueType::Dimension,
            "Laid-out horizontal scroll position.",
        )
        .observed(ObservationKind::ScrollX),
    )
    .property(
        PropertySchema::new(
            VIRTUAL_VISIBLE_WIDTH,
            "visibleWidth",
            ValueType::Dimension,
            "Laid-out horizontal viewport width.",
        )
        .observed(ObservationKind::ViewportWidth),
    )
    .property(
        PropertySchema::new(
            CONTENT_WIDTH,
            "contentWidth",
            ValueType::Dimension,
            "Laid-out full content width.",
        )
        .observed(ObservationKind::ContentWidth),
    )
    .property(
        PropertySchema::new(
            OFFSET_Y,
            "offsetY",
            ValueType::Dimension,
            "Laid-out vertical scroll position in logical pixels.",
        )
        .observed(ObservationKind::ScrollY),
    )
    .property(
        PropertySchema::new(
            VISIBLE_HEIGHT,
            "visibleHeight",
            ValueType::Dimension,
            "Laid-out visible viewport height in logical pixels.",
        )
        .observed(ObservationKind::ViewportHeight),
    )
    .property(
        PropertySchema::new(
            CONTENT_HEIGHT,
            "contentHeight",
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
        "itemCount",
        ValueType::Int,
        "Host-provided collection length.",
    ))
    .property(PropertySchema::new(
        WINDOW_START,
        "windowStart",
        ValueType::Int,
        "Host-provided first mounted row index.",
    ))
    .property(PropertySchema::new(
        VIRTUAL_DATA_VERSION,
        "dataVersion",
        ValueType::Int,
        "Increment after a middle insertion, removal, or reorder to reset item measurements.",
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
    .event(EventSchema::new(
        VIRTUAL_MEASURE,
        "measure",
        EventType::VirtualMeasure,
        "Mounted item extents changed after native layout.",
    ))
    .event(EventSchema::new(
        VIRTUAL_WINDOW_CHANGE,
        "window",
        EventType::VirtualWindow,
        "Native scrolling selected another bounded item range.",
    ))
    .slot(SlotSchema {
        id: CHILDREN,
        name: "children".into(),
        arity: SlotArity::Many,
        documentation: "Only keyed rows in the current virtual window.".into(),
    });
    registry.register(schema, |input: &NativeElementInput| {
        let key = required_string(input, ID, "id")?;
        let row_height = float(input, ROW_HEIGHT, "rowHeight", 0.0)?;
        let horizontal = optional_bool(input, VIRTUAL_HORIZONTAL).unwrap_or(false);
        let viewport = if horizontal {
            float(input, VIRTUAL_VIEWPORT_WIDTH, "viewportWidth", 0.0)?
        } else {
            float(input, VIEWPORT_HEIGHT, "viewportHeight", 0.0)?
        };
        let offset = float(input, SCROLL_OFFSET, "offset", 0.0)?;
        if row_height <= 0.0 || viewport < 0.0 {
            return Err(SchemaError::Adapter(
                "VirtualWindow requires positive row_height and nonnegative viewport extent".into(),
            ));
        }
        let count = nonnegative_int(input, ITEM_COUNT, "__item_count", None)?;
        let first = nonnegative_int(input, WINDOW_START, "__window_start", None)?;
        let overscan = nonnegative_int(input, OVERSCAN, "overscan", Some(3))?;
        let rows = input.children(CHILDREN);
        let variable = optional_bool(input, VARIABLE_HEIGHT).unwrap_or(false);
        let list = input.virtual_list.clone().unwrap_or_else(|| {
            let list = if variable {
                VirtualList::variable(count, row_height, viewport)
            } else {
                VirtualList::fixed(count, row_height, viewport)
            };
            list.overscan(overscan)
        });
        let list = if horizontal { list.horizontal() } else { list };
        if first > count || rows.len() > count - first || rows.len() > 4096 {
            return Err(SchemaError::Adapter(format!(
                "VirtualWindow received an invalid or unbounded range of {} rows at {first}",
                rows.len()
            )));
        }
        let mut element =
            list.build_range(key.clone(), offset, first..first + rows.len(), |index| {
                rows[index - first].clone()
            });
        let mut scroll = element.scroll.as_deref().cloned().unwrap_or_default();
        if input.get(SCROLLBAR_THUMB).is_some()
            || optional_bool(input, VIRTUAL_SCROLLBAR_VISIBLE).unwrap_or(false)
        {
            let width = float(input, VIRTUAL_SCROLLBAR_WIDTH, "scrollbarWidth", 8.0)?;
            if width <= 0.0 {
                return Err(SchemaError::Adapter(
                    "VirtualWindow requires positive scrollbar_width".into(),
                ));
            }
            let scrollbar = ScrollbarStyle::new(
                ScrollbarPartStyle::new(argui_paint::QuadStyle::default()),
                ScrollbarPartStyle::new(argui_paint::QuadStyle::solid(
                    match input.get(SCROLLBAR_THUMB) {
                        Some(SchemaValue::Color(color)) => *color,
                        _ => argui_core::Color::srgba(0.55, 0.55, 0.55, 0.75),
                    },
                )),
            )
            .width(width)
            .visibility(
                if optional_bool(input, VIRTUAL_SCROLLBAR_VISIBLE).unwrap_or(true) {
                    ScrollbarVisibility::Always
                } else {
                    ScrollbarVisibility::Hidden
                },
            );
            scroll = scroll.scrollbar(scrollbar);
            element = element.scrollbar_gutter(ScrollbarGutter::Stable);
        }
        let shadow_width = float(input, VIRTUAL_SHADOW_WIDTH, "shadowWidth", 0.0)?;
        let shadow_intensity = float(input, VIRTUAL_SHADOW_INTENSITY, "shadowIntensity", 1.0)?;
        if shadow_width < 0.0 || !(0.0..=1.0).contains(&shadow_intensity) {
            return Err(SchemaError::Adapter(
                "VirtualWindow requires nonnegative shadow_width and shadow_intensity in [0,1]"
                    .into(),
            ));
        }
        if shadow_width > 0.0 {
            let start = f32::from(optional_bool(input, VIRTUAL_SHADOW_START).unwrap_or(true));
            let end = f32::from(optional_bool(input, VIRTUAL_SHADOW_END).unwrap_or(true));
            let strengths = if horizontal {
                [start, 0.0, end, 0.0]
            } else {
                [0.0, start, 0.0, end]
            };
            let effect = match input.get(VIRTUAL_SHADOW_COLOR) {
                Some(SchemaValue::Color(color)) => EdgeShadow::new(shadow_width, *color)
                    .intensity(shadow_intensity)
                    .strengths(strengths)
                    .scroll(),
                _ => EdgeFade::new(shadow_width)
                    .intensity(shadow_intensity)
                    .strengths(strengths)
                    .scroll(),
            };
            scroll = scroll.effect(effect);
        }
        element = element.scroll_config(scroll);
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
