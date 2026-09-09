use argui_core::{Color, ColorScheme, Transform2D};
use argui_ui::{Element, Orientation, Role, UiEvent, UiEventKind, UiTree, UserSelect};
use argui_widgets::{
    Checkbox, RadioGroup, RadioGroupAction, RadioGroupBehavior, RadioGroupPart, RadioOption,
    Switch, ToggleAction, ToggleBehavior, TogglePart, shadcn,
};

fn click(key: &str) -> UiEvent {
    let tree = UiTree::new(argui_ui::Element::container([]));
    UiEvent::new(
        tree.node_ids()[0],
        Some(key.into()),
        UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
    )
}

#[test]
fn boolean_and_exclusive_controls_publish_controlled_state() {
    let themes = shadcn(Color::srgb(0.2, 0.5, 0.9));
    let theme = themes.resolve(ColorScheme::Dark);
    let check = Checkbox::new("metrics", "Metrics", argui_ui::CheckedState::Checked).build(theme);
    assert_eq!(check.user_select, UserSelect::None);
    assert_eq!(check.semantics.as_ref().unwrap().role, Role::CheckBox);
    assert_eq!(
        check.semantics.as_ref().unwrap().state.checked,
        Some(argui_ui::CheckedState::Checked)
    );
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
    let toggle = ToggleBehavior::new(
        "metrics",
        "Metrics",
        Role::CheckBox,
        argui_ui::CheckedState::Checked,
    );
    assert_eq!(
        toggle.action(&click("metrics")),
        Some(ToggleAction::SetChecked(argui_ui::CheckedState::Unchecked))
    );
    assert_eq!(toggle.enabled(false).action(&click("metrics")), None);

    let disabled = RadioGroupBehavior::new(
        "quality",
        "Quality",
        [("High".into(), true), ("Low".into(), false)],
        None,
    );
    assert_eq!(disabled.action(&click("quality::option::1")), None);
}

#[test]
fn switch_thumb_uses_a_retained_transform_transition() {
    let themes = shadcn(Color::srgb(0.2, 0.5, 0.9));
    let theme = themes.resolve(ColorScheme::Dark);
    let mut tree = UiTree::new(Switch::new("profile", "Profile", false).build(theme));
    let thumb = tree.node_id_at(2).unwrap();
    assert_eq!(
        tree.resolved_transform(thumb, tree.element_at(2).unwrap()),
        Transform2D::IDENTITY
    );

    tree.set_reduced_motion(true);
    tree.update(Switch::new("profile", "Profile", true).build(theme));
    assert_eq!(
        tree.resolved_transform(thumb, tree.element_at(2).unwrap()),
        Transform2D::IDENTITY.translate(18.0, 0.0)
    );
}

#[test]
fn checkbox_variants_keep_indicator_and_disabled_state_consistent() {
    let themes = shadcn(Color::srgb(0.2, 0.5, 0.9));
    let theme = themes.resolve(ColorScheme::Light);
    let unchecked =
        Checkbox::new("unchecked", "Unchecked", argui_ui::CheckedState::Unchecked).build(theme);
    assert_eq!(unchecked.children[0].children.len(), 0);
    assert_eq!(
        unchecked.semantics.as_ref().unwrap().state.checked,
        Some(argui_ui::CheckedState::Unchecked)
    );
    assert!(unchecked.interaction.as_ref().unwrap().enabled);
    assert!(
        unchecked
            .interaction
            .as_ref()
            .unwrap()
            .focus_policy
            .is_focusable()
    );

    let custom = Element::text("custom mark");
    let checked = Checkbox::new("checked", "Checked", argui_ui::CheckedState::Checked)
        .indicator(custom.clone())
        .build(theme);
    assert_eq!(checked.children[0].children.len(), 1);
    assert_eq!(checked.children[0].children[0], custom);

    let disabled = Checkbox::new("disabled", "Disabled", argui_ui::CheckedState::Checked)
        .enabled(false)
        .build(theme);
    let interaction = disabled.interaction.as_ref().unwrap();
    assert!(!interaction.enabled);
    assert!(!interaction.focus_policy.is_focusable());
    assert_eq!(interaction.cursor, argui_ui::CursorIcon::NotAllowed);
    assert!(disabled.semantics.as_ref().unwrap().state.disabled);
}

