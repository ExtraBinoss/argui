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
            .filter(|node| node.semantics.role == argui_ui::Role::Cell
                && node.semantics.focus_policy.is_tab_stop())
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

#[test]
fn month_buttons_navigate_without_selecting_and_stop_at_constraints() {
    use argui_ui::{ClickEvent, Element, UiEvent, UiEventKind, UiTree};
    let date = Date::from_calendar_date(2026, Month::September, 30).unwrap();
    let state = CalendarState::new(date, CalendarSelection::Single(Some(date)));
    let mut calendar = Calendar::new("calendar", "Date", &state, date);
    let id = UiTree::new(Element::container([])).node_ids()[0];
    let click = |key: &str| {
        UiEvent::new(
            id,
            Some(key.into()),
            UiEventKind::Click(ClickEvent::accessibility()),
        )
    };
    for (part, month) in [("previous", Month::August), ("next", Month::October)] {
        let next = calendar
            .action(&click(&format!("calendar::{part}")))
            .unwrap();
        assert_eq!(next.active.month(), month);
        assert_eq!(next.selection, state.selection);
    }
    calendar.constraints.minimum = Some(date);
    calendar.constraints.maximum = Some(date);
    assert!(calendar.action(&click("calendar::next")).is_none());
    assert!(calendar.action(&click("calendar::previous")).is_none());
    assert!(calendar.action(&click("calendar::unrelated")).is_none());
}

#[test]
fn calendar_days_have_equal_columns_and_single_line_labels_in_both_themes() {
    use argui_core::Size;
    use argui_layout::LayoutEngine;
    use argui_text::TextEngine;
    use argui_ui::{Element, ElementKind, UiTree, length};
    let date = Date::from_calendar_date(2026, Month::September, 9).unwrap();
    let state = CalendarState::new(date, CalendarSelection::Single(Some(date)));
    let calendar = Calendar::new("calendar", "Date", &state, date);
    let themes = shadcn(Color::from_srgb8(80, 80, 200));
    let mut text = TextEngine::from_embedded_fonts(
        [include_bytes!("../../argui-web-demo/assets/fonts/NotoSans-Regular.ttf").as_slice()],
        "Noto Sans",
        "Noto Sans",
        "Noto Sans",
    );
    for scheme in [ColorScheme::Light, ColorScheme::Dark] {
        for width in [280.0, 420.0] {
            let mut tree = UiTree::new(
                Element::column([calendar.build(themes.resolve(scheme))]).width(length(width)),
            );
            let output = LayoutEngine::new()
                .compute(&mut tree, &mut text, Size::new(width, 480.0))
                .unwrap();
            let days: Vec<_> = tree
                .node_ids()
                .iter()
                .enumerate()
                .filter(|(_, id)| {
                    tree.key(**id)
                        .is_some_and(|key| key.starts_with("calendar::day::"))
                })
                .collect();
            assert_eq!(days.len(), 42);
            let bounds_of = |id| {
                output
                    .nodes
                    .iter()
                    .find(|node| node.node == id)
                    .unwrap()
                    .bounds
            };
            let first = bounds_of(*days[0].1);
            for (index, id) in &days {
                let bounds = bounds_of(**id);
                assert!((bounds.size.width - first.size.width).abs() <= 1.0);
                assert!(bounds.size.width >= 30.0);
                assert_eq!(bounds.size.height, 36.0);
                let child = tree.element_at(*index).unwrap().children.first().unwrap();
                let ElementKind::Text { style, .. } = &child.kind else {
                    panic!("day label");
                };
                assert_eq!(style.wrap, argui_text::TextWrap::None);
                assert!(tree.key(**id).unwrap().starts_with("calendar::day::"));
            }
            assert!(bounds_of(*days[7].1).origin.y >= first.origin.y + 40.0);
        }
    }
}
