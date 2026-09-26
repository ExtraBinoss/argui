use argui_core::Color;
use argui_paint::Fill;
use argui_schema::{NativeElementInput, SchemaError, SchemaValue, ValueType, builtin};
use argui_text::TextWrap;
use argui_ui::{
    CaretHeight, ElementKind, Overflow, Role, ScrollbarGutter, ScrollbarVisibility, TextPrivacy,
    UiTree,
};

/// Controlled text uses the delta event while the full-value event stays optional.
#[test]
fn text_input_schema_separates_edit_from_full_input() {
    let registry = builtin::registry().unwrap();
    let schema = registry.schema(builtin::TEXT_INPUT).unwrap();
    let value = schema
        .properties
        .iter()
        .find(|property| property.id == builtin::VALUE)
        .unwrap();
    assert_eq!(value.change_event, Some(builtin::TEXT_EDIT));
    let edit = schema
        .events
        .iter()
        .find(|event| event.id == builtin::TEXT_EDIT)
        .unwrap();
    assert_eq!(edit.name.as_str(), "edit");
    assert_eq!(edit.payload, None);
    assert_eq!(edit.event_type, argui_ui::EventType::TextEdit);
    assert!(schema.events.iter().any(
        |event| event.id == builtin::INPUT_CHANGED && event.payload == Some(ValueType::String)
    ));
}

/// Search editors expose the search role without changing their controlled value behavior.
#[test]
fn text_input_can_expose_search_semantics() {
    let editor = builtin::registry()
        .unwrap()
        .construct(
            builtin::TEXT_INPUT,
            &NativeElementInput::new()
                .property(builtin::ID, SchemaValue::String("search".into()))
                .property(builtin::SEARCH_INPUT, SchemaValue::Bool(true)),
        )
        .unwrap();
    assert_eq!(editor.semantics.as_ref().unwrap().role, Role::SearchInput);
    let ElementKind::TextEditor {
        text,
        placeholder_text,
        ..
    } = &editor.kind
    else {
        panic!("search input must use a text editor");
    };
    assert_eq!(text.wrap, TextWrap::None);
    assert_eq!(placeholder_text.wrap, TextWrap::None);
    assert_eq!(editor.style.overflow.x, Overflow::Hidden);
    assert_eq!(editor.style.overflow.y, Overflow::Hidden);
}

/// Applications can opt out of the default input-bound clipping from TSX.
#[test]
fn text_input_clip_property_controls_its_own_paint_boundary() {
    let editor = builtin::registry()
        .unwrap()
        .construct(
            builtin::TEXT_INPUT,
            &NativeElementInput::new()
                .property(builtin::ID, SchemaValue::String("search".into()))
                .property(builtin::CLIP, SchemaValue::Bool(false)),
        )
        .unwrap();
    assert_eq!(editor.style.overflow.x, Overflow::Visible);
    assert_eq!(editor.style.overflow.y, Overflow::Visible);
}

/// Password and revealed-password TSX values keep protected native semantics.
#[test]
fn text_input_privacy_masks_passwords_and_protects_revealed_text() {
    let registry = builtin::registry().unwrap();
    for (name, expected, privacy) in [
        ("password", "••••••", TextPrivacy::Password),
        ("revealedPassword", "secret", TextPrivacy::RevealedPassword),
    ] {
        let editor = registry
            .construct(
                builtin::TEXT_INPUT,
                &NativeElementInput::new()
                    .property(builtin::ID, SchemaValue::String("login".into()))
                    .property(builtin::VALUE, SchemaValue::String("secret".into()))
                    .property(builtin::TEXT_PRIVACY, SchemaValue::String(name.into())),
            )
            .unwrap();
        assert_eq!(editor.text_privacy, privacy);
        let semantics = editor.semantics.as_ref().unwrap();
        assert!(semantics.state.protected);
        assert!(semantics.value.is_none());
        let tree = UiTree::new(editor);
        assert_eq!(
            tree.text_input_display(tree.node_ids()[0]).as_deref(),
            Some(expected)
        );
    }
    let error = registry
        .construct(
            builtin::TEXT_INPUT,
            &NativeElementInput::new()
                .property(builtin::ID, SchemaValue::String("login".into()))
                .property(builtin::TEXT_PRIVACY, SchemaValue::String("unknown".into())),
        )
        .unwrap_err();
    assert!(matches!(error, SchemaError::InvalidPropertyValue { .. }));
}

