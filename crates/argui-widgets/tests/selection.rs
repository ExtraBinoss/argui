use argui_core::{Color, ColorScheme};
use argui_ui::{Role, UiEvent, UiEventKind, UiTree};
use argui_widgets::{Checkbox, RadioGroup, RadioOption, Switch, shadcn};

fn click(key: &str) -> UiEvent {
    let tree = UiTree::new(argui_ui::Element::container([]));
    UiEvent {
        target: tree.node_ids()[0],
        key: Some(key.into()),
        kind: UiEventKind::Clicked,
    }
}

#[test]
fn boolean_and_exclusive_controls_publish_controlled_state() {
    let themes = shadcn(Color::rgb(0.2, 0.5, 0.9));
    let theme = themes.resolve(ColorScheme::Dark);
    let check = Checkbox::new("metrics", "Metrics", true).build(theme);
    assert_eq!(check.semantics.as_ref().unwrap().role, Role::CheckBox);
    assert_eq!(check.semantics.as_ref().unwrap().state.checked, Some(true));
    let switch = Switch::new("profile", "Profile", false)
        .enabled(false)
        .build(theme);
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
    assert_eq!(
        RadioGroup::selection("quality", &click("quality::option::1")),
        Some(1)
    );
    assert_eq!(
        RadioGroup::selection("other", &click("quality::option::1")),
        None
    );
}
