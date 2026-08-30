use argui_paint::{Color, PaintStyle, QuadStyle};
use argui_text::{TextStyle, TextWrap};
use argui_ui::{
    Button, ButtonStyle, CursorIcon, ElementKind, KeyboardActivation, UiTree, VisualState,
};

#[test]
fn button_is_only_a_composed_interactive_element() {
    let resting = QuadStyle::solid(Color::rgb(0.0, 0.0, 0.0));
    let hovered = QuadStyle::solid(Color::WHITE);
    let element = Button::new(
        "save",
        "Save",
        ButtonStyle::new(PaintStyle::new(resting.clone()), TextStyle::default())
            .hovered(hovered.clone()),
    )
    .build();

    assert_eq!(element.key.as_deref(), Some("save"));
    assert_eq!(element.paint.quad, resting);
    assert_eq!(element.style.shrink, 0.0);
    let interaction = element.interaction.as_ref().unwrap();
    assert_eq!(interaction.cursor, CursorIcon::Pointer);
    assert_eq!(
        interaction.keyboard_activation,
        KeyboardActivation::EnterOrSpace
    );
    let mut tree = UiTree::new(element.clone());
    let node = tree.node_ids()[0];
    let regions = [argui_ui::HitRegion {
        node,
        bounds: argui_core::Rect::new(
            argui_core::Point::default(),
            argui_core::Size::new(20.0, 20.0),
        ),
        transform: argui_core::Affine2D::IDENTITY,
        clips: argui_paint::ClipChain::default(),
        focusable: true,
        cursor: CursorIcon::Pointer,
        gestures: argui_ui::GestureSet::NONE,
        window_drag: None,
    }];
    tree.pointer_moved(argui_core::Point::new(2.0, 2.0), &regions);
    assert!(tree.visual_states(node).contains(VisualState::Hovered));
    assert_eq!(element.children.len(), 1);
    assert!(matches!(
        &element.children[0].kind,
        ElementKind::Text { style, .. } if style.wrap == TextWrap::None
    ));
}
