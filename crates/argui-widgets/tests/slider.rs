use argui_core::{Key, KeyInput, KeyState, Modifiers, Point, Rect, Size};
use argui_runtime::{LayoutBounds, LayoutSnapshot};
use argui_ui::{
    Element, GestureEvent, GestureKind, GesturePhase, SemanticAction, SemanticValue, UiEvent,
    UiEventKind, UiTree, UserSelect,
};
use argui_widgets::{
    RangeAction, RangeAxis, RangeBehavior, RangeConfig, RangeDetents, RangeDirection, RangeState,
    Slider, shadcn,
};

fn event(kind: UiEventKind) -> UiEvent {
    let tree = UiTree::new(Element::container([]));
    UiEvent::new(tree.node_ids()[0], Some("scale".into()), kind)
}

#[test]
fn slider_uses_one_clamped_path_for_keys_pointer_and_semantics() {
    let config = RangeConfig::new(10.0, 0.0, 2.0);
    assert_eq!(config.clamp(9.1), 10.0);
    assert_eq!(config.clamp(3.0), 4.0);
    let mut state = RangeState::default();
    let behavior = |value| RangeBehavior::new("scale", "Scale", value, config);
    let node = UiTree::new(Element::container([])).node_ids()[0];
    state.layout_changed(
        &LayoutSnapshot {
            viewport: Rect::default(),
            nodes: vec![LayoutBounds {
                node,
                key: Some("scale::track".into()),
                retained_identity: None,
                bounds: Rect::new(Point::new(10.0, 0.0), Size::new(100.0, 20.0)),
            }],
        },
        &behavior(4.0),
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
        state.update(&pressed(Key::Home), &behavior(4.0)),
        Some(RangeAction::Begin(0.0))
    );
    assert_eq!(
        state.update(&pressed(Key::End), &behavior(4.0)),
        Some(RangeAction::Update(10.0))
    );
    let tap = event(UiEventKind::Gesture(GestureEvent {
        target: node,
        pointer: argui_core::PointerId::MOUSE,
        phase: GesturePhase::Ended,
        delivery: Default::default(),
        kind: GestureKind::Tap {
            position: Point::new(60.0, 10.0),
        },
    }));
    assert_eq!(
        state.update(&tap, &behavior(0.0)),
        Some(RangeAction::Commit(6.0))
    );

    let semantic = event(UiEventKind::SemanticAction {
        action: SemanticAction::SetValue,
        value: Some(SemanticValue::Number {
            value: 7.0,
            minimum: None,
            maximum: None,
            step: None,
        }),
    });
    assert_eq!(
        state.update(&semantic, &behavior(0.0)),
        Some(RangeAction::Commit(8.0))
    );

    let pan = |phase, position| {
        event(UiEventKind::Gesture(GestureEvent {
            target: node,
            pointer: argui_core::PointerId::MOUSE,
            phase,
            delivery: Default::default(),
            kind: GestureKind::Pan {
                position,
                delta: Point::default(),
                total: Point::default(),
                velocity: Point::default(),
            },
        }))
    };
    assert_eq!(
        state.update(
            &pan(GesturePhase::Started, Point::new(50.0, 10.0)),
            &behavior(4.0)
        ),
        Some(RangeAction::Begin(4.0))
    );
    assert_eq!(
        state.update(
            &pan(GesturePhase::Ended, Point::new(70.0, 10.0)),
            &behavior(4.0)
        ),
        Some(RangeAction::Commit(6.0))
    );
    assert_eq!(
        state.update(
            &pan(GesturePhase::Started, Point::new(50.0, 10.0)),
            &behavior(4.0),
        ),
        Some(RangeAction::Begin(4.0))
    );
    assert_eq!(
        state.update(
            &pan(GesturePhase::Cancelled, Point::new(40.0, 0.0)),
            &behavior(4.0),
        ),
        Some(RangeAction::Cancel(4.0))
    );

    let themes = shadcn(argui_core::Color::srgb(0.2, 0.5, 0.9));
    let zero = Slider::new("zero", "Zero", 0.0, RangeConfig::new(0.0, 0.0, 1.0))
        .enabled(false)
        .build(themes.resolve(argui_core::ColorScheme::Dark));
    assert_eq!(zero.user_select, UserSelect::None);
    assert!(!zero.interaction.as_ref().unwrap().enabled);
    assert_eq!(
        state.update(&pressed(Key::ArrowRight), &behavior(4.0).enabled(false)),
        None
    );
}

