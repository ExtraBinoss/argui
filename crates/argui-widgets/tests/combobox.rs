use argui_core::{Color, ColorScheme, Key, KeyInput, KeyState};
use argui_ui::{ClickEvent, Element, UiEvent, UiEventKind, UiTree};
use argui_widgets::{Combobox, ComboboxAction, SelectOption, shadcn};

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
fn filtering_keeps_source_indices_and_active_relations_point_at_mounted_options() {
    let mut combo = Combobox::new(
        "language",
        "Language",
        [
            SelectOption::new("Rust"),
            SelectOption::new("Ruby").enabled(false),
            SelectOption::new("Python"),
        ],
    );
    let theme = shadcn(Color::BLACK);
    assert_eq!(
        combo.action(&event("language", key(Key::ArrowDown))),
        Some(ComboboxAction::Open)
    );
    assert_eq!(
        combo.action(&event("language", UiEventKind::TextChanged("ru".into()))),
        Some(ComboboxAction::Query("ru".into()))
    );
    combo.open = true;
    combo.query = "RU".into();
    combo.highlighted = Some(999);
    assert_eq!(combo.visible_indices(), vec![0, 1]);
    assert_eq!(
        combo.action(&event("language", key(Key::Enter))),
        Some(ComboboxAction::Select(0))
    );
    assert_eq!(
        combo.action(&event("language", key(Key::ArrowDown))),
        Some(ComboboxAction::Highlight(0))
    );
    assert_eq!(
        combo.action(&event("language", key(Key::ArrowUp))),
        Some(ComboboxAction::Highlight(0))
    );
    assert_eq!(
        combo.action(&event(
            "language::option::1",
            UiEventKind::Click(ClickEvent::accessibility())
        )),
        None
    );
    combo.selected = Some(0);
    for query in ["ru", "python", "nothing"] {
        combo.query = query.into();
        let tree = UiTree::new(combo.build(theme.resolve(ColorScheme::Light)));
        assert!(tree.semantic_diagnostics().is_empty());
        let semantic = tree.semantic_tree(&[], 1.0);
        assert_eq!(
            semantic
                .nodes
                .iter()
                .filter(|node| node.semantics.focus_policy.is_tab_stop())
                .count(),
            1
        );
    }
    combo.query = "python".into();
    combo.highlighted = Some(2);
    assert_eq!(
        combo.action(&event(
            "language::option::2",
            UiEventKind::Click(ClickEvent::accessibility())
        )),
        Some(ComboboxAction::Select(2))
    );
    for close in [key(Key::Escape), key(Key::Tab)] {
        assert_eq!(
            combo.action(&event("language", close)),
            Some(ComboboxAction::Close)
        );
    }
    assert_eq!(
        combo.action(&event("language::list", UiEventKind::DismissRequested)),
        Some(ComboboxAction::Close)
    );
    assert_eq!(combo.action(&event("language", key(Key::Home))), None);
    assert_eq!(combo.action(&event("other", key(Key::Enter))), None);
    combo.open = false;
    assert_eq!(combo.action(&event("language", key(Key::Escape))), None);
    assert_eq!(
        combo.action(&event(
            "language",
            UiEventKind::Click(ClickEvent::accessibility())
        )),
        Some(ComboboxAction::Open)
    );
    combo.enabled = false;
    assert_eq!(combo.action(&event("language", key(Key::ArrowDown))), None);
    assert_eq!(
        combo.build(theme.resolve(ColorScheme::Dark)).children.len(),
        1
    );
}
