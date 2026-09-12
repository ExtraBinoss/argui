use super::*;
use crate::app::text;
use argui::{
    core::Color,
    ui::{length, percent},
    widgets::*,
};

impl CatalogueDemo {
    fn sheet(&self, theme: &WidgetTheme) -> Sheet {
        Sheet::new(
            "settings",
            "Settings",
            self.open,
            Button::new("open", "Open settings", theme.button()).build(),
            Element::column([
                text("Appearance", 24.0, theme.foreground, 650),
                text(
                    "Keep your preferences close at hand.",
                    14.0,
                    theme.muted_foreground,
                    400,
                ),
                Button::new("settings::close", "Done", theme.outline_button()).build(),
            ])
            .gap(16.0),
        )
    }

    fn alert_dialog(&self, theme: &WidgetTheme) -> AlertDialog {
        AlertDialog::new(
            "confirm",
            "Archive this project?",
            "The project will move to your archive. You can restore it later.",
            self.open,
            Button::new("open", "Archive project", theme.destructive_button()).build(),
        )
    }

    fn drawer(&self, theme: &WidgetTheme) -> Drawer {
        let mut drawer = Drawer::new(
            "details",
            "Project details",
            self.open,
            Button::new("open", "Open project details", theme.button()).build(),
            Element::column([
                text("Summer collection", 24.0, theme.foreground, 700),
                text(
                    "12 designs · Updated today",
                    14.0,
                    theme.muted_foreground,
                    400,
                ),
            ])
            .gap(12.0),
        );
        drawer.offset = self.offset;
        drawer
    }

    fn carousel(&self, theme: &WidgetTheme) -> Carousel {
        Carousel::new(
            "collection",
            "Collections",
            ["Summer", "Autumn", "Winter"]
                .into_iter()
                .enumerate()
                .map(|(index, name)| {
                    Element::column([
                        text(format!("0{}", index + 1), 14.0, theme.muted_foreground, 600),
                        text(name, 32.0, theme.foreground, 700),
                        text(
                            "A new collection to explore",
                            14.0,
                            theme.muted_foreground,
                            400,
                        ),
                    ])
                    .height(length(220.0))
                    .width(percent(1.0))
                    .padding(argui::ui::Sides::length(24.0))
                    .gap(12.0)
                    .background(theme.muted)
                    .radius(argui::paint::CornerRadii::all(12.0))
                }),
            self.selected,
        )
    }

    fn chart(&self) -> Chart {
        let mut chart = Chart::new(
            "sales",
            "Quarterly revenue",
            ["Q1", "Q2", "Q3", "Q4"].map(str::to_owned),
            [
                ChartSeries {
                    label: "Studio".into(),
                    values: vec![18.0, 32.0, 24.0, 46.0],
                    color: Color::from_srgb8(59, 130, 246),
                },
                ChartSeries {
                    label: "Team".into(),
                    values: vec![8.0, 22.0, 30.0, 38.0],
                    color: Color::from_srgb8(16, 185, 129),
                },
            ],
        );
        chart.kind = if self.open {
            ChartKind::Line
        } else {
            ChartKind::Bar
        };
        chart
    }

    pub(super) fn surfaces_view(&self, theme: &WidgetTheme, cx: &mut Context<Self>) -> Element {
        match self.page {
            Page::Sheet => self.sheet(theme).build(theme),
            Page::AlertDialog => self.alert_dialog(theme).build(theme),
            Page::Drawer => self.drawer(theme).build(theme),
            Page::Carousel => self.carousel(theme).build(theme),
            Page::Chart => Element::column([
                Toggle::new("chart-line", "Line chart", self.open).build(theme),
                ScrollArea {
                    orientation: argui::ui::Orientation::Horizontal,
                    ..ScrollArea::new(
                        "chart-scroll",
                        "Revenue chart",
                        300.0,
                        self.chart().build(theme).expect("finite chart data"),
                    )
                }
                .build(theme),
            ])
            .gap(18.0),
            Page::HoverCard => {
                let card = HoverCard::new(
                    "profile",
                    "Ada Lovelace",
                    self.hover.is_open(),
                    Button::new("profile", "Ada Lovelace", theme.outline_button()).build(),
                    Element::column([
                        text("Ada Lovelace", 18.0, theme.foreground, 700),
                        text(
                            "Designing thoughtful interfaces with Argui.",
                            14.0,
                            theme.muted_foreground,
                            400,
                        ),
                        Button::new("profile-follow", "Follow", theme.button()).build(),
                    ])
                    .gap(12.0),
                );
                let mut root = card.build(theme);
                for child in &mut root.children {
                    for kind in [
                        EventType::PointerEnter,
                        EventType::PointerLeave,
                        EventType::Focus,
                        EventType::Blur,
                        EventType::Key,
                    ] {
                        *child = child
                            .clone()
                            .on(cx.listener(kind, Self::event).capture(true));
                    }
                }
                root
            }
            _ => unreachable!("surface page"),
        }
    }

    pub(super) fn surfaces_event(
        &mut self,
        event: &UiEvent,
        theme: &WidgetTheme,
        cx: &mut Context<Self>,
    ) -> bool {
        match self.page {
            Page::Sheet => {
                if let Some(action) = self.sheet(theme).action(event) {
                    self.open = action == DialogAction::Open;
                    return true;
                }
            }
            Page::AlertDialog => {
                if let Some(action) = self.alert_dialog(theme).action(event) {
                    self.open = action == AlertDialogAction::Open;
                    if action == AlertDialogAction::Confirm {
                        self.status = "Project archived.".into();
                    }
                    return true;
                }
            }
            Page::Drawer => {
                if let Some(action) = self.drawer(theme).action(event) {
                    match action {
                        DrawerAction::Open => {
                            self.open = true;
                            self.offset = 0.0;
                        }
                        DrawerAction::Close => {
                            self.open = false;
                            self.offset = 0.0;
                        }
                        DrawerAction::Drag(offset) => self.offset = offset,
                    }
                    return true;
                }
            }
            Page::Carousel => {
                if let Some(index) = self.carousel(theme).action(event) {
                    self.selected = index;
                    return true;
                }
            }
            Page::Chart => {
                if let Some(value) =
                    Toggle::new("chart-line", "Line chart", self.open).action(event)
                {
                    self.open = value;
                    return true;
                }
                if let Some((series, category)) = self.chart().action(event) {
                    self.status = format!(
                        "{} · {}: {}",
                        self.chart().series[series].label,
                        self.chart().categories[category],
                        self.chart().series[series].values[category]
                    );
                    return true;
                }
            }
            Page::HoverCard => {
                if ButtonBehavior::new("profile-follow", "Follow")
                    .action(event)
                    .is_some()
                {
                    self.status = "You are now following Ada.".into();
                    return true;
                }
                if self.hover.update(event, self.origin.elapsed()) {
                    self.timer.cancel();
                    if let Some(deadline) = self.hover.next_deadline() {
                        let delay = deadline.saturating_sub(self.origin.elapsed());
                        if cx
                            .spawn_latest(
                                &mut self.timer,
                                argui::runtime::tasks::sleep(delay),
                                |demo, _, cx| {
                                    demo.hover.advance(demo.origin.elapsed());
                                    cx.notify();
                                },
                            )
                            .is_err()
                        {
                            self.status = "Preview timer unavailable.".into();
                        }
                    }
                    return true;
                }
            }
            _ => unreachable!("surface page"),
        }
        false
    }
}
