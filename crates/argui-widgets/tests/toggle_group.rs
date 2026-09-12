use argui_core::{Color, ColorScheme, Key, KeyInput, KeyState};
use argui_ui::{ClickEvent, Element, Orientation, UiEvent, UiEventKind, UiTree};
use argui_widgets::{ChoiceMode, Toggle, ToggleGroup, ToggleGroupAction, shadcn};

fn event(key: &str, kind: UiEventKind) -> UiEvent {
    UiEvent::new(
        UiTree::new(Element::container([])).node_ids()[0],
        Some(key.into()),
        kind,
    )
}

#[test]
fn selection_cardinality_and_roving_focus_survive_disabled_items_and_stale_active_ids() {
    let mut disabled = Toggle::new("b", "B", false);
    disabled.enabled = false;
    let mut group = ToggleGroup::new(
        "format",
        "Format",
        [
            Toggle::new("a", "A", true),
            disabled,
            Toggle::new("c", "C", false),
        ],
    );
    let click = |id: &str| {
        event(
            &format!("format::item::{id}"),
            UiEventKind::Click(ClickEvent::accessibility()),
        )
    };
    assert_eq!(
        group.action(&click("c")),
        Some(ToggleGroupAction::Change(vec!["c".into()]))
    );
    group.mode = ChoiceMode::Multiple;
    assert_eq!(
        group.action(&click("c")),
        Some(ToggleGroupAction::Change(vec!["a".into(), "c".into()]))
    );
    assert_eq!(
        group.action(&click("a")),
        Some(ToggleGroupAction::Change(vec![]))
    );
    assert_eq!(group.action(&click("b")), None);
    assert_eq!(group.action(&click("missing")), None);
    let palette = shadcn(Color::BLACK);
    for active in [None, Some("c".into()), Some("missing".into())] {
        for orientation in [Orientation::Horizontal, Orientation::Vertical] {
            group.active = active.clone();
            group.orientation = orientation;
            let root = group.build(palette.resolve(ColorScheme::Dark));
            assert_eq!(
                root.children
                    .iter()
                    .filter(|child| child
                        .interaction
                        .as_ref()
                        .unwrap()
                        .focus_policy
                        .is_tab_stop())
                    .count(),
                1
            );
        }
    }
    for (rtl, key, expected) in [
        (false, Key::ArrowRight, "c"),
        (true, Key::ArrowLeft, "c"),
        (false, Key::Home, "a"),
        (false, Key::End, "c"),
    ] {
        group.rtl = rtl;
        group.orientation = Orientation::Horizontal;
        let key = UiEventKind::KeyInput(KeyInput {
            key,
            state: KeyState::Pressed,
            repeat: false,
            modifiers: Default::default(),
            text: None,
        });
        assert_eq!(
            group.action(&event("format::item::a", key)),
            Some(ToggleGroupAction::Focus(expected.into()))
        );
    }
}
