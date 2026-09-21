#[path = "builtin/media.rs"]
mod media;
#[path = "builtin/switch.rs"]
mod switch;
#[path = "builtin/text_editor.rs"]
mod text_editor;
#[path = "builtin/virtual_list.rs"]
mod virtual_list;

#[test]
fn builtin_catalogue_preserves_stable_names_and_public_members() {
    let registry = builtin::registry().unwrap();
    for (id, name) in [
        (builtin::CONTAINER, "Container"),
        (builtin::ROW, "Row"),
        (builtin::COLUMN, "Column"),
        (builtin::TEXT, "Text"),
        (builtin::PRESSABLE, "Pressable"),
        (builtin::TEXT_EDITOR, "TextEditor"),
        (builtin::POPOVER_PANEL, "PopoverPanel"),
        (builtin::IMAGE, "Image"),
        (builtin::SVG, "Svg"),
        (builtin::SWITCH, "SwitchControl"),
        (builtin::VIRTUAL_LIST, "VList"),
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
    }
    assert_eq!(registry.schemas().count(), 11);
}
use argui_core::Color;
use argui_schema::{NativeElementInput, NativeSlotValue, SchemaValue, builtin};
use argui_text::TextWrap;
use argui_ui::{Element, ElementKind, FlexWrap, Overflow, Role, length};

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
        let border = element.paint.quad.border.unwrap();
        assert_eq!(border.widths.left, 1.0);
        assert_eq!(border.color, Color::BLACK);
        assert_eq!(element.style.overflow.y, Overflow::Auto);
    }
}

#[test]
fn enabled_pressable_keeps_tooltip_and_select_trigger_semantics() {
    let registry = builtin::registry().unwrap();
    let button = registry
        .construct(
            builtin::PRESSABLE,
            &NativeElementInput::new()
                .property(builtin::KEY, SchemaValue::String("menu".into()))
                .property(builtin::LABEL, SchemaValue::String("Menu".into()))
                .property(builtin::TOOLTIP, SchemaValue::String("Open menu".into()))
                .property(builtin::SELECT_TRIGGER, SchemaValue::Bool(true))
                .property(builtin::BORDER_COLOR, SchemaValue::Color(Color::BLACK))
                .property(builtin::HOVER_BACKGROUND, SchemaValue::Color(Color::BLACK))
                .property(
                    builtin::PRESSED_BACKGROUND,
                    SchemaValue::Color(Color::BLACK),
                )
                .property(
                    builtin::FOCUS_BORDER_COLOR,
                    SchemaValue::Color(Color::BLACK),
                )
                .property(builtin::RADIUS, SchemaValue::Float(6.0))
                .slot(NativeSlotValue::new(
                    builtin::CHILDREN,
                    [Element::text("Menu")],
                )),
        )
        .unwrap();
    assert_eq!(button.tooltip.as_deref(), Some("Open menu"));
    assert_eq!(button.semantics.as_ref().unwrap().role, Role::ComboBox);
    assert_eq!(
        button.semantics.as_ref().unwrap().state.expanded,
        Some(false)
    );
    assert!(button.interaction.as_ref().unwrap().enabled);
    assert!(button.children[0].semantic_hidden);
    let border = button.paint.quad.border.unwrap();
    assert_eq!(border.widths.left, 1.0);
    assert_eq!(border.color, Color::BLACK);

    let empty_tooltip = registry
        .construct(
            builtin::PRESSABLE,
            &NativeElementInput::new()
                .property(builtin::KEY, SchemaValue::String("plain".into()))
                .property(builtin::LABEL, SchemaValue::String("Plain".into()))
                .property(builtin::TOOLTIP, SchemaValue::String(String::new())),
        )
        .unwrap();
    assert_eq!(empty_tooltip.tooltip, None);
    assert_eq!(empty_tooltip.semantics.as_ref().unwrap().role, Role::Button);
    assert_eq!(
        empty_tooltip.semantics.as_ref().unwrap().state.expanded,
        None
    );
}
