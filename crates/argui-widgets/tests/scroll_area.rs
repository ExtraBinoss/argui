use argui_core::{Color, ColorScheme, Size};
use argui_layout::LayoutEngine;
use argui_text::TextEngine;
use argui_ui::{Element, Orientation, ScrollConfig, UiTree, length};
use argui_widgets::{ScrollArea, shadcn};

#[test]
fn viewports_expose_real_overflow_and_keep_custom_scroll_configuration() {
    let palette = shadcn(Color::BLACK);
    let theme = palette.resolve(ColorScheme::Light);
    for orientation in [Orientation::Vertical, Orientation::Horizontal] {
        let content = Element::container([])
            .width(length(800.0))
            .height(length(800.0));
        let mut area = ScrollArea::new("log", "Log", 180.0, content);
        area.orientation = orientation;
        let mut tree = UiTree::new(area.clone().build(theme));
        let output = LayoutEngine::new()
            .compute(&mut tree, &mut TextEngine::new(), Size::new(300.0, 300.0))
            .unwrap();
        let scroll = &output.scroll_regions[0];
        assert!(
            if orientation == Orientation::Vertical {
                scroll.max_offset.y
            } else {
                scroll.max_offset.x
            } > 400.0
        );
        area.config = Some(ScrollConfig::default());
        assert!(
            area.build(theme)
                .scroll
                .as_ref()
                .unwrap()
                .scrollbar
                .is_none()
        );
    }
}

#[test]
fn keyboard_scrolling_clamps_emits_scroll_and_leaves_editor_and_modified_keys_alone() {
    use argui_core::{Key, KeyInput, KeyState, Modifiers, Point};
    use argui_ui::{EventHandlerId, EventListener, EventOwnerId, EventType, UiEventKind};
    let palette = shadcn(Color::BLACK);
    let theme = palette.resolve(ColorScheme::Light);
    let root = ScrollArea::new(
        "log",
        "Log",
        200.0,
        Element::text("Content").height(length(1000.0)),
    )
    .build(theme)
    .on(EventListener::new(
        EventType::Scroll,
        EventHandlerId::new(EventOwnerId(1), 0),
    ));
    let mut tree = UiTree::new(root);
    let output = LayoutEngine::new()
        .compute(&mut tree, &mut TextEngine::new(), Size::new(300.0, 400.0))
        .unwrap();
    let node = output.scroll_regions[0].node;
    let mut input = KeyInput {
        key: Key::ArrowDown,
        state: KeyState::Pressed,
        modifiers: Modifiers::default(),
        repeat: false,
        text: None,
    };
    assert!(
        !tree
            .scroll_keyboard(&input, &output.scroll_regions)
            .scroll_changed
    );
    tree.pointer_moved(Point::new(10.0, 10.0), &output.hit_regions);
    tree.primary_pressed(&output.hit_regions);
    assert_eq!(tree.focused_node(), Some(node));
    for (key, expected) in [
        (Key::ArrowDown, 40.0),
        (Key::PageDown, 220.0),
        (Key::Character(" ".into()), 400.0),
        (Key::PageUp, 220.0),
        (Key::ArrowUp, 180.0),
        (Key::Home, 0.0),
        (Key::End, 800.0),
    ] {
        input.key = key;
        let update = tree.scroll_keyboard(&input, &output.scroll_regions);
        assert!(update.scroll_changed);
        assert!(
            matches!(update.events[0].kind, UiEventKind::Scrolled { offset, .. } if offset.y == expected)
        );
        assert_eq!(tree.scroll_offset(node).y, expected);
    }
    assert!(
        !tree
            .scroll_keyboard(&input, &output.scroll_regions)
            .scroll_changed
    );
    input.key = Key::Character(" ".into());
    input.modifiers.shift = true;
    tree.scroll_keyboard(&input, &output.scroll_regions);
    assert_eq!(tree.scroll_offset(node).y, 620.0);
    input.modifiers.shift = false;
    for key in [
        Key::ArrowLeft,
        Key::ArrowRight,
        Key::Tab,
        Key::Character("x".into()),
    ] {
        input.key = key;
        assert!(
            !tree
                .scroll_keyboard(&input, &output.scroll_regions)
                .scroll_changed
        );
    }
    input.key = Key::Home;
    input.modifiers.alt = true;
    assert!(
        !tree
            .scroll_keyboard(&input, &output.scroll_regions)
            .scroll_changed
    );
    input.modifiers.alt = false;
    input.modifiers.control = true;
    assert!(
        !tree
            .scroll_keyboard(&input, &output.scroll_regions)
            .scroll_changed
    );
    input.modifiers.control = false;
    input.state = KeyState::Released;
    assert!(
        !tree
            .scroll_keyboard(&input, &output.scroll_regions)
            .scroll_changed
    );
    input.state = KeyState::Pressed;
    assert!(!tree.scroll_keyboard(&input, &[]).scroll_changed);
    let mut disabled = output.scroll_regions.clone();
    disabled[0].config.enabled = false;
    assert!(!tree.scroll_keyboard(&input, &disabled).scroll_changed);
    for element in [
        argui_widgets::Input::new("log", "text", "", theme.input()).build(),
        argui_widgets::Button::new("log", "Button", theme.button()).build(),
    ] {
        tree.update(element);
        assert!(
            !tree
                .scroll_keyboard(&input, &output.scroll_regions)
                .scroll_changed
        );
    }
}
