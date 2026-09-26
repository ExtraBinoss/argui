use argui_core::Color;
use argui_schema::{NativeElementInput, SchemaValue, builtin};
use argui_ui::UiTree;

#[test]
fn layout_gap_and_text_color_retarget_through_native_transitions() {
    let registry = builtin::registry().unwrap();
    let row = |gap| {
        registry
            .construct(
                builtin::ROW,
                &NativeElementInput::new()
                    .property(builtin::ID, SchemaValue::String("gap".into()))
                    .property(builtin::GAP, SchemaValue::Float(gap))
                    .property(builtin::TRANSITION_MS, SchemaValue::Float(480.0)),
            )
            .unwrap()
    };
    let mut layout = UiTree::new(row(5.0));
    layout.update(row(24.0));
    assert!(layout.wants_animation_frame());

    let text = |color| {
        registry
            .construct(
                builtin::TEXT,
                &NativeElementInput::new()
                    .property(builtin::ID, SchemaValue::String("color".into()))
                    .property(builtin::TEXT_VALUE, SchemaValue::String("Argui".into()))
                    .property(builtin::TEXT_COLOR, SchemaValue::Color(color))
                    .property(builtin::TRANSITION_MS, SchemaValue::Float(500.0)),
            )
            .unwrap()
    };
    let mut paint = UiTree::new(text(Color::BLACK));
    paint.update(text(Color::WHITE));
    assert!(paint.wants_animation_frame());
}

#[test]
fn layout_and_text_reject_conflicting_transition_drivers() {
    let registry = builtin::registry().unwrap();
    for native in [builtin::ROW, builtin::TEXT] {
        let mut input = NativeElementInput::new()
            .property(builtin::TRANSITION_MS, SchemaValue::Float(160.0))
            .property(builtin::TRANSITION_SPRING, SchemaValue::Bool(true));
        if native == builtin::TEXT {
            input = input.property(builtin::TEXT_VALUE, SchemaValue::String("Argui".into()));
        }
        assert!(
            registry
                .construct(native, &input)
                .unwrap_err()
                .to_string()
                .contains("cannot combine")
        );
    }
}