#[test]
fn radio_group_supports_horizontal_layout_and_invalid_option_decoration() {
    let themes = shadcn(Color::srgb(0.2, 0.5, 0.9));
    let theme = themes.resolve(ColorScheme::Dark);
    let group = RadioGroup::new(
        "layout",
        "Layout",
        [
            RadioOption::new("Grid"),
            RadioOption::new("List").enabled(false),
        ],
        Some(99),
    )
    .orientation(Orientation::Horizontal)
    .build(theme);
    assert_eq!(
        group.semantics.as_ref().unwrap().orientation,
        Some(Orientation::Horizontal)
    );
    assert_eq!(group.style.flex_direction, argui_ui::FlexDirection::Row);
    assert!(!group.children[1].interaction.as_ref().unwrap().enabled);
    assert_eq!(
        group.children[1]
            .semantics
            .as_ref()
            .unwrap()
            .position_in_set,
        Some(2)
    );
    assert!(!group.children[0].semantics.as_ref().unwrap().state.selected);

    let behavior = RadioGroupBehavior::new(
        "layout",
        "Layout",
        [("Grid".into(), true), ("List".into(), false)],
        None,
    );
    let invalid = behavior.decorate(RadioGroupPart::Option(7), Element::container([]));
    assert_eq!(invalid.key.as_deref(), Some("layout::option::7"));
    assert_eq!(
        invalid.semantics.as_ref().unwrap().label.as_deref(),
        Some("")
    );
    assert!(!invalid.interaction.as_ref().unwrap().enabled);
    assert!(
        behavior
            .decorate(RadioGroupPart::Indicator, Element::text("dot"))
            .semantic_hidden
    );
    assert!(
        behavior
            .decorate(RadioGroupPart::Label, Element::text("label"))
            .semantic_hidden
    );
}

#[test]
fn toggle_decorations_publish_position_and_ignore_non_click_events() {
    let behavior = ToggleBehavior::new(
        "notifications",
        "Notifications",
        Role::Switch,
        argui_ui::CheckedState::Unchecked,
    )
    .enabled(false)
    .position_in_set(2, 4);
    let root = behavior.decorate(TogglePart::Root, Element::container([]));
    let semantics = root.semantics.as_ref().unwrap();
    assert_eq!(semantics.role, Role::Switch);
    assert_eq!(semantics.position_in_set, Some(2));
    assert_eq!(semantics.set_size, Some(4));
    assert_eq!(
        semantics.state.checked,
        Some(argui_ui::CheckedState::Unchecked)
    );
    assert!(semantics.state.disabled);
    assert_eq!(root.user_select, UserSelect::None);
    assert_eq!(
        root.interaction.as_ref().unwrap().cursor,
        argui_ui::CursorIcon::NotAllowed
    );

    let tree = UiTree::new(Element::container([]));
    let target = tree.node_ids()[0];
    let focused = UiEvent::new(target, Some("notifications".into()), UiEventKind::Focused);
    assert_eq!(behavior.action(&focused), None);
    assert_eq!(behavior.action(&click("other")), None);
    assert_eq!(
        ToggleBehavior::new(
            "notifications",
            "Notifications",
            Role::Switch,
            argui_ui::CheckedState::Checked
        )
        .action(&click("notifications")),
        Some(ToggleAction::SetChecked(argui_ui::CheckedState::Unchecked))
    );
    assert_eq!(
        ToggleBehavior::new(
            "notifications",
            "Notifications",
            Role::Switch,
            argui_ui::CheckedState::Checked
        )
        .action(&click("other")),
        None
    );
}

#[test]
fn mixed_checkbox_has_its_own_semantics_and_activates_to_checked() {
    use argui_ui::CheckedState;
    let themes = shadcn(argui_core::Color::WHITE);
    let checkbox = Checkbox::new("all", "All rows", CheckedState::Mixed)
        .build(themes.resolve(argui_core::ColorScheme::Dark));
    assert_eq!(
        checkbox.semantics.as_ref().unwrap().state.checked,
        Some(CheckedState::Mixed)
    );
    let behavior = ToggleBehavior::new("all", "All rows", Role::CheckBox, CheckedState::Mixed);
    assert_eq!(
        behavior.action(&click("all")),
        Some(ToggleAction::SetChecked(CheckedState::Checked))
    );
    assert_eq!(CheckedState::Unchecked.toggled(), CheckedState::Checked);
}
