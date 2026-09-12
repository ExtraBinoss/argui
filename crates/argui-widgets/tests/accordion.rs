use argui_core::{Color, ColorScheme, Key, KeyInput, KeyState};
use argui_ui::{ClickEvent, Element, UiEvent, UiEventKind, UiTree};
use argui_widgets::{Accordion, AccordionAction, AccordionItem, ChoiceMode, shadcn};

fn event(key: &str, kind: UiEventKind) -> UiEvent {
    UiEvent::new(
        UiTree::new(Element::container([])).node_ids()[0],
        Some(key.into()),
        kind,
    )
}

#[test]
fn disclosures_select_one_or_many_and_closed_panels_unmount() {
    let mut a = AccordionItem::new("a", "A", Element::text("first").keyed("first"));
    a.open = true;
    let b = AccordionItem::new("b", "B", Element::text("second").keyed("second"));
    let mut c = AccordionItem::new("c", "C", Element::text("third"));
    c.enabled = false;
    let mut accordion = Accordion::new("faq", [a, b, c]);
    let click = |id: &str| {
        event(
            &format!("faq::item::{id}::trigger"),
            UiEventKind::Click(ClickEvent::accessibility()),
        )
    };
    assert_eq!(
        accordion.action(&click("b")),
        Some(AccordionAction::Change(vec!["b".into()]))
    );
    accordion.mode = ChoiceMode::Multiple;
    assert_eq!(
        accordion.action(&click("b")),
        Some(AccordionAction::Change(vec!["a".into(), "b".into()]))
    );
    assert_eq!(
        accordion.action(&click("a")),
        Some(AccordionAction::Change(vec![]))
    );
    assert_eq!(accordion.action(&click("c")), None);
    assert_eq!(accordion.action(&click("unknown")), None);
    accordion.mode = ChoiceMode::Single;
    accordion.collapsible = false;
    assert_eq!(accordion.action(&click("a")), None);
    let tree = UiTree::new(accordion.build(shadcn(Color::BLACK).resolve(ColorScheme::Light)));
    assert!(tree.semantic_diagnostics().is_empty());
    assert!(
        tree.node_ids()
            .iter()
            .any(|node| tree.key(*node) == Some("first"))
    );
    assert!(
        !tree
            .node_ids()
            .iter()
            .any(|node| tree.key(*node) == Some("second"))
    );
    for (key, expected) in [
        (Key::ArrowDown, Some("b")),
        (Key::ArrowUp, Some("b")),
        (Key::End, Some("b")),
        (Key::Home, Some("a")),
        (Key::Tab, None),
    ] {
        let input = KeyInput {
            key,
            state: KeyState::Pressed,
            repeat: false,
            modifiers: Default::default(),
            text: None,
        };
        assert_eq!(
            accordion.action(&event(
                "faq::item::a::trigger",
                UiEventKind::KeyInput(input)
            )),
            expected.map(|id| AccordionAction::Focus(id.into()))
        );
    }
}
