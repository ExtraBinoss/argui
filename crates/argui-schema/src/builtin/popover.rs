//! Anchored, dismissible native popup surface.

use argui_paint::{Border, CornerRadii};
use argui_ui::{
    DismissPolicy, Element, EventType, FloatingPlacement, FocusScope, Placement, Role, Semantics,
    WindowLayer,
};

use super::{
    ANCHOR, BORDER_COLOR, CHILDREN, CommonProperty, DISMISS, POPOVER_PANEL, RADIUS, apply_common,
    apply_container, apply_events, common_event, common_property, required_string,
};
use crate::{
    NativeElementInput, NativeSchema, PropertySchema, SchemaError, SchemaRegistry, SchemaValue,
    SlotArity, SlotSchema, ValueType,
};

/// Registers an anchored, dismissible popup surface for DSL-owned panels.
///
/// * `registry` — canonical native registry receiving this primitive.
///
/// # Errors
///
/// Returns when the schema conflicts with an existing native primitive.
pub(super) fn register(registry: &mut SchemaRegistry) -> Result<(), SchemaError> {
    let schema = NativeSchema::new(
        POPOVER_PANEL,
        "PopoverPanel",
        "Anchored floating selection panel.",
    )
    .property(common_property(CommonProperty::Key))
    .property(
        PropertySchema::new(
            ANCHOR,
            "anchor",
            ValueType::String,
            "Key of the trigger anchor.",
        )
        .required(),
    )
    .property(common_property(CommonProperty::Width))
    .property(common_property(CommonProperty::Rotation))
    .property(common_property(CommonProperty::Opacity))
    .property(common_property(CommonProperty::Background))
    .property(common_property(CommonProperty::Padding))
    .property(common_property(CommonProperty::Gap))
    .property(PropertySchema::new(
        BORDER_COLOR,
        "border_color",
        ValueType::Color,
        "Panel outline color.",
    ))
    .property(PropertySchema::new(
        RADIUS,
        "radius",
        ValueType::Float,
        "Panel corner radius.",
    ))
    .event(common_event(DISMISS, "dismiss", EventType::Dismiss))
    .slot(SlotSchema {
        id: CHILDREN,
        name: "children".into(),
        arity: SlotArity::Many,
        documentation: "Selection options.".into(),
    });
    registry.register(schema, |input: &NativeElementInput| {
        let anchor = required_string(input, ANCHOR, "anchor")?;
        let mut element = apply_container(
            apply_common(Element::column(input.children(CHILDREN).to_vec()), input),
            input,
        );
        if let Some(SchemaValue::Color(color)) = input.get(BORDER_COLOR) {
            element = element.border(Border::all(1.0, *color));
        }
        if let Some(SchemaValue::Float(radius)) = input.get(RADIUS) {
            element = element.radius(CornerRadii::all(*radius));
        }
        element = element
            .anchored_portal(
                WindowLayer::Popover,
                anchor.clone(),
                FloatingPlacement::new(Placement::BottomStart),
            )
            .portal_dismiss(DismissPolicy::OutsidePointer)
            .focus_scope(FocusScope::restoring())
            .semantics(Semantics::new(Role::Group).label("Selection options"));
        Ok(apply_events(element, input))
    })
}
