use crate::{Button, WidgetTheme};
use argui_core::{Key, KeyState};
use argui_paint::{Border, CornerRadii};
use argui_text::{TextStyle, TextWrap};
use argui_ui::{
    AlignItems, Element, EventType, FocusPolicy, GridPosition, JustifyContent, Role, SemanticState,
    Semantics, Sides, UiEvent, UiEventKind, ValueHandler, length, percent,
};
use time::{Date, Duration, Weekday};

mod state;
pub use state::{CalendarConstraints, CalendarSelection, CalendarState};

pub trait CalendarLocale {
    /// Returns whether this locale uses right-to-left layout.
    fn rtl(&self) -> bool {
        false
    }
    /// Returns the first weekday used to lay out the calendar grid.
    fn first_weekday(&self) -> Weekday;
    /// Formats a full weekday name for `weekday`.
    fn weekday(&self, weekday: Weekday) -> String;
    /// Formats a compact weekday name; defaults to the first three characters.
    fn short_weekday(&self, weekday: Weekday) -> String {
        self.weekday(weekday).chars().take(3).collect()
    }
    /// Formats the month containing `date` for the calendar heading.
    fn month(&self, date: Date) -> String;
    /// Formats `date` for day labels and input values.
    fn format(&self, date: Date) -> String;
    /// Parses a date from user-provided `text`.
    ///
    /// # Errors
    ///
    /// Returns a message when the input cannot be parsed as a date.
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
        format!("{} {}", date.month(), date.year())
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
    compact: bool,
    select_handlers: Vec<ValueHandler<String>>,
}

