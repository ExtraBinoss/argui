use argui_schema::{NativeElementInput, NativeSlotValue, SchemaError, SchemaValue, builtin};
use argui_ui::{Element, ElementKind, VirtualList};

/// Collects the text of the rows physically mounted under a viewport.
fn texts(element: &Element, output: &mut Vec<String>) {
    if let ElementKind::Text { content, .. } = &element.kind {
        output.push(content.as_str().to_owned());
    }
    for child in &element.children {
        texts(child, output);
    }
}

/// Builds a schema input with the same mounted range as the engine window.
fn input(count: usize, offset: f32) -> NativeElementInput {
    let window = VirtualList::fixed(count, 20.0, 60.0)
        .overscan(2)
        .window(offset);
    let rows = window
        .range
        .clone()
        .map(|index| Element::text(index.to_string()));
    NativeElementInput::new()
        .property(builtin::KEY, SchemaValue::String("items".into()))
        .property(builtin::ROW_HEIGHT, SchemaValue::Float(20.0))
        .property(builtin::VIEWPORT_HEIGHT, SchemaValue::Float(60.0))
        .property(builtin::SCROLL_OFFSET, SchemaValue::Float(offset))
        .property(builtin::OVERSCAN, SchemaValue::Int(2))
        .property(builtin::ITEM_COUNT, SchemaValue::Int(count as i64))
        .property(
            builtin::WINDOW_START,
            SchemaValue::Int(window.range.start as i64),
        )
        .slot(NativeSlotValue::new(builtin::CHILDREN, rows))
}

#[test]
fn virtual_list_builds_only_the_mounted_rows_and_exposes_scroll_payload() {
    let registry = builtin::registry().unwrap();
    let schema = registry.schema_named("VList").unwrap();
    assert_eq!(schema.id, builtin::VIRTUAL_LIST);
    let scroll = schema
        .events
        .iter()
        .find(|event| event.id == builtin::SCROLL)
        .unwrap();
    assert_eq!(scroll.payload, Some(argui_schema::ValueType::Float));
    assert!(
        schema
            .properties
            .iter()
            .any(|property| property.id == builtin::ROTATION)
    );
    let root = registry
        .construct(builtin::VIRTUAL_LIST, &input(100_000, 600.0))
        .unwrap();
    assert_eq!(root.key.as_deref(), Some("items"));
    assert!(root.children[0].children.len() < 25);
    assert!(root.children[0].children.len() > 1);
}

#[test]
fn virtual_list_handles_empty_models_and_clamps_far_offsets() {
    let registry = builtin::registry().unwrap();
    let empty = registry
        .construct(builtin::VIRTUAL_LIST, &input(0, 0.0))
        .unwrap();
    let mut visible = Vec::new();
    texts(&empty, &mut visible);
    assert!(visible.is_empty());

    let last = registry
        .construct(builtin::VIRTUAL_LIST, &input(100_000, f32::MAX))
        .unwrap();
    texts(&last, &mut visible);
    assert!(visible.len() < 25);
    assert!(visible.iter().any(|value| value == "99999"));
}

#[test]
fn virtual_list_uses_optional_defaults_and_theme_scrollbar_color() {
    let registry = builtin::registry().unwrap();
    let mut defaults = input(20, 0.0);
    defaults
        .properties
        .retain(|(id, _)| *id != builtin::SCROLL_OFFSET && *id != builtin::OVERSCAN);
    let default_window = VirtualList::fixed(20, 20.0, 60.0).overscan(3).window(0.0);
    defaults.slots[0].elements = default_window
        .range
        .map(|index| Element::text(index.to_string()))
        .collect();
    let root = registry
        .construct(builtin::VIRTUAL_LIST, &defaults)
        .unwrap();
    assert_eq!(root.key.as_deref(), Some("items"));
    let mut colored = input(20, 0.0)
        .property(
            builtin::SCROLLBAR_THUMB,
            SchemaValue::Color(argui_core::Color::srgb(1.0, 0.0, 0.0)),
        )
        .property(builtin::GROW, SchemaValue::Float(1.0));
    colored
        .properties
        .iter_mut()
        .find(|(id, _)| *id == builtin::SCROLL_OFFSET)
        .unwrap()
        .1 = SchemaValue::Float(f32::NAN);
    let root = registry.construct(builtin::VIRTUAL_LIST, &colored).unwrap();
    assert_eq!(root.style.flex_grow, 1.0);
    assert!(root.scroll.as_ref().unwrap().effects.is_empty());
}

