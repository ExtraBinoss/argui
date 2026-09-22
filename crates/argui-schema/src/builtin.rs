//! Stable schemas for the deliberately small Rust-backed visual and behavior surface.

use argui_paint::{Border, CornerRadii};
use argui_text::{
    EllipsisPosition, FontStyle, LetterSpacing, TextAlign, TextOverflow, TextStyle, TextWrap,
    UnderlineStyle,
};
use argui_ui::{AlignItems, Element, EventType, FlexWrap, JustifyContent};

use crate::{
    EventId, NativeElementInput, NativeSchema, NativeTypeId, PropertyId, PropertySchema,
    SchemaError, SchemaRegistry, SchemaValue, SlotArity, SlotId, SlotSchema, ValueType,
};

mod common;
mod flickable;
mod focus_scope;
mod key_binding;
mod media;
mod path;
mod popup_window;
mod rectangle;
mod text_editor;
mod touch_area;
mod virtual_window;
use common::*;

pub const CONTAINER: NativeTypeId = NativeTypeId::from_raw(1);
pub const ROW: NativeTypeId = NativeTypeId::from_raw(2);
pub const COLUMN: NativeTypeId = NativeTypeId::from_raw(3);
pub const TEXT: NativeTypeId = NativeTypeId::from_raw(4);
pub const IMAGE: NativeTypeId = NativeTypeId::from_raw(8);
pub const SVG: NativeTypeId = NativeTypeId::from_raw(9);
pub const RECTANGLE: NativeTypeId = NativeTypeId::from_raw(12);
pub const TOUCH_AREA: NativeTypeId = NativeTypeId::from_raw(13);
pub const FOCUS_SCOPE: NativeTypeId = NativeTypeId::from_raw(14);
pub const PATH: NativeTypeId = NativeTypeId::from_raw(15);
pub const TEXT_INPUT: NativeTypeId = NativeTypeId::from_raw(16);
pub const KEY_BINDING: NativeTypeId = NativeTypeId::from_raw(17);
pub const POPUP_WINDOW: NativeTypeId = NativeTypeId::from_raw(18);
pub const FLICKABLE: NativeTypeId = NativeTypeId::from_raw(19);
pub const VIRTUAL_WINDOW: NativeTypeId = NativeTypeId::from_raw(20);

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
pub const BORDER_WIDTH: PropertyId = PropertyId::from_raw(65);
pub const CLIP: PropertyId = PropertyId::from_raw(66);
pub const HAS_HOVER: PropertyId = PropertyId::from_raw(67);
pub const PRESSED: PropertyId = PropertyId::from_raw(68);
pub const MOUSE_X: PropertyId = PropertyId::from_raw(69);
pub const MOUSE_Y: PropertyId = PropertyId::from_raw(70);
pub const PRESSED_X: PropertyId = PropertyId::from_raw(71);
pub const PRESSED_Y: PropertyId = PropertyId::from_raw(72);
pub const MOUSE_CURSOR: PropertyId = PropertyId::from_raw(73);
pub const X: PropertyId = PropertyId::from_raw(74);
pub const Y: PropertyId = PropertyId::from_raw(75);
pub const VISIBLE: PropertyId = PropertyId::from_raw(76);
pub const FOCUS_ON_CLICK: PropertyId = PropertyId::from_raw(77);
pub const FOCUS_ON_TAB: PropertyId = PropertyId::from_raw(78);
pub const HAS_FOCUS: PropertyId = PropertyId::from_raw(79);
pub const FOCUS_VISIBLE: PropertyId = PropertyId::from_raw(80);
pub const FOCUS_CONTAINMENT: PropertyId = PropertyId::from_raw(81);
pub const RESTORE_FOCUS: PropertyId = PropertyId::from_raw(82);
pub const INITIAL_FOCUS: PropertyId = PropertyId::from_raw(83);
pub const KEYBOARD_ACTIVATION: PropertyId = PropertyId::from_raw(84);
pub const SEMANTIC_ROLE: PropertyId = PropertyId::from_raw(85);
pub const SEMANTIC_LABEL: PropertyId = PropertyId::from_raw(86);
pub const SEMANTIC_EXPANDABLE: PropertyId = PropertyId::from_raw(87);
pub const ALIGN_ITEMS: PropertyId = PropertyId::from_raw(88);
pub const JUSTIFY_CONTENT: PropertyId = PropertyId::from_raw(89);
pub const MULTILINE: PropertyId = PropertyId::from_raw(90);
pub const SHORTCUT: PropertyId = PropertyId::from_raw(91);
pub const PLACEMENT: PropertyId = PropertyId::from_raw(92);
pub const DISMISS_POLICY: PropertyId = PropertyId::from_raw(93);
pub const WINDOW_LAYER: PropertyId = PropertyId::from_raw(94);
pub const SCROLL_X: PropertyId = PropertyId::from_raw(95);
pub const OFFSET_X: PropertyId = PropertyId::from_raw(96);
pub const OFFSET_Y: PropertyId = PropertyId::from_raw(97);
pub const FLICK_VIEWPORT_WIDTH: PropertyId = PropertyId::from_raw(98);
pub const FLICK_VIEWPORT_HEIGHT: PropertyId = PropertyId::from_raw(99);
pub const CONTENT_WIDTH: PropertyId = PropertyId::from_raw(100);
pub const CONTENT_HEIGHT: PropertyId = PropertyId::from_raw(101);
pub const SHRINK: PropertyId = PropertyId::from_raw(102);
pub const VISIBLE_HEIGHT: PropertyId = PropertyId::from_raw(103);
pub const BACKDROP_FILTER: PropertyId = PropertyId::from_raw(104);
pub const TEXT_LINE_HEIGHT: PropertyId = PropertyId::from_raw(105);
pub const TEXT_FONT_STYLE: PropertyId = PropertyId::from_raw(106);
pub const TEXT_LETTER_SPACING: PropertyId = PropertyId::from_raw(107);
pub const TEXT_UNDERLINE: PropertyId = PropertyId::from_raw(108);
pub const TEXT_STRIKETHROUGH: PropertyId = PropertyId::from_raw(109);
pub const TEXT_ALIGN: PropertyId = PropertyId::from_raw(110);
pub const TEXT_LINE_CLAMP: PropertyId = PropertyId::from_raw(111);
pub const TEXT_OVERFLOW: PropertyId = PropertyId::from_raw(112);
pub const SHADOW_BLUR: PropertyId = PropertyId::from_raw(113);
pub const SHADOW_OFFSET_Y: PropertyId = PropertyId::from_raw(114);
pub const SHADOW_COLOR: PropertyId = PropertyId::from_raw(115);
pub const MAX_DIGITS: PropertyId = PropertyId::from_raw(116);
pub const MOUSE_GLOBAL_X: PropertyId = PropertyId::from_raw(117);
pub const MOUSE_GLOBAL_Y: PropertyId = PropertyId::from_raw(118);

