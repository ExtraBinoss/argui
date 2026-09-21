//! Stable schemas for the deliberately small Rust-backed visual and behavior surface.

use argui_paint::{Border, CornerRadii};
use argui_text::{TextStyle, TextWrap};
use argui_ui::{
    AlignItems, CursorIcon, Element, EventType, FlexWrap, FocusPolicy, GestureSet, Interaction,
    JustifyContent, KeyboardActivation, Role, SemanticAction, SemanticState, Semantics, StateName,
    StateScopeId, StylePatch, UserSelect, VisualState, property,
};

use crate::{
    EventId, NativeElementInput, NativeSchema, NativeTypeId, PropertyId, PropertySchema,
    SchemaError, SchemaRegistry, SchemaValue, SlotArity, SlotId, SlotSchema, ValueType,
};

mod common;
mod media;
mod popover;
mod switch;
mod text_editor;
mod virtual_list;
use common::*;

pub const CONTAINER: NativeTypeId = NativeTypeId::from_raw(1);
pub const ROW: NativeTypeId = NativeTypeId::from_raw(2);
pub const COLUMN: NativeTypeId = NativeTypeId::from_raw(3);
pub const TEXT: NativeTypeId = NativeTypeId::from_raw(4);
pub const PRESSABLE: NativeTypeId = NativeTypeId::from_raw(5);
pub const TEXT_EDITOR: NativeTypeId = NativeTypeId::from_raw(6);
pub const POPOVER_PANEL: NativeTypeId = NativeTypeId::from_raw(7);
pub const IMAGE: NativeTypeId = NativeTypeId::from_raw(8);
pub const SVG: NativeTypeId = NativeTypeId::from_raw(9);
pub const SWITCH: NativeTypeId = NativeTypeId::from_raw(10);
pub const VIRTUAL_LIST: NativeTypeId = NativeTypeId::from_raw(11);

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
pub const BUSY: PropertyId = PropertyId::from_raw(17);
pub const BORDER_COLOR: PropertyId = PropertyId::from_raw(18);
pub const HOVER_BACKGROUND: PropertyId = PropertyId::from_raw(19);
pub const PRESSED_BACKGROUND: PropertyId = PropertyId::from_raw(20);
pub const FOCUS_BORDER_COLOR: PropertyId = PropertyId::from_raw(21);
pub const RADIUS: PropertyId = PropertyId::from_raw(22);
pub const TEXT_COLOR: PropertyId = PropertyId::from_raw(23);
pub const TEXT_WEIGHT: PropertyId = PropertyId::from_raw(24);
pub const WRAP: PropertyId = PropertyId::from_raw(25);
pub const GROW: PropertyId = PropertyId::from_raw(26);
pub const ANCHOR: PropertyId = PropertyId::from_raw(27);
pub const SELECT_TRIGGER: PropertyId = PropertyId::from_raw(28);
pub const EXPANDED: PropertyId = PropertyId::from_raw(29);
pub const TEXT_SIZE: PropertyId = PropertyId::from_raw(30);
pub const NO_WRAP: PropertyId = PropertyId::from_raw(31);
pub const MIN_WIDTH: PropertyId = PropertyId::from_raw(32);
pub const SOURCE: PropertyId = PropertyId::from_raw(33);
pub const FIT: PropertyId = PropertyId::from_raw(34);
pub const SAMPLING: PropertyId = PropertyId::from_raw(35);
pub const SCROLL_Y: PropertyId = PropertyId::from_raw(36);
pub const SCROLLBAR_THUMB: PropertyId = PropertyId::from_raw(37);
pub const CHECKED: PropertyId = PropertyId::from_raw(38);
pub const PLACEHOLDER_COLOR: PropertyId = PropertyId::from_raw(39);
pub const MIN_HEIGHT: PropertyId = PropertyId::from_raw(40);
pub const ROTATION: PropertyId = PropertyId::from_raw(41);
pub const SELECTION_COLOR: PropertyId = PropertyId::from_raw(42);
pub const CARET_COLOR: PropertyId = PropertyId::from_raw(43);
pub const OPACITY: PropertyId = PropertyId::from_raw(44);
pub const ROW_HEIGHT: PropertyId = PropertyId::from_raw(45);
pub const VIEWPORT_HEIGHT: PropertyId = PropertyId::from_raw(46);
pub const SCROLL_OFFSET: PropertyId = PropertyId::from_raw(47);
pub const OVERSCAN: PropertyId = PropertyId::from_raw(48);
pub const ITEM_COUNT: PropertyId = PropertyId::from_raw(49);
pub const WINDOW_START: PropertyId = PropertyId::from_raw(50);
pub const EDGE_SHADOW_WIDTH: PropertyId = PropertyId::from_raw(51);
pub const EDGE_SHADOW_INTENSITY: PropertyId = PropertyId::from_raw(52);
pub const EDGE_SHADOW_COLOR: PropertyId = PropertyId::from_raw(53);
pub const SELECTION_FILL: PropertyId = PropertyId::from_raw(54);
pub const SELECTION_RADIUS: PropertyId = PropertyId::from_raw(55);
pub const CARET_FILL: PropertyId = PropertyId::from_raw(56);
pub const CARET_WIDTH: PropertyId = PropertyId::from_raw(57);
pub const CARET_HEIGHT: PropertyId = PropertyId::from_raw(58);
pub const CARET_RADIUS: PropertyId = PropertyId::from_raw(59);
pub const CARET_COUNT: PropertyId = PropertyId::from_raw(60);
pub const CARET_SPACING: PropertyId = PropertyId::from_raw(61);
pub const CARET_OFFSET_Y: PropertyId = PropertyId::from_raw(62);
pub const CARET_BLINK: PropertyId = PropertyId::from_raw(63);
pub const SEARCH_INPUT: PropertyId = PropertyId::from_raw(64);

