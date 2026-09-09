use argui_core::{Color, ColorScheme, Key, KeyInput, KeyState, Modifiers};
use argui_ui::{Element, UiEvent, UiEventKind, UiTree};
use argui_widgets::{
    CalendarConstraints, CalendarLocale, Date, DatePicker, DatePickerState, IsoCalendarLocale,
    shadcn,
};

fn date(text: &str) -> Date {
    IsoCalendarLocale.parse(text).unwrap()
}
fn event(key: &str, kind: UiEventKind) -> UiEvent {
    UiEvent::new(
        UiTree::new(Element::container([])).node_ids()[0],
        Some(key.into()),
        kind,
    )
}

#[test]
fn drafts_are_validated_atomically_and_escape_restores_the_committed_value() {
    let today = date("2024-02-29");
    let mut state = DatePickerState::new(Some(today), today, &IsoCalendarLocale);
    let constraints = CalendarConstraints {
        minimum: Some(today),
        ..Default::default()
    };
    for draft in ["2023-02-29", "2024-02-28"] {
        state.draft = draft.into();
        assert!(!state.commit(&IsoCalendarLocale, &constraints, "Unavailable"));
        assert_eq!(state.value, Some(today));
        assert!(state.error.is_some());
    }
    let themes = shadcn(Color::WHITE);
    let built =
        DatePicker::new("date", "Date", &state, today).build(themes.resolve(ColorScheme::Light));
    let tree = UiTree::new(built);
    assert!(tree.semantic_diagnostics().is_empty());
    assert!(
        tree.semantic_tree(&[], 1.0).nodes.iter().any(|node| !node
            .semantics
            .relations
            .described_by
            .is_empty())
    );
    state.open = true;
    let response = DatePicker::new("date", "Date", &state, today)
        .action(&event(
            "date::input",
            UiEventKind::KeyInput(KeyInput {
                key: Key::Escape,
                state: KeyState::Pressed,
                repeat: false,
                modifiers: Modifiers::default(),
                text: None,
            }),
        ))
        .unwrap();
    assert_eq!(response.state.draft, "2024-02-29");
    assert!(!response.state.open);
    assert!(response.state.error.is_none());
    assert!(!response.committed);
    state.draft.clear();
    assert!(state.commit(&IsoCalendarLocale, &constraints, "Unavailable"));
    assert_eq!(state.value, None);
}

#[test]
fn text_changes_do_not_commit_and_valid_submission_normalizes_input() {
    let today = date("2024-02-29");
    let initial = DatePickerState::new(None, today, &IsoCalendarLocale);
    let response = DatePicker::new("date", "Date", &initial, today)
        .action(&event(
            "date::input",
            UiEventKind::TextChanged("2025-01-01".into()),
        ))
        .unwrap();
    assert_eq!(response.state.value, None);
    assert_eq!(response.state.draft, "2025-01-01");
    let response = DatePicker::new("date", "Date", &response.state, today)
        .action(&event(
            "date::input",
            UiEventKind::Submitted(" 2025-01-01 ".into()),
        ))
        .unwrap();
    assert!(response.committed);
    assert_eq!(response.state.value, Some(date("2025-01-01")));
    assert_eq!(response.state.draft, "2025-01-01");
    assert!(
        DatePicker::new("date", "Date", &initial, today)
            .action(&event("other", UiEventKind::Submitted("2025-01-01".into())))
            .is_none()
    );
}

#[test]
fn popup_navigation_selects_dates_and_ignores_unrelated_or_released_keys() {
    let today = date("2024-02-29");
    let key = |key, state| {
        UiEventKind::KeyInput(KeyInput {
            key,
            state,
            repeat: false,
            modifiers: Default::default(),
            text: None,
        })
    };
    let mut state = DatePickerState::new(Some(today), today, &IsoCalendarLocale);
    for input in [Key::ArrowDown, Key::Escape] {
        assert!(
            DatePicker::new("date", "Date", &state, today)
                .action(&event("date::input", key(input, KeyState::Released)))
                .is_none()
        );
    }
    assert!(
        DatePicker::new("date", "Date", &state, today)
            .action(&event("date::unknown", UiEventKind::Focused))
            .is_none()
    );
    state = DatePicker::new("date", "Date", &state, today)
        .action(&event(
            "date::popup",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        ))
        .unwrap()
        .state;
    assert!(state.open);
    let next = DatePicker::new("date", "Date", &state, today)
        .action(&event(
            "date::calendar::day::2024-02-29",
            key(Key::ArrowRight, KeyState::Pressed),
        ))
        .unwrap();
    assert!(!next.committed);
    assert_eq!(next.state.calendar.active, date("2024-03-01"));
    state = next.state;
    let selected = DatePicker::new("date", "Date", &state, today)
        .action(&event(
            "date::calendar::day::2024-03-01",
            key(Key::Enter, KeyState::Pressed),
        ))
        .unwrap();
    assert!(selected.committed);
    assert_eq!(selected.state.value, Some(date("2024-03-01")));
    assert!(!selected.state.open);
}
