use argui_schema::{
    NativeElementCache, NativeElementInput, NativeEventValue, NativeSlotValue, SchemaValue, builtin,
};
use argui_ui::{Element, EventHandler, EventHandlerId, EventOwnerId, RetainedIdentity};

/// Unchanged inputs reuse the exact retained subtree; changed inputs rebuild it.
#[test]
fn reuses_equal_inputs_and_rebuilds_changed_values_and_children() {
    let registry = builtin::registry().unwrap();
    let mut cache = NativeElementCache::new();
    let id = RetainedIdentity::new(1, 2);
    let input =
        NativeElementInput::new().property(builtin::TEXT_VALUE, SchemaValue::String("one".into()));
    cache.begin_render();
    let first = cache
        .construct(&registry, builtin::TEXT, id.clone(), input.clone(), false)
        .unwrap();
    cache.end_render();
    cache.begin_render();
    let same = cache
        .construct(&registry, builtin::TEXT, id.clone(), input, false)
        .unwrap();
    assert!(same.ptr_eq(&first));
    let invalid = NativeElementInput::new().property(builtin::TEXT_VALUE, SchemaValue::Bool(true));
    assert!(
        cache
            .construct(&registry, builtin::TEXT, id.clone(), invalid, false)
            .is_err()
    );
    let input =
        NativeElementInput::new().property(builtin::TEXT_VALUE, SchemaValue::String("two".into()));
    let changed = cache
        .construct(&registry, builtin::TEXT, id.clone(), input.clone(), false)
        .unwrap();
    assert!(!changed.ptr_eq(&first));
    let interactive = cache
        .construct(&registry, builtin::TEXT, id.clone(), input, true)
        .unwrap();
    assert!(interactive.interaction.is_some());
    assert_eq!(interactive.source_identity(), Some(&id));
    let input = NativeElementInput::new().slot(NativeSlotValue::new(builtin::CHILDREN, [first]));
    let parent = cache
        .construct(&registry, builtin::COLUMN, id.clone(), input.clone(), false)
        .unwrap();
    assert!(
        parent.ptr_eq(
            &cache
                .construct(&registry, builtin::COLUMN, id.clone(), input, false)
                .unwrap()
        )
    );
    let input = NativeElementInput::new().slot(NativeSlotValue::new(builtin::CHILDREN, [changed]));
    assert!(
        !parent.ptr_eq(
            &cache
                .construct(&registry, builtin::COLUMN, id, input, false)
                .unwrap()
        )
    );
    cache.end_render();
    assert_eq!(cache.len(), 1);
    cache.begin_render();
    cache.end_render();
    assert!(cache.is_empty());
}

/// Handler changes and explicit registry invalidation cannot reuse stale elements.
#[test]
fn handler_identity_and_clear_invalidate_cached_nodes() {
    let registry = builtin::registry().unwrap();
    let mut cache = NativeElementCache::new();
    let id = RetainedIdentity::new(1, 2);
    let input = |slot| {
        NativeElementInput::new().event(NativeEventValue::new(
            builtin::CLICK,
            EventHandler::from_identity(EventHandlerId::new(EventOwnerId(1), slot)),
        ))
    };
    cache.begin_render();
    let first = cache
        .construct(&registry, builtin::TOUCH_AREA, id.clone(), input(1), false)
        .unwrap();
    assert!(
        first.ptr_eq(
            &cache
                .construct(&registry, builtin::TOUCH_AREA, id.clone(), input(1), false)
                .unwrap()
        )
    );
    assert!(
        !first.ptr_eq(
            &cache
                .construct(&registry, builtin::TOUCH_AREA, id.clone(), input(2), false)
                .unwrap()
        )
    );
    cache.clear();
    assert!(cache.is_empty());
    assert!(
        !first.ptr_eq(
            &cache
                .construct(&registry, builtin::TOUCH_AREA, id, input(1), false)
                .unwrap()
        )
    );
}

/// Reusing a parent preserves sharing even when its caller clones child handles.
#[test]
fn unchanged_subtrees_remain_shared() {
    let registry = builtin::registry().unwrap();
    let mut cache = NativeElementCache::new();
    let child = Element::container([]);
    let id = RetainedIdentity::new(4, 2);
    let input =
        || NativeElementInput::new().slot(NativeSlotValue::new(builtin::CHILDREN, [child.clone()]));
    let first = cache
        .construct(&registry, builtin::COLUMN, id.clone(), input(), false)
        .unwrap();
    let next = cache
        .construct(&registry, builtin::COLUMN, id, input(), false)
        .unwrap();
    assert!(first.ptr_eq(&next));
    assert!(next.children[0].ptr_eq(&child));
}
