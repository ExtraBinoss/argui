use crate::{Button, WidgetTheme};
use argui_core::{Key, KeyState};
use argui_paint::{Border, CornerRadii};
use argui_text::{TextStyle, TextWrap};
use argui_ui::{
    AlignItems, Element, FocusPolicy, GridPosition, JustifyContent, Role, SemanticState, Semantics,
    Sides, UiEvent, UiEventKind, length, percent,
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
    fn short_weekday(&self, weekday: Weekday) -> String {
        self.weekday(weekday).chars().take(3).collect()
    }
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
            .height(length(28.0))
            .flex_basis(length(0.0))
            .min_width(length(0.0))
            .grow(1.0)
            .semantics(Semantics::new(Role::ColumnHeader).label(self.locale.weekday(weekday)))
        }))
        .gap(4.0)
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
                    .flex_basis(length(0.0))
                    .min_width(length(0.0))
                    .grow(1.0)
                    .semantics(semantics)
            }))
            .gap(4.0)
            .shrink(0.0)
            .semantics(Semantics::new(Role::Row))
        });
        let mut semantics = Semantics::new(Role::Grid).label(&self.label);
        semantics.grid = GridPosition {
            row_count: Some(7),
            column_count: Some(7),
            ..Default::default()
        };
        Element::column([
            Element::row([
                self.month_button("previous", "Previous month", "‹", Key::PageUp, theme),
                Element::text(self.locale.month(self.state.month()))
                    .text_style(theme.ghost_button().label)
                    .semantics(
                        Semantics::new(Role::Heading).label(self.locale.month(self.state.month())),
                    ),
                self.month_button("next", "Next month", "›", Key::PageDown, theme),
            ])
            .align_items(AlignItems::CENTER)
            .justify_content(JustifyContent::SPACE_BETWEEN),
            Element::column(std::iter::once(header).chain(rows))
                .gap(4.0)
                .semantics(semantics),
        ])
        .width(length(300.0))
        .max_width(percent(1.0))
        .padding(Sides::length(12.0))
        .gap(12.0)
        .background(theme.card)
        .border(Border::all(1.0, theme.border))
        .radius(CornerRadii::all(10.0))
        .keyed(&self.key)
        .semantic_scope()
    }

    fn month_button(
        &self,
        part: &str,
        label: &str,
        glyph: &str,
        key: Key,
        theme: &WidgetTheme,
    ) -> Element {
        let enabled = self.state.clone().navigate(
            &key,
            Default::default(),
            self.locale.first_weekday(),
            &self.constraints,
        );
        Button::icon(
            format!("{}::{part}", self.key),
            label,
            Element::text(glyph).text_style(TextStyle {
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

    pub fn action(&self, event: &UiEvent) -> Option<CalendarState> {
        let key = event.target_key()?;
        if matches!(event.kind, UiEventKind::Click(_)) {
            let navigation = if key == format!("{}::previous", self.key) {
                Some(Key::PageUp)
            } else if key == format!("{}::next", self.key) {
                Some(Key::PageDown)
            } else {
                None
            };
            if let Some(key) = navigation {
                let mut next = self.state.clone();
                return next
                    .navigate(
                        &key,
                        Default::default(),
                        self.locale.first_weekday(),
                        &self.constraints,
                    )
                    .then_some(next);
            }
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
