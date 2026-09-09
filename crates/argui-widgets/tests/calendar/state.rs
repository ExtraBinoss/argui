use argui_core::{Key, Modifiers};
use argui_widgets::{CalendarConstraints, CalendarSelection, CalendarState, Date, Month, Weekday};

fn date(year: i32, month: u8, day: u8) -> Date {
    Date::from_calendar_date(year, Month::try_from(month).unwrap(), day).unwrap()
}

#[test]
fn month_and_year_navigation_preserve_the_desired_day() {
    let mut state = CalendarState::new(date(2024, 1, 31), CalendarSelection::Single(None));
    let constraints = CalendarConstraints::default();
    for expected in [date(2024, 2, 29), date(2024, 3, 31)] {
        assert!(state.navigate(
            &Key::PageDown,
            Modifiers::default(),
            Weekday::Monday,
            &constraints
        ));
        assert_eq!(state.active, expected);
    }
    assert!(state.navigate(
        &Key::PageUp,
        Modifiers::default(),
        Weekday::Monday,
        &constraints
    ));
    assert!(state.navigate(
        &Key::PageDown,
        Modifiers {
            shift: true,
            ..Modifiers::default()
        },
        Weekday::Monday,
        &constraints
    ));
    assert_eq!(state.active, date(2025, 2, 28));
    assert!(state.navigate(
        &Key::PageDown,
        Modifiers::default(),
        Weekday::Monday,
        &constraints
    ));
    assert_eq!(state.active, date(2025, 3, 31));
}

#[test]
fn ranges_reject_disabled_interiors_without_changing_the_draft() {
    let mut state = CalendarState::new(
        date(2024, 1, 1),
        CalendarSelection::Range {
            start: None,
            end: None,
        },
    );
    let disabled = |day: Date| day.day() == 3;
    let constraints = CalendarConstraints {
        disabled: &disabled,
        ..Default::default()
    };
    assert!(state.select(date(2024, 1, 1), &constraints));
    let before = state.clone();
    assert!(!state.select(date(2024, 1, 5), &constraints));
    assert_eq!(state, before);
    assert!(state.select(date(2024, 1, 2), &constraints));
    assert!(state.selection.contains(date(2024, 1, 1)));
    assert!(state.selection.contains(date(2024, 1, 2)));
    assert!(!state.selection.contains(date(2024, 1, 3)));
    assert!(state.select(date(2024, 1, 7), &constraints));
    assert!(state.select(date(2024, 1, 5), &constraints));
    assert!(state.selection.contains(date(2024, 1, 6)));
}

#[test]
fn week_navigation_skips_disabled_days_and_respects_bounds() {
    let mut state = CalendarState::new(
        date(2024, 1, 10),
        CalendarSelection::Multiple(Default::default()),
    );
    let constraints = CalendarConstraints {
        minimum: Some(date(2024, 1, 1)),
        maximum: Some(date(2024, 1, 31)),
        disabled: &|date| date.weekday() == Weekday::Sunday,
    };
    for (key, expected) in [
        (Key::Home, 7 + 1),
        (Key::End, 13),
        (Key::ArrowRight, 15),
        (Key::ArrowLeft, 13),
        (Key::ArrowUp, 6),
        (Key::ArrowDown, 13),
    ] {
        assert!(state.navigate(&key, Modifiers::default(), Weekday::Sunday, &constraints));
        assert_eq!(state.active.day(), expected);
    }
    assert!(!state.select(date(2024, 2, 1), &constraints));
    assert!(state.select(date(2024, 1, 2), &constraints));
    assert!(state.selection.contains(date(2024, 1, 2)));
    assert!(state.select(date(2024, 1, 2), &constraints));
    assert!(!state.selection.contains(date(2024, 1, 2)));
    assert!(!state.navigate(
        &Key::Escape,
        Modifiers::default(),
        Weekday::Monday,
        &constraints
    ));
    state.active = Date::MIN;
    assert!(!state.navigate(
        &Key::PageUp,
        Modifiers::default(),
        Weekday::Monday,
        &CalendarConstraints::default()
    ));
    state.active = Date::MAX;
    assert!(!state.navigate(
        &Key::PageDown,
        Modifiers::default(),
        Weekday::Monday,
        &CalendarConstraints::default()
    ));
}
