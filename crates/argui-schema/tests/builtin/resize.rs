use argui_schema::{NativeElementInput, NativeEventValue, SchemaValue, builtin};
use argui_ui::{EventHandler, EventHandlerId, EventOwnerId, EventType, ResizeAxis};

fn configuration(target: &str, axis: &str, minimum: f32, maximum: f32) -> NativeElementInput {
    NativeElementInput::new()
        .property(builtin::RESIZE_TARGET, SchemaValue::String(target.into()))
        .property(builtin::RESIZE_AXIS, SchemaValue::String(axis.into()))
        .property(builtin::RESIZE_MINIMUM, SchemaValue::Float(minimum))
        .property(builtin::RESIZE_MAXIMUM, SchemaValue::Float(maximum))
}

#[test]
fn resize_schema_declares_and_constructs_native_commit_only_interaction() {
    let registry = builtin::registry().unwrap();
    for (axis, expected) in [
        ("horizontal", ResizeAxis::Horizontal),
        ("vertical", ResizeAxis::Vertical),
    ] {
        let input = configuration("pane", axis, 100., 400.)
            .property(builtin::RESIZE_TRAILING, SchemaValue::Bool(true))
            .event(NativeEventValue::new(
                builtin::RESIZE_COMMIT,
                EventHandler::from_identity(EventHandlerId::new(EventOwnerId(1), 0)),
            ));
        let area = registry.construct(builtin::TOUCH_AREA, &input).unwrap();
        let interaction = area.interaction.as_ref().unwrap();
        let resize = interaction.resize.as_ref().unwrap();
        assert_eq!(resize.axis, expected);
        assert!(resize.trailing && interaction.capture_on_press);
        assert_eq!((resize.minimum, resize.maximum), (100., 400.));
        assert_eq!(area.event_listeners.len(), 1);
        assert_eq!(area.event_listeners[0].event, EventType::ResizeCommit);
    }
    assert!(
        registry
            .construct(builtin::TOUCH_AREA, &NativeElementInput::new())
            .unwrap()
            .interaction
            .as_ref()
            .unwrap()
            .resize
            .is_none()
    );
}

#[test]
fn resize_schema_rejects_incomplete_or_invalid_configuration() {
    let registry = builtin::registry().unwrap();
    for input in [
        NativeElementInput::new().property(
            builtin::RESIZE_AXIS,
            SchemaValue::String("horizontal".into()),
        ),
        NativeElementInput::new()
            .property(builtin::RESIZE_TARGET, SchemaValue::String("pane".into())),
        NativeElementInput::new()
            .property(builtin::RESIZE_TARGET, SchemaValue::String("pane".into()))
            .property(
                builtin::RESIZE_AXIS,
                SchemaValue::String("horizontal".into()),
            ),
        configuration(" ", "horizontal", 100., 400.),
        configuration("pane", "horizontal", -1., 400.),
        configuration("pane", "horizontal", 100., 99.),
        configuration("pane", "diagonal", 100., 400.),
        configuration("pane", "horizontal", f32::NAN, 400.),
        configuration("pane", "horizontal", 100., f32::INFINITY),
    ] {
        assert!(registry.construct(builtin::TOUCH_AREA, &input).is_err());
    }
}

#[test]
fn resize_schema_accepts_a_zero_width_boundary_and_equal_limits() {
    let registry = builtin::registry().unwrap();
    for boundary in [0., 100., 1_000_000.] {
        let input = configuration("pane", "horizontal", boundary, boundary);
        let area = registry.construct(builtin::TOUCH_AREA, &input).unwrap();
        assert_eq!(
            area.interaction
                .as_ref()
                .unwrap()
                .resize
                .as_ref()
                .unwrap()
                .minimum,
            boundary
        );
    }
}
