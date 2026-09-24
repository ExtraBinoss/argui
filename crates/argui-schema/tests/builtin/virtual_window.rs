use argui_schema::{NativeElementInput, NativeSlotValue, SchemaError, SchemaValue, builtin};
use argui_ui::{Element, RetainedIdentity, UiTree, VirtualList};

/// Builds the compiler-shaped input for a keyed million-row model.
///
/// * `count` — full model length without materializing its rows.
/// * `offset` — current scroll offset in logical pixels.
fn input(count: usize, offset: f32) -> NativeElementInput {
    let window = VirtualList::fixed(count, 20.0, 60.0)
        .overscan(2)
        .window(offset);
    let rows = window.range.clone().map(|index| {
        Element::text(index.to_string())
            .keyed(format!("row-{index}"))
            .retained_identity(RetainedIdentity::new(7, 4).with_unsigned_key(index as u64))
    });
    NativeElementInput::new()
        .property(builtin::KEY, SchemaValue::String("window".into()))
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

/// Finds one keyed row's retained node ID in a mounted tree.
///
/// * `tree` — tree containing the virtual window.
/// * `key` — stable row key to find.
fn row_id(tree: &UiTree, key: &str) -> argui_ui::NodeId {
    tree.node_ids()
        .iter()
        .copied()
        .enumerate()
        .find_map(|(index, node)| {
            (tree
                .element_at(index)
                .and_then(|element| element.key.as_deref())
                == Some(key))
            .then_some(node)
        })
        .expect("visible keyed row")
}

#[test]
fn virtual_window_mounts_only_visible_rows_without_visual_policy() {
    let registry = builtin::registry().unwrap();
    let schema = registry.schema(builtin::VIRTUAL_WINDOW).unwrap();
    assert_eq!(schema.name.as_str(), "VirtualWindow");
    for name in ["offset_y", "visible_height", "content_height"] {
        assert!(schema.properties.iter().any(|entry| entry.name == name));
    }
    for property in [builtin::BACKGROUND, builtin::EDGE_SHADOW_WIDTH] {
        assert!(!schema.properties.iter().any(|entry| entry.id == property));
    }
    let root = registry
        .construct(builtin::VIRTUAL_WINDOW, &input(1_000_000, 600.0))
        .unwrap();
    assert_eq!(root.key.as_deref(), Some("window"));
    assert!(root.children[0].children.len() < 25);
    assert!(root.paint.quad.background.is_none());
    let scroll = root.scroll.as_ref().unwrap();
    assert!(scroll.scrollbar.is_none());
    assert!(scroll.effects.is_empty());
}

#[test]
fn virtual_window_can_expose_an_explicit_scrollbar() {
    let registry = builtin::registry().unwrap();
    let root = registry
        .construct(
            builtin::VIRTUAL_WINDOW,
            &input(100, 600.0).property(
                builtin::SCROLLBAR_THUMB,
                SchemaValue::Color(argui_core::Color::srgba(0.3, 0.4, 0.5, 1.0)),
            ),
        )
        .unwrap();
    assert!(root.scroll.as_ref().unwrap().scrollbar.is_some());
}

#[test]
fn visible_row_identity_survives_a_small_scroll() {
    let registry = builtin::registry().unwrap();
    let first = registry
        .construct(builtin::VIRTUAL_WINDOW, &input(1_000_000, 600.0))
        .unwrap();
    let mut tree = UiTree::new(first);
    let row = row_id(&tree, "row-30");
    let moved = registry
        .construct(builtin::VIRTUAL_WINDOW, &input(1_000_000, 605.0))
        .unwrap();
    tree.update(moved);
    assert_eq!(row_id(&tree, "row-30"), row);
    let mounted = tree
        .node_ids()
        .iter()
        .copied()
        .enumerate()
        .filter(|(index, _)| {
            tree.element_at(*index)
                .and_then(|element| element.key.as_deref())
                .is_some_and(|key| key.starts_with("row-"))
        })
        .count();
    assert!(mounted < 25);
}

#[test]
fn virtual_window_accepts_a_stale_bounded_window_but_rejects_invalid_values() {
    let registry = builtin::registry().unwrap();
    let mut wrong = input(100, 0.0);
    wrong.slots[0].elements.pop();
    let root = registry.construct(builtin::VIRTUAL_WINDOW, &wrong).unwrap();
    assert_eq!(
        root.virtual_viewport().unwrap().mounted.len(),
        wrong.slots[0].elements.len()
    );
    let mut invalid = input(100, 0.0);
    invalid
        .properties
        .iter_mut()
        .find(|(id, _)| *id == builtin::ROW_HEIGHT)
        .unwrap()
        .1 = SchemaValue::Float(f32::NAN);
    assert!(matches!(
        registry.construct(builtin::VIRTUAL_WINDOW, &invalid),
        Err(SchemaError::Adapter(_))
    ));
}

#[test]
fn virtual_window_rejects_invalid_dimensions_indices_and_counts() {
    let registry = builtin::registry().unwrap();
    for (property, value) in [
        (builtin::ROW_HEIGHT, SchemaValue::Float(0.0)),
        (builtin::VIEWPORT_HEIGHT, SchemaValue::Float(-1.0)),
        (builtin::SCROLL_OFFSET, SchemaValue::Float(f32::INFINITY)),
        (builtin::ITEM_COUNT, SchemaValue::Int(-1)),
        (builtin::WINDOW_START, SchemaValue::Int(-1)),
        (builtin::OVERSCAN, SchemaValue::Int(-1)),
    ] {
        let mut invalid = input(100, 0.0);
        invalid
            .properties
            .iter_mut()
            .find(|(id, _)| *id == property)
            .unwrap()
            .1 = value;
        assert!(
            matches!(
                registry.construct(builtin::VIRTUAL_WINDOW, &invalid),
                Err(SchemaError::Adapter(_))
            ),
            "invalid property {property:?}"
        );
    }
    let mut wrong_start = input(100, 0.0);
    wrong_start
        .properties
        .iter_mut()
        .find(|(id, _)| *id == builtin::WINDOW_START)
        .unwrap()
        .1 = SchemaValue::Int(101);
    assert!(matches!(
        registry.construct(builtin::VIRTUAL_WINDOW, &wrong_start),
        Err(SchemaError::Adapter(_))
    ));
}

#[test]
fn horizontal_window_uses_widths_and_axis_specific_scroll_effects() {
    let registry = builtin::registry().unwrap();
    let input = NativeElementInput::new()
        .property(builtin::KEY, SchemaValue::String("horizontal".into()))
        .property(builtin::ROW_HEIGHT, SchemaValue::Float(30.0))
        .property(builtin::VIRTUAL_HORIZONTAL, SchemaValue::Bool(true))
        .property(builtin::VIRTUAL_VIEWPORT_WIDTH, SchemaValue::Float(90.0))
        .property(builtin::ITEM_COUNT, SchemaValue::Int(100))
        .property(builtin::WINDOW_START, SchemaValue::Int(0))
        .property(builtin::VIRTUAL_SHADOW_WIDTH, SchemaValue::Float(12.0))
        .property(builtin::VIRTUAL_SCROLLBAR_VISIBLE, SchemaValue::Bool(true))
        .slot(NativeSlotValue::new(
            builtin::CHILDREN,
            (0..12).map(|index| Element::text(index.to_string()).keyed(index.to_string())),
        ));
    let root = registry.construct(builtin::VIRTUAL_WINDOW, &input).unwrap();
    let scroll = root.scroll.as_ref().unwrap();
    assert_eq!(scroll.axes, argui_ui::ScrollAxes::Horizontal);
    assert_eq!(scroll.effects.len(), 1);
    assert!(scroll.scrollbar.is_some());
    assert_eq!(root.virtual_viewport().unwrap().mounted, 0..12);
    assert_eq!(root.children[0].children[1].style.size.width.value(), 30.0);
}
