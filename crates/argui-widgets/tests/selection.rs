use argui_core::{Color, ColorScheme};
use argui_ui::{Role, UiEvent, UiEventKind, UiTree, UserSelect};
use argui_widgets::{
    Checkbox, RadioGroup, RadioGroupAction, RadioGroupBehavior, RadioOption, Switch, ToggleAction,
    ToggleBehavior, shadcn,
};

fn click(key: &str) -> UiEvent {
    let tree = UiTree::new(argui_ui::Element::container([]));
    UiEvent::new(tree.node_ids()[0], Some(key.into()), UiEventKind::Clicked)
}

#[test]
fn boolean_and_exclusive_controls_publish_controlled_state() {
    let themes = shadcn(Color::rgb(0.2, 0.5, 0.9));
    let theme = themes.resolve(ColorScheme::Dark);
    let check = Checkbox::new("metrics", "Metrics", true).build(theme);
    assert_eq!(check.user_select, UserSelect::None);
    assert_eq!(check.semantics.as_ref().unwrap().role, Role::CheckBox);
    assert_eq!(check.semantics.as_ref().unwrap().state.checked, Some(true));
    let switch = Switch::new("profile", "Profile", false)
        .enabled(false)
        .build(theme);
    assert_eq!(switch.user_select, UserSelect::None);
    assert_eq!(switch.semantics.as_ref().unwrap().role, Role::Switch);
    assert!(switch.semantics.as_ref().unwrap().state.disabled);

    let group = RadioGroup::new(
        "quality",
        "Quality",
        [RadioOption::new("High"), RadioOption::new("Low")],
        Some(1),
    )
    .build(theme);
    assert_eq!(group.semantics.as_ref().unwrap().role, Role::Group);
    assert_eq!(group.children.len(), 2);
    assert!(
        group
            .children
            .iter()
            .all(|option| option.user_select == UserSelect::None)
    );
    let behavior = RadioGroupBehavior::new(
        "quality",
        "Quality",
        [("High".into(), true), ("Low".into(), true)],
        Some(1),
    );
    assert_eq!(
        behavior.action(&click("quality::option::1")),
        Some(RadioGroupAction::Select(1))
    );
    assert_eq!(behavior.action(&click("other::option::1")), None);
    let toggle = ToggleBehavior::new("metrics", "Metrics", Role::CheckBox, true);
    assert_eq!(toggle.action(&click("metrics")), Some(ToggleAction::Toggle));
    assert_eq!(toggle.enabled(false).action(&click("metrics")), None);

    let disabled = RadioGroupBehavior::new(
        "quality",
        "Quality",
        [("High".into(), true), ("Low".into(), false)],
        None,
    );
    assert_eq!(disabled.action(&click("quality::option::1")), None);
}
