//! Native schema and adapter for generic layout containers.

use super::*;

/// Registers a native layout container with the shared visual contract.
/// `registry` receives the schema, `id` and `name` identify it, and
/// `constructor` creates an element from visual children. Returns a schema
/// registration error when IDs or names collide.
pub(super) fn register(
    registry: &mut SchemaRegistry,
    id: NativeTypeId,
    name: &'static str,
    constructor: fn(Vec<Element>) -> Element,
) -> Result<(), SchemaError> {
    let mut schema = NativeSchema::new(id, name, format!("Native {name} layout primitive."))
        .property(common_property(CommonProperty::Key))
        .property(common_property(CommonProperty::Tooltip))
        .property(common_property(CommonProperty::Width))
        .property(common_property(CommonProperty::Height))
        .property(common_property(CommonProperty::Position))
        .property(common_property(CommonProperty::Inset))
        .property(
            PropertySchema::new(
                MEASURED_WIDTH,
                "measuredWidth",
                ValueType::Dimension,
                "Last completed layout width in logical pixels; read-only and one frame delayed.",
            )
            .observed(crate::ObservationKind::MeasuredWidth)
            .not_animatable(),
        )
        .property(
            PropertySchema::new(
                MEASURED_HEIGHT,
                "measuredHeight",
                ValueType::Dimension,
                "Last completed layout height in logical pixels; read-only and one frame delayed.",
            )
            .observed(crate::ObservationKind::MeasuredHeight)
            .not_animatable(),
        )
        .property(common_property(CommonProperty::Rotation))
        .property(common_property(CommonProperty::Opacity))
        .property(common_property(CommonProperty::BackdropFilter))
        .property(common_property(CommonProperty::DesktopBackdropTint))
        .property(common_property(CommonProperty::DesktopBackdropFallback))
        .property(common_property(CommonProperty::Visible))
        .property(common_property(CommonProperty::MinWidth))
        .property(common_property(CommonProperty::MinHeight))
        .property(common_property(CommonProperty::Background))
        .property(common_property(CommonProperty::SelectionFill))
        .property(common_property(CommonProperty::SelectionColor))
        .property(common_property(CommonProperty::SelectionRadius))
        .property(common_property(CommonProperty::Gap))
        .property(loop_motion::properties()[1].clone())
        .property(loop_motion::properties()[2].clone())
        .property(loop_motion::properties()[11].clone())
        .property(common_property(CommonProperty::Padding))
        .property(PropertySchema::new(
            DIRECTION_SCOPE,
            "directionScope",
            ValueType::String,
            "Writing direction inherited by visual descendants and portal content: ltr or rtl.",
        ))
        .property(PropertySchema::new(
            WRAP,
            "wrap",
            ValueType::Bool,
            "Wrap children when space is narrow.",
        ))
        .property(PropertySchema::new(
            GROW,
            "grow",
            ValueType::Float,
            "Flex growth factor.",
        ))
        .property(PropertySchema::new(
            SHRINK,
            "shrink",
            ValueType::Float,
            "Flex shrink factor.",
        ))
        .property(PropertySchema::new(
            ALIGN_ITEMS,
            "alignItems",
            ValueType::String,
            "Cross-axis alignment of children.",
        ))
        .property(PropertySchema::new(
            JUSTIFY_CONTENT,
            "justifyContent",
            ValueType::String,
            "Main-axis distribution of children.",
        ))
        .event(common_event(CLICK, "click", EventType::Click))
        .event(common_event(FOCUS, "focus", EventType::Focus))
        .event(common_event(BLUR, "blur", EventType::Blur))
        .slot(SlotSchema {
            id: CHILDREN,
            name: "children".into(),
            arity: SlotArity::Many,
            documentation: "Ordered visual children.".into(),
        });
    for property in layout::properties() {
        schema = schema.property(property);
    }
    for property in transition::properties() {
        schema = schema.property(property);
    }
    registry.register(schema, move |input: &NativeElementInput| {
        let children = input.children(CHILDREN).to_vec();
        let mut element = apply_container(apply_common(constructor(children), input)?, input);
        if let Some(SchemaValue::String(value)) = input.get(DIRECTION_SCOPE) {
            element = element.direction_scope(match value.as_str() {
                "ltr" => WritingDirection::Ltr,
                "rtl" => WritingDirection::Rtl,
                _ => {
                    return Err(SchemaError::Adapter(format!(
                        "{name} does not support direction_scope `{value}`"
                    )));
                }
            });
        }
        if optional_bool(input, WRAP) == Some(true) {
            element = element.flex_wrap(FlexWrap::Wrap);
        }
        if let Some(SchemaValue::Float(grow)) = input.get(GROW) {
            element = element.grow(*grow);
        }
        if let Some(SchemaValue::Float(shrink)) = input.get(SHRINK) {
            element = element.shrink(*shrink);
        }
        if let Some(SchemaValue::String(value)) = input.get(ALIGN_ITEMS) {
            element = element.align_items(match value.as_str() {
                "start" => AlignItems::START,
                "center" => AlignItems::CENTER,
                "end" => AlignItems::END,
                "stretch" => AlignItems::STRETCH,
                _ => {
                    return Err(SchemaError::Adapter(format!(
                        "{name} does not support align_items `{value}`"
                    )));
                }
            });
        }
        if let Some(SchemaValue::String(value)) = input.get(JUSTIFY_CONTENT) {
            element = element.justify_content(match value.as_str() {
                "start" => JustifyContent::START,
                "center" => JustifyContent::CENTER,
                "end" => JustifyContent::END,
                "spaceBetween" => JustifyContent::SPACE_BETWEEN,
                "spaceAround" => JustifyContent::SPACE_AROUND,
                "spaceEvenly" => JustifyContent::SPACE_EVENLY,
                _ => {
                    return Err(SchemaError::Adapter(format!(
                        "{name} does not support justify_content `{value}`"
                    )));
                }
            });
        }
        Ok(apply_events(
            loop_motion::apply(
                transition::apply(layout::apply(element, input, name)?, input, name)?,
                input,
            )?,
            input,
        ))
    })
}