#[test]
fn virtual_list_adds_only_opted_in_scroll_reactive_edge_shadow() {
    let registry = builtin::registry().unwrap();
    let shadowed = input(100, 0.0)
        .property(builtin::EDGE_SHADOW_WIDTH, SchemaValue::Float(14.0))
        .property(builtin::EDGE_SHADOW_INTENSITY, SchemaValue::Float(0.2))
        .property(
            builtin::EDGE_SHADOW_COLOR,
            SchemaValue::Color(argui_core::Color::srgb(0.1, 0.2, 0.3)),
        );
    let root = registry
        .construct(builtin::VIRTUAL_LIST, &shadowed)
        .unwrap();
    let scroll = root.scroll.as_ref().unwrap();
    assert_eq!(scroll.effects.len(), 1);
    assert!(matches!(
        scroll.effects[0].layer.filters.as_slice(),
        [argui_paint::Filter::Effect(effect)] if effect.id == argui_effects::EDGE_SHADOW_ID
    ));
}

#[test]
fn virtual_list_rejects_mismatched_windows_and_invalid_extents() {
    let registry = builtin::registry().unwrap();
    let mut wrong_window = input(100, 0.0);
    wrong_window
        .properties
        .iter_mut()
        .find(|(id, _)| *id == builtin::WINDOW_START)
        .unwrap()
        .1 = SchemaValue::Int(50);
    assert!(matches!(
        registry.construct(builtin::VIRTUAL_LIST, &wrong_window),
        Err(SchemaError::Adapter(_))
    ));
    let mut invalid_extent = input(100, 0.0);
    invalid_extent
        .properties
        .iter_mut()
        .find(|(id, _)| *id == builtin::ROW_HEIGHT)
        .unwrap()
        .1 = SchemaValue::Float(0.0);
    assert!(matches!(
        registry.construct(builtin::VIRTUAL_LIST, &invalid_extent),
        Err(SchemaError::Adapter(_))
    ));
    let mut wrong_count = input(100, 0.0);
    wrong_count.slots[0].elements.pop();
    assert!(matches!(
        registry.construct(builtin::VIRTUAL_LIST, &wrong_count),
        Err(SchemaError::Adapter(_))
    ));
    let mut negative_overscan = input(100, 0.0);
    negative_overscan
        .properties
        .iter_mut()
        .find(|(id, _)| *id == builtin::OVERSCAN)
        .unwrap()
        .1 = SchemaValue::Int(-1);
    assert!(matches!(
        registry.construct(builtin::VIRTUAL_LIST, &negative_overscan),
        Err(SchemaError::Adapter(_))
    ));
    for (property, value) in [
        (builtin::EDGE_SHADOW_WIDTH, f32::NAN),
        (builtin::EDGE_SHADOW_WIDTH, -1.0),
        (builtin::EDGE_SHADOW_INTENSITY, f32::INFINITY),
    ] {
        let invalid = input(100, 0.0).property(property, SchemaValue::Float(value));
        assert!(matches!(
            registry.construct(builtin::VIRTUAL_LIST, &invalid),
            Err(SchemaError::Adapter(_))
        ));
    }
    for (id, value) in [
        (builtin::ITEM_COUNT, SchemaValue::Int(-1)),
        (builtin::WINDOW_START, SchemaValue::Int(-1)),
        (builtin::VIEWPORT_HEIGHT, SchemaValue::Float(-1.0)),
        (builtin::VIEWPORT_HEIGHT, SchemaValue::Float(f32::NAN)),
        (builtin::ROW_HEIGHT, SchemaValue::Float(f32::NAN)),
    ] {
        let mut invalid = input(100, 0.0);
        invalid
            .properties
            .iter_mut()
            .find(|(candidate, _)| *candidate == id)
            .unwrap()
            .1 = value;
        assert!(matches!(
            registry.construct(builtin::VIRTUAL_LIST, &invalid),
            Err(SchemaError::Adapter(_))
        ));
    }
    for missing in [builtin::KEY, builtin::ITEM_COUNT, builtin::WINDOW_START] {
        let mut invalid = input(100, 0.0);
        invalid.properties.retain(|(id, _)| *id != missing);
        assert!(matches!(
            registry.construct(builtin::VIRTUAL_LIST, &invalid),
            Err(SchemaError::Adapter(_))
        ));
    }
}