pub const CLICK: EventId = EventId::from_raw(1);
pub const FOCUS: EventId = EventId::from_raw(2);
pub const BLUR: EventId = EventId::from_raw(3);
pub const INPUT_CHANGED: EventId = EventId::from_raw(4);
pub const SUBMIT: EventId = EventId::from_raw(5);
pub const DISMISS: EventId = EventId::from_raw(6);
pub const SCROLL: EventId = EventId::from_raw(7);
pub const POINTER_ENTER: EventId = EventId::from_raw(8);
pub const POINTER_LEAVE: EventId = EventId::from_raw(9);
pub const POINTER_DOWN: EventId = EventId::from_raw(10);
pub const POINTER_UP: EventId = EventId::from_raw(11);
pub const POINTER_MOVE: EventId = EventId::from_raw(12);
pub const POINTER_CANCEL: EventId = EventId::from_raw(13);
pub const WHEEL: EventId = EventId::from_raw(14);
pub const MOVED: EventId = EventId::from_raw(15);
pub const DOUBLE_CLICK: EventId = EventId::from_raw(16);
pub const KEY_INPUT: EventId = EventId::from_raw(17);
pub const CAPTURE_KEY_INPUT: EventId = EventId::from_raw(18);
pub const ACTIVATED: EventId = EventId::from_raw(19);
pub const CONTEXT_MENU: EventId = EventId::from_raw(20);
pub const DRAG_X: EventId = EventId::from_raw(21);
pub const DRAG_Y: EventId = EventId::from_raw(22);
pub const TEXT_EDIT: EventId = EventId::from_raw(23);

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
    rectangle::register(&mut registry)?;
    touch_area::register(&mut registry)?;
    focus_scope::register(&mut registry)?;
    path::register(&mut registry)?;
    flickable::register(&mut registry)?;
    key_binding::register(&mut registry)?;
    popup_window::register(&mut registry)?;
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
        .property(common_property(CommonProperty::X))
        .property(common_property(CommonProperty::Y))
        .property(common_property(CommonProperty::Rotation))
        .property(common_property(CommonProperty::Opacity))
        .property(common_property(CommonProperty::BackdropFilter))
        .property(common_property(CommonProperty::Visible))
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
        ))
        .property(PropertySchema::new(
            TEXT_LINE_HEIGHT,
            "line_height",
            ValueType::Float,
            "Line height in logical pixels.",
        ))
        .property(PropertySchema::new(
            TEXT_FONT_STYLE,
            "font_style",
            ValueType::String,
            "Font style: normal, italic or oblique.",
        ))
        .property(PropertySchema::new(
            TEXT_LETTER_SPACING,
            "letter_spacing",
            ValueType::Float,
            "Additional spacing between glyphs in logical pixels.",
        ))
        .property(PropertySchema::new(
            TEXT_UNDERLINE,
            "underline",
            ValueType::String,
            "Underline style: none, single or double.",
        ))
        .property(PropertySchema::new(
            TEXT_STRIKETHROUGH,
            "strikethrough",
            ValueType::Bool,
            "Strike a line through the text.",
        ))
        .property(PropertySchema::new(
            TEXT_ALIGN,
            "text_align",
            ValueType::String,
            "Text alignment: start, end, left, right, center or justify.",
        ))
        .property(PropertySchema::new(
            TEXT_LINE_CLAMP,
            "line_clamp",
            ValueType::Int,
            "Maximum number of visual lines; zero means unlimited.",
        ))
        .property(PropertySchema::new(
            TEXT_OVERFLOW,
            "text_overflow",
            ValueType::String,
            "Overflow behavior: clip or ellipsis_end.",
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
        if let Some(SchemaValue::Float(height)) = input.get(TEXT_LINE_HEIGHT) {
            style.line_height = (*height).max(0.0);
        }
        if let Some(SchemaValue::String(value)) = input.get(TEXT_FONT_STYLE) {
            style.font_style = match value.as_str() {
                "normal" => FontStyle::Normal,
                "italic" => FontStyle::Italic,
                "oblique" => FontStyle::Oblique,
                _ => {
                    return Err(SchemaError::Adapter(format!(
                        "unsupported font_style `{value}`"
                    )));
                }
            };
        }
        if let Some(SchemaValue::Float(spacing)) = input.get(TEXT_LETTER_SPACING) {
            style.letter_spacing = LetterSpacing::Px(*spacing);
        }
        if let Some(SchemaValue::String(value)) = input.get(TEXT_UNDERLINE) {
            style.decoration.underline = match value.as_str() {
                "none" => UnderlineStyle::None,
                "single" => UnderlineStyle::Single,
                "double" => UnderlineStyle::Double,
                _ => {
                    return Err(SchemaError::Adapter(format!(
                        "unsupported underline `{value}`"
                    )));
                }
            };
        }
        if let Some(SchemaValue::Bool(strikethrough)) = input.get(TEXT_STRIKETHROUGH) {
            style.decoration.strikethrough = *strikethrough;
        }
        if let Some(SchemaValue::String(value)) = input.get(TEXT_ALIGN) {
            style.align = match value.as_str() {
                "start" => TextAlign::Start,
                "end" => TextAlign::End,
                "left" => TextAlign::Left,
                "right" => TextAlign::Right,
                "center" => TextAlign::Center,
                "justify" => TextAlign::Justify,
                _ => {
                    return Err(SchemaError::Adapter(format!(
                        "unsupported text_align `{value}`"
                    )));
                }
            };
        }
        if let Some(SchemaValue::Int(lines)) = input.get(TEXT_LINE_CLAMP) {
            style.line_clamp = usize::try_from(*lines)
                .ok()
                .and_then(std::num::NonZeroUsize::new);
        }
        if let Some(SchemaValue::String(value)) = input.get(TEXT_OVERFLOW) {
            style.overflow = match value.as_str() {
                "clip" => TextOverflow::Clip,
                "ellipsis_end" => TextOverflow::Ellipsis(EllipsisPosition::End),
                _ => {
                    return Err(SchemaError::Adapter(format!(
                        "unsupported text_overflow `{value}`"
                    )));
                }
            };
        }
        apply_common(Element::text(content.clone()).text_style(style), input)
    })?;
    text_editor::register(&mut registry)?;
    media::register(&mut registry)?;
    virtual_window::register(&mut registry)?;
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
        .property(common_property(CommonProperty::X))
        .property(common_property(CommonProperty::Y))
        .property(common_property(CommonProperty::Rotation))
        .property(common_property(CommonProperty::Opacity))
        .property(common_property(CommonProperty::BackdropFilter))
        .property(common_property(CommonProperty::Visible))
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
            SHRINK,
            "shrink",
            ValueType::Float,
            "Flex shrink factor.",
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
        .property(PropertySchema::new(
            ALIGN_ITEMS,
            "align_items",
            ValueType::String,
            "Cross-axis alignment of children.",
        ))
        .property(PropertySchema::new(
            JUSTIFY_CONTENT,
            "justify_content",
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
    registry.register(schema, move |input: &NativeElementInput| {
        let children = input.children(CHILDREN).to_vec();
        let mut element = apply_container(apply_common(constructor(children), input)?, input);
        if optional_bool(input, WRAP) == Some(true) {
            element = element.flex_wrap(FlexWrap::Wrap);
        }
        if let Some(SchemaValue::Float(grow)) = input.get(GROW) {
            element = element.grow(*grow);
        }
        if let Some(SchemaValue::Float(shrink)) = input.get(SHRINK) {
            element = element.shrink(*shrink);
        }
        if let Some(SchemaValue::Color(color)) = input.get(BORDER_COLOR) {
            element = element.border(Border::all(1.0, *color));
        }
        if let Some(SchemaValue::Float(radius)) = input.get(RADIUS) {
            element = element.radius(CornerRadii::all(*radius));
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
                "space_between" => JustifyContent::SPACE_BETWEEN,
                "space_around" => JustifyContent::SPACE_AROUND,
                "space_evenly" => JustifyContent::SPACE_EVENLY,
                _ => {
                    return Err(SchemaError::Adapter(format!(
                        "{name} does not support justify_content `{value}`"
                    )));
                }
            });
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
                .scroll_config(argui_ui::ScrollConfig::default().scrollbar(scrollbar))
                .scrollbar_gutter(argui_ui::ScrollbarGutter::Stable);
        }
        Ok(apply_events(element, input))
    })
}
