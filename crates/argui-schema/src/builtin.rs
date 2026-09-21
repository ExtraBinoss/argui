//! Stable schemas for the deliberately small Rust-backed visual and behavior surface.

use argui_text::{TextColor, TextStyle};
use argui_ui::{
    CaretStyle, CursorIcon, Element, EventType, FocusPolicy, GestureSet, Interaction,
    KeyboardActivation, Role, SemanticAction, SemanticState, SemanticValue, Semantics, Sides,
    TextEditorSpec, TextInputFilter, UserSelect,
};

use crate::{
    EventId, EventSchema, NativeElementInput, NativeSchema, NativeTypeId, PropertyId,
    PropertySchema, SchemaError, SchemaRegistry, SchemaValue, SlotArity, SlotId, SlotSchema,
    ValueType,
};

pub const CONTAINER: NativeTypeId = NativeTypeId::from_raw(1);
pub const ROW: NativeTypeId = NativeTypeId::from_raw(2);
pub const COLUMN: NativeTypeId = NativeTypeId::from_raw(3);
pub const TEXT: NativeTypeId = NativeTypeId::from_raw(4);
pub const PRESSABLE: NativeTypeId = NativeTypeId::from_raw(5);
pub const TEXT_EDITOR: NativeTypeId = NativeTypeId::from_raw(6);

pub const CHILDREN: SlotId = SlotId::from_raw(1);
pub const KEY: PropertyId = PropertyId::from_raw(1);
pub const TOOLTIP: PropertyId = PropertyId::from_raw(2);
pub const WIDTH: PropertyId = PropertyId::from_raw(3);
pub const HEIGHT: PropertyId = PropertyId::from_raw(4);
pub const BACKGROUND: PropertyId = PropertyId::from_raw(5);
pub const GAP: PropertyId = PropertyId::from_raw(6);
pub const PADDING: PropertyId = PropertyId::from_raw(7);
pub const CONTENT: PropertyId = PropertyId::from_raw(8);
pub const TEXT_VALUE: PropertyId = PropertyId::from_raw(9);
pub const VALUE: PropertyId = PropertyId::from_raw(10);
pub const INPUT_PLACEHOLDER: PropertyId = PropertyId::from_raw(11);
pub const ENABLED: PropertyId = PropertyId::from_raw(12);
pub const READ_ONLY: PropertyId = PropertyId::from_raw(13);
pub const LABEL: PropertyId = PropertyId::from_raw(14);
pub const DESCRIPTION: PropertyId = PropertyId::from_raw(15);
pub const INVALID: PropertyId = PropertyId::from_raw(16);

pub const CLICK: EventId = EventId::from_raw(1);
pub const FOCUS: EventId = EventId::from_raw(2);
pub const BLUR: EventId = EventId::from_raw(3);
pub const INPUT_CHANGED: EventId = EventId::from_raw(4);
pub const SUBMIT: EventId = EventId::from_raw(5);

/// Creates the canonical registry of built-in native visual primitives.
///
/// # Errors
///
/// Returns a schema error if the built-in definitions violate registry invariants.
pub fn registry() -> Result<SchemaRegistry, SchemaError> {
    let mut registry = SchemaRegistry::new();
    register_container(&mut registry, CONTAINER, "Container", Element::container)?;
    register_container(&mut registry, ROW, "Row", Element::row)?;
    register_container(&mut registry, COLUMN, "Column", Element::column)?;
    let text = NativeSchema::new(TEXT, "Text", "Displays styled text content.")
        .property(PropertySchema::new(
            CONTENT,
            "content",
            ValueType::String,
            "Displayed text.",
        ))
        .property(PropertySchema::new(
            TEXT_VALUE,
            "text",
            ValueType::String,
            "Displayed text using the standard-library spelling.",
        ))
        .property(common_property(CommonProperty::Key))
        .property(common_property(CommonProperty::Tooltip))
        .property(common_property(CommonProperty::Width))
        .property(common_property(CommonProperty::Height))
        .property(common_property(CommonProperty::Background));
    registry.register(text, |input: &NativeElementInput| {
        let content = optional_string(input, TEXT_VALUE)
            .or_else(|| optional_string(input, CONTENT))
            .ok_or_else(|| SchemaError::Adapter("Text requires `text` or `content`".into()))?;
        Ok(apply_common(Element::text(content.clone()), input))
    })?;
    register_pressable(&mut registry)?;
    register_text_editor(&mut registry)?;
    Ok(registry)
}

