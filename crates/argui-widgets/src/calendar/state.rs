use argui_core::{Key, Modifiers};
use std::collections::BTreeSet;
use time::{Date, Duration, Month, Weekday};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CalendarSelection {
    Single(Option<Date>),
    Multiple(BTreeSet<Date>),
    Range {
        start: Option<Date>,
        end: Option<Date>,
    },
}

impl CalendarSelection {
    /// Returns whether `date` belongs to the current selection.
    pub fn contains(&self, date: Date) -> bool {
        match self {
            Self::Single(selected) => *selected == Some(date),
            Self::Multiple(selected) => selected.contains(&date),
            Self::Range {
                start: Some(start),
                end,
            } => (*start..=end.unwrap_or(*start)).contains(&date),
            Self::Range { start: None, .. } => false,
        }
    }
}

/// All dates are Gregorian civil dates; the owner supplies today and locale explicitly.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalendarState {
    pub active: Date,
    pub selection: CalendarSelection,
    desired_day: u8,
}

impl CalendarState {
    /// Creates calendar state focused on `active` with the supplied selection.
    pub fn new(active: Date, selection: CalendarSelection) -> Self {
        Self {
            active,
            selection,
            desired_day: active.day(),
        }
    }

    /// Returns the first day of the month containing the active date.
    pub fn month(&self) -> Date {
        self.active.replace_day(1).expect("first day of month")
    }

    /// Selection is atomic: a range containing a disabled date is rejected unchanged.
    ///
    /// Returns whether the date was accepted under `constraints`.
    pub fn select(&mut self, date: Date, constraints: &CalendarConstraints<'_>) -> bool {
        if !constraints.enabled(date) {
            return false;
        }
        match &mut self.selection {
            CalendarSelection::Single(selected) => *selected = Some(date),
            CalendarSelection::Multiple(selected) => {
                if !selected.insert(date) {
                    selected.remove(&date);
                }
            }
            CalendarSelection::Range { start, end } => {
                if let Some(anchor) = *start
                    && end.is_none()
                {
                    let first = anchor.min(date);
                    let last = anchor.max(date);
                    let mut cursor = first;
                    loop {
                        if !constraints.enabled(cursor) {
                            return false;
                        }
                        if cursor == last {
                            break;
                        }
                        cursor = cursor.next_day().expect("range endpoint exists");
                    }
                    *start = Some(first);
                    *end = Some(last);
                } else {
                    *start = Some(date);
                    *end = None;
                }
            }
        }
        self.active = date;
        self.desired_day = date.day();
        true
    }

    /// Moves the active date according to a calendar navigation key.
    ///
    /// `key` is the pressed key, `modifiers` controls modified navigation, and
    /// `first_weekday` defines the week boundary used by Home and End. `constraints`
    /// bounds navigation and skips disabled dates. Returns whether the active date changed.
    pub fn navigate(
        &mut self,
        key: &Key,
        modifiers: Modifiers,
        first_weekday: Weekday,
        constraints: &CalendarConstraints<'_>,
    ) -> bool {
        if constraints
            .minimum
            .zip(constraints.maximum)
            .is_some_and(|(min, max)| min > max)
        {
            return false;
        }
        let weekday = (self.active.weekday().number_days_from_monday() + 7
            - first_weekday.number_days_from_monday())
            % 7;
        let (candidate, direction, preserve_day) = match key {
            Key::ArrowLeft => (self.active.previous_day(), -1, false),
            Key::ArrowRight => (self.active.next_day(), 1, false),
            Key::ArrowUp => (self.active.checked_sub(Duration::days(7)), -1, false),
            Key::ArrowDown => (self.active.checked_add(Duration::days(7)), 1, false),
            Key::Home => (
                self.active.checked_sub(Duration::days(i64::from(weekday))),
                1,
                false,
            ),
            Key::End => (
                self.active
                    .checked_add(Duration::days(i64::from(6 - weekday))),
                -1,
                false,
            ),
            Key::PageUp | Key::PageDown => {
                let direction = if *key == Key::PageUp { -1 } else { 1 };
                (
                    shift_month(
                        self.active,
                        direction * if modifiers.shift { 12 } else { 1 },
                        self.desired_day,
                    ),
                    direction,
                    true,
                )
            }
            _ => return false,
        };
        let Some(mut candidate) = candidate else {
            return false;
        };
        candidate = candidate.clamp(
            constraints.minimum.unwrap_or(Date::MIN),
            constraints.maximum.unwrap_or(Date::MAX),
        );
        while !constraints.enabled(candidate) {
            let next = if direction > 0 {
                candidate.next_day()
            } else {
                candidate.previous_day()
            };
            let Some(next) = next.filter(|date| constraints.in_bounds(*date)) else {
                return false;
            };
            candidate = next;
        }
        let changed = candidate != self.active;
        self.active = candidate;
        if !preserve_day {
            self.desired_day = candidate.day();
        }
        changed
    }
}

pub struct CalendarConstraints<'a> {
    pub minimum: Option<Date>,
    pub maximum: Option<Date>,
    pub disabled: &'a dyn Fn(Date) -> bool,
}

impl Default for CalendarConstraints<'_> {
    fn default() -> Self {
        Self {
            minimum: None,
            maximum: None,
            disabled: &|_| false,
        }
    }
}

impl CalendarConstraints<'_> {
    /// Returns whether `date` falls between the optional inclusive bounds.
    pub fn in_bounds(&self, date: Date) -> bool {
        self.minimum.is_none_or(|min| date >= min) && self.maximum.is_none_or(|max| date <= max)
    }
    /// Returns whether `date` is in bounds and is not rejected by the disabled-date predicate.
    pub fn enabled(&self, date: Date) -> bool {
        self.in_bounds(date) && !(self.disabled)(date)
    }
}

fn shift_month(date: Date, months: i32, desired_day: u8) -> Option<Date> {
    let month = date.year() * 12 + i32::from(date.month() as u8) - 1 + months;
    let first = Date::from_calendar_date(
        month.div_euclid(12),
        Month::try_from((month.rem_euclid(12) + 1) as u8).ok()?,
        1,
    )
    .ok()?;
    (1..=desired_day)
        .rev()
        .find_map(|day| first.replace_day(day).ok())
}
