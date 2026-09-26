//! Declarative painted rectangle with no control-specific interaction or styling.

use argui_ui::{
    Element, StateScopeId, StateSelector, StyleCondition, StylePatch, Transform2D, VisualState,
    property,
};

use super::{
    BACKGROUND, BORDER, CHILDREN, CLIP, CommonProperty, FOCUS_BORDER_COLOR, HOVER_BACKGROUND,
    PRESSED_BACKGROUND, PRESSED_SCALE, RADII, SHADOW, apply_common, apply_container,
    common_property, loop_motion, touch_area::TOUCH_AREA_SCOPE, transition,
};
use crate::{
    NativeElementInput, NativeSchema, SchemaError, SchemaRegistry, SchemaValue, SlotArity,
    SlotSchema, ValueType,
};

/// Registers the brush-painted Rectangle base element.
///
/// * `registry` — native schema registry receiving the element and its adapter.
///
/// # Errors
///
/// Returns a schema error for conflicting identifiers or invalid metadata.
pub(super) fn register(registry: &mut SchemaRegistry) -> Result<(), SchemaError> {
    let schema = NativeSchema::new(
        super::RECTANGLE,
        "Rectangle",
        "Painted rectangle with generic brush, border, radius, and clipping.",
    )
    .property(common_property(CommonProperty::Key))
    .property(common_property(CommonProperty::Tooltip))
    .property(common_property(CommonProperty::Width))
    .property(common_property(CommonProperty::Height))
    .property(common_property(CommonProperty::Position))
    .property(common_property(CommonProperty::Inset))
    .property(common_property(CommonProperty::MinWidth))
    .property(common_property(CommonProperty::MinHeight))
    .property(common_property(CommonProperty::Rotation))
    .property(loop_motion::properties()[0].clone())
    .property(loop_motion::properties()[1].clone())
    .property(loop_motion::properties()[2].clone())
    .property(loop_motion::properties()[3].clone())
    .property(loop_motion::properties()[4].clone())
    .property(loop_motion::properties()[5].clone())
    .property(loop_motion::properties()[6].clone())
    .property(loop_motion::properties()[7].clone())
    .property(loop_motion::properties()[8].clone())
    .property(loop_motion::properties()[9].clone())
    .property(loop_motion::properties()[10].clone())
    .property(common_property(CommonProperty::Opacity))
    .property(transition::properties()[0].clone())
    .property(transition::properties()[1].clone())
    .property(common_property(CommonProperty::BackdropFilter))
    .property(common_property(CommonProperty::DesktopBackdropTint))
    .property(common_property(CommonProperty::DesktopBackdropFallback))
    .property(common_property(CommonProperty::Visible))
    .property(common_property(CommonProperty::MaxWidth))
    .property(common_property(CommonProperty::MaxHeight))
    .property(common_property(CommonProperty::Grow))
    .property(common_property(CommonProperty::Shrink))
    .property(common_property(CommonProperty::AlignSelf))
    .property(common_property(CommonProperty::Margin))
    .property(common_property(CommonProperty::Padding))
    .property(crate::PropertySchema::new(
        BACKGROUND,
        "background",
        ValueType::Brush,
        "Brush painted inside the rectangle.",
    ))
    .property(crate::PropertySchema::new(
        HOVER_BACKGROUND,
        "hoverBackground",
        ValueType::Brush,
        "Brush painted while the nearest TouchArea or FocusScope is hovered.",
    ))
    .property(crate::PropertySchema::new(
        PRESSED_BACKGROUND,
        "pressedBackground",
        ValueType::Brush,
        "Brush painted while the nearest TouchArea or FocusScope is pressed.",
    ))
    .property(crate::PropertySchema::new(
        PRESSED_SCALE,
        "pressedScale",
        ValueType::Float,
        "Scale around the center while the nearest TouchArea or FocusScope is pressed.",
    ))
    .property(crate::PropertySchema::new(
        BORDER,
        "border",
        ValueType::Border,
        "Solid border with uniform or per-edge widths.",
    ))
    .property(crate::PropertySchema::new(
        RADII,
        "radii",
        ValueType::Radii,
        "Corner radii in logical pixels.",
    ))
    .property(crate::PropertySchema::new(
        SHADOW,
        "shadow",
        ValueType::Shadow,
        "Drop or inset shadow.",
    ))
    .property(crate::PropertySchema::new(
        FOCUS_BORDER_COLOR,
        "focusBorderColor",
        ValueType::Color,
        "Border color while the nearest FocusScope has visible keyboard focus.",
    ))
    .property(crate::PropertySchema::new(
        CLIP,
        "clip",
        ValueType::Bool,
        "Clip descendants to the rounded rectangle bounds.",
    ))
    .slot(SlotSchema {
        id: CHILDREN,
        name: "children".into(),
        arity: SlotArity::Many,
        documentation: "Elements painted within the rectangle.".into(),
    });
    registry.register(schema, |input: &NativeElementInput| {
        let mut element = transition::apply(
            apply_container(
                apply_common(Element::container(input.children(CHILDREN).to_vec()), input)?,
                input,
            ),
            input,
            "Rectangle",
        )?;
        if let Some(SchemaValue::Brush(brush)) = input.get(BACKGROUND) {
            element = element.fill(brush.clone());
        }
        let pressed_scale = match input.get(PRESSED_SCALE) {
            Some(SchemaValue::Float(scale))
                if scale.is_finite() && *scale > 0.0 && *scale <= 1.0 =>
            {
                Some(*scale)
            }
            Some(SchemaValue::Float(_)) => {
                return Err(SchemaError::Adapter(
                    "Rectangle pressedScale must be finite and in (0, 1]".into(),
                ));
            }
            _ => None,
        };
        for (id, state) in [
            (HOVER_BACKGROUND, VisualState::Hovered),
            (PRESSED_BACKGROUND, VisualState::Pressed),
        ] {
            let mut patch = StylePatch::new();
            let mut has_style = false;
            if let Some(SchemaValue::Brush(brush)) = input.get(id) {
                patch = patch.set(property::Background, Some(brush.clone()));
                has_style = true;
            }
            if state == VisualState::Pressed
                && let Some(scale) = pressed_scale
            {
                patch = patch.set(
                    property::Transform,
                    Transform2D::IDENTITY.scale(scale, scale),
                );
                has_style = true;
            }
            if has_style {
                element = element.when(
                    StyleCondition::any([
                        StyleCondition::state(StateSelector::scope(
                            StateScopeId::new(TOUCH_AREA_SCOPE),
                            state,
                        )),
                        StyleCondition::state(StateSelector::scope(
                            StateScopeId::new(super::focus_scope::FOCUS_SCOPE_SCOPE),
                            state,
                        )),
                    ]),
                    patch,
                );
            }
        }
        if let Some(SchemaValue::Border(border)) = input.get(BORDER) {
            element = element.border(*border);
        }
        let radii = match input.get(RADII) {
            Some(SchemaValue::Radii(value)) => *value,
            _ => argui_paint::CornerRadii::all(0.0),
        };
        if radii
            .as_array()
            .iter()
            .any(|value| !value.is_finite() || *value < 0.0)
        {
            return Err(SchemaError::Adapter(
                "Rectangle radii must be finite and nonnegative".into(),
            ));
        }
        element = if matches!(input.get(CLIP), Some(SchemaValue::Bool(true))) {
            element.clip(radii)
        } else {
            element.radius(radii)
        };
        if let Some(SchemaValue::Shadow(shadow)) = input.get(SHADOW) {
            element = element.shadow(*shadow);
        }
        if let Some(SchemaValue::Color(color)) = input.get(FOCUS_BORDER_COLOR) {
            element = element.when(
                StyleCondition::any([
                    StyleCondition::state(StateSelector::scope(
                        StateScopeId::new(super::focus_scope::FOCUS_SCOPE_SCOPE),
                        VisualState::FocusVisible,
                    )),
                    StyleCondition::state(VisualState::FocusWithin),
                ]),
                StylePatch::new().set(property::BorderColor, *color),
            );
        }
        loop_motion::apply(element, input)
    })
}
