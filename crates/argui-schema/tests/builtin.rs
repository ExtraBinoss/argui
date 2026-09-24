#[path = "builtin/flickable.rs"]
mod flickable;
#[path = "builtin/focus_scope.rs"]
mod focus_scope;
#[path = "builtin/gpu_canvas.rs"]
mod gpu_canvas;
#[path = "builtin/key_binding.rs"]
mod key_binding;
#[path = "builtin/layout.rs"]
mod layout;
#[path = "builtin/loop_motion.rs"]
mod loop_motion;
#[cfg(feature = "media")]
#[path = "builtin/media.rs"]
mod media;
#[path = "builtin/path.rs"]
mod path;
#[path = "builtin/popup_window.rs"]
mod popup_window;
#[path = "builtin/rectangle.rs"]
mod rectangle;
#[path = "builtin/text_editor.rs"]
mod text_editor;
#[path = "builtin/touch_area.rs"]
mod touch_area;
#[path = "builtin/transition.rs"]
mod transition;
#[path = "builtin/virtual_window.rs"]
mod virtual_window;

#[test]
fn builtin_catalogue_preserves_stable_names_and_public_members() {
    let registry = builtin::registry().unwrap();
    for (id, name) in [
        (builtin::CONTAINER, "Container"),
        (builtin::ROW, "Row"),
        (builtin::COLUMN, "Column"),
        (builtin::GRID, "Grid"),
        (builtin::TEXT, "Text"),
        (builtin::TEXT_INPUT, "TextInput"),
        (builtin::GPU_CANVAS, "GpuCanvas"),
        (builtin::RECTANGLE, "Rectangle"),
        (builtin::TOUCH_AREA, "TouchArea"),
        (builtin::FOCUS_SCOPE, "FocusScope"),
        (builtin::PATH, "Path"),
        (builtin::FLICKABLE, "Flickable"),
        (builtin::KEY_BINDING, "KeyBinding"),
        (builtin::POPUP_WINDOW, "PopupWindow"),
        (builtin::VIRTUAL_WINDOW, "VirtualWindow"),
    ] {
        let schema = registry.schema(id).unwrap();
        assert_eq!(schema.name.as_str(), name);
        assert_eq!(registry.schema_named(name), Some(schema));
        assert!(!schema.documentation.is_empty());
        assert!(
            schema
                .properties
                .iter()
                .any(|property| property.id == builtin::OPACITY)
        );
        assert!(
            schema
                .properties
                .iter()
                .any(|property| property.id == builtin::VISIBLE)
        );
        if id != builtin::KEY_BINDING {
            assert!(
                schema
                    .properties
                    .iter()
                    .any(|property| property.id == builtin::BACKDROP_FILTER),
                "{name} must support backdrop_filter"
            );
        }
    }
    #[cfg(feature = "media")]
    for (id, name) in [(builtin::IMAGE, "Image"), (builtin::SVG, "Svg")] {
        assert_eq!(registry.schema(id).unwrap().name.as_str(), name);
    }
    assert_eq!(
        registry.schemas().count(),
        if cfg!(feature = "media") { 17 } else { 15 }
    );
    for name in [
        "Pressable",
        "PopoverPanel",
        "SwitchControl",
        "TextEditor",
        "VList",
    ] {
        assert!(
            registry.schema_named(name).is_none(),
            "{name} is still native"
        );
    }
}
use argui_core::Color;
use argui_paint::Filter;
use argui_schema::{NativeElementInput, NativeSlotValue, SchemaValue, builtin};
use argui_text::{
    EllipsisPosition, FontStyle, LetterSpacing, TextAlign, TextOverflow, TextWrap, UnderlineStyle,
};
use argui_ui::{
    AlignItems, Display, Element, ElementKind, FlexWrap, JustifyContent, Overflow, length,
};

#[test]
fn visibility_is_shared_by_visual_primitives() {
    let registry = builtin::registry().unwrap();
    let hidden = registry
        .construct(
            builtin::RECTANGLE,
            &NativeElementInput::new().property(builtin::VISIBLE, SchemaValue::Bool(false)),
        )
        .unwrap();
    assert_eq!(hidden.style.display, Display::None);
}

#[test]
fn ordered_backdrop_filters_apply_to_text_and_layout_surfaces() {
    let registry = builtin::registry().unwrap();
    for id in [builtin::TEXT, builtin::CONTAINER] {
        let input = NativeElementInput::new().property(
            builtin::BACKDROP_FILTER,
            SchemaValue::String("blur(4px) brightness(60%)".into()),
        );
        let input = if id == builtin::TEXT {
            input.property(builtin::CONTENT, SchemaValue::String("Glass".into()))
        } else {
            input
        };
        let element = registry.construct(id, &input).unwrap();
        assert_eq!(
            element.layer.as_ref().unwrap().backdrop_filters,
            vec![Filter::Blur(4.0), Filter::Brightness(0.6)]
        );
    }
}

