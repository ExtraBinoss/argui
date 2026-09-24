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

pub(crate) mod accessibility;
mod accessibility_contract;
mod common;
mod container;
mod flickable;
mod focus_scope;
mod focus_scope_parse;
mod gpu_canvas;
mod key_binding;
mod layout;
mod loop_motion;
#[cfg(feature = "media")]
mod media;
mod path;
mod popup_window;
mod rectangle;
mod text_editor;
mod touch_area;
mod transition;
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
pub const GRID: NativeTypeId = NativeTypeId::from_raw(21);
pub const GPU_CANVAS: NativeTypeId = NativeTypeId::from_raw(22);

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
pub const DESKTOP_BACKDROP_TINT: PropertyId = PropertyId::from_raw(236);
pub const DESKTOP_BACKDROP_FALLBACK: PropertyId = PropertyId::from_raw(237);
pub const TEXT_PRIVACY: PropertyId = PropertyId::from_raw(238);
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
pub const POSITION: PropertyId = PropertyId::from_raw(163);
pub const INSET_LEFT: PropertyId = PropertyId::from_raw(164);
pub const INSET_RIGHT: PropertyId = PropertyId::from_raw(165);
pub const INSET_TOP: PropertyId = PropertyId::from_raw(166);
pub const INSET_BOTTOM: PropertyId = PropertyId::from_raw(167);
pub const Z_INDEX: PropertyId = PropertyId::from_raw(168);
pub const VARIABLE_HEIGHT: PropertyId = PropertyId::from_raw(169);
pub const MEASURED_WIDTH: PropertyId = PropertyId::from_raw(170);
pub const MEASURED_HEIGHT: PropertyId = PropertyId::from_raw(171);
pub const SEMANTIC_DESCRIPTION: PropertyId = PropertyId::from_raw(172);
pub const SEMANTIC_VALUE: PropertyId = PropertyId::from_raw(173);
pub const SEMANTIC_SELECTED: PropertyId = PropertyId::from_raw(174);
pub const SEMANTIC_CURRENT: PropertyId = PropertyId::from_raw(175);
pub const SEMANTIC_CONTROLS: PropertyId = PropertyId::from_raw(176);
pub const SEMANTIC_ACTIVE_DESCENDANT: PropertyId = PropertyId::from_raw(177);
pub const SEMANTIC_LABELLED_BY: PropertyId = PropertyId::from_raw(178);
pub const SEMANTIC_DESCRIBED_BY: PropertyId = PropertyId::from_raw(179);
pub const SEMANTIC_INVALID: PropertyId = PropertyId::from_raw(180);
pub const SEMANTIC_LIVE: PropertyId = PropertyId::from_raw(181);
pub const TRANSITION_MS: PropertyId = PropertyId::from_raw(182);
pub const TRANSITION_SPRING: PropertyId = PropertyId::from_raw(183);
pub const ROTATION_LOOP_MS: PropertyId = PropertyId::from_raw(184);
pub const LOOP_MS: PropertyId = PropertyId::from_raw(185);
pub const LOOP_PLAYING: PropertyId = PropertyId::from_raw(186);
pub const LOOP_TRANSLATE_X: PropertyId = PropertyId::from_raw(187);
pub const LOOP_TRANSLATE_Y: PropertyId = PropertyId::from_raw(188);
pub const LOOP_SCALE: PropertyId = PropertyId::from_raw(189);
pub const LOOP_OPACITY: PropertyId = PropertyId::from_raw(190);
pub const LOOP_BACKGROUND: PropertyId = PropertyId::from_raw(191);
pub const LOOP_HOLD: PropertyId = PropertyId::from_raw(192);
pub const LOOP_WIDTH: PropertyId = PropertyId::from_raw(193);
pub const LOOP_RADIUS: PropertyId = PropertyId::from_raw(194);
pub const LOOP_GAP: PropertyId = PropertyId::from_raw(195);
pub const VIRTUAL_HORIZONTAL: PropertyId = PropertyId::from_raw(196);
pub const VIRTUAL_VIEWPORT_WIDTH: PropertyId = PropertyId::from_raw(197);
pub const VIRTUAL_SHADOW_COLOR: PropertyId = PropertyId::from_raw(198);
pub const VIRTUAL_SHADOW_INTENSITY: PropertyId = PropertyId::from_raw(199);
pub const VIRTUAL_SHADOW_WIDTH: PropertyId = PropertyId::from_raw(200);
pub const VIRTUAL_SHADOW_START: PropertyId = PropertyId::from_raw(201);
pub const VIRTUAL_SHADOW_END: PropertyId = PropertyId::from_raw(202);
pub const VIRTUAL_VISIBLE_WIDTH: PropertyId = PropertyId::from_raw(203);
pub const VIRTUAL_SCROLLBAR_VISIBLE: PropertyId = PropertyId::from_raw(204);
pub const VIRTUAL_SCROLLBAR_WIDTH: PropertyId = PropertyId::from_raw(205);
pub const VIRTUAL_DATA_VERSION: PropertyId = PropertyId::from_raw(206);
pub const SEMANTIC_FOCUSABLE: PropertyId = PropertyId::from_raw(207);
pub const SEMANTIC_PRESSED: PropertyId = PropertyId::from_raw(208);
pub const SEMANTIC_REQUIRED: PropertyId = PropertyId::from_raw(209);
pub const SEMANTIC_READ_ONLY: PropertyId = PropertyId::from_raw(210);
pub const SEMANTIC_MULTISELECTABLE: PropertyId = PropertyId::from_raw(211);
pub const SEMANTIC_NUMERIC_VALUE: PropertyId = PropertyId::from_raw(212);
pub const SEMANTIC_MINIMUM_VALUE: PropertyId = PropertyId::from_raw(213);
pub const SEMANTIC_MAXIMUM_VALUE: PropertyId = PropertyId::from_raw(214);
pub const SEMANTIC_VALUE_STEP: PropertyId = PropertyId::from_raw(215);
pub const SEMANTIC_CAN_INCREMENT: PropertyId = PropertyId::from_raw(216);
pub const SEMANTIC_CAN_DECREMENT: PropertyId = PropertyId::from_raw(217);
pub const SEMANTIC_CAN_SET_VALUE: PropertyId = PropertyId::from_raw(218);
pub const SEMANTIC_CAN_EXPAND: PropertyId = PropertyId::from_raw(219);
pub const SEMANTIC_CAN_COLLAPSE: PropertyId = PropertyId::from_raw(220);
pub const SEMANTIC_CAN_SCROLL_INTO_VIEW: PropertyId = PropertyId::from_raw(221);
pub const SEMANTIC_DISABLED: PropertyId = PropertyId::from_raw(222);
pub const SEMANTIC_CHECKED_STATE: PropertyId = PropertyId::from_raw(223);
pub const SEMANTIC_HIDDEN: PropertyId = PropertyId::from_raw(224);
pub const SEMANTIC_ORIENTATION: PropertyId = PropertyId::from_raw(225);
pub const SEMANTIC_LEVEL: PropertyId = PropertyId::from_raw(226);
pub const SEMANTIC_POSITION_IN_SET: PropertyId = PropertyId::from_raw(227);
pub const SEMANTIC_SET_SIZE: PropertyId = PropertyId::from_raw(228);
pub const SEMANTIC_MODAL: PropertyId = PropertyId::from_raw(229);
pub const SEMANTIC_HAS_POPUP: PropertyId = PropertyId::from_raw(230);
pub const SEMANTIC_SORT: PropertyId = PropertyId::from_raw(231);
pub const ALT: PropertyId = PropertyId::from_raw(232);
pub const CANVAS_ID: PropertyId = PropertyId::from_raw(233);
pub const CANVAS_REVISION: PropertyId = PropertyId::from_raw(234);
pub const RESOLUTION_SCALE: PropertyId = PropertyId::from_raw(235);
pub const MAX_DIGITS: PropertyId = PropertyId::from_raw(116);
pub const MOUSE_GLOBAL_X: PropertyId = PropertyId::from_raw(117);
pub const MOUSE_GLOBAL_Y: PropertyId = PropertyId::from_raw(118);
pub const GRID_ROWS: PropertyId = PropertyId::from_raw(119);
pub const GRID_COLUMNS: PropertyId = PropertyId::from_raw(120);
pub const GRID_ROW_START: PropertyId = PropertyId::from_raw(121);
pub const GRID_ROW_SPAN: PropertyId = PropertyId::from_raw(122);
pub const GRID_COLUMN_START: PropertyId = PropertyId::from_raw(123);
pub const GRID_COLUMN_SPAN: PropertyId = PropertyId::from_raw(124);
pub const MAX_WIDTH: PropertyId = PropertyId::from_raw(125);
pub const MAX_HEIGHT: PropertyId = PropertyId::from_raw(126);
pub const ASPECT_RATIO: PropertyId = PropertyId::from_raw(127);
pub const FLEX_BASIS: PropertyId = PropertyId::from_raw(128);
pub const ROW_GAP: PropertyId = PropertyId::from_raw(129);
pub const COLUMN_GAP: PropertyId = PropertyId::from_raw(130);
pub const PADDING_LEFT: PropertyId = PropertyId::from_raw(131);
pub const PADDING_RIGHT: PropertyId = PropertyId::from_raw(132);
pub const PADDING_TOP: PropertyId = PropertyId::from_raw(133);
pub const PADDING_BOTTOM: PropertyId = PropertyId::from_raw(134);
pub const MARGIN_LEFT: PropertyId = PropertyId::from_raw(135);
pub const MARGIN_RIGHT: PropertyId = PropertyId::from_raw(136);
pub const MARGIN_TOP: PropertyId = PropertyId::from_raw(137);
pub const MARGIN_BOTTOM: PropertyId = PropertyId::from_raw(138);
pub const SCALE_X: PropertyId = PropertyId::from_raw(139);
pub const SCALE_Y: PropertyId = PropertyId::from_raw(140);
pub const TRANSLATE_X: PropertyId = PropertyId::from_raw(141);
pub const TRANSLATE_Y: PropertyId = PropertyId::from_raw(142);
pub const ORIGIN_X: PropertyId = PropertyId::from_raw(143);
pub const ORIGIN_Y: PropertyId = PropertyId::from_raw(144);
pub const QUERY_SCOPE: PropertyId = PropertyId::from_raw(145);
pub const QUERY_MIN_WIDTH: PropertyId = PropertyId::from_raw(146);
pub const QUERY_COLUMNS: PropertyId = PropertyId::from_raw(147);
pub const ALIGN_SELF: PropertyId = PropertyId::from_raw(148);
pub const JUSTIFY_ITEMS: PropertyId = PropertyId::from_raw(149);
pub const JUSTIFY_SELF: PropertyId = PropertyId::from_raw(150);
pub const ALIGN_CONTENT: PropertyId = PropertyId::from_raw(151);
pub const GRID_AUTO_FLOW: PropertyId = PropertyId::from_raw(152);
pub const RADIUS_TOP_LEFT: PropertyId = PropertyId::from_raw(153);
pub const RADIUS_TOP_RIGHT: PropertyId = PropertyId::from_raw(154);
pub const RADIUS_BOTTOM_RIGHT: PropertyId = PropertyId::from_raw(155);
pub const RADIUS_BOTTOM_LEFT: PropertyId = PropertyId::from_raw(156);
pub const BORDER_LEFT: PropertyId = PropertyId::from_raw(157);
pub const BORDER_RIGHT: PropertyId = PropertyId::from_raw(158);
pub const BORDER_TOP: PropertyId = PropertyId::from_raw(159);
pub const BORDER_BOTTOM: PropertyId = PropertyId::from_raw(160);
pub const SHADOW_OFFSET_X: PropertyId = PropertyId::from_raw(161);
pub const SHADOW_SPREAD: PropertyId = PropertyId::from_raw(162);

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
pub const VIRTUAL_MEASURE: EventId = EventId::from_raw(24);
pub const VIRTUAL_WINDOW_CHANGE: EventId = EventId::from_raw(25);
pub const SEMANTIC_ACTION: EventId = EventId::from_raw(26);