fn register_container(
    registry: &mut SchemaRegistry,
    id: NativeTypeId,
    name: &'static str,
    constructor: fn(Vec<Element>) -> Element,
) -> Result<(), SchemaError> {
    let schema = NativeSchema::new(id, name, format!("Native {name} layout primitive."))
        .property(common_property(CommonProperty::Key))
        .property(common_property(CommonProperty::Tooltip))
        .property(common_property(CommonProperty::Width))
        .property(common_property(CommonProperty::Height))
        .property(common_property(CommonProperty::Background))
        .property(common_property(CommonProperty::Gap))
        .property(common_property(CommonProperty::Padding))
        .event(common_event(CLICK, "click", EventType::Click))
        .event(common_event(FOCUS, "focus", EventType::Focus))
        .event(common_event(BLUR, "blur", EventType::Blur))
        .slot(SlotSchema {
            id: CHILDREN,
            name: "children".into(),
            arity: SlotArity::Many,
            documentation: "Ordered visual children.".into(),
        });
    registry.register(schema, move |input: &NativeElementInput| {
        let children = input.children(CHILDREN).to_vec();
        Ok(apply_events(
            apply_container(apply_common(constructor(children), input), input),
            input,
        ))
    })
}

/// Registers the accessible press behavior used by standard-library controls.
fn register_pressable(registry: &mut SchemaRegistry) -> Result<(), SchemaError> {
    let schema = NativeSchema::new(PRESSABLE, "Pressable", "Accessible press behavior.")
        .property(common_property(CommonProperty::Key))
        .property(
            PropertySchema::new(
                LABEL,
                "label",
                ValueType::String,
                "Accessible control label.",
            )
            .required(),
        )
        .property(common_property(CommonProperty::Tooltip))
        .property(common_property(CommonProperty::Width))
        .property(common_property(CommonProperty::Height))
        .property(common_property(CommonProperty::Background))
        .property(common_property(CommonProperty::Gap))
        .property(common_property(CommonProperty::Padding))
        .property(
            PropertySchema::new(
                ENABLED,
                "enabled",
                ValueType::Bool,
                "Whether the control accepts activation.",
            )
            .default_value(SchemaValue::Bool(true)),
        )
        .event(common_event(CLICK, "click", EventType::Click))
        .event(common_event(FOCUS, "focus", EventType::Focus))
        .event(common_event(BLUR, "blur", EventType::Blur))
        .slot(SlotSchema {
            id: CHILDREN,
            name: "children".into(),
            arity: SlotArity::Many,
            documentation: "Visual button content.".into(),
        });
    registry.register(schema, |input: &NativeElementInput| {
        let key = required_string(input, KEY, "key")?;
        let label = required_string(input, LABEL, "label")?;
        let enabled = optional_bool(input, ENABLED).unwrap_or(true);
        let children = input
            .children(CHILDREN)
            .iter()
            .cloned()
            .map(|child| child.semantic_hidden(true));
        let element = Element::row(children)
            .keyed(key.clone())
            .user_select(UserSelect::None)
            .interaction(
                Interaction::default()
                    .enabled(enabled)
                    .focus_policy(if enabled {
                        FocusPolicy::TabStop
                    } else {
                        FocusPolicy::None
                    })
                    .cursor(if enabled {
                        CursorIcon::Pointer
                    } else {
                        CursorIcon::NotAllowed
                    })
                    .gestures(GestureSet::default().tap(argui_ui::TapGesture::default()))
                    .keyboard_activation(KeyboardActivation::EnterOrSpace),
            )
            .semantics(
                Semantics::new(Role::Button)
                    .label(label.clone())
                    .state(SemanticState {
                        disabled: !enabled,
                        ..SemanticState::default()
                    })
                    .action(SemanticAction::Click)
                    .action(SemanticAction::Focus),
            );
        Ok(apply_events(
            apply_container(apply_common(element, input), input),
            input,
        ))
    })
}

