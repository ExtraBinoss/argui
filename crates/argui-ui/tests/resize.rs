use argui_core::{Point, Size};
use argui_ui::{
    CursorIcon, Element, GestureEvent, GestureKind, GesturePhase, LayoutStyle, Length, ResizeAxes,
    ResizeConfig, ResizeEvent, ResizeState, UiEvent, UiEventKind, UiTree,
};

fn event(phase: GesturePhase, total: Point) -> UiEvent {
    let tree = UiTree::new(Element::container([]));
    let target = tree.node_ids()[0];
    UiEvent {
        target,
        key: Some("resize".into()),
        kind: UiEventKind::Gesture(GestureEvent {
            target,
            phase,
            kind: GestureKind::Pan {
                delta: total,
                total,
                velocity: Point::default(),
            },
        }),
    }
}

#[test]
fn resize_clamps_and_cancel_restores_the_start_size() {
    let config = ResizeConfig::new(Size::new(100.0, 80.0), Size::new(300.0, 240.0));
    let mut state = ResizeState::new(Size::new(160.0, 120.0));
    assert_eq!(
        state.update(
            &event(GesturePhase::Started, Point::default()),
            "resize",
            config
        ),
        Some(ResizeEvent::Started(Size::new(160.0, 120.0)))
    );
    state.update(
        &event(GesturePhase::Changed, Point::new(500.0, -100.0)),
        "resize",
        config,
    );
    assert_eq!(state.size(), Size::new(300.0, 80.0));
    state.update(
        &event(GesturePhase::Cancelled, Point::default()),
        "resize",
        config,
    );
    assert_eq!(state.size(), Size::new(160.0, 120.0));
}

#[test]
fn resize_axes_end_and_unrelated_events_are_explicit() {
    let bounds = ResizeConfig::new(Size::new(100.0, 80.0), Size::new(300.0, 240.0));
    let mut state = ResizeState::new(Size::new(160.0, 120.0));
    assert_eq!(
        state.update(
            &event(GesturePhase::Changed, Point::new(25.0, 70.0)),
            "resize",
            bounds.axes(ResizeAxes::Horizontal),
        ),
        Some(ResizeEvent::Changed(Size::new(185.0, 120.0)))
    );
    assert_eq!(
        state.update(
            &event(GesturePhase::Changed, Point::new(25.0, 70.0)),
            "resize",
            bounds.axes(ResizeAxes::Vertical),
        ),
        Some(ResizeEvent::Changed(Size::new(185.0, 190.0)))
    );
    assert_eq!(
        state.update(
            &event(GesturePhase::Ended, Point::default()),
            "resize",
            bounds,
        ),
        Some(ResizeEvent::Ended(Size::new(185.0, 190.0)))
    );
    assert_eq!(
        state.update(
            &event(GesturePhase::Cancelled, Point::default()),
            "resize",
            bounds,
        ),
        Some(ResizeEvent::Cancelled(Size::new(185.0, 190.0)))
    );

    let mut unrelated = event(GesturePhase::Started, Point::default());
    assert_eq!(state.update(&unrelated, "other", bounds), None);
    unrelated.key = Some("resize".into());
    unrelated.kind = UiEventKind::Clicked;
    assert_eq!(state.update(&unrelated, "resize", bounds), None);
    unrelated.kind = UiEventKind::Gesture(GestureEvent {
        target: unrelated.target,
        phase: GesturePhase::Started,
        kind: GestureKind::Tap {
            position: Point::default(),
        },
    });
    assert_eq!(state.update(&unrelated, "resize", bounds), None);
}

#[test]
fn first_pan_sample_resizes_without_waiting_for_another_pointer_event() {
    let config = ResizeConfig::new(Size::new(100.0, 80.0), Size::new(300.0, 240.0));
    let mut state = ResizeState::new(Size::new(160.0, 120.0));
    assert_eq!(
        state.update(
            &event(GesturePhase::Started, Point::new(12.0, 8.0)),
            "resize",
            config,
        ),
        Some(ResizeEvent::Started(Size::new(172.0, 128.0)))
    );
    assert_eq!(state.size(), Size::new(172.0, 128.0));
}

#[test]
fn resizable_composes_arbitrary_content_and_handle() {
    let element = argui_ui::Resizable::new(
        Size::new(240.0, 160.0),
        Element::text("editor"),
        "resize",
        Element::text("handle"),
    )
    .layout(LayoutStyle {
        min_width: Length::Px(120.0),
        ..LayoutStyle::default()
    })
    .build();

    assert_eq!(element.children.len(), 2);
    assert_eq!(element.style.width, Length::Px(240.0));
    assert_eq!(element.style.height, Length::Px(160.0));
    assert_eq!(element.style.min_width, Length::Px(120.0));
    let handle = &element.children[1];
    assert_eq!(handle.key.as_deref(), Some("resize"));
    assert_eq!(
        handle.interaction.as_ref().unwrap().cursor,
        CursorIcon::NwseResize
    );
}
