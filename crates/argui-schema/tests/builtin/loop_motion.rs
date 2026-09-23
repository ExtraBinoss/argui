use argui_schema::{NativeElementInput, SchemaValue, builtin};
use argui_ui::{PropertyBinding, UiTree};

#[test]
fn row_gap_loop_advances_layout_and_pauses() {
    let registry = builtin::registry().unwrap();
    let row = registry
        .construct(
            builtin::ROW,
            &NativeElementInput::new()
                .property(builtin::GAP, SchemaValue::Float(5.0))
                .property(builtin::LOOP_MS, SchemaValue::Float(1000.0))
                .property(builtin::LOOP_GAP, SchemaValue::Float(24.0)),
        )
        .unwrap();
    assert!(matches!(row.bindings[0], PropertyBinding::Layout(..)));
    let mut tree = UiTree::new(row);
    tree.advance_animations(argui_animation::Time::from_nanos(1));
    assert_eq!(
        tree.advance_animations(argui_animation::Time::from_nanos(500_000_001)),
        argui_ui::TreeUpdate::Layout
    );
    assert!(tree.wants_animation_frame());
    let paused = registry
        .construct(
            builtin::ROW,
            &NativeElementInput::new()
                .property(builtin::GAP, SchemaValue::Float(5.0))
                .property(builtin::LOOP_MS, SchemaValue::Float(1000.0))
                .property(builtin::LOOP_GAP, SchemaValue::Float(24.0))
                .property(builtin::LOOP_PLAYING, SchemaValue::Bool(false)),
        )
        .unwrap();
    assert!(!UiTree::new(paused).wants_animation_frame());
}

#[test]
fn text_opacity_loop_uses_native_compositor_binding() {
    let text = builtin::registry()
        .unwrap()
        .construct(
            builtin::TEXT,
            &NativeElementInput::new()
                .property(builtin::TEXT_VALUE, SchemaValue::String("Argui".into()))
                .property(builtin::LOOP_MS, SchemaValue::Float(950.0))
                .property(builtin::LOOP_OPACITY, SchemaValue::Float(0.2)),
        )
        .unwrap();
    assert!(matches!(
        text.bindings[0],
        PropertyBinding::LayerOpacity(..)
    ));
    assert!(UiTree::new(text).wants_animation_frame());
}

#[test]
fn gap_loop_requires_an_authored_gap() {
    let error = builtin::registry()
        .unwrap()
        .construct(
            builtin::ROW,
            &NativeElementInput::new()
                .property(builtin::LOOP_MS, SchemaValue::Float(1000.0))
                .property(builtin::LOOP_GAP, SchemaValue::Float(24.0)),
        )
        .unwrap_err();
    assert!(error.to_string().contains("loop_gap requires gap"));
}
