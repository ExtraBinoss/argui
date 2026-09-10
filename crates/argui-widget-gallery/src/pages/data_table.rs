use argui::{
    core::Point,
    runtime::{Context, Render},
    ui::{
        Element, EventType, ScrollRequest, UiEvent, UiEventKind, VirtualAlignment, VirtualList,
        VirtualWindow,
    },
    widgets::{DataColumn, DataRow, DataTable, DataTableModel, TableColumn, shadcn},
};

#[derive(Clone)]
struct Task {
    title: String,
    owner: &'static str,
    status: &'static str,
    hours: i32,
}

pub(crate) struct TableDemo {
    icons: argui::widgets::WidgetAssets,
    model: DataTableModel<Task>,
    heights: VirtualList,
    offset: f32,
    window: Option<VirtualWindow>,
}
impl TableDemo {
    pub(crate) fn new(icons: &argui::widgets::WidgetAssets) -> Self {
        let mut title = DataColumn::new("task", TableColumn::new("Task", 280.0), |task: &Task| {
            task.title.clone()
        });
        title.compare = Some(Box::new(|a, b| a.title.cmp(&b.title)));
        let mut status = DataColumn::new(
            "status",
            TableColumn::new("Status", 148.0),
            |task: &Task| task.status.into(),
        );
        status.compare = Some(Box::new(|a, b| a.status.cmp(b.status)));
        status.render = Some(Box::new(|task, theme| {
            Element::text(task.status)
                .text_style(argui::text::TextStyle {
                    color: theme.foreground,
                    font_size: 12.0,
                    line_height: 18.0,
                    wrap: argui::text::TextWrap::None,
                    ..Default::default()
                })
                .padding(argui::ui::sides(8.0, 3.0))
                .background(theme.muted)
                .radius(argui::paint::CornerRadii::all(6.0))
        }));
        let mut owner = DataColumn::new(
            "owner",
            TableColumn::new("Assignee", 172.0),
            |task: &Task| task.owner.into(),
        );
        owner.compare = Some(Box::new(|a, b| a.owner.cmp(b.owner)));
        let mut column =
            DataColumn::new("value", TableColumn::new("Hours", 120.0), |task: &Task| {
                task.hours.to_string()
            });
        column.compare = Some(Box::new(|a, b| a.hours.cmp(&b.hours)));
        column.validate = Some(Box::new(|_, value| {
            value
                .parse::<i32>()
                .map(|_| ())
                .map_err(|_| "Enter an integer".into())
        }));
        Self {
            icons: icons.clone(),
            model: DataTableModel::new(
                (0..10_000)
                    .map(|value| DataRow {
                        id: value.to_string(),
                        value: Task {
                            title: [
                                "Audit the design system",
                                "Build the settings page",
                                "Review keyboard navigation",
                                "Update the documentation",
                                "Polish empty states",
                                "Ship the component gallery",
                            ][value as usize % 6]
                                .into(),
                            owner: ["Alex Morgan", "Sam Chen", "Jordan Lee"][value as usize % 3],
                            status: ["In progress", "Done", "Backlog"][value as usize % 3],
                            hours: value % 24,
                        },
                    })
                    .collect(),
                vec![title, status, owner, column],
            )
            .expect("unique rows"),
            heights: VirtualList::variable(10_000, 52.0, 364.0),
            offset: 0.0,
            window: None,
        }
    }
}
impl TableDemo {
    fn event(&mut self, event: &UiEvent, cx: &mut Context<Self>) {
        if let UiEventKind::Scrolled { offset, .. } = event.kind
            && event.target_key() == Some("grid::rows")
        {
            self.offset = offset.y;
            if self.window.as_ref() != Some(&self.heights.window(self.offset)) {
                cx.notify();
            }
            return;
        }
        let Some(action) =
            DataTable::new("grid", &self.model, &self.heights, self.offset).action(event)
        else {
            return;
        };
        let previous = self.model.collection().clone();
        let resizing = matches!(action, argui::widgets::DataTableAction::ResizeColumn { .. });
        if let Ok(Some(commit)) = self.model.apply(action) {
            let value = commit.value.parse::<i32>().expect("validated integer edit");
            let rows = (0..self.model.collection().len())
                .map(|index| {
                    let row = self.model.row(index).expect("current row");
                    DataRow {
                        id: row.id.clone(),
                        value: Task {
                            hours: if row.id == commit.address.row {
                                value
                            } else {
                                row.value.hours
                            },
                            ..row.value.clone()
                        },
                    }
                })
                .collect();
            self.model
                .replace_rows(rows)
                .expect("existing row identities");
        }
        self.model
            .collection()
            .remap_heights(&previous, &mut self.heights);
        if resizing {
            let _ = event.prevent_default();
            event.stop_propagation();
            cx.notify();
            return;
        }
        if let Some(edit) = self.model.edit() {
            cx.request_focus(
                DataTable::new("grid", &self.model, &self.heights, self.offset)
                    .editor_key(&edit.address),
            );
        } else {
            cx.request_focus("grid");
        }
        if let Some(index) = self
            .model
            .active
            .as_ref()
            .and_then(|address| self.model.collection().index_of(&address.row))
        {
            self.offset = self
                .heights
                .scroll_to(index, VirtualAlignment::Nearest, self.offset);
            cx.scroll(ScrollRequest::offset(
                "grid::rows",
                Point::new(0.0, self.offset),
            ));
        }
        let _ = event.prevent_default();
        event.stop_propagation();
        cx.notify();
    }
}
impl Render for TableDemo {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = shadcn(cx.environment().primary);
        let theme = themes.resolve(cx.environment().color_scheme);
        self.window = Some(self.heights.window(self.offset));
        Element::column([
            DataTable::new("grid", &self.model, &self.heights, self.offset)
                .icons(&self.icons)
                .build(theme),
            Element::text(format!(
                "{} of {} rows selected",
                self.model.selection().selected.len(),
                self.model.collection().len()
            ))
            .text_style(argui::text::TextStyle {
                color: theme.muted_foreground,
                font_size: 13.0,
                line_height: 20.0,
                ..Default::default()
            }),
        ])
        .gap(12.0)
        .on(cx.listener(EventType::Click, Self::event))
        .on(cx.listener(EventType::Key, Self::event))
        .on(cx.listener(EventType::Input, Self::event))
        .on(cx.listener(EventType::Submit, Self::event))
        .on(cx.listener(EventType::Scroll, Self::event))
        .on(cx.listener(EventType::Gesture, Self::event))
    }
}