/// TextInput exposes multiline editing without supplying a surrounding frame.
#[test]
fn text_input_multiline_uses_the_shared_editing_engine() {
    let editor = builtin::registry()
        .unwrap()
        .construct(
            builtin::TEXT_INPUT,
            &NativeElementInput::new()
                .property(builtin::ID, SchemaValue::String("notes".into()))
                .property(builtin::VALUE, SchemaValue::String("first\nsecond".into()))
                .property(builtin::MULTILINE, SchemaValue::Bool(true))
                .property(builtin::LABEL, SchemaValue::String("Notes".into())),
        )
        .unwrap();
    let ElementKind::TextEditor {
        multiline,
        text,
        placeholder_text,
        ..
    } = &editor.kind
    else {
        panic!("TextInput must use the native editing engine");
    };
    assert!(*multiline);
    assert_eq!(text.wrap, TextWrap::WordOrGlyph);
    assert_eq!(placeholder_text.wrap, TextWrap::WordOrGlyph);
    assert_eq!(editor.style.overflow.y, Overflow::Auto);
    assert_eq!(editor.style.overflow.x, Overflow::Hidden);
    assert_eq!(editor.style.scrollbar_gutter, ScrollbarGutter::Stable);
    assert_eq!(
        editor
            .scroll
            .as_ref()
            .unwrap()
            .scrollbar
            .as_ref()
            .unwrap()
            .visibility,
        ScrollbarVisibility::Always
    );
    assert_eq!(editor.semantics.as_ref().unwrap().role, Role::TextInput);
}

/// Ordinary Text and container scopes share the same selected-text style API as editors.
#[test]
fn ordinary_text_and_container_accept_selection_styles() {
    let registry = builtin::registry().unwrap();
    let color = Color::srgba(0.65, 0.25, 0.75, 0.4);
    let text = registry
        .construct(
            builtin::TEXT,
            &NativeElementInput::new()
                .property(builtin::TEXT_VALUE, SchemaValue::String("select me".into()))
                .property(builtin::SELECTION_COLOR, SchemaValue::Color(color)),
        )
        .unwrap();
    let highlight = text.selection_highlight.as_ref().unwrap();
    assert_eq!(highlight.background, Fill::Solid(color));
    assert_eq!(highlight.radii.as_array(), [3.0; 4]);

    let fill = Fill::Solid(Color::WHITE);
    let container = registry
        .construct(
            builtin::CONTAINER,
            &NativeElementInput::new()
                .property(builtin::SELECTION_FILL, SchemaValue::Brush(fill.clone()))
                .property(builtin::SELECTION_COLOR, SchemaValue::Color(color))
                .property(builtin::SELECTION_RADIUS, SchemaValue::Float(6.0)),
        )
        .unwrap();
    let highlight = container.selection_highlight.as_ref().unwrap();
    assert_eq!(highlight.background, fill);
    assert_eq!(highlight.radii.as_array(), [6.0; 4]);
}

