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
    /// Creates picker state from an optional value, current date and locale.
    /// `today` supplies the initial calendar date when no value is selected.
    pub fn new(value: Option<Date>, today: Date, locale: &dyn CalendarLocale) -> Self {
        Self {
            value,
            draft: value.map(|date| locale.format(date)).unwrap_or_default(),
            error: None,
            open: false,
            calendar: CalendarState::new(value.unwrap_or(today), CalendarSelection::Single(value)),
        }
    }

    /// Discards the draft and restores the committed value formatted with `locale`.
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

    /// Parses and validates the draft, committing it when available.
    ///
    /// `locale` parses the draft; `unavailable` is shown when constraints reject the date.
    /// Returns whether a value was committed.
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
    icon: Option<Element>,
    surface: Option<argui_ui::OverlaySurface>,
}

impl<'a> DatePicker<'a> {
    /// Creates a date picker identified by `key`, labelled for accessibility and initialized with `today`.
    /// `label` names the control for assistive technology; `state` supplies its retained value and draft.
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
            icon: None,
            surface: None,
        }
    }

    #[must_use]
    /// Sets the overlay surface policy for the calendar popup.
    pub fn surface(mut self, surface: argui_ui::OverlaySurface) -> Self {
        self.surface = Some(surface);
        self
    }

    /// Replaces the default calendar icon with `icon`.
    pub fn icon(mut self, icon: Element) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Returns the state key used by the editable date input.
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

    /// Builds the date input and calendar popup using `theme` for styling.
    pub fn build(&self, theme: &WidgetTheme) -> Element {
        let error_key = format!("{}::error", self.key);
        let mut input = Input::new(
            self.input_key(),
            &self.state.draft,
            self.locale.format(self.today),
            theme.input(),
        )
        .label(&self.label)
        .invalid(self.state.error.is_some())
        .build()
        .width(argui_ui::length(0.0))
        .grow(1.0)
        .shrink(1.0)
        .min_width(argui_ui::length(0.0));
        if self.state.open {
            input = input.controls([format!("{}::content", self.popup_key())]);
        }
        if self.state.error.is_some() {
            input = input.described_by([error_key.clone()]);
        }
        let mut trigger = Button::new(self.popup_key(), &self.label, theme.outline_button());
        if let Some(icon) = &self.icon {
            trigger = trigger.content(icon.clone());
        }
        let trigger = trigger.build().padding(argui_ui::Sides::length(8.0));
        let mut popup = Popover::new(
            self.popup_key(),
            &self.label,
            self.state.open,
            trigger,
            self.calendar()
                .build(theme)
                .width(argui_ui::percent(1.0))
                .background(theme.popover)
                .border(argui_paint::Border::all(0.0, theme.popover)),
        )
        .placement(argui_ui::FloatingPlacement::new(
            argui_ui::Placement::BottomEnd,
        ))
        .trap_focus(true)
        .padding(0.0)
        .size(300.0, 380.0);
        if let Some(surface) = self.surface {
            popup = popup.surface(surface);
        }
        let popup = popup.build(theme);
        let mut children = vec![
            Element::row([input, popup])
                .gap(8.0)
                .align_items(argui_ui::AlignItems::CENTER),
        ];
        if let Some(error) = &self.state.error {
            let mut semantics = Semantics::new(Role::Alert).label(error);
            semantics.live = LiveRegion::Polite;
            children.push(
                Element::text(error.as_str())
                    .text_style(argui_text::TextStyle {
                        color: theme.destructive,
                        font_size: 13.0,
                        line_height: 18.0,
                        ..Default::default()
                    })
                    .keyed(error_key)
                    .semantics(semantics),
            );
        }
        Element::column(children)
            .width(argui_ui::length(300.0))
            .max_width(argui_ui::percent(1.0))
            .gap(8.0)
            .keyed(&self.key)
            .semantic_scope()
    }

    /// Returns the updated picker state and focus request resulting from `event`, if any.
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
            let selected = key.starts_with(&format!("{}::calendar::day::", self.key))
                && (matches!(event.kind, UiEventKind::Click(_))
                    || matches!(&event.kind, UiEventKind::KeyInput(input) if input.key == Key::Enter || input.key == Key::Character(" ".into())));
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