impl<'a> Calendar<'a> {
    /// Creates a calendar identified by `key`, labelled for accessibility, and anchored to `state`.
    ///
    /// `today` marks the current date in the rendered grid.
    /// `label` supplies the accessible calendar name.
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
            compact: false,
            select_handlers: Vec::new(),
        }
    }

    /// Uses tighter mobile-friendly day cells, spacing, and outer padding.
    #[must_use]
    pub const fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }

    /// Adds a handler that receives the selected date in ISO `YYYY-MM-DD` form.
    ///
    /// `handler` is additive and is not delivered for constrained or disabled days.
    #[must_use]
    pub fn on_select(mut self, handler: ValueHandler<String>) -> Self {
        self.select_handlers.push(handler);
        self
    }

    /// Returns the interaction key used for `date`'s day cell.
    pub fn day_key(&self, date: Date) -> String {
        format!("{}::day::{date}", self.key)
    }

    /// Returns the interaction key for the month/year heading button.
    pub fn heading_key(&self) -> String {
        format!("{}::heading", self.key)
    }

    /// Returns the interaction key for `year` in the year-selection grid.
    pub fn year_key(&self, year: i32) -> String {
        format!("{}::year::{year}", self.key)
    }

    /// Returns the key that should receive focus for the calendar's current view.
    pub fn focus_key(&self) -> String {
        if self.state.year_picker_open() {
            self.year_key(self.state.active.year())
        } else {
            self.day_key(self.state.active)
        }
    }

    /// Builds the calendar with localized headings and themed day controls.
    /// `theme` supplies the calendar's colors and text styles.
    pub fn build(&self, theme: &WidgetTheme) -> Element {
        self.build_days(theme, |date, _today, selected| {
            Element::text(date.day().to_string()).text_style(TextStyle {
                color: if selected {
                    theme.primary_foreground
                } else if date.month() != self.state.month().month() {
                    theme.muted_foreground
                } else {
                    theme.foreground
                },
                font_size: 14.0,
                line_height: 20.0,
                wrap: TextWrap::None,
                ..Default::default()
            })
        })
    }

    /// Custom day content retains the same semantics, identity and keyboard behavior.
    ///
    /// `day` receives the date, whether it is today, and whether it is selected.
    /// `theme` supplies the surrounding calendar styling.
    pub fn build_days(
        &self,
        theme: &WidgetTheme,
        mut day: impl FnMut(Date, bool, bool) -> Element,
    ) -> Element {
        let cell_height = if self.compact { 32.0 } else { 36.0 };
        let grid_gap = if self.compact { 2.0 } else { 4.0 };
        let first_weekday = self.locale.first_weekday();
        let weekday = (self.state.month().weekday().number_days_from_monday() + 7
            - first_weekday.number_days_from_monday())
            % 7;
        let header = Element::row((0..7).map(|offset| {
            let mut weekday = first_weekday;
            for _ in 0..offset {
                weekday = weekday.next();
            }
            Element::row([
                Element::text(self.locale.short_weekday(weekday)).text_style(TextStyle {
                    color: theme.muted_foreground,
                    font_size: 12.0,
                    line_height: 20.0,
                    wrap: TextWrap::None,
                    ..Default::default()
                }),
            ])
            .justify_content(JustifyContent::CENTER)
            .height(length(if self.compact { 24.0 } else { 28.0 }))
            .flex_basis(length(0.0))
            .min_width(length(0.0))
            .grow(1.0)
            .semantics(Semantics::new(Role::ColumnHeader).label(self.locale.weekday(weekday)))
        }))
        .gap(grid_gap)
        .semantics(Semantics::new(Role::Row));
        let rows = (0..6).map(|row| {
            Element::row((0..7).map(|column| {
                let date = self
                    .state
                    .month()
                    .checked_add(Duration::days(row * 7 + column - i64::from(weekday)));
                let Some(date) = date else {
                    return Element::container([]).flex_basis(length(0.0)).grow(1.0);
                };
                let selected = self.state.selection.contains(date);
                let enabled = self.constraints.enabled(date);
                let mut style = if selected {
                    theme.button()
                } else if date == self.today {
                    theme.secondary_button()
                } else {
                    theme.ghost_button()
                };
                style.layout.padding = Sides::length(0.0);
                style.focused = Some(argui_ui::StylePatch::from_quad(
                    style
                        .paint
                        .quad
                        .clone()
                        .border(Border::all(2.0, theme.ring)),
                ));
                let mut element = Button::new(self.day_key(date), self.locale.format(date), style)
                    .enabled(enabled)
                    .build();
                if enabled {
                    for handler in &self.select_handlers {
                        element = element
                            .on(handler.direct_listener_value(EventType::Click, date.to_string()));
                    }
                }
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
                if !enabled {
                    element = element.opacity(0.35);
                }
                element
                    .height(length(cell_height))
                    .flex_basis(length(0.0))
                    .min_width(length(0.0))
                    .grow(1.0)
                    .semantics(semantics)
            }))
            .gap(grid_gap)
            .shrink(0.0)
            .semantics(Semantics::new(Role::Row))
        });
        let mut semantics = Semantics::new(Role::Grid).label(&self.label);
        semantics.grid = GridPosition {
            row_count: Some(7),
            column_count: Some(7),
            ..Default::default()
        };
        let days = Element::column(std::iter::once(header).chain(rows))
            .gap(grid_gap)
            .semantics(semantics);
        let picking_year = self.state.year_picker_open();
        let heading = self.locale.month(self.state.month());
        let mut heading_style = theme.ghost_button();
        heading_style.layout.padding = argui_ui::sides(8.0, 4.0);
        let body = if picking_year {
            self.year_grid(theme, cell_height, grid_gap)
        } else {
            days
        };
        Element::column([
            Element::row([
                self.navigation_button("previous", false, theme),
                Button::new(self.heading_key(), &heading, heading_style)
                    .without_tooltip()
                    .build()
                    .height(length(32.0))
                    .min_width(length(0.0))
                    .grow(1.0),
                self.navigation_button("next", true, theme),
            ])
            .align_items(AlignItems::CENTER)
            .justify_content(JustifyContent::SPACE_BETWEEN),
            body,
        ])
        .width(length(if self.compact { 288.0 } else { 300.0 }))
        .max_width(percent(1.0))
        .padding(Sides::length(if self.compact { 8.0 } else { 12.0 }))
        .gap(if self.compact { 8.0 } else { 12.0 })
        .background(theme.card)
        .border(Border::all(1.0, theme.border))
        .radius(CornerRadii::all(10.0))
        .keyed(&self.key)
        .semantic_scope()
    }

    /// Builds one month or year-page navigation button for the current view.
    fn navigation_button(&self, part: &str, forward: bool, theme: &WidgetTheme) -> Element {
        let picking_year = self.state.year_picker_open();
        let mut next = self.state.clone();
        let enabled = if picking_year {
            next.navigate_years(if forward { 12 } else { -12 }, &self.constraints)
        } else {
            next.navigate(
                if forward {
                    &Key::PageDown
                } else {
                    &Key::PageUp
                },
                Default::default(),
                self.locale.first_weekday(),
                &self.constraints,
            )
        };
        Button::icon(
            format!("{}::{part}", self.key),
            match (picking_year, forward) {
                (true, false) => "Previous years",
                (true, true) => "Next years",
                (false, false) => "Previous month",
                (false, true) => "Next month",
            },
            Element::text(if forward { "›" } else { "‹" }).text_style(TextStyle {
                font_size: 24.0,
                line_height: 24.0,
                color: theme.foreground,
                wrap: TextWrap::None,
                ..Default::default()
            }),
            theme.ghost_button(),
        )
        .enabled(enabled)
        .build()
        .width(length(32.0))
        .height(length(32.0))
        .padding(Sides::length(0.0))
        .opacity(if enabled { 1.0 } else { 0.35 })
    }

    /// Builds the accessible twelve-year grid around the active year.
    fn year_grid(&self, theme: &WidgetTheme, cell_height: f32, gap: f32) -> Element {
        let start = self
            .state
            .active
            .year()
            .saturating_sub(5)
            .clamp(Date::MIN.year(), Date::MAX.year().saturating_sub(11));
        let rows = (0..4).map(|row| {
            Element::row((0..3).map(|column| {
                let year = start + row * 3 + column;
                let selected = year == self.state.active.year();
                let enabled = self
                    .constraints
                    .minimum
                    .is_none_or(|minimum| minimum.year() <= year)
                    && self
                        .constraints
                        .maximum
                        .is_none_or(|maximum| maximum.year() >= year);
                let mut style = if selected {
                    theme.button()
                } else {
                    theme.ghost_button()
                };
                style.layout.padding = Sides::length(0.0);
                let mut button = Button::new(self.year_key(year), year.to_string(), style)
                    .without_tooltip()
                    .enabled(enabled)
                    .build()
                    .height(length(cell_height))
                    .flex_basis(length(0.0))
                    .min_width(length(0.0))
                    .grow(1.0);
                if let Some(interaction) = &mut button.interaction {
                    interaction.focus_policy = if selected && enabled {
                        FocusPolicy::TabStop
                    } else {
                        FocusPolicy::Programmatic
                    };
                }
                let mut semantics =
                    Semantics::new(Role::Cell)
                        .label(year.to_string())
                        .state(SemanticState {
                            selected,
                            disabled: !enabled,
                            ..Default::default()
                        });
                semantics.grid = GridPosition {
                    row_index: Some(row as u32 + 1),
                    column_index: Some(column as u32 + 1),
                    ..Default::default()
                };
                button.semantics(semantics)
            }))
            .gap(gap)
            .semantics(Semantics::new(Role::Row))
        });
        let mut semantics = Semantics::new(Role::Grid).label("Choose year");
        semantics.grid = GridPosition {
            row_count: Some(4),
            column_count: Some(3),
            ..Default::default()
        };
        Element::column(rows).gap(gap).semantics(semantics)
    }

    /// Applies a calendar interaction to a cloned state and returns it when changed.
    /// `event` is the UI interaction to interpret.
    pub fn action(&self, event: &UiEvent) -> Option<CalendarState> {
        let key = event.target_key()?;
        if matches!(event.kind, UiEventKind::Click(_)) {
            if key == self.heading_key() {
                let mut next = self.state.clone();
                next.set_year_picker_open(!next.year_picker_open());
                return Some(next);
            }
            let navigation = if key == format!("{}::previous", self.key) {
                Some(false)
            } else if key == format!("{}::next", self.key) {
                Some(true)
            } else {
                None
            };
            if let Some(forward) = navigation {
                let mut next = self.state.clone();
                let changed = if next.year_picker_open() {
                    next.navigate_years(if forward { 12 } else { -12 }, &self.constraints)
                } else {
                    next.navigate(
                        if forward {
                            &Key::PageDown
                        } else {
                            &Key::PageUp
                        },
                        Default::default(),
                        self.locale.first_weekday(),
                        &self.constraints,
                    )
                };
                return changed.then_some(next);
            }
        }
        if let Some(year) = key
            .strip_prefix(&format!("{}::year::", self.key))
            .and_then(|value| value.parse::<i32>().ok())
        {
            let mut next = self.state.clone();
            let changed = match &event.kind {
                UiEventKind::Click(_) => next.select_year(year, &self.constraints),
                UiEventKind::KeyInput(input) if input.state == KeyState::Pressed => {
                    if input.key == Key::Enter || input.key == Key::Character(" ".into()) {
                        return next.select_year(year, &self.constraints).then_some(next);
                    }
                    let step = match input.key {
                        Key::ArrowLeft => -1,
                        Key::ArrowRight => 1,
                        Key::ArrowUp => -3,
                        Key::ArrowDown => 3,
                        Key::PageUp => -12,
                        Key::PageDown => 12,
                        _ => return None,
                    };
                    next.navigate_years(
                        if self.locale.rtl() { -step } else { step },
                        &self.constraints,
                    )
                }
                _ => false,
            };
            return changed.then_some(next);
        }
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
