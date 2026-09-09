use argui_core::{Color, Key, KeyInput, KeyState};
use argui_ui::{ActionId, ActionInvocation, ActionState, Element, UiEvent, UiEventKind, UiTree};
use argui_widgets::{CommandPalette, MenuItem, MenuResponse, shadcn};

#[test]
fn palette_presence_opens_immediately_retains_exit_and_can_reopen() {
    let themes = shadcn(Color::BLACK);
    let theme = themes.resolve(argui_core::ColorScheme::Dark);
    let mut presence = argui_widgets::Presence::default().fade_in(false);
    let build = |presence: &argui_widgets::Presence| {
        palette("", false)
            .presence(presence)
            .build(Element::text("Open"), theme)
    };
    presence.set_open(true, false);
    let root = build(&presence);
    let content = &root.children[1];
    let tree = UiTree::new(content.clone());
    let node = tree.node_ids()[0];
    assert_eq!(
        tree.resolved_layer(node, content, content.layer.as_ref().unwrap())
            .opacity,
        1.0
    );
    assert_eq!(
        content.layer.as_ref().unwrap().shadows,
        theme.overlay_shadows
    );
    assert!(presence.animating());
    presence.set_open(false, false);
    assert!(build(&presence).children[1].semantic_hidden);
    presence.advance(argui_animation::Duration::from_millis(50));
    assert!(presence.visible());
    presence.set_open(true, false);
    assert!(!build(&presence).children[1].semantic_hidden);
    presence.set_open(false, true);
    assert_eq!(build(&presence).children.len(), 1);
}

fn palette(query: &str, open: bool) -> CommandPalette {
    CommandPalette::new(
        "palette",
        open,
        query,
        vec![
            MenuItem::new(
                "disabled",
                ActionInvocation::new(ActionId("disabled")),
                ActionState::new("Save unavailable").enabled(false),
            ),
            MenuItem::new(
                "save",
                ActionInvocation::new(ActionId("save")),
                ActionState::new("Save draft"),
            ),
            MenuItem::new(
                "undo",
                ActionInvocation::new(ActionId("undo")),
                ActionState::new("Undo"),
            ),
        ],
    )
}
#[test]
fn palette_filters_case_insensitively_and_enter_skips_disabled_commands() {
    let mut event = UiEvent::new(
        UiTree::new(Element::container([])).node_ids()[0],
        Some("palette::query".into()),
        UiEventKind::KeyInput(KeyInput {
            key: Key::Enter,
            state: KeyState::Pressed,
            modifiers: Default::default(),
            repeat: false,
            text: None,
        }),
    );
    assert_eq!(
        palette("SAVE", true).response(&event),
        Some(MenuResponse::Invoke(ActionInvocation::new(ActionId(
            "save"
        ))))
    );
    assert!(palette("absent", true).response(&event).is_none());
    assert!(palette("save", false).response(&event).is_none());
    if let UiEventKind::KeyInput(input) = &mut event.kind {
        input.state = KeyState::Released;
    }
    assert!(palette("save", true).response(&event).is_none());
}
#[test]
fn empty_results_keep_a_real_search_input_and_explain_the_empty_state() {
    let themes = shadcn(Color::WHITE);
    let theme = themes.resolve(argui_core::ColorScheme::Dark);
    let tree = UiTree::new(palette("absent", true).build(Element::text("Commands"), theme));
    let snapshot = tree.semantic_tree(&[], 1.0);
    assert!(
        snapshot
            .nodes
            .iter()
            .any(|node| node.semantics.label.as_deref() == Some("Search commands"))
    );
    assert!(
        snapshot
            .nodes
            .iter()
            .any(|node| node.semantics.label.as_deref() == Some("No matching commands"))
    );
}
