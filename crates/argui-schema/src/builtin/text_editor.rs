//! Accessible controlled text editing with themeable native paint properties.

use argui_paint::{CornerRadii, Fill, QuadStyle};
use argui_text::{TextColor, TextStyle};
use argui_ui::{
    CaretAlign, CaretHeight, CaretPrimitive, CaretStyle, CaretVisual, CursorIcon, Element,
    EventType, FocusPolicy, GestureSet, Interaction, Role, SemanticAction, SemanticState,
    SemanticValue, Semantics, TextEditorSpec, TextInputFilter,
};

use super::{
    BLUR, CARET_BLINK, CARET_COLOR, CARET_COUNT, CARET_FILL, CARET_HEIGHT, CARET_OFFSET_Y,
    CARET_RADIUS, CARET_SPACING, CARET_WIDTH, DESCRIPTION, ENABLED, FOCUS, INPUT_CHANGED,
    INPUT_PLACEHOLDER, INVALID, KEY, LABEL, PLACEHOLDER_COLOR, READ_ONLY, SEARCH_INPUT,
    SELECTION_COLOR, SUBMIT, TEXT_COLOR, TEXT_EDITOR, VALUE,
    common::{
        CommonProperty, apply_common, apply_events, common_event, common_property, optional_bool,
        optional_string, required_string,
    },
};
use crate::{
    NativeElementInput, NativeSchema, PropertySchema, SchemaError, SchemaRegistry, SchemaValue,
    ValueType,
};

