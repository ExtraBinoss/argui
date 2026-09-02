use argui_animation::{Duration, Keyframe, Keyframes, Time};
use argui_core::{Affine2D, Color, Key, KeyInput, KeyState, Modifiers, Point, Rect, Size};
use argui_paint::{ClipChain, PaintStyle, QuadStyle};
use argui_text::TextStyle;
use argui_ui::{
    CaretAlign, CaretAnimation, CaretFrame, CaretHeight, CaretPrimitive, CaretStyle, CaretVisual,
    CursorIcon, FocusRequest, GestureSet, HitRegion, TreeUpdate, UiTree,
};
use argui_widgets::{Input, InputStyle};

fn region(node: argui_ui::NodeId) -> HitRegion {
    HitRegion {
        node,
        bounds: Rect::new(Point::default(), Size::new(200.0, 40.0)),
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
        shape: argui_ui::HitShape::Bounds,
        slop: argui_ui::HitTestStyle::default().slop,
        enabled: true,
        focusable: true,
        cursor: CursorIcon::Text,
        gestures: GestureSet::NONE,
        window_drag: None,
    }
}

fn input(caret: CaretStyle) -> argui_ui::Element {
    Input::new(
        "editor",
        "text",
        "",
        InputStyle::new(PaintStyle::default(), TextStyle::default()).caret(caret),
    )
    .build()
}

#[test]
fn primitives_resolve_line_relative_geometry() {
    let line = Rect::new(Point::new(10.0, 20.0), Size::new(1.0, 18.0));
    let dot = CaretPrimitive::new(
        6.0,
        CaretHeight::Pixels(6.0),
        QuadStyle::solid(Color::WHITE),
    )
    .align(CaretAlign::End)
    .offset(2.0, -1.0);
    assert_eq!(
        dot.bounds(line),
        Rect::new(Point::new(12.0, 31.0), Size::new(6.0, 6.0))
    );
    let bar = CaretPrimitive::new(2.0, CaretHeight::Line, QuadStyle::solid(Color::WHITE));
    assert_eq!(bar.bounds(line).size, Size::new(2.0, 18.0));
}

#[test]
fn generic_keyframes_drive_color_opacity_and_transform() {
    let frames = Keyframes::new([
        Keyframe::new(0.0, CaretFrame::new(1.0, Color::WHITE)),
        Keyframe::new(
            1.0,
            CaretFrame::new(0.0, Color::rgb(1.0, 0.0, 0.0))
                .transform(argui_core::Transform2D::IDENTITY.scale(2.0, 2.0)),
        ),
    ])
    .unwrap();
    let animation = CaretAnimation::new(frames, Duration::from_millis(1_000)).unwrap();
    let sample = animation.sample(Duration::from_millis(500));
    assert!((sample.opacity - 0.5).abs() < 0.001);
    assert!((sample.tint.as_array()[1] - 0.5).abs() < 0.001);
    assert!((sample.transform.scale.x - 1.5).abs() < 0.001);
}

#[test]
fn zero_duration_caret_animation_is_rejected() {
    let frames = Keyframes::new([
        Keyframe::new(0.0, CaretFrame::default()),
        Keyframe::new(1.0, CaretFrame::default()),
    ])
    .unwrap();
    assert_eq!(
        CaretAnimation::new(frames, Duration::ZERO),
        Err(argui_animation::TimingError::ZeroDuration)
    );
}

#[test]
fn focused_animated_caret_requests_paint_and_resets_after_editing() {
    let element = input(CaretStyle::default());
    let mut tree = UiTree::new(element.clone());
    let node = tree.node_id_at(0).unwrap();
    let regions = [region(node)];
    assert!(!tree.wants_animation_frame());
    tree.sync_focus(&regions, Some(FocusRequest::Focus(node.into())));
    assert!(tree.wants_animation_frame());
    assert_eq!(
        tree.advance_animations(Time::from_nanos(1)),
        TreeUpdate::Paint
    );
    tree.advance_animations(Time::from_nanos(600_000_001));
    let caret = match &element.kind {
        argui_ui::ElementKind::TextEditor { caret, .. } => caret,
        _ => panic!("expected a text editor"),
    };
    assert_eq!(tree.resolved_caret_frame(node, caret).opacity, 0.0);

    tree.edit_text_input(&KeyInput {
        key: Key::Character("x".into()),
        state: KeyState::Pressed,
        text: Some("x".into()),
        repeat: false,
        modifiers: Modifiers::default(),
    });
    assert_eq!(tree.resolved_caret_frame(node, caret).opacity, 1.0);
    assert_eq!(tree.set_reduced_motion(true), TreeUpdate::Paint);
    assert!(!tree.wants_animation_frame());
    assert_eq!(tree.resolved_caret_frame(node, caret).opacity, 1.0);
}

#[test]
fn user_authored_steady_multi_primitive_caret_stays_idle() {
    let primitive = CaretPrimitive::new(
        4.0,
        CaretHeight::Pixels(4.0),
        QuadStyle::solid(Color::WHITE),
    );
    let style = CaretStyle::new(CaretVisual::new([
        primitive.clone(),
        primitive.offset(6.0, 0.0),
    ]));
    let mut tree = UiTree::new(input(style));
    let node = tree.node_id_at(0).unwrap();
    tree.sync_focus(&[region(node)], Some(FocusRequest::Focus(node.into())));
    assert!(!tree.wants_animation_frame());
}
