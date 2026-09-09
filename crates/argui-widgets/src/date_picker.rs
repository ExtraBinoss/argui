use crate::{
    Button, Calendar, CalendarConstraints, CalendarLocale, CalendarSelection, CalendarState, Date,
    Input, IsoCalendarLocale, Popover, PopoverAction, PopoverBehavior, WidgetTheme,
};
use argui_core::{Key, KeyState};
use argui_ui::{Element, FocusTarget, LiveRegion, Role, Semantics, UiEvent, UiEventKind};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DatePickerState {
    pub value: Option<Date>,
    pub draft: String,
    pub error: Option<String>,
    pub open: bool,
    pub calendar: CalendarState,
}

impl DatePickerState {
    pub fn new(value: Option<Date>, today: Date, locale: &dyn CalendarLocale) -> Self {
        Self {
            value,
            draft: value.map(|date| locale.format(date)).unwrap_or_default(),
            error: None,
            open: false,
            calendar: CalendarState::new(value.unwrap_or(today), CalendarSelection::Single(value)),
        }
    }

    pub fn cancel(&mut self, locale: &dyn CalendarLocale) {
        self.draft = self
            .value
            .map(|date| locale.format(date))
            .unwrap_or_default();
        self.error = None;
        self.open = false;
        self.calendar = CalendarState::new(
            self.value.unwrap_or(self.calendar.active),
            CalendarSelection::Single(self.value),
        );
    }

    pub fn commit(
        &mut self,
        locale: &dyn CalendarLocale,
        constraints: &CalendarConstraints<'_>,
        unavailable: &str,
    ) -> bool {
        let parsed = if self.draft.trim().is_empty() {
            Ok(None)
        } else {
            locale.parse(self.draft.trim()).map(Some)
        };
        match parsed {
            Ok(value) if value.is_none_or(|date| constraints.enabled(date)) => {
                self.value = value;
                self.cancel(locale);
                if let Some(date) = value {
                    self.calendar = CalendarState::new(date, CalendarSelection::Single(value));
                }
                true
            }
            result => {
                self.error = Some(result.err().unwrap_or_else(|| unavailable.into()));
                false
            }
        }
    }
}

pub struct DatePickerResponse {
    pub state: DatePickerState,
    pub focus: Option<FocusTarget>,
    pub committed: bool,
}

pub struct DatePicker<'a> {
    key: String,
    label: String,
    pub state: &'a DatePickerState,
    pub today: Date,
    pub locale: &'a dyn CalendarLocale,
    pub constraints: CalendarConstraints<'a>,
    pub unavailable_message: &'a str,
}

impl<'a> DatePicker<'a> {
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        state: &'a DatePickerState,
        today: Date,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            state,
            today,
            locale: &IsoCalendarLocale,
            constraints: CalendarConstraints::default(),
            unavailable_message: "This date is unavailable",
        }
    }

    pub fn input_key(&self) -> String {
        format!("{}::input", self.key)
    }
    fn popup_key(&self) -> String {
        format!("{}::popup", self.key)
    }
    fn calendar(&self) -> Calendar<'_> {
        let mut calendar = Calendar::new(
            format!("{}::calendar", self.key),
            &self.label,
            &self.state.calendar,
            self.today,
        );
        calendar.locale = self.locale;
        calendar.constraints = CalendarConstraints {
            minimum: self.constraints.minimum,
            maximum: self.constraints.maximum,
            disabled: self.constraints.disabled,
        };
        calendar
    }

    pub fn build(&self, theme: &WidgetTheme) -> Element {
        let error_key = format!("{}::error", self.key);
        let mut input = Input::new(self.input_key(), &self.state.draft, "", theme.input())
            .label(&self.label)
            .invalid(self.state.error.is_some())
            .build();
        if self.state.open {
            input = input.controls([format!("{}::content", self.popup_key())]);
        }
        if self.state.error.is_some() {
            input = input.described_by([error_key.clone()]);
        }
        let trigger = Button::new(self.popup_key(), &self.label, theme.outline_button()).build();
        let popup = Popover::new(
            self.popup_key(),
            &self.label,
            self.state.open,
            trigger,
            self.calendar().build(theme),
        )
        .trap_focus(true)
        .size(360.0, 420.0)
        .build(theme);
        let mut children = vec![Element::row([input, popup])];
        if let Some(error) = &self.state.error {
            let mut semantics = Semantics::new(Role::Alert).label(error);
            semantics.live = LiveRegion::Polite;
            children.push(
                Element::text(error.as_str())
                    .keyed(error_key)
                    .semantics(semantics),
            );
        }
        Element::column(children).keyed(&self.key).semantic_scope()
    }

    pub fn action(&self, event: &UiEvent) -> Option<DatePickerResponse> {
        let key = event.target_key()?;
        if key != self.key && !key.starts_with(&format!("{}::", self.key)) {
            return None;
        }
        let mut state = self.state.clone();
        let mut focus = None;
        let mut committed = false;
        if key == self.input_key() {
            match &event.kind {
                UiEventKind::TextChanged(value) => {
                    state.draft = value.clone();
                    state.error = None;
                }
                UiEventKind::Submitted(value) => {
                    state.draft = value.clone();
                    committed =
                        state.commit(self.locale, &self.constraints, self.unavailable_message);
                }
                UiEventKind::KeyInput(input)
                    if input.state == KeyState::Pressed && input.key == Key::Escape =>
                {
                    state.cancel(self.locale)
                }
                _ => return None,
            }
        } else if let Some(action) =
            PopoverBehavior::new(self.popup_key(), &self.label, state.open).action(event)
        {
            match action {
                PopoverAction::Toggle if !state.open => state.open = true,
                _ => {
                    state.cancel(self.locale);
                    focus = Some(self.input_key().into());
                }
            }
        } else if state.open {
            let calendar = self.calendar();
            let next = calendar.action(event)?;
            let selected = matches!(event.kind, UiEventKind::Click(_))
                || matches!(&event.kind, UiEventKind::KeyInput(input) if input.key == Key::Enter || input.key == Key::Character(" ".into()));
            if selected {
                state.draft = self.locale.format(next.active);
                committed = state.commit(self.locale, &self.constraints, self.unavailable_message);
                focus = Some(self.input_key().into());
            } else {
                focus = Some(calendar.day_key(next.active).into());
            }
            state.calendar = next;
        } else {
            return None;
        }
        Some(DatePickerResponse {
            state,
            focus,
            committed,
        })
    }
}
