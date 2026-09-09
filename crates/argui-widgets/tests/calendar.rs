#[path = "calendar/state.rs"]
mod state;
use argui_core::{Color, ColorScheme};
use argui_widgets::{
    Calendar, CalendarLocale, CalendarSelection, CalendarState, Date, IsoCalendarLocale, Month,
    shadcn,
};

#[test]
fn calendar_builds_accessible_days_and_iso_dates_round_trip() {
    let date = Date::from_calendar_date(2024, Month::February, 29).unwrap();
    let locale = IsoCalendarLocale;
    assert_eq!(locale.parse(&locale.format(date)), Ok(date));
    assert!(locale.parse("2023-02-29").is_err());
    let state = CalendarState::new(date, CalendarSelection::Single(Some(date)));
    let calendar = Calendar::new("calendar", "Choose a date", &state, date);
    let themes = shadcn(Color::WHITE);
    let built = calendar.build(themes.resolve(ColorScheme::Light));
    let tree = argui_ui::UiTree::new(built);
    let semantic = tree.semantic_tree(&[], 1.0);
    assert_eq!(
        semantic
            .nodes
            .iter()
            .filter(|node| node.semantics.role == argui_ui::Role::Cell)
            .count(),
        42
    );
    assert_eq!(
        semantic
            .nodes
            .iter()
            .filter(|node| node.semantics.state.selected)
            .count(),
        1
    );
    assert_eq!(
        semantic
            .nodes
            .iter()
            .filter(|node| node.semantics.focus_policy.is_tab_stop())
            .count(),
        1
    );
    assert!(tree.semantic_diagnostics().is_empty());
}

#[test]
fn calendar_bounds_and_keyboard_events_preserve_selection_until_activation() {
    use argui_core::{Key, KeyInput, KeyState};
    use argui_ui::{ClickEvent, Element, UiEvent, UiEventKind, UiTree};
    let themes = shadcn(Color::WHITE);
    let theme = themes.resolve(ColorScheme::Light);
    for date in [Date::MIN, Date::MAX] {
        let state = CalendarState::new(date, CalendarSelection::Single(None));
        let calendar = Calendar::new("calendar", "Date", &state, date);
        assert!(
            UiTree::new(calendar.build(theme))
                .semantic_diagnostics()
                .is_empty()
        );
    }
    let date = Date::from_calendar_date(2024, Month::February, 29).unwrap();
    let state = CalendarState::new(date, CalendarSelection::Single(None));
    let mut calendar = Calendar::new("calendar", "Date", &state, date);
    calendar.constraints.disabled = &|_| true;
    let tree = UiTree::new(calendar.build(theme));
    assert!(
        tree.semantic_tree(&[], 1.0)
            .nodes
            .iter()
            .filter(|node| node.semantics.role == argui_ui::Role::Cell)
            .all(|node| node.semantics.state.disabled)
    );
    calendar.constraints = Default::default();
    let id = UiTree::new(Element::container([])).node_ids()[0];
    let event = |target: &str, kind| UiEvent::new(id, Some(target.into()), kind);
    for key in [Key::Enter, Key::Character(" ".into())] {
        let selected = calendar
            .action(&event(
                "calendar",
                UiEventKind::KeyInput(KeyInput {
                    key,
                    state: KeyState::Pressed,
                    modifiers: Default::default(),
                    repeat: false,
                    text: None,
                }),
            ))
            .unwrap();
        assert!(selected.selection.contains(date));
    }
    assert!(
        calendar
            .action(&event(
                &calendar.day_key(date),
                UiEventKind::Click(ClickEvent::accessibility())
            ))
            .is_some()
    );
    for target in ["other", "calendar::day::invalid"] {
        assert!(
            calendar
                .action(&event(target, UiEventKind::Focused))
                .is_none()
        );
    }
    assert!(
        calendar
            .action(&event("calendar", UiEventKind::Focused))
            .is_none()
    );
    let invalid = argui_widgets::CalendarConstraints {
        minimum: Some(Date::MAX),
        maximum: Some(Date::MIN),
        ..Default::default()
    };
    let mut state = state;
    assert!(!state.navigate(
        &Key::ArrowDown,
        Default::default(),
        argui_widgets::Weekday::Monday,
        &invalid
    ));
}
