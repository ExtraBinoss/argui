use argui_core::{Color, ColorScheme, Key, KeyInput, KeyState};
use argui_ui::{ClickEvent, Element, UiEvent, UiEventKind, UiTree};
use argui_widgets::{NativeSelect, SelectAction, SelectOption, Typeahead, shadcn};

#[test]
fn compact_select_exposes_form_state_and_reuses_selection_and_typeahead() {
    let mut select = NativeSelect::new(
        "os",
        "OS",
        [SelectOption::new("Linux"), SelectOption::new("Windows")],
        Some(0),
    );
    select.required = true;
    let node = UiTree::new(Element::container([])).node_ids()[0];
    for enabled in [true, false] {
        select.enabled = enabled;
        select.open = true;
        let element = select.build(shadcn(Color::BLACK).resolve(ColorScheme::Light));
        let semantics = element.children[0].semantics.as_ref().unwrap();
        assert!(semantics.state.required);
        assert_eq!(semantics.state.disabled, !enabled);
        assert_eq!(element.children.len(), if enabled { 2 } else { 1 });
        let event = UiEvent::new(
            node,
            Some("os".into()),
            UiEventKind::Click(ClickEvent::accessibility()),
        );
        assert_eq!(
            select.action(&event),
            enabled.then_some(SelectAction::Toggle)
        );
        let event = UiEvent::new(
            node,
            Some("os".into()),
            UiEventKind::KeyInput(KeyInput {
                key: Key::Character("w".into()),
                state: KeyState::Pressed,
                repeat: false,
                modifiers: Default::default(),
                text: None,
            }),
        );
        assert_eq!(
            select.search(&event, &mut Typeahead::default(), Default::default()),
            enabled.then_some(SelectAction::Highlight(1))
        );
    }
    select.selected = None;
    assert!(
        select
            .build(shadcn(Color::BLACK).resolve(ColorScheme::Dark))
            .children[0]
            .semantics
            .as_ref()
            .unwrap()
            .value
            .is_none()
    );
}