#[test]
fn text_alias_takes_precedence_and_clamps_font_weight() {
    let registry = builtin::registry().unwrap();
    let text = registry
        .construct(
            builtin::TEXT,
            &NativeElementInput::new()
                .property(builtin::CONTENT, SchemaValue::String("old".into()))
                .property(builtin::TEXT_VALUE, SchemaValue::String("new".into()))
                .property(builtin::NO_WRAP, SchemaValue::Bool(true))
                .property(builtin::TEXT_WEIGHT, SchemaValue::Int(2000))
                .property(builtin::TEXT_SIZE, SchemaValue::Float(20.0))
                .property(builtin::TEXT_COLOR, SchemaValue::Color(Color::BLACK)),
        )
        .unwrap();
    assert!(
        matches!(text.kind, ElementKind::Text { ref content, ref style } if content.as_str() == "new" && style.wrap == TextWrap::None && style.weight == 1000 && style.font_size == 20.0 && style.line_height == 25.0 && style.color == Color::BLACK)
    );

    let low_weight = registry
        .construct(
            builtin::TEXT,
            &NativeElementInput::new()
                .property(builtin::CONTENT, SchemaValue::String("low".into()))
                .property(builtin::TEXT_WEIGHT, SchemaValue::Int(-3)),
        )
        .unwrap();
    assert!(matches!(&low_weight.kind, ElementKind::Text { style, .. } if style.weight == 1));
}

#[test]
fn optional_text_typography_reaches_the_text_style() {
    let text = builtin::registry()
        .unwrap()
        .construct(
            builtin::TEXT,
            &NativeElementInput::new()
                .property(builtin::TEXT_VALUE, SchemaValue::String("styled".into()))
                .property(builtin::TEXT_LINE_HEIGHT, SchemaValue::Float(28.0))
                .property(
                    builtin::TEXT_FONT_STYLE,
                    SchemaValue::String("italic".into()),
                )
                .property(builtin::TEXT_LETTER_SPACING, SchemaValue::Float(1.5))
                .property(
                    builtin::TEXT_UNDERLINE,
                    SchemaValue::String("double".into()),
                )
                .property(builtin::TEXT_STRIKETHROUGH, SchemaValue::Bool(true))
                .property(builtin::TEXT_ALIGN, SchemaValue::String("center".into()))
                .property(builtin::TEXT_LINE_CLAMP, SchemaValue::Int(2))
                .property(
                    builtin::TEXT_OVERFLOW,
                    SchemaValue::String("ellipsis_end".into()),
                ),
        )
        .unwrap();
    let ElementKind::Text { style, .. } = &text.kind else {
        panic!("expected text")
    };
    assert_eq!(style.line_height, 28.0);
    assert_eq!(style.font_style, FontStyle::Italic);
    assert_eq!(style.letter_spacing, LetterSpacing::Px(1.5));
    assert_eq!(style.decoration.underline, UnderlineStyle::Double);
    assert!(style.decoration.strikethrough);
    assert_eq!(style.align, TextAlign::Center);
    assert_eq!(style.line_clamp.unwrap().get(), 2);
    assert_eq!(
        style.overflow,
        TextOverflow::Ellipsis(EllipsisPosition::End)
    );
}

#[test]
fn layout_primitives_apply_optional_spacing_shape_and_scroll_policy() {
    let registry = builtin::registry().unwrap();
    for id in [builtin::CONTAINER, builtin::ROW, builtin::COLUMN] {
        let element = registry
            .construct(
                id,
                &NativeElementInput::new()
                    .property(builtin::GAP, SchemaValue::Float(6.0))
                    .property(builtin::PADDING, SchemaValue::Float(4.0))
                    .property(builtin::WRAP, SchemaValue::Bool(true))
                    .property(builtin::GROW, SchemaValue::Float(2.0))
                    .property(builtin::BORDER_COLOR, SchemaValue::Color(Color::BLACK))
                    .property(builtin::RADIUS, SchemaValue::Float(5.0))
                    .property(builtin::ALIGN_ITEMS, SchemaValue::String("center".into()))
                    .property(
                        builtin::JUSTIFY_CONTENT,
                        SchemaValue::String("center".into()),
                    )
                    .property(builtin::SCROLL_Y, SchemaValue::Bool(true))
                    .slot(NativeSlotValue::new(
                        builtin::CHILDREN,
                        [Element::text("child")],
                    )),
            )
            .unwrap();
        assert_eq!(element.children.len(), 1);
        assert_eq!(element.style.gap.width, length(6.0));
        assert_eq!(element.style.padding.left, length(4.0));
        assert_eq!(element.style.flex_wrap, FlexWrap::Wrap);
        assert_eq!(element.style.flex_grow, 2.0);
        assert_eq!(element.style.align_items, Some(AlignItems::CENTER));
        assert_eq!(element.style.justify_content, Some(JustifyContent::CENTER));
        let border = element.paint.quad.border.unwrap();
        assert_eq!(border.widths.left, 1.0);
        assert_eq!(border.color, Color::BLACK);
        assert_eq!(element.style.overflow.y, Overflow::Auto);
    }
}
#[path = "builtin/accessibility.rs"]
mod accessibility;