pub const CLICK: EventId = EventId::from_raw(1);
pub const FOCUS: EventId = EventId::from_raw(2);
pub const BLUR: EventId = EventId::from_raw(3);
pub const INPUT_CHANGED: EventId = EventId::from_raw(4);
pub const SUBMIT: EventId = EventId::from_raw(5);
pub const DISMISS: EventId = EventId::from_raw(6);
pub const SCROLL: EventId = EventId::from_raw(7);

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
        .property(common_property(CommonProperty::Rotation))
        .property(common_property(CommonProperty::Opacity))
        .property(common_property(CommonProperty::Background));
    let text = text
        .property(common_property(CommonProperty::SelectionFill))
        .property(common_property(CommonProperty::SelectionColor))
        .property(common_property(CommonProperty::SelectionRadius));
    let text = text
        .property(PropertySchema::new(
            TEXT_COLOR,
            "color",
            ValueType::Color,
            "Text foreground color.",
        ))
        .property(PropertySchema::new(
            TEXT_WEIGHT,
            "weight",
            ValueType::Int,
            "Font weight.",
        ))
        .property(PropertySchema::new(
            TEXT_SIZE,
            "font_size",
            ValueType::Float,
            "Font size in logical pixels.",
        ))
        .property(PropertySchema::new(
            NO_WRAP,
            "no_wrap",
            ValueType::Bool,
            "Keep text on one line.",
        ));
    registry.register(text, |input: &NativeElementInput| {
        let content = optional_string(input, TEXT_VALUE)
            .or_else(|| optional_string(input, CONTENT))
            .ok_or_else(|| SchemaError::Adapter("Text requires `text` or `content`".into()))?;
        let mut style = TextStyle::default();
        if optional_bool(input, NO_WRAP) == Some(true) {
            style.wrap = TextWrap::None;
        }
        if let Some(SchemaValue::Color(color)) = input.get(TEXT_COLOR) {
            style.color = *color;
        }
        if let Some(SchemaValue::Int(weight)) = input.get(TEXT_WEIGHT) {
            style.weight = (*weight).clamp(1, 1000) as u16;
        }
        if let Some(SchemaValue::Float(size)) = input.get(TEXT_SIZE) {
            style.font_size = *size;
            style.line_height = *size * 1.25;
        }
        Ok(apply_common(
            Element::text(content.clone()).text_style(style),
            input,
        ))
    })?;
    register_pressable(&mut registry)?;
    text_editor::register(&mut registry)?;
    popover::register(&mut registry)?;
    media::register(&mut registry)?;
    switch::register(&mut registry)?;
    virtual_list::register(&mut registry)?;
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
        .property(common_property(CommonProperty::Rotation))
        .property(common_property(CommonProperty::Opacity))
        .property(common_property(CommonProperty::MinWidth))
        .property(common_property(CommonProperty::MinHeight))
        .property(common_property(CommonProperty::Background))
        .property(common_property(CommonProperty::SelectionFill))
        .property(common_property(CommonProperty::SelectionColor))
        .property(common_property(CommonProperty::SelectionRadius))
        .property(common_property(CommonProperty::Gap))
        .property(common_property(CommonProperty::Padding))
        .property(PropertySchema::new(
            SCROLL_Y,
            "scroll_y",
            ValueType::Bool,
            "Enable vertical scrolling and a visible scrollbar.",
        ))
        .property(PropertySchema::new(
            SCROLLBAR_THUMB,
            "scrollbar_thumb",
            ValueType::Color,
            "Color of the vertical scrollbar thumb.",
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
            BORDER_COLOR,
            "border_color",
            ValueType::Color,
            "Outline color.",
        ))
        .property(PropertySchema::new(
            RADIUS,
            "radius",
            ValueType::Float,
            "Corner radius in logical pixels.",
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
    registry.register(schema, move |input: &NativeElementInput| {
        let children = input.children(CHILDREN).to_vec();
        let mut element = apply_container(apply_common(constructor(children), input), input);
        if optional_bool(input, WRAP) == Some(true) {
            element = element.flex_wrap(FlexWrap::Wrap);
        }
        if let Some(SchemaValue::Float(grow)) = input.get(GROW) {
            element = element.grow(*grow);
        }
        if let Some(SchemaValue::Color(color)) = input.get(BORDER_COLOR) {
            element = element.border(Border::all(1.0, *color));
        }
        if let Some(SchemaValue::Float(radius)) = input.get(RADIUS) {
            element = element.radius(CornerRadii::all(*radius));
        }
        if optional_bool(input, SCROLL_Y) == Some(true) {
            let thumb = match input.get(SCROLLBAR_THUMB) {
                Some(SchemaValue::Color(color)) => *color,
                _ => argui_core::Color::srgba(0.45, 0.50, 0.57, 0.75),
            };
            let scrollbar = argui_ui::ScrollbarStyle::new(
                argui_ui::ScrollbarPartStyle::new(argui_paint::QuadStyle::default()),
                argui_ui::ScrollbarPartStyle::new(argui_paint::QuadStyle::solid(thumb)),
            )
            .width(8.0)
            .visibility(argui_ui::ScrollbarVisibility::Always);
            element = element
                .overflow(argui_ui::Axes {
                    x: argui_ui::Overflow::Hidden,
                    y: argui_ui::Overflow::Auto,
                })
                .scroll_config(argui_ui::ScrollConfig::default().scrollbar(scrollbar));
        }
        Ok(apply_events(element, input))
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
        .property(common_property(CommonProperty::Rotation))
        .property(common_property(CommonProperty::Opacity))
        .property(common_property(CommonProperty::MinWidth))
        .property(common_property(CommonProperty::Background))
        .property(common_property(CommonProperty::Gap))
        .property(common_property(CommonProperty::Padding))
        .property(PropertySchema::new(
            BUSY,
            "busy",
            ValueType::Bool,
            "Reject activation while work is in progress.",
        ))
        .property(PropertySchema::new(
            BORDER_COLOR,
            "border_color",
            ValueType::Color,
            "Resting border color.",
        ))
        .property(PropertySchema::new(
            HOVER_BACKGROUND,
            "hover_background",
            ValueType::Color,
            "Hover background color.",
        ))
        .property(PropertySchema::new(
            PRESSED_BACKGROUND,
            "pressed_background",
            ValueType::Color,
            "Pressed background color.",
        ))
        .property(PropertySchema::new(
            FOCUS_BORDER_COLOR,
            "focus_border_color",
            ValueType::Color,
            "Visible keyboard focus border color.",
        ))
        .property(PropertySchema::new(
            RADIUS,
            "radius",
            ValueType::Float,
            "Corner radius in logical pixels.",
        ))
        .property(PropertySchema::new(
            SELECT_TRIGGER,
            "select_trigger",
            ValueType::Bool,
            "Expose selection-trigger semantics.",
        ))
        .property(PropertySchema::new(
            EXPANDED,
            "expanded",
            ValueType::Bool,
            "Whether the selection popup is expanded.",
        ))
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
        let busy = optional_bool(input, BUSY).unwrap_or(false);
        let interactive = enabled && !busy;
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
                    .enabled(interactive)
                    .focus_policy(if interactive {
                        FocusPolicy::TabStop
                    } else {
                        FocusPolicy::None
                    })
                    .cursor(if !enabled {
                        CursorIcon::NotAllowed
                    } else if busy {
                        CursorIcon::Progress
                    } else {
                        CursorIcon::Pointer
                    })
                    .gestures(GestureSet::default().tap(argui_ui::TapGesture::default()))
                    .keyboard_activation(KeyboardActivation::EnterOrSpace),
            )
            .semantics(
                Semantics::new(if optional_bool(input, SELECT_TRIGGER) == Some(true) {
                    Role::ComboBox
                } else {
                    Role::Button
                })
                .label(label.clone())
                .state(SemanticState {
                    disabled: !interactive,
                    busy,
                    expanded: optional_bool(input, SELECT_TRIGGER)
                        .filter(|value| *value)
                        .map(|_| optional_bool(input, EXPANDED).unwrap_or(false)),
                    ..SemanticState::default()
                })
                .action(SemanticAction::Click)
                .action(SemanticAction::Focus),
            );
        let mut element = apply_container(apply_common(element, input), input)
            .align_items(AlignItems::CENTER)
            .justify_content(JustifyContent::CENTER)
            .state_scope(StateScopeId::new("button"))
            .active_state(StateName::new("busy"), busy);
        if let Some(SchemaValue::Color(color)) = input.get(BORDER_COLOR) {
            element = element.border(Border::all(1.0, *color));
        }
        if let Some(SchemaValue::Float(radius)) = input.get(RADIUS) {
            element = element.radius(CornerRadii::all(*radius));
        }
        for (id, state) in [
            (HOVER_BACKGROUND, VisualState::Hovered),
            (PRESSED_BACKGROUND, VisualState::Pressed),
        ] {
            if let Some(SchemaValue::Color(color)) = input.get(id) {
                element = element.when(
                    state,
                    StylePatch::new().set(property::BackgroundColor, *color),
                );
            }
        }
        if let Some(SchemaValue::Color(color)) = input.get(FOCUS_BORDER_COLOR) {
            element = element.when(
                VisualState::FocusVisible,
                StylePatch::new()
                    .set(property::BorderColor, *color)
                    .set(property::BorderWidths, [2.0; 4]),
            );
        }
        if !interactive || element.tooltip.as_deref() == Some("") {
            element.tooltip = None;
        }
        Ok(apply_events(element, input))
    })
}
