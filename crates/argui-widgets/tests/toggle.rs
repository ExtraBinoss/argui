use argui_core::{Color, ColorScheme};
use argui_ui::{ClickEvent, Element, UiEvent, UiEventKind, UiTree};
use argui_widgets::{Toggle, shadcn};

#[test]
fn pressed_state_is_accessible_and_only_enabled_clicks_change_it() {
    let theme = shadcn(Color::BLACK);
    let node = UiTree::new(Element::container([])).node_ids()[0];
    for pressed in [false, true] {
        for enabled in [false, true] {
            for outline in [false, true] {
                let mut toggle = Toggle::new("bold", "Bold", pressed);
                toggle.enabled = enabled;
                toggle.outline = outline;
                let root = toggle.build(theme.resolve(ColorScheme::Light));
                let semantics = root.semantics.as_ref().unwrap();
                assert_eq!(semantics.state.pressed, Some(pressed));
                assert_eq!(semantics.state.checked, None);
                assert_eq!(semantics.state.disabled, !enabled);
                for (key, kind, click) in [
                    (
                        "bold",
                        UiEventKind::Click(ClickEvent::accessibility()),
                        true,
                    ),
                    (
                        "other",
                        UiEventKind::Click(ClickEvent::accessibility()),
                        false,
                    ),
                    ("bold", UiEventKind::Focused, false),
                ] {
                    assert_eq!(
                        toggle.action(&UiEvent::new(node, Some(key.into()), kind)),
                        (enabled && click).then_some(!pressed)
                    );
                }
            }
        }
    }
}
