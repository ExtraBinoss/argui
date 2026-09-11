use argui_core::{Color, ColorScheme, Key, KeyInput, KeyState, Size};
use argui_layout::LayoutEngine;
use argui_text::TextEngine;
use argui_ui::{
    ClickEvent, Element, EventHandlerId, EventListener, EventOwnerId, EventType, FocusRequest,
    Role, UiEvent, UiEventKind, UiTree,
};
use argui_widgets::{Breadcrumb, BreadcrumbLink, shadcn};

#[test]
fn ancestors_are_links_and_the_current_page_and_separators_are_not_interactive() {
    let themes = shadcn(Color::WHITE);
    let breadcrumb = Breadcrumb::new("path", [BreadcrumbLink::new("home", "Accueil")], "Projets")
        .label("Emplacement")
        .current_description("Page actuelle")
        .separator("›");
    let tree = UiTree::new(breadcrumb.build(themes.resolve(ColorScheme::Light)));
    let semantic = tree.semantic_tree(&[], 1.0);
    assert_eq!(semantic.nodes.len(), 3);
    assert_eq!(
        semantic.nodes[0].semantics.label.as_deref(),
        Some("Emplacement")
    );
    let current = semantic
        .nodes
        .iter()
        .find(|node| node.semantics.label.as_deref() == Some("Projets"))
        .unwrap();
    assert_eq!(
        current.semantics.description.as_deref(),
        Some("Page actuelle")
    );
    assert!(!current.semantics.focus_policy.is_focusable());
    let links: Vec<_> = semantic
        .nodes
        .iter()
        .filter(|node| node.semantics.role == Role::Link)
        .collect();
    assert_eq!(links.len(), 1);
    assert!(links[0].semantics.focus_policy.is_tab_stop());
    let current_only = Breadcrumb::new("root", [], "Home").build(themes.resolve(ColorScheme::Dark));
    assert_eq!(current_only.children.len(), 1);
}

#[test]
fn events_return_stable_ancestor_ids_without_activating_the_current_page() {
    let breadcrumb = Breadcrumb::new(
        "path",
        [
            BreadcrumbLink::new("home", "Home"),
            BreadcrumbLink::new("all", "All"),
        ],
        "Current",
    );
    let node = UiTree::new(Element::container([])).node_ids()[0];
    for (key, expected) in [
        (Some("path::link::home"), Some("home")),
        (Some("path::link::all"), Some("all")),
        (Some("path::current"), None),
        (Some("unrelated"), None),
        (None, None),
    ] {
        assert_eq!(
            breadcrumb.action(&UiEvent::new(
                node,
                key.map(Into::into),
                UiEventKind::Click(ClickEvent::accessibility())
            )),
            expected
        );
        assert_eq!(
            breadcrumb.action(&UiEvent::new(
                node,
                key.map(Into::into),
                UiEventKind::Focused
            )),
            None
        );
    }
}

#[test]
fn links_activate_with_enter_and_do_not_consume_space() {
    let themes = shadcn(Color::WHITE);
    let breadcrumb = Breadcrumb::new("path", [BreadcrumbLink::new("home", "Home")], "Current");
    let root = breadcrumb
        .build(themes.resolve(ColorScheme::Light))
        .on(EventListener::new(
            EventType::Click,
            EventHandlerId::new(EventOwnerId(1), 0),
        ));
    let mut tree = UiTree::new(root);
    let output = LayoutEngine::new()
        .compute(&mut tree, &mut TextEngine::new(), Size::new(300.0, 100.0))
        .unwrap();
    tree.sync_focus(
        &output.hit_regions,
        Some(FocusRequest::Focus(breadcrumb.link_key("home").into())),
    );
    for (key, state, expected) in [
        (Key::Enter, KeyState::Pressed, Some("home")),
        (Key::Enter, KeyState::Released, None),
        (Key::Character(" ".into()), KeyState::Pressed, None),
        (Key::Character(" ".into()), KeyState::Released, None),
    ] {
        let update = tree.key_input(
            &KeyInput {
                key,
                state,
                modifiers: Default::default(),
                repeat: false,
                text: None,
            },
            &output.hit_regions,
        );
        assert_eq!(
            update
                .events
                .iter()
                .find_map(|event| breadcrumb.action(event)),
            expected
        );
    }
}

#[test]
fn long_paths_wrap_within_narrow_containers() {
    const FONT: &[u8] = include_bytes!("../../argui-web-demo/assets/fonts/NotoSans-Regular.ttf");
    let themes = shadcn(Color::WHITE);
    let breadcrumb = Breadcrumb::new(
        "path",
        [
            BreadcrumbLink::new("home", "Home"),
            BreadcrumbLink::new("library", "A shared component library with a longer name"),
        ],
        "Current page with a longer title",
    );
    for width in [180.0, 480.0] {
        let mut text =
            TextEngine::from_embedded_fonts([FONT], "Noto Sans", "Noto Sans", "Noto Sans");
        let mut tree = UiTree::new(breadcrumb.build(themes.resolve(ColorScheme::Dark)));
        let output = LayoutEngine::new()
            .compute(&mut tree, &mut text, Size::new(width, 500.0))
            .unwrap();
        for node in &output.nodes {
            assert!(
                node.bounds.origin.x + node.bounds.size.width <= width + 0.1,
                "overflow at {width}: {:?}",
                node.bounds
            );
        }
        let prepared = text.prepare(&output.text, 1.0);
        for (index, block) in output.text.blocks().iter().enumerate() {
            assert_eq!(
                prepared
                    .glyphs
                    .iter()
                    .filter(|glyph| glyph.block == index)
                    .map(|glyph| glyph.end)
                    .max(),
                Some(block.content.as_str().len()),
                "truncated {} in {:?}",
                block.content.as_str(),
                block.bounds
            );
        }
    }
}

#[test]
#[should_panic(expected = "breadcrumb IDs must be unique")]
fn duplicate_ancestor_ids_are_rejected() {
    let _ = Breadcrumb::new(
        "path",
        [
            BreadcrumbLink::new("same", "First"),
            BreadcrumbLink::new("same", "Second"),
        ],
        "Current",
    );
}
