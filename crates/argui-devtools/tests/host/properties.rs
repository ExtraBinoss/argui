use super::*;
use argui_inspect::{StyleField, StyleLength, StyleUnit};

fn click(key: &str) -> UiEvent {
    event(
        key,
        UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
    )
}

fn editors() -> DevtoolsHost<App> {
    let host = populated_host();
    let inspector = host.inspector();
    let mut snapshot = inspector.tree();
    snapshot.nodes[1].properties = vec![
        (
            StyleProperty::Width,
            StyleValue::Length(StyleLength {
                unit: StyleUnit::Auto,
                value: 0.0,
            }),
        ),
        (
            StyleProperty::Height,
            StyleValue::Length(StyleLength {
                unit: StyleUnit::Px,
                value: 32.0,
            }),
        ),
        (StyleProperty::Opacity, StyleValue::Number(1.0)),
        (StyleProperty::Background, StyleValue::Srgba([1.0; 4])),
        (
            StyleProperty::Border,
            StyleValue::Parameters(
                [
                    ("width", 2.0),
                    ("red", 0.0),
                    ("green", 0.0),
                    ("blue", 0.0),
                    ("alpha", 1.0),
                ]
                .map(|(label, value)| StyleField {
                    label: label.into(),
                    value,
                })
                .to_vec(),
            ),
        ),
        (
            StyleProperty::Overflow,
            StyleValue::Choice("Visible / Visible".into()),
        ),
    ]
    .into_iter()
    .map(|(property, value)| PropertySnapshot {
        property,
        value,
        authored: true,
    })
    .collect();
    inspector.publish_tree(snapshot);
    inspector.select(Some(InspectNodeId(2)));
    host
}

#[test]
fn lengths_use_actual_bounds_percent_inputs_and_validate_drafts_without_losing_overrides() {
    let host = Entity::new(editors()).mount().unwrap();
    let inspector = host.read(|tools| tools.inspector());
    let update = |event: UiEvent| change_tools(&host, |tools| tools.update(&event));
    update(click("__devtools-unit-2-width-px"));
    assert_eq!(
        inspector.property_value(InspectNodeId(2), StyleProperty::Width),
        Some(StyleValue::Length(StyleLength {
            unit: StyleUnit::Px,
            value: 80.0
        }))
    );
    update(click("__devtools-unit-2-width-percent"));
    update(event(
        "__devtools-value-2-width-0",
        UiEventKind::TextChanged("37.5".into()),
    ));
    let valid = Some(StyleValue::Length(StyleLength {
        unit: StyleUnit::Percent,
        value: 0.375,
    }));
    assert_eq!(
        inspector.property_value(InspectNodeId(2), StyleProperty::Width),
        valid
    );
    for text in ["-", "-1", "NaN", "inf"] {
        update(event(
            "__devtools-value-2-width-0",
            UiEventKind::TextChanged(text.into()),
        ));
        assert_eq!(
            inspector.property_value(InspectNodeId(2), StyleProperty::Width),
            valid
        );
        assert!(contains_text(
            &host.render(Default::default()).unwrap(),
            "Invalid value"
        ));
    }
    update(event("__devtools-value-2-width-0", UiEventKind::Blurred));
    assert!(!contains_text(
        &host.render(Default::default()).unwrap(),
        "Invalid value"
    ));
    update(event(
        "__devtools-value-1-width-0",
        UiEventKind::TextChanged("55".into()),
    ));
    assert_eq!(
        inspector.property_value(InspectNodeId(2), StyleProperty::Width),
        valid
    );
    update(click("__devtools-overflow-2-Hidden"));
    assert_eq!(
        inspector.property_value(InspectNodeId(2), StyleProperty::Overflow),
        Some(StyleValue::Choice("Hidden / Hidden".into()))
    );
    update(click("__devtools-reset-property-2-width"));
    assert_eq!(
        inspector.property_value(InspectNodeId(2), StyleProperty::Width),
        None
    );
    assert!(
        inspector
            .property_value(InspectNodeId(2), StyleProperty::Overflow)
            .is_some()
    );
}

#[test]
fn escape_cancels_invalid_property_drafts_only_on_key_press() {
    let host = Entity::new(editors()).mount().unwrap();
    let inspector = host.read(|tools| tools.inspector());
    let update = |event: UiEvent| change_tools(&host, |tools| tools.update(&event));
    update(event(
        "__devtools-value-2-height-0",
        UiEventKind::TextChanged("42".into()),
    ));
    let saved = inspector.property_value(InspectNodeId(2), StyleProperty::Height);
    update(event(
        "__devtools-value-2-height-0",
        UiEventKind::TextChanged("-".into()),
    ));
    for (key, state, cancelled) in [
        (
            argui_core::Key::ArrowLeft,
            argui_core::KeyState::Pressed,
            false,
        ),
        (
            argui_core::Key::Escape,
            argui_core::KeyState::Released,
            false,
        ),
        (argui_core::Key::Escape, argui_core::KeyState::Pressed, true),
    ] {
        let input = event(
            "__devtools-value-2-height-0",
            UiEventKind::KeyInput(argui_core::KeyInput {
                key,
                state,
                modifiers: Default::default(),
                text: None,
                repeat: false,
            }),
        );
        change_tools(&host, |tools| tools.update(&input));
        assert_eq!(input.default_prevented(), cancelled);
        assert_eq!(
            contains_text(&host.render(Default::default()).unwrap(), "Invalid value"),
            !cancelled
        );
        assert_eq!(
            inspector.property_value(InspectNodeId(2), StyleProperty::Height),
            saved
        );
    }
}

#[test]
fn color_editor_preserves_border_width_and_opacity_slider_keeps_editor_input_local() {
    let mut host = editors();
    let inspector = host.inspector();
    host.update(&click("__devtools-swatch-2-border"));
    host.update(&event(
        "__devtools-color-2-border::field::0",
        UiEventKind::TextChanged("#FF800080".into()),
    ));
    let StyleValue::Parameters(fields) = inspector
        .property_value(InspectNodeId(2), StyleProperty::Border)
        .unwrap()
    else {
        panic!("border parameters")
    };
    assert_eq!(fields[0].value, 2.0);
    assert!((fields[1].value - 1.0).abs() < 0.001);
    assert!((fields[2].value - 128.0 / 255.0).abs() < 0.001);
    assert!((fields[4].value - 128.0 / 255.0).abs() < 0.001);
    let key = argui_core::KeyInput {
        key: argui_core::Key::ArrowLeft,
        state: argui_core::KeyState::Pressed,
        modifiers: Default::default(),
        repeat: false,
        text: None,
    };
    let event = event("__devtools-opacity", UiEventKind::KeyInput(key));
    assert_eq!(host.update(&event), ViewUpdate::Rebuild);
    assert!(event.default_prevented());
    assert!(
        matches!(inspector.property_value(InspectNodeId(2), StyleProperty::Opacity),
        Some(StyleValue::Number(value)) if (value-0.99).abs() < 0.0001)
    );
    host.update(&click("__devtools-style-border"));
    // Disabling a property closes its editor and prevents stale color events.
    host.update(&super::event(
        "__devtools-color-2-border::field::0",
        UiEventKind::TextChanged("#000000FF".into()),
    ));
    assert_eq!(
        inspector.property_enabled(InspectNodeId(2), StyleProperty::Border),
        Some(false)
    );
}
