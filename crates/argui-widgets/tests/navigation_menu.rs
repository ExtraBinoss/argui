use argui_core::{Color, ColorScheme, Key, KeyInput, KeyState};
use argui_ui::{ClickEvent, Element, UiEvent, UiEventKind, UiTree};
use argui_widgets::{NavigationItem, NavigationMenu, NavigationMenuAction, shadcn};

fn event(key: &str, kind: UiEventKind) -> UiEvent {
    UiEvent::new(
        UiTree::new(Element::container([])).node_ids()[0],
        Some(key.into()),
        kind,
    )
}
fn key(value: Key) -> UiEventKind {
    UiEventKind::KeyInput(KeyInput {
        key: value,
        state: KeyState::Pressed,
        repeat: false,
        modifiers: Default::default(),
        text: None,
    })
}

#[test]
fn navigation_distinguishes_links_panels_and_disabled_triggers() {
    let mut docs = NavigationItem::new("docs", "Docs");
    docs.panel = Some(Element::text("Documentation"));
    let mut home = NavigationItem::new("home", "Home");
    home.current = true;
    let mut disabled = NavigationItem::new("disabled", "Disabled");
    disabled.panel = Some(Element::text("Hidden"));
    disabled.enabled = false;
    let mut menu = NavigationMenu::new("nav", "Main navigation", [home, docs, disabled]);
    let click = |id: &str| {
        event(
            &format!("nav::item::{id}"),
            UiEventKind::Click(ClickEvent::accessibility()),
        )
    };
    assert_eq!(
        menu.action(&click("home")),
        Some(NavigationMenuAction::Activate("home".into()))
    );
    assert_eq!(
        menu.action(&click("docs")),
        Some(NavigationMenuAction::Open {
            id: "docs".into(),
            focus_panel: false
        })
    );
    assert_eq!(
        menu.action(&event("nav::item::docs", key(Key::ArrowDown))),
        Some(NavigationMenuAction::Open {
            id: "docs".into(),
            focus_panel: true
        })
    );
    assert_eq!(menu.action(&click("disabled")), None);
    assert_eq!(
        menu.action(&event("nav::item::home", key(Key::ArrowRight))),
        Some(NavigationMenuAction::Focus("docs".into()))
    );
    menu.rtl = true;
    assert_eq!(
        menu.action(&event("nav::item::home", key(Key::ArrowLeft))),
        Some(NavigationMenuAction::Focus("docs".into()))
    );
    let palette = shadcn(Color::BLACK);
    for open in [None, Some("docs".into()), Some("disabled".into())] {
        menu.open = open;
        assert!(
            UiTree::new(menu.build(palette.resolve(ColorScheme::Light)))
                .semantic_diagnostics()
                .is_empty()
        );
    }
    menu.open = Some("docs".into());
    assert_eq!(
        menu.action(&click("docs")),
        Some(NavigationMenuAction::Close)
    );
    assert_eq!(
        menu.action(&event(
            "nav::item::docs::content",
            UiEventKind::DismissRequested
        )),
        Some(NavigationMenuAction::Close)
    );
    assert_eq!(
        menu.action(&event("nav::item::docs", key(Key::Escape))),
        Some(NavigationMenuAction::Close)
    );
    assert_eq!(
        menu.action(&event("nav::item::home", UiEventKind::Focused)),
        None
    );
}

#[test]
fn panel_triggers_reserve_their_full_width_between_links() {
    let theme = shadcn(Color::BLACK);
    let mut guide = NavigationItem::new("guide", "Guide");
    guide.panel = Some(Element::text("Getting started"));
    let menu = NavigationMenu::new(
        "docs",
        "Documentation",
        [
            NavigationItem::new("overview", "Overview"),
            guide,
            NavigationItem::new("examples", "Examples"),
        ],
    );
    let mut tree = UiTree::new(menu.build(theme.resolve(ColorScheme::Light)));
    let output = argui_layout::LayoutEngine::new()
        .compute(
            &mut tree,
            &mut argui_text::TextEngine::new(),
            argui_core::Size::new(600.0, 300.0),
        )
        .unwrap();
    let bounds = |key: &str| {
        output
            .nodes
            .iter()
            .find(|node| tree.key(node.node) == Some(key))
            .unwrap()
            .bounds
    };
    let guide = bounds("docs::item::guide");
    let next = bounds("docs::item::examples");
    assert!(
        guide.origin.x + guide.size.width + 3.0 <= next.origin.x,
        "{guide:?}, {next:?}"
    );
}
