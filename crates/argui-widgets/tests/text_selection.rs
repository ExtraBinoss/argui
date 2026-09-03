use argui_core::{ColorScheme, Point, Rect, Size};
use argui_ui::{ElementKind, SelectionCapabilities, UserSelect};
use argui_widgets::{TextSelectionToolbar, shadcn};

#[test]
fn touch_selection_toolbar_is_a_clamped_non_selectable_widget() {
    let palette = shadcn(argui_core::Color::srgb(0.2, 0.4, 0.8));
    let theme = palette.resolve(ColorScheme::Light);
    let toolbar = TextSelectionToolbar::new(
        "copy",
        Rect::new(Point::new(2.0, 2.0), Size::new(20.0, 20.0)),
        Size::new(200.0, 120.0),
        SelectionCapabilities {
            copy: true,
            select_all: true,
            ..SelectionCapabilities::default()
        },
    )
    .build(theme);

    assert!(matches!(toolbar.kind, ElementKind::Container));
    assert_eq!(toolbar.user_select, UserSelect::None);
    assert!(!toolbar.interaction.as_ref().unwrap().focusable);
    assert_eq!(toolbar.children.len(), 2);
}

#[test]
fn editable_toolbar_keeps_unavailable_commands_visible_but_disabled() {
    let palette = shadcn(argui_core::Color::srgb(0.2, 0.4, 0.8));
    let theme = palette.resolve(ColorScheme::Dark);
    let toolbar = TextSelectionToolbar::new(
        "editor",
        Rect::default(),
        Size::new(500.0, 300.0),
        SelectionCapabilities {
            editable: true,
            paste: true,
            select_all: true,
            ..SelectionCapabilities::default()
        },
    )
    .build(theme);

    assert_eq!(toolbar.children.len(), 4);
    assert!(!toolbar.children[0].interaction.as_ref().unwrap().enabled);
    assert!(!toolbar.children[1].interaction.as_ref().unwrap().enabled);
    assert!(toolbar.children[2].interaction.as_ref().unwrap().enabled);
    assert!(toolbar.children[3].interaction.as_ref().unwrap().enabled);
}