/// Creates the canonical registry of built-in native visual primitives.
///
/// # Errors
///
/// Returns a schema error if the built-in definitions violate registry invariants.
pub fn registry() -> Result<SchemaRegistry, SchemaError> {
    let mut registry = SchemaRegistry::new();
    container::register(&mut registry, CONTAINER, "Container", Element::container)?;
    container::register(&mut registry, ROW, "Row", Element::row)?;
    container::register(&mut registry, COLUMN, "Column", Element::column)?;
    container::register(&mut registry, GRID, "Grid", Element::grid)?;
    rectangle::register(&mut registry)?;
    touch_area::register(&mut registry)?;
    focus_scope::register(&mut registry)?;
    path::register(&mut registry)?;
    gpu_canvas::register(&mut registry)?;
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
        .property(common_property(CommonProperty::DesktopBackdropTint))
        .property(common_property(CommonProperty::DesktopBackdropFallback))
        .property(common_property(CommonProperty::Visible))
        .property(common_property(CommonProperty::Background))
        .property(common_property(CommonProperty::Padding))
        .property(PropertySchema::new(
            RADIUS,
            "radius",
            ValueType::Float,
            "Corner radius for a text background in logical pixels.",
        ))
        .property(loop_motion::properties()[1].clone())
        .property(loop_motion::properties()[2].clone())
        .property(loop_motion::properties()[6].clone())
        .property(transition::properties()[0].clone())
        .property(transition::properties()[1].clone());
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
        let mut element = apply_container(
            apply_common(Element::text(content.clone()).text_style(style), input)?,
            input,
        );
        if let Some(SchemaValue::Float(radius)) = input.get(RADIUS) {
            element = element.radius(CornerRadii::all((*radius).max(0.0)));
        }
        loop_motion::apply(transition::apply(element, input, "Text")?, input)
    })?;
    text_editor::register(&mut registry)?;
    #[cfg(feature = "media")]
    media::register(&mut registry)?;
    virtual_window::register(&mut registry)?;
    registry.enable_builtin_accessibility()?;
    Ok(registry)
}
