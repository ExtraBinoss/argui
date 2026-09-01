use argui_core::{Color, ColorScheme};
use argui_ui::{CursorIcon, Element, Role, UiEvent, UiEventKind, UiTree};
use argui_widgets::{Tab, Tabs, TabsAction, TabsBehavior, TabsPart, shadcn};

fn click(key: &str) -> UiEvent {
    let tree = UiTree::new(Element::container([]));
    UiEvent {
        target: tree.node_ids()[0],
        key: Some(key.into()),
        kind: UiEventKind::Clicked,
    }
}

#[test]
fn tabs_mount_only_the_selected_panel_and_decode_selection() {
    let themes = shadcn(Color::rgb(0.2, 0.5, 0.9));
    let tabs = Tabs::new(
        "settings",
        [
            Tab::new("General", Element::text("general")),
            Tab::new("GPU", Element::text("gpu")),
        ],
        9,
    )
    .build(themes.resolve(ColorScheme::Light));
    assert_eq!(tabs.children.len(), 2);
    assert_eq!(
        tabs.children[0].semantics.as_ref().unwrap().role,
        Role::TabList
    );
    assert_eq!(
        tabs.children[1].semantics.as_ref().unwrap().role,
        Role::TabPanel
    );
    assert_eq!(tabs.children[1].key.as_deref(), Some("settings::panel::1"));
    let behavior = TabsBehavior::new(
        "settings",
        [("General".into(), true), ("GPU".into(), true)],
        1,
    );
    assert_eq!(
        behavior.action(&click("settings::tab::0")),
        Some(TabsAction::Select(0))
    );
    assert_eq!(behavior.action(&click("other::tab::0")), None);
    assert_eq!(
        behavior.action(&UiEvent {
            kind: UiEventKind::Focused,
            ..click("settings::tab::0")
        }),
        None
    );

    let disabled = TabsBehavior::new("disabled", [("GPU".into(), false)], 0)
        .decorate(TabsPart::Trigger(0), Element::container([]));
    assert_eq!(
        disabled.interaction.as_ref().unwrap().cursor,
        CursorIcon::NotAllowed
    );
}
