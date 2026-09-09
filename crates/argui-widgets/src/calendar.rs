use crate::{Button, WidgetTheme};
use argui_core::{Key, KeyState};
use argui_ui::{
    Element, FocusPolicy, GridPosition, Role, SemanticState, Semantics, UiEvent, UiEventKind,
};
use time::{Date, Duration, Weekday};

mod state;
pub use state::{CalendarConstraints, CalendarSelection, CalendarState};

pub trait CalendarLocale {
    fn rtl(&self) -> bool {
        false
    }
    fn first_weekday(&self) -> Weekday;
    fn weekday(&self, weekday: Weekday) -> String;
    fn month(&self, date: Date) -> String;
    fn format(&self, date: Date) -> String;
    fn parse(&self, text: &str) -> Result<Date, String>;
}

/// ISO dates and Monday-first weeks. Supply a locale for translated presentation and input.
pub struct IsoCalendarLocale;
impl CalendarLocale for IsoCalendarLocale {
    fn first_weekday(&self) -> Weekday {
        Weekday::Monday
    }
    fn weekday(&self, weekday: Weekday) -> String {
        weekday.to_string()
    }
    fn month(&self, date: Date) -> String {
        format!("{}-{:02}", date.year(), date.month() as u8)
    }
    fn format(&self, date: Date) -> String {
        date.to_string()
    }
    fn parse(&self, text: &str) -> Result<Date, String> {
        let format =
            time::format_description::parse("[year]-[month]-[day]").expect("ISO date format");
        Date::parse(text, &format).map_err(|error| error.to_string())
    }
}

pub struct Calendar<'a> {
    key: String,
    label: String,
    pub state: &'a CalendarState,
    pub today: Date,
    pub locale: &'a dyn CalendarLocale,
    pub constraints: CalendarConstraints<'a>,
}

impl<'a> Calendar<'a> {
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        state: &'a CalendarState,
        today: Date,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            state,
            today,
            locale: &IsoCalendarLocale,
            constraints: CalendarConstraints::default(),
        }
    }

    pub fn day_key(&self, date: Date) -> String {
        format!("{}::day::{date}", self.key)
    }

    pub fn build(&self, theme: &WidgetTheme) -> Element {
        self.build_days(theme, |date, _today, _selected| {
            Element::text(date.day().to_string())
        })
    }

    /// Custom day content retains the same semantics, identity and keyboard behavior.
    pub fn build_days(
        &self,
        theme: &WidgetTheme,
        mut day: impl FnMut(Date, bool, bool) -> Element,
    ) -> Element {
        let first_weekday = self.locale.first_weekday();
        let weekday = (self.state.month().weekday().number_days_from_monday() + 7
            - first_weekday.number_days_from_monday())
            % 7;
        let header = Element::row((0..7).map(|offset| {
            let mut weekday = first_weekday;
            for _ in 0..offset {
                weekday = weekday.next();
            }
            Element::text(self.locale.weekday(weekday))
                .grow(1.0)
                .semantics(Semantics::new(Role::ColumnHeader).label(self.locale.weekday(weekday)))
        }))
        .semantics(Semantics::new(Role::Row));
        let rows = (0..6).map(|row| {
            Element::row((0..7).map(|column| {
                let date = self
                    .state
                    .month()
                    .checked_add(Duration::days(row * 7 + column - i64::from(weekday)));
                let Some(date) = date else {
                    return Element::container([]).grow(1.0);
                };
                let selected = self.state.selection.contains(date);
                let enabled = self.constraints.enabled(date);
                let mut element = Button::new(
                    self.day_key(date),
                    self.locale.format(date),
                    if selected {
                        theme.button()
                    } else {
                        theme.ghost_button()
                    },
                )
                .enabled(enabled)
                .build();
                element.children =
                    vec![day(date, date == self.today, selected).semantic_hidden(true)];
                if let Some(interaction) = &mut element.interaction {
                    interaction.focus_policy = if date == self.state.active && enabled {
                        FocusPolicy::TabStop
                    } else {
                        FocusPolicy::Programmatic
                    };
                }
                let mut semantics = Semantics::new(Role::Cell)
                    .label(self.locale.format(date))
                    .state(SemanticState {
                        selected,
                        disabled: !enabled,
                        ..Default::default()
                    });
                semantics.grid = GridPosition {
                    row_index: Some(row as u32 + 2),
                    column_index: Some(column as u32 + 1),
                    ..Default::default()
                };
                element.grow(1.0).semantics(semantics)
            }))
            .semantics(Semantics::new(Role::Row))
        });
        let mut semantics = Semantics::new(Role::Grid).label(&self.label);
        semantics.grid = GridPosition {
            row_count: Some(7),
            column_count: Some(7),
            ..Default::default()
        };
        Element::column([
            Element::text(self.locale.month(self.state.month())).semantics(
                Semantics::new(Role::Heading).label(self.locale.month(self.state.month())),
            ),
            Element::column(std::iter::once(header).chain(rows)).semantics(semantics),
        ])
        .keyed(&self.key)
        .semantic_scope()
    }

    pub fn action(&self, event: &UiEvent) -> Option<CalendarState> {
        let key = event.target_key()?;
        let date = key
            .strip_prefix(&format!("{}::day::", self.key))
            .and_then(|value| IsoCalendarLocale.parse(value).ok());
        if key != self.key && date.is_none() {
            return None;
        }
        let mut next = self.state.clone();
        let changed = match &event.kind {
            UiEventKind::Click(_) => next.select(date?, &self.constraints),
            UiEventKind::KeyInput(input) if input.state == KeyState::Pressed => {
                if input.key == Key::Enter || input.key == Key::Character(" ".into()) {
                    next.select(date.unwrap_or(next.active), &self.constraints)
                } else {
                    let key = if self.locale.rtl() {
                        match input.key {
                            Key::ArrowLeft => Key::ArrowRight,
                            Key::ArrowRight => Key::ArrowLeft,
                            _ => input.key.clone(),
                        }
                    } else {
                        input.key.clone()
                    };
                    next.navigate(
                        &key,
                        input.modifiers,
                        self.locale.first_weekday(),
                        &self.constraints,
                    )
                }
            }
            _ => false,
        };
        changed.then_some(next)
    }
}
