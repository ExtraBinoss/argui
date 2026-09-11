use argui_core::{Color, ColorScheme, Key, KeyInput, KeyState, Size};
use argui_layout::LayoutEngine;
use argui_text::TextEngine;
use argui_ui::{
    ClickEvent, Element, EventHandlerId, EventListener, EventOwnerId, EventType, FocusRequest,
    Role, UiEvent, UiEventKind, UiTree,
};
use argui_widgets::{Pagination, PaginationLabels, shadcn};

fn event(key: Option<String>, kind: UiEventKind) -> UiEvent {
    UiEvent::new(UiTree::new(Element::container([])).node_ids()[0], key, kind)
}

#[test]
fn visible_ranges_clamp_and_stay_bounded_without_overflow() {
    let themes = shadcn(Color::WHITE);
    let theme = themes.resolve(ColorScheme::Dark);
    for (page, total, expected, pages) in [
        (7, 0, 0, vec![]),
        (0, 1, 1, vec![1]),
        (99, 4, 4, vec![1, 2, 3, 4]),
        (1, 12, 1, vec![1, 2, 3, 4, 5, 12]),
        (6, 12, 6, vec![1, 5, 6, 7, 12]),
        (12, 12, 12, vec![1, 8, 9, 10, 11, 12]),
        (
            usize::MAX,
            usize::MAX,
            usize::MAX,
            vec![
                1,
                usize::MAX - 4,
                usize::MAX - 3,
                usize::MAX - 2,
                usize::MAX - 1,
                usize::MAX,
            ],
        ),
    ] {
        let pagination = Pagination::new("pages", page, total);
        assert_eq!(pagination.page(), expected);
        let root = pagination.build(theme);
        assert!(root.children.len() <= 9);
        let visible: Vec<_> = root
            .children
            .iter()
            .filter_map(|child| {
                child
                    .key
                    .as_deref()?
                    .strip_prefix("pages::page::")?
                    .parse::<usize>()
                    .ok()
            })
            .collect();
        assert_eq!(visible, pages);
        let semantic = UiTree::new(root).semantic_tree(&[], 1.0);
        let current: Vec<_> = semantic
            .nodes
            .iter()
            .filter(|node| node.semantics.description.as_deref() == Some("Current page"))
            .collect();
        assert_eq!(current.len(), usize::from(total > 0));
        assert_eq!(
            semantic
                .nodes
                .iter()
                .filter(|node| node.semantics.role == Role::Button)
                .count(),
            pages.len() + 2
        );
    }
}

#[test]
fn only_enabled_visible_destinations_change_the_controlled_page() {
    for (page, total) in [(0, 0), (1, 1), (1, 12), (6, 12), (12, 12)] {
        let pagination = Pagination::new("pages", page, total);
        for (key, expected) in [
            (pagination.previous_key(), (page > 1).then(|| page - 1)),
            (pagination.next_key(), (page < total).then_some(page + 1)),
            (pagination.page_key(page), None),
            (pagination.page_key(1), (page > 1).then_some(1)),
            (pagination.page_key(11), (page == 12).then_some(11)),
            (pagination.page_key(0), None),
            ("other::next".into(), None),
        ] {
            let click = event(
                Some(key.clone()),
                UiEventKind::Click(ClickEvent::accessibility()),
            );
            assert_eq!(pagination.action(&click), expected, "{page}/{total}: {key}");
            assert_eq!(pagination.clone().enabled(false).action(&click), None);
            assert_eq!(
                pagination.action(&event(Some(key), UiEventKind::Focused)),
                None
            );
        }
        assert_eq!(
            pagination.action(&event(
                None,
                UiEventKind::Click(ClickEvent::accessibility())
            )),
            None
        );
    }
}

#[test]
fn translated_navigation_wraps_without_clipping_and_disabled_controls_leave_tab_order() {
    let themes = shadcn(Color::WHITE);
    let pagination = Pagination::new("pages", 6, 12)
        .enabled(false)
        .labels(PaginationLabels {
            navigation: "Pages des résultats".into(),
            previous: "Précédent".into(),
            next: "Suivant".into(),
            page: "Page".into(),
            current: "Page actuelle".into(),
        });
    let mut tree = UiTree::new(pagination.build(themes.resolve(ColorScheme::Light)));
    let output = LayoutEngine::new()
        .compute(&mut tree, &mut TextEngine::new(), Size::new(280.0, 300.0))
        .unwrap();
    for node in &output.nodes {
        assert!(node.bounds.origin.x + node.bounds.size.width <= 280.1);
    }
    let semantics = tree.semantic_tree(&output.semantic_bounds, 1.0);
    assert!(
        semantics
            .nodes
            .iter()
            .any(|node| node.semantics.label.as_deref() == Some("Pages des résultats"))
    );
    for node in semantics
        .nodes
        .iter()
        .filter(|node| node.semantics.role == Role::Button)
    {
        assert!(node.semantics.state.disabled);
        assert!(!node.semantics.focus_policy.is_tab_stop());
    }
    assert!(
        semantics
            .nodes
            .iter()
            .any(|node| node.semantics.description.as_deref() == Some("Page actuelle"))
    );
}

#[test]
fn keyboard_activation_uses_enter_and_space_with_retained_focus() {
    let themes = shadcn(Color::WHITE);
    let pagination = Pagination::new("pages", 1, 12);
    let root = pagination
        .build(themes.resolve(ColorScheme::Light))
        .on(EventListener::new(
            EventType::Click,
            EventHandlerId::new(EventOwnerId(1), 0),
        ));
    let mut tree = UiTree::new(root);
    let output = LayoutEngine::new()
        .compute(&mut tree, &mut TextEngine::new(), Size::new(700.0, 100.0))
        .unwrap();
    tree.sync_focus(
        &output.hit_regions,
        Some(FocusRequest::Focus(pagination.next_key().into())),
    );
    for (key, state, expected) in [
        (Key::Enter, KeyState::Pressed, Some(2)),
        (Key::Enter, KeyState::Released, None),
        (Key::Character(" ".into()), KeyState::Pressed, None),
        (Key::Character(" ".into()), KeyState::Released, Some(2)),
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
                .find_map(|event| pagination.action(event)),
            expected
        );
    }
}

#[test]
fn multi_digit_pages_remain_complete_with_the_gallery_font() {
    const FONT: &[u8] = include_bytes!("../../argui-web-demo/assets/fonts/NotoSans-Regular.ttf");
    let themes = shadcn(Color::WHITE);
    let mut text = TextEngine::from_embedded_fonts([FONT], "Noto Sans", "Noto Sans", "Noto Sans");
    for (page, total) in [(12, 12), (50, 100), (usize::MAX, usize::MAX)] {
        let mut tree = UiTree::new(
            Pagination::new("pages", page, total).build(themes.resolve(ColorScheme::Light)),
        );
        let output = LayoutEngine::new()
            .compute(&mut tree, &mut text, Size::new(440.0, 400.0))
            .unwrap();
        let prepared = text.prepare(&output.text, 1.0);
        for (index, block) in output.text.blocks().iter().enumerate() {
            if block.content.as_str().parse::<usize>().is_ok() {
                assert_eq!(
                    prepared
                        .glyphs
                        .iter()
                        .filter(|glyph| glyph.block == index)
                        .map(|glyph| glyph.end)
                        .max(),
                    Some(block.content.as_str().len()),
                    "truncated {}",
                    block.content.as_str()
                );
            }
        }
    }
}