#[test]
fn range_covers_vertical_reverse_and_interaction_boundaries() {
    let config = RangeConfig::new(0.0, 10.0, 1.0)
        .axis(RangeAxis::Vertical)
        .direction(RangeDirection::Reverse);
    assert_eq!(config.ratio(2.0), 0.8);
    let behavior = |value| RangeBehavior::new("scale", "Scale", value, config);
    let node = UiTree::new(Element::container([])).node_ids()[0];
    let mut state = RangeState::default();
    state.layout_changed(
        &LayoutSnapshot {
            viewport: Rect::default(),
            nodes: vec![LayoutBounds {
                node,
                key: Some("scale::track".into()),
                retained_identity: None,
                bounds: Rect::new(Point::new(0.0, 10.0), Size::new(20.0, 100.0)),
            }],
        },
        &behavior(2.0),
    );

    let key = |key, state, repeat| {
        event(UiEventKind::KeyInput(KeyInput {
            key,
            state,
            modifiers: Modifiers::default(),
            repeat,
            text: None,
        }))
    };
    assert_eq!(
        state.update(&key(Key::ArrowUp, KeyState::Pressed, true), &behavior(2.0)),
        Some(RangeAction::Update(3.0))
    );
    assert_eq!(
        state.update(
            &key(Key::ArrowUp, KeyState::Released, false),
            &behavior(3.0)
        ),
        None
    );
    assert_eq!(
        state.update(&key(Key::ArrowUp, KeyState::Pressed, false), &behavior(3.0)),
        Some(RangeAction::Begin(4.0))
    );
    assert_eq!(
        state.update(
            &key(Key::ArrowUp, KeyState::Released, false),
            &behavior(4.0)
        ),
        Some(RangeAction::Commit(4.0))
    );

    let gesture = |phase, kind| {
        event(UiEventKind::Gesture(GestureEvent {
            target: node,
            pointer: argui_core::PointerId::MOUSE,
            phase,
            delivery: Default::default(),
            kind,
        }))
    };
    assert_eq!(
        state.update(
            &gesture(
                GesturePhase::Ended,
                GestureKind::Tap {
                    position: Point::new(10.0, 35.0),
                }
            ),
            &behavior(2.0)
        ),
        Some(RangeAction::Commit(8.0))
    );
    assert_eq!(
        state.update(
            &gesture(
                GesturePhase::Started,
                GestureKind::Pan {
                    position: Point::new(10.0, 30.0),
                    delta: Point::default(),
                    total: Point::default(),
                    velocity: Point::default(),
                }
            ),
            &behavior(8.0)
        ),
        Some(RangeAction::Begin(8.0))
    );
    assert_eq!(
        state.update(
            &gesture(
                GesturePhase::Changed,
                GestureKind::Pan {
                    position: Point::new(10.0, 50.0),
                    delta: Point::new(0.0, 20.0),
                    total: Point::new(0.0, 20.0),
                    velocity: Point::default(),
                }
            ),
            &behavior(8.0)
        ),
        Some(RangeAction::Update(6.0))
    );

    state.layout_changed(&LayoutSnapshot::default(), &behavior(8.0));
    assert_eq!(
        state.update(
            &gesture(
                GesturePhase::Ended,
                GestureKind::Tap {
                    position: Point::default(),
                }
            ),
            &behavior(8.0)
        ),
        None
    );

    let flat = RangeBehavior::new(
        "scale",
        "Scale",
        5.0,
        RangeConfig::new(0.0, 10.0, 1.0).axis(RangeAxis::Vertical),
    );
    state.layout_changed(
        &LayoutSnapshot {
            viewport: Rect::default(),
            nodes: vec![LayoutBounds {
                node,
                key: Some(flat.track_key()),
                retained_identity: None,
                bounds: Rect::new(Point::default(), Size::new(20.0, 0.0)),
            }],
        },
        &flat,
    );
    assert_eq!(
        state.update(
            &gesture(
                GesturePhase::Ended,
                GestureKind::Tap {
                    position: Point::default(),
                }
            ),
            &flat
        ),
        None
    );

    let flat_horizontal =
        RangeBehavior::new("scale", "Scale", 5.0, RangeConfig::new(0.0, 10.0, 1.0));
    state.layout_changed(
        &LayoutSnapshot {
            viewport: Rect::default(),
            nodes: vec![LayoutBounds {
                node,
                key: Some(flat_horizontal.track_key()),
                retained_identity: None,
                bounds: Rect::new(Point::default(), Size::new(0.0, 20.0)),
            }],
        },
        &flat_horizontal,
    );
    assert_eq!(
        state.update(
            &gesture(
                GesturePhase::Started,
                GestureKind::Pan {
                    position: Point::default(),
                    delta: Point::default(),
                    total: Point::default(),
                    velocity: Point::default(),
                }
            ),
            &flat_horizontal
        ),
        None
    );
}

#[test]
fn range_detents_are_magnetic_only_during_slow_pointer_motion() {
    let config = RangeConfig::new(0.0, 100.0, 1.0).detents(RangeDetents::new(10.0, 2.0, 200.0));
    let behavior = |value| RangeBehavior::new("scale", "Scale", value, config);
    let node = UiTree::new(Element::container([])).node_ids()[0];
    let mut state = RangeState::default();
    state.layout_changed(
        &LayoutSnapshot {
            viewport: Rect::default(),
            nodes: vec![LayoutBounds {
                node,
                key: Some("scale::track".into()),
                retained_identity: None,
                bounds: Rect::new(Point::default(), Size::new(100.0, 20.0)),
            }],
        },
        &behavior(0.0),
    );
    let pan = |velocity| {
        event(UiEventKind::Gesture(GestureEvent {
            target: node,
            pointer: argui_core::PointerId::MOUSE,
            phase: GesturePhase::Changed,
            delivery: Default::default(),
            kind: GestureKind::Pan {
                position: Point::new(48.0, 10.0),
                delta: Point::default(),
                total: Point::default(),
                velocity: Point::new(velocity, 0.0),
            },
        }))
    };

    assert_eq!(
        state.update(&pan(80.0), &behavior(48.0)),
        Some(RangeAction::Update(50.0))
    );
    assert_eq!(
        state.update(&pan(600.0), &behavior(48.0)),
        Some(RangeAction::Update(48.0))
    );
}