/// The same generic fill and geometry inputs compose text selection and carets.
#[test]
fn text_input_composes_rounded_selection_and_repeated_caret_primitives() {
    let registry = builtin::registry().unwrap();
    let colors = [Color::WHITE, Color::BLACK, Color::WHITE];
    let fill = Fill::conic_gradient(
        &colors,
        &[0.0, 0.5, 1.0],
        argui_core::Point::new(0.5, 0.5),
        45.0,
        "oklab",
    )
    .unwrap();
    let editor = registry
        .construct(
            builtin::TEXT_INPUT,
            &NativeElementInput::new()
                .property(builtin::ID, SchemaValue::String("caret".into()))
                .property(builtin::SELECTION_FILL, SchemaValue::Brush(fill.clone()))
                .property(builtin::SELECTION_RADIUS, SchemaValue::Float(7.0))
                .property(builtin::CARET_FILL, SchemaValue::Brush(fill.clone()))
                .property(builtin::CARET_WIDTH, SchemaValue::Float(4.0))
                .property(builtin::CARET_HEIGHT, SchemaValue::Float(4.0))
                .property(builtin::CARET_RADIUS, SchemaValue::Float(2.0))
                .property(builtin::CARET_COUNT, SchemaValue::Int(3))
                .property(builtin::CARET_SPACING, SchemaValue::Float(6.0))
                .property(builtin::CARET_BLINK, SchemaValue::Bool(false)),
        )
        .unwrap();
    let highlight = editor.selection_highlight.as_ref().unwrap();
    assert_eq!(highlight.background, fill);
    assert_eq!(highlight.radii.as_array(), [7.0; 4]);
    let ElementKind::TextEditor { caret, .. } = &editor.kind else {
        panic!("TextInput should use the editor engine");
    };
    assert_eq!(caret.visual.primitives.len(), 3);
    assert!(
        caret
            .visual
            .primitives
            .iter()
            .all(|part| part.height == CaretHeight::Pixels(4.0)
                && part.paint.radii.as_array() == [2.0; 4])
    );
    assert_eq!(caret.visual.primitives[2].offset.x, 12.0);
    assert!(!caret.is_animated());
}

/// The native TextInput adapter applies selection and caret colors without
/// replacing its retained caret animation.
#[test]
fn text_input_applies_themeable_selection_and_caret_colors() {
    let registry = builtin::registry().unwrap();
    let selection_color = Color::srgba(0.4, 0.3, 0.8, 0.32);
    let caret_color = Color::srgba(0.7, 0.2, 0.5, 1.0);
    let editor = registry
        .construct(
            builtin::TEXT_INPUT,
            &NativeElementInput::new()
                .property(builtin::ID, SchemaValue::String("query".into()))
                .property(builtin::VALUE, SchemaValue::String("hello".into()))
                .property(
                    builtin::SELECTION_COLOR,
                    SchemaValue::Color(selection_color),
                )
                .property(builtin::CARET_COLOR, SchemaValue::Color(caret_color)),
        )
        .unwrap();
    let ElementKind::TextEditor {
        selection, caret, ..
    } = &editor.kind
    else {
        panic!("TextInput should use the editor engine");
    };
    assert_eq!(*selection, selection_color);
    assert_eq!(
        caret.visual.primitives[0].paint.background,
        Some(Fill::Solid(caret_color))
    );
    assert!(caret.is_animated());
}

/// The declared native colors are validated before reaching the adapter.
#[test]
fn text_input_color_schema_rejects_wrong_value_type() {
    let registry = builtin::registry().unwrap();
    let schema = registry.schema(builtin::TEXT_INPUT).unwrap();
    for (id, name) in [
        (builtin::SELECTION_COLOR, "selectionColor"),
        (builtin::CARET_COLOR, "caretColor"),
    ] {
        assert!(schema.properties.iter().any(|property| {
            property.id == id
                && property.name.as_str() == name
                && property.value_type == ValueType::Color
        }));
        let error = registry
            .construct(
                builtin::TEXT_INPUT,
                &NativeElementInput::new()
                    .property(builtin::ID, SchemaValue::String("query".into()))
                    .property(id, SchemaValue::String("wrong".into())),
            )
            .unwrap_err();
        assert!(matches!(error, SchemaError::PropertyType { .. }));
    }
}
