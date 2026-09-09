use argui::{
    core::Point,
    runtime::{Context, Render},
    ui::{Element, EventType, ScrollRequest, UiEvent, UiEventKind, VirtualAlignment, VirtualList},
    widgets::{DataColumn, DataRow, DataTable, DataTableModel, TableColumn, shadcn},
};

pub(crate) struct TableDemo {
    model: DataTableModel<i32>,
    heights: VirtualList,
    offset: f32,
}
impl Default for TableDemo {
    fn default() -> Self {
        let mut column =
            DataColumn::new("value", TableColumn::new("Value", 240.0), |value: &i32| {
                value.to_string()
            });
        column.compare = Some(Box::new(i32::cmp));
        column.validate = Some(Box::new(|_, value| {
            value
                .parse::<i32>()
                .map(|_| ())
                .map_err(|_| "Enter an integer".into())
        }));
        Self {
            model: DataTableModel::new(
                (0..10_000)
                    .map(|value| DataRow {
                        id: value.to_string(),
                        value,
                    })
                    .collect(),
                vec![column],
            )
            .expect("unique rows"),
            heights: VirtualList::variable(10_000, 44.0, 320.0),
            offset: 0.0,
        }
    }
}
impl TableDemo {
    fn event(&mut self, event: &UiEvent, cx: &mut Context<Self>) {
        if let UiEventKind::Scrolled { offset, .. } = event.kind
            && event.target_key() == Some("grid::rows")
        {
            self.offset = offset.y;
            cx.notify();
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
                        value: if row.id == commit.address.row {
                            value
                        } else {
                            row.value
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
        DataTable::new("grid", &self.model, &self.heights, self.offset)
            .build(themes.resolve(cx.environment().color_scheme))
            .on(cx.listener(EventType::Click, Self::event))
            .on(cx.listener(EventType::Key, Self::event))
            .on(cx.listener(EventType::Input, Self::event))
            .on(cx.listener(EventType::Submit, Self::event))
            .on(cx.listener(EventType::Scroll, Self::event))
            .on(cx.listener(EventType::Gesture, Self::event))
    }
}
