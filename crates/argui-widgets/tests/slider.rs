use argui_core::{Key, KeyInput, KeyState, Modifiers, Point, Rect, Size};
use argui_runtime::{LayoutBounds, LayoutSnapshot};
use argui_ui::{
    Element, GestureEvent, GestureKind, GesturePhase, SemanticAction, SemanticValue, UiEvent,
    UiEventKind, UiTree,
};
use argui_widgets::{Slider, SliderConfig, SliderState, shadcn};

fn event(kind: UiEventKind) -> UiEvent {
    let tree = UiTree::new(Element::container([]));
    UiEvent {
        target: tree.node_ids()[0],
        key: Some("scale".into()),
        kind,
    }
}

#[test]
fn slider_uses_one_clamped_path_for_keys_pointer_and_semantics() {
    let config = SliderConfig::new(10.0, 0.0, 2.0);
    assert_eq!(config.clamp(9.1), 10.0);
    assert_eq!(config.clamp(3.0), 4.0);
    let mut state = SliderState::default();
    let node = UiTree::new(Element::container([])).node_ids()[0];
    state.layout_changed(
        &LayoutSnapshot {
            viewport: Rect::default(),
            nodes: vec![LayoutBounds {
                node,
                key: Some("scale".into()),
                bounds: Rect::new(Point::new(10.0, 0.0), Size::new(100.0, 20.0)),
            }],
        },
        "scale",
    );
    let pressed = |key| {
        event(UiEventKind::KeyInput(KeyInput {
            key,
            state: KeyState::Pressed,
            modifiers: Modifiers::default(),
            repeat: false,
            text: None,
        }))
    };
    assert_eq!(
        state.update(&pressed(Key::Home), "scale", 4.0, config),
        Some(0.0)
    );
    assert_eq!(
        state.update(&pressed(Key::End), "scale", 4.0, config),
        Some(10.0)
    );
    let tap = event(UiEventKind::Gesture(GestureEvent {
        target: node,
        phase: GesturePhase::Ended,
        kind: GestureKind::Tap {
            position: Point::new(60.0, 10.0),
        },
    }));
    assert_eq!(state.update(&tap, "scale", 0.0, config), Some(6.0));

    let semantic = event(UiEventKind::SemanticAction {
        action: SemanticAction::SetValue,
        value: Some(SemanticValue::Number {
            value: 7.0,
            minimum: None,
            maximum: None,
            step: None,
        }),
    });
    assert_eq!(state.update(&semantic, "scale", 0.0, config), Some(8.0));

    let pan = |phase, total| {
        event(UiEventKind::Gesture(GestureEvent {
            target: node,
            phase,
            kind: GestureKind::Pan {
                delta: total,
                total,
                velocity: Point::default(),
            },
        }))
    };
    assert_eq!(
        state.update(
            &pan(GesturePhase::Started, Point::default()),
            "scale",
            4.0,
            config
        ),
        Some(4.0)
    );
    assert_eq!(
        state.update(
            &pan(GesturePhase::Ended, Point::new(20.0, 0.0)),
            "scale",
            4.0,
            config
        ),
        Some(6.0)
    );

    let themes = shadcn(argui_core::Color::rgb(0.2, 0.5, 0.9));
    let zero = Slider::new("zero", "Zero", 0.0, SliderConfig::new(0.0, 0.0, 1.0))
        .enabled(false)
        .build(themes.resolve(argui_core::ColorScheme::Dark));
    assert!(!zero.interaction.as_ref().unwrap().enabled);
}
