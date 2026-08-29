use argui_paint::{Color, PaintStyle, QuadStyle};
use argui_text::{TextStyle, TextWrap};
use argui_ui::{Button, ButtonStyle, CursorIcon, ElementKind, KeyboardActivation};

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
    assert_eq!(interaction.styles.hovered, Some(hovered));
    assert_eq!(element.children.len(), 1);
    assert!(matches!(
        &element.children[0].kind,
        ElementKind::Text { style, .. } if style.wrap == TextWrap::None
    ));
}