/// Registers the accessible controlled text-editing behavior used by DSL forms.
fn register_text_editor(registry: &mut SchemaRegistry) -> Result<(), SchemaError> {
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
    .property(common_property(CommonProperty::Background))
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
        let placeholder_style = TextStyle {
            color: TextColor::srgba(0.55, 0.60, 0.68, 1.0),
            ..TextStyle::default()
        };
        let mut semantics = Semantics::new(Role::TextInput)
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
        let element = Element::text_editor(TextEditorSpec {
            value,
            placeholder,
            multiline: false,
            read_only,
            filter: TextInputFilter::Any,
            text: TextStyle::default(),
            placeholder_text: placeholder_style,
            selection: argui_core::Color::srgba(0.20, 0.68, 0.94, 0.38),
            caret: CaretStyle::default(),
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

/// Creates a common event declaration whose name is shared across primitives.
fn common_event(id: EventId, name: &'static str, event_type: EventType) -> EventSchema {
    EventSchema::new(id, name, event_type, format!("Native `{name}` event."))
}

/// Attaches every schema-validated opaque handler to its engine event kind.
fn apply_events(mut element: Element, input: &NativeElementInput) -> Element {
    for event in &input.events {
        let event_type = match event.id {
            CLICK => EventType::Click,
            FOCUS => EventType::Focus,
            BLUR => EventType::Blur,
            INPUT_CHANGED => EventType::Input,
            SUBMIT => EventType::Submit,
            _ => continue,
        };
        element = element.on(event.handler.direct_listener(event_type));
    }
    element
}

#[derive(Clone, Copy)]
enum CommonProperty {
    Key,
    Tooltip,
    Width,
    Height,
    Background,
    Gap,
    Padding,
}

fn common_property(property: CommonProperty) -> PropertySchema {
    match property {
        CommonProperty::Key => {
            PropertySchema::new(KEY, "key", ValueType::String, "Public reconciliation key.")
        }
        CommonProperty::Tooltip => PropertySchema::new(
            TOOLTIP,
            "tooltip",
            ValueType::String,
            "Tooltip description.",
        ),
        CommonProperty::Width => {
            PropertySchema::new(WIDTH, "width", ValueType::Dimension, "Preferred width.")
        }
        CommonProperty::Height => {
            PropertySchema::new(HEIGHT, "height", ValueType::Dimension, "Preferred height.")
        }
        CommonProperty::Background => PropertySchema::new(
            BACKGROUND,
            "background",
            ValueType::Color,
            "Background color.",
        ),
        CommonProperty::Gap => {
            PropertySchema::new(GAP, "gap", ValueType::Float, "Uniform child gap.")
        }
        CommonProperty::Padding => {
            PropertySchema::new(PADDING, "padding", ValueType::Float, "Uniform padding.")
        }
    }
}

fn apply_common(mut element: Element, input: &NativeElementInput) -> Element {
    if let Some(SchemaValue::String(value)) = input.get(KEY) {
        element = element.keyed(value.clone());
    }
    if let Some(SchemaValue::String(value)) = input.get(TOOLTIP) {
        element = element.tooltip(value.clone());
    }
    if let Some(SchemaValue::Dimension(value)) = input.get(WIDTH) {
        element = element.width(*value);
    }
    if let Some(SchemaValue::Dimension(value)) = input.get(HEIGHT) {
        element = element.height(*value);
    }
    if let Some(SchemaValue::Color(value)) = input.get(BACKGROUND) {
        element = element.background(*value);
    }
    element
}

fn apply_container(mut element: Element, input: &NativeElementInput) -> Element {
    if let Some(SchemaValue::Float(value)) = input.get(GAP) {
        element = element.gap(*value);
    }
    if let Some(SchemaValue::Float(value)) = input.get(PADDING) {
        element = element.padding(Sides::length(*value));
    }
    element
}

fn required_string<'a>(
    input: &'a NativeElementInput,
    id: PropertyId,
    name: &str,
) -> Result<&'a String, SchemaError> {
    let value = input.get(id).ok_or_else(|| {
        SchemaError::Adapter(format!(
            "required property `{name}` disappeared after validation"
        ))
    })?;
    let SchemaValue::String(value) = value else {
        return Err(SchemaError::Adapter(format!(
            "validated property `{name}` changed type"
        )));
    };
    Ok(value)
}

/// Reads an optional string property after registry type validation.
fn optional_string(input: &NativeElementInput, id: PropertyId) -> Option<&String> {
    match input.get(id) {
        Some(SchemaValue::String(value)) => Some(value),
        _ => None,
    }
}

/// Reads an optional boolean property after registry type validation.
fn optional_bool(input: &NativeElementInput, id: PropertyId) -> Option<bool> {
    match input.get(id) {
        Some(SchemaValue::Bool(value)) => Some(*value),
        _ => None,
    }
}
