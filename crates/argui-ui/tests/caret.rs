use argui_animation::{Duration, Keyframe, Keyframes, Time};
use argui_core::{Affine2D, Color, Key, KeyInput, KeyState, Modifiers, Point, Rect, Size};
use argui_paint::{ClipChain, QuadStyle};
use argui_ui::{
    CaretAlign, CaretAnimation, CaretFrame, CaretHeight, CaretPrimitive, CaretStyle, CaretVisual,
    CursorIcon, Element, FocusPolicy, FocusRequest, GestureSet, HitRegion, Interaction,
    TextEditorSpec, TextInputFilter, TreeUpdate, UiTree,
};

fn region(node: argui_ui::NodeId) -> HitRegion {
    HitRegion {
        node,
        bounds: Rect::new(Point::default(), Size::new(200.0, 40.0)),
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
        shape: argui_ui::HitShape::Bounds,
        slop: argui_ui::HitTestStyle::default().slop,
        enabled: true,
        focus_policy: argui_ui::FocusPolicy::TabStop,
        cursor: CursorIcon::Text,
        gestures: GestureSet::EMPTY,
        window_drag: None,
    }
}

fn input(caret: CaretStyle) -> argui_ui::Element {
    Element::text_editor(TextEditorSpec {
        value: "text".to_owned(),
        placeholder: String::new(),
        multiline: false,
        read_only: false,
        filter: TextInputFilter::Any,
        text: Default::default(),
        placeholder_text: Default::default(),
        selection: Color::WHITE,
        caret,
    })
    .keyed("editor")
    .interaction(Interaction::default().focus_policy(FocusPolicy::TabStop))
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
            CaretFrame::new(0.0, Color::srgb(1.0, 0.0, 0.0))
                .transform(argui_core::Transform2D::IDENTITY.scale(2.0, 2.0)),
        ),
    ])
    .unwrap();
    let animation = CaretAnimation::new(frames, Duration::from_millis(1_000)).unwrap();
    let sample = animation.sample(Duration::from_millis(500));
    assert!((sample.opacity - 0.5).abs() < 0.001);
    assert_eq!(
        sample.tint,
        Color::WHITE.mix(
            Color::srgb(1.0, 0.0, 0.0),
            0.5,
            argui_core::ColorInterpolation::Oklab,
        )
    );
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
fn focused_animated_caret_requests_composition_and_resets_after_editing() {
    let element = input(CaretStyle::default());
    let mut tree = UiTree::new(element.clone());
    let node = tree.node_id_at(0).unwrap();
    let regions = [region(node)];
    assert!(!tree.wants_animation_frame());
    tree.sync_focus(&regions, Some(FocusRequest::Focus(node.into())));
    assert!(tree.wants_animation_frame());
    assert_eq!(
        tree.advance_animations(Time::from_nanos(1)),
        TreeUpdate::Composite
    );
    assert!(!tree.wants_animation_frame());
    assert_eq!(
        tree.next_animation_frame_at(),
        Some(Time::from_nanos(500_000_001))
    );
    tree.advance_animations(Time::from_nanos(500_000_001));
    let caret = match &element.kind {
        argui_ui::ElementKind::TextEditor { caret, .. } => caret,
        _ => panic!("expected a text editor"),
    };
    assert_eq!(tree.resolved_caret_frame(node, caret).opacity, 0.0);
    assert_eq!(
        tree.next_animation_frame_at(),
        Some(Time::from_nanos(1_000_000_001))
    );

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
    assert_eq!(tree.next_animation_frame_at(), None);
    assert_eq!(tree.resolved_caret_frame(node, caret).opacity, 1.0);
}

#[test]
fn interpolated_caret_keeps_display_linked_frames() {
    let frames = Keyframes::new([
        Keyframe::new(0.0, CaretFrame::new(1.0, Color::WHITE)),
        Keyframe::new(1.0, CaretFrame::new(0.0, Color::WHITE)),
    ])
    .unwrap();
    let style = CaretStyle::new(CaretVisual::new([CaretPrimitive::new(
        1.5,
        CaretHeight::Line,
        QuadStyle::solid(Color::WHITE),
    )]))
    .animated(CaretAnimation::new(frames, Duration::from_millis(1_000)).unwrap());
    let mut tree = UiTree::new(input(style));
    let node = tree.node_id_at(0).unwrap();
    tree.sync_focus(&[region(node)], Some(FocusRequest::Focus(node.into())));
    tree.advance_animations(Time::from_nanos(1));
    assert!(tree.wants_animation_frame());
    assert_eq!(tree.next_animation_frame_at(), None);
}

#[test]
fn tint_animation_requests_paint_instead_of_composition() {
    let frames = Keyframes::new([
        Keyframe::new(0.0, CaretFrame::new(1.0, Color::WHITE)),
        Keyframe::new(1.0, CaretFrame::new(1.0, Color::BLACK)),
    ])
    .unwrap();
    let style = CaretStyle::new(CaretVisual::new([CaretPrimitive::new(
        1.5,
        CaretHeight::Line,
        QuadStyle::solid(Color::WHITE),
    )]))
    .animated(CaretAnimation::new(frames, Duration::from_millis(1_000)).unwrap());
    let mut tree = UiTree::new(input(style));
    let node = tree.node_id_at(0).unwrap();
    tree.sync_focus(&[region(node)], Some(FocusRequest::Focus(node.into())));
    assert_eq!(
        tree.advance_animations(Time::from_nanos(1)),
        TreeUpdate::Paint
    );
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
