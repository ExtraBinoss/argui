use argui_core::{Color, ColorScheme, Size};
use argui_layout::LayoutEngine;
use argui_text::TextEngine;
use argui_ui::{ClickEvent, Element, UiEvent, UiEventKind, UiTree};
use argui_widgets::{Button, DialogAction, Sheet, SheetSide, shadcn};

#[test]
fn sheets_touch_the_requested_viewport_edge_and_restore_focus() {
    let palette = shadcn(Color::BLACK);
    let theme = palette.resolve(ColorScheme::Light);
    for side in [
        SheetSide::Left,
        SheetSide::Right,
        SheetSide::Top,
        SheetSide::Bottom,
    ] {
        let mut sheet = Sheet::new(
            "settings",
            "Settings",
            true,
            Button::new("open", "Open", theme.button()).build(),
            Element::text("Settings content"),
        );
        sheet.side = side;
        let mut tree = UiTree::new(sheet.clone().build(theme));
        let output = LayoutEngine::new()
            .compute(&mut tree, &mut TextEngine::new(), Size::new(800.0, 600.0))
            .unwrap();
        let panel = output
            .nodes
            .iter()
            .find(|node| tree.key(node.node) == Some("settings::panel"))
            .unwrap()
            .bounds;
        match side {
            SheetSide::Left => assert_eq!(panel.origin.x, 0.0),
            SheetSide::Right => assert!((panel.origin.x + panel.size.width - 800.0).abs() < 1.0),
            SheetSide::Top => assert_eq!(panel.origin.y, 0.0),
            SheetSide::Bottom => assert!((panel.origin.y + panel.size.height - 600.0).abs() < 1.0),
        }
        let event = UiEvent::new(
            tree.node_ids()[0],
            Some("settings::close".into()),
            UiEventKind::Click(ClickEvent::accessibility()),
        );
        assert_eq!(sheet.action(&event), Some(DialogAction::Close));
        let root = sheet.build(theme);
        assert!(root.children[1].focus_scope.as_ref().unwrap().restore);
    }
}