/// Registers the accessible controlled text-editing primitive.
///
/// * `registry` — canonical native registry receiving the editor schema and adapter.
///
/// # Errors
///
/// Returns when the schema conflicts with another registered native definition.
pub(super) fn register(registry: &mut SchemaRegistry) -> Result<(), SchemaError> {
    let schema = NativeSchema::new(
        TEXT_EDITOR,
        "TextEditor",
        "Accessible controlled text-editing behavior.",
    )
    .property(common_property(CommonProperty::Key))
    .property(
        PropertySchema::new(VALUE, "value", ValueType::String, "Controlled text value.")
            .default_value(SchemaValue::String(String::new()))
            .changed_by(INPUT_CHANGED),
    )
    .property(
        PropertySchema::new(
            INPUT_PLACEHOLDER,
            "placeholder",
            ValueType::String,
            "Placeholder text.",
        )
        .default_value(SchemaValue::String(String::new())),
    )
    .property(
        PropertySchema::new(
            ENABLED,
            "enabled",
            ValueType::Bool,
            "Whether editing is enabled.",
        )
        .default_value(SchemaValue::Bool(true)),
    )
    .property(
        PropertySchema::new(
            READ_ONLY,
            "read_only",
            ValueType::Bool,
            "Whether the value is read-only.",
        )
        .default_value(SchemaValue::Bool(false)),
    )
    .property(
        PropertySchema::new(
            SEARCH_INPUT,
            "search",
            ValueType::Bool,
            "Expose search-input semantics.",
        )
        .default_value(SchemaValue::Bool(false)),
    )
    .property(PropertySchema::new(
        LABEL,
        "label",
        ValueType::String,
        "Accessible input label.",
    ))
    .property(PropertySchema::new(
        DESCRIPTION,
        "description",
        ValueType::String,
        "Accessible description.",
    ))
    .property(
        PropertySchema::new(
            INVALID,
            "invalid",
            ValueType::Bool,
            "Whether the current value is invalid.",
        )
        .default_value(SchemaValue::Bool(false)),
    )
    .property(common_property(CommonProperty::Width))
    .property(common_property(CommonProperty::Height))
    .property(common_property(CommonProperty::Rotation))
    .property(common_property(CommonProperty::Opacity))
    .property(common_property(CommonProperty::Background))
    .property(common_property(CommonProperty::SelectionFill))
    .property(common_property(CommonProperty::SelectionRadius))
    .property(PropertySchema::new(
        TEXT_COLOR,
        "text_color",
        ValueType::Color,
        "Input text foreground color.",
    ))
    .property(PropertySchema::new(
        PLACEHOLDER_COLOR,
        "placeholder_color",
        ValueType::Color,
        "Placeholder text foreground color.",
    ))
    .property(common_property(CommonProperty::SelectionColor))
    .property(PropertySchema::new(
        CARET_COLOR,
        "caret_color",
        ValueType::Color,
        "Text caret color.",
    ))
    .property(PropertySchema::new(
        CARET_FILL,
        "caret_fill",
        ValueType::Brush,
        "GPU fill shared by the caret's composed primitives.",
    ))
    .property(PropertySchema::new(
        CARET_WIDTH,
        "caret_width",
        ValueType::Float,
        "Width of each caret primitive.",
    ))
    .property(PropertySchema::new(
        CARET_HEIGHT,
        "caret_height",
        ValueType::Float,
        "Fixed caret height; zero uses the text line height.",
    ))
    .property(PropertySchema::new(
        CARET_RADIUS,
        "caret_radius",
        ValueType::Float,
        "Corner radius of each caret primitive.",
    ))
    .property(PropertySchema::new(
        CARET_COUNT,
        "caret_count",
        ValueType::Int,
        "Number of repeated caret primitives.",
    ))
    .property(PropertySchema::new(
        CARET_SPACING,
        "caret_spacing",
        ValueType::Float,
        "Horizontal distance between caret primitives.",
    ))
    .property(PropertySchema::new(
        CARET_OFFSET_Y,
        "caret_offset_y",
        ValueType::Float,
        "Vertical offset of the caret primitives.",
    ))
    .property(PropertySchema::new(
        CARET_BLINK,
        "caret_blink",
        ValueType::Bool,
        "Whether to use the standard blinking timeline.",
    ))
    .event(common_event(INPUT_CHANGED, "input", EventType::Input).payload(ValueType::String))
    .event(common_event(SUBMIT, "submit", EventType::Submit).payload(ValueType::String))
    .event(common_event(FOCUS, "focus", EventType::Focus))
    .event(common_event(BLUR, "blur", EventType::Blur));
    registry.register(schema, |input: &NativeElementInput| {
        let key = required_string(input, KEY, "key")?;
        let value = optional_string(input, VALUE).cloned().unwrap_or_default();
        let placeholder = optional_string(input, INPUT_PLACEHOLDER)
            .cloned()
            .unwrap_or_default();
        let enabled = optional_bool(input, ENABLED).unwrap_or(true);
        let read_only = optional_bool(input, READ_ONLY).unwrap_or(false);
        let invalid = optional_bool(input, INVALID).unwrap_or(false);
        let label = optional_string(input, LABEL)
            .unwrap_or(&placeholder)
            .clone();
        let mut placeholder_style = TextStyle {
            color: TextColor::srgba(0.55, 0.60, 0.68, 1.0),
            ..TextStyle::default()
        };
        if let Some(SchemaValue::Color(color)) = input.get(PLACEHOLDER_COLOR) {
            placeholder_style.color = *color;
        }
        let mut input_style = TextStyle::default();
        if let Some(SchemaValue::Color(color)) = input.get(TEXT_COLOR) {
            input_style.color = *color;
        }
        let role = if optional_bool(input, SEARCH_INPUT) == Some(true) {
            Role::SearchInput
        } else {
            Role::TextInput
        };
        let mut semantics = Semantics::new(role)
            .label(label)
            .value(SemanticValue::Text(value.clone()))
            .state(SemanticState {
                disabled: !enabled,
                read_only,
                invalid,
                ..SemanticState::default()
            })
            .action(SemanticAction::Focus)
            .action(SemanticAction::SetValue);
        if let Some(description) = optional_string(input, DESCRIPTION) {
            semantics = semantics.description(description.clone());
        }
        let selection = input
            .get(SELECTION_COLOR)
            .and_then(|value| match value {
                SchemaValue::Color(color) => Some(*color),
                _ => None,
            })
            .unwrap_or_else(|| argui_core::Color::srgba(0.20, 0.68, 0.94, 0.38));
        let mut caret = CaretStyle::default();
        let caret_fill = match input.get(CARET_FILL) {
            Some(SchemaValue::Brush(fill)) => fill.clone(),
            _ => Fill::Solid(match input.get(CARET_COLOR) {
                Some(SchemaValue::Color(color)) => *color,
                _ => argui_core::Color::WHITE,
            }),
        };
        let float = |id, default| match input.get(id) {
            Some(SchemaValue::Float(value)) => *value,
            _ => default,
        };
        let width = float(CARET_WIDTH, 1.5).max(0.0);
        let height = float(CARET_HEIGHT, 0.0);
        let radius = float(CARET_RADIUS, 0.0).max(0.0);
        let spacing = float(CARET_SPACING, 6.0);
        let offset_y = float(CARET_OFFSET_Y, 0.0);
        let count = match input.get(CARET_COUNT) {
            Some(SchemaValue::Int(value)) => (*value).clamp(1, 16) as usize,
            _ => 1,
        };
        let visual = (0..count).map(|index| {
            let paint = QuadStyle {
                background: Some(caret_fill.clone()),
                ..QuadStyle::default()
            }
            .radius(CornerRadii::all(radius));
            CaretPrimitive::new(
                width,
                if height > 0.0 {
                    CaretHeight::Pixels(height)
                } else {
                    CaretHeight::Line
                },
                paint,
            )
            .align(if height > 0.0 {
                CaretAlign::End
            } else {
                CaretAlign::Start
            })
            .offset(index as f32 * spacing, offset_y)
        });
        caret.visual = CaretVisual::new(visual);
        if optional_bool(input, CARET_BLINK) == Some(false) {
            caret.animation = None;
        }
        let element = Element::text_editor(TextEditorSpec {
            value,
            placeholder,
            multiline: false,
            read_only,
            filter: TextInputFilter::Any,
            text: input_style,
            placeholder_text: placeholder_style,
            selection,
            caret,
        })
        .keyed(key.clone())
        .interaction(
            Interaction::default()
                .enabled(enabled)
                .focus_policy(if enabled {
                    FocusPolicy::TabStop
                } else {
                    FocusPolicy::None
                })
                .cursor(CursorIcon::Text)
                .gestures(
                    GestureSet::default()
                        .tap(argui_ui::TapGesture::default())
                        .pan(argui_ui::PanGesture::default()),
                ),
        )
        .semantics(semantics);
        Ok(apply_events(apply_common(element, input), input))
    })
}
