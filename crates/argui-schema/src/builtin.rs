//! Stable schemas for the deliberately small Rust-backed visual and behavior surface.

use argui_paint::CornerRadii;
use argui_text::{
    EllipsisPosition, FontStyle, LetterSpacing, TextAlign, TextOverflow, TextStyle, TextWrap,
    UnderlineStyle,
};
use argui_ui::{AlignItems, Element, EventType, FlexWrap, JustifyContent, WritingDirection};

use crate::{
    NativeElementInput, NativeSchema, NativeTypeId, PropertyId, PropertySchema, SchemaError,
    SchemaRegistry, SchemaValue, SlotArity, SlotSchema, ValueType,
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
mod options;
mod path;
mod popup_opening;
mod popup_window;
mod rectangle;
mod resize;
mod text_editor;
mod touch_area;
mod transition;
mod virtual_window;
use common::*;

mod ids;
pub use ids::*;

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
            TEXT_VALUE,
            "text",
            ValueType::String,
            "Displayed text; TSX also accepts text children.",
        ))
        .property(common_property(CommonProperty::Key))
        .property(common_property(CommonProperty::Tooltip))
        .property(common_property(CommonProperty::Width))
        .property(common_property(CommonProperty::Height))
        .property(common_property(CommonProperty::Position))
        .property(common_property(CommonProperty::Inset))
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
            "fontSize",
            ValueType::Float,
            "Font size in logical pixels.",
        ))
        .property(PropertySchema::new(
            NO_WRAP,
            "noWrap",
            ValueType::Bool,
            "Keep text on one line.",
        ))
        .property(PropertySchema::new(
            TEXT_LINE_HEIGHT,
            "lineHeight",
            ValueType::Float,
            "Line height in logical pixels.",
        ))
        .property(PropertySchema::new(
            TEXT_FONT_STYLE,
            "fontStyle",
            ValueType::String,
            "Font style: normal, italic or oblique.",
        ))
        .property(PropertySchema::new(
            TEXT_LETTER_SPACING,
            "letterSpacing",
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
            "textAlign",
            ValueType::String,
            "Text alignment: start, end, left, right, center or justify.",
        ))
        .property(PropertySchema::new(
            TEXT_LINE_CLAMP,
            "lineClamp",
            ValueType::Int,
            "Maximum number of visual lines; zero means unlimited.",
        ))
        .property(PropertySchema::new(
            TEXT_OVERFLOW,
            "textOverflow",
            ValueType::String,
            "CSS-like overflow behavior: clip or ellipsis; ellipsis_start, ellipsis_middle, and ellipsis_end are also supported.",
        ));
    registry.register(text, |input: &NativeElementInput| {
        let content = optional_string(input, TEXT_VALUE)
            .ok_or_else(|| SchemaError::Adapter("Text requires `text`".into()))?;
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
                "ellipsis" | "ellipsisEnd" => TextOverflow::Ellipsis(EllipsisPosition::End),
                "ellipsisStart" => TextOverflow::Ellipsis(EllipsisPosition::Start),
                "ellipsisMiddle" => TextOverflow::Ellipsis(EllipsisPosition::Middle),
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
    registry.set_builtin_allowed_values(options::allowed_values)?;
    Ok(registry)
}
