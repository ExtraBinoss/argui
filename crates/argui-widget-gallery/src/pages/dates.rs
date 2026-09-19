use argui::{
    runtime::{Context, Entity, Render},
    ui::{Element, EventType, UiEvent},
    widgets::{
        Calendar, CalendarSelection, CalendarState, Date, DatePicker, DatePickerState,
        IsoCalendarLocale, Month, shadcn,
    },
};

pub(crate) struct DatesDemo {
    icons: argui::widgets::WidgetAssets,
    picker: bool,
    today: Date,
    calendar: CalendarState,
    date: DatePickerState,
}
impl DatesDemo {
    pub(crate) fn new(icons: &argui::widgets::WidgetAssets) -> Self {
        let epoch = Date::from_calendar_date(1970, Month::January, 1).expect("epoch");
        let days = web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            / 86_400;
        let today = Date::from_julian_day(
            epoch
                .to_julian_day()
                .saturating_add(days.min(i32::MAX as u64) as i32),
        )
        .unwrap_or(Date::MAX);
        Self {
            icons: icons.clone(),
            picker: false,
            today,
            calendar: CalendarState::new(today, CalendarSelection::Single(None)),
            date: DatePickerState::new(None, today, &IsoCalendarLocale),
        }
    }
}

pub(crate) fn render(
    entity: &Entity<DatesDemo>,
    picker: bool,
    cx: &mut Context<crate::WidgetGallery>,
) -> Element {
    if entity.read(|demo| demo.picker != picker) {
        entity.update(|demo, cx| {
            demo.picker = picker;
            cx.notify();
        });
    }
    cx.entity(entity)
}
impl DatesDemo {
    fn event(&mut self, event: &UiEvent, cx: &mut Context<Self>) {
        if self.picker {
            if let Some(response) =
                DatePicker::new("date", "Choose a date", &self.date, self.today).action(event)
            {
                self.date = response.state;
                if let Some(focus) = response.focus {
                    cx.request_focus(focus);
                }
                let _ = event.prevent_default();
                event.stop_propagation();
                cx.notify();
            }
        } else if let Some(next) =
            Calendar::new("calendar", "Calendar", &self.calendar, self.today).action(event)
        {
            let key = Calendar::new("calendar", "Calendar", &next, self.today).focus_key();
            self.calendar = next;
            cx.request_focus(key);
            let _ = event.prevent_default();
            event.stop_propagation();
            cx.notify();
        }
    }
}
impl Render for DatesDemo {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = shadcn(cx.environment());
        let theme = themes.resolve(cx.environment().color_scheme);
        let content = if self.picker {
            DatePicker::new("date", "Choose a date", &self.date, self.today)
                .icon(
                    self.icons
                        .icon(argui::widgets::TablerIcon::Calendar, 18.0)
                        .vector_color(theme.foreground),
                )
                .build(theme)
        } else {
            Calendar::new("calendar", "Calendar", &self.calendar, self.today).build(theme)
        };
        content
            .on(cx.listener(EventType::Click, Self::event))
            .on(cx.listener(EventType::Key, Self::event))
            .on(cx.listener(EventType::Input, Self::event))
            .on(cx.listener(EventType::Submit, Self::event))
            .on(cx.listener(EventType::PointerOutside, Self::event))
            .on(cx.listener(EventType::Dismiss, Self::event))
    }
}
