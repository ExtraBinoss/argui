use crate::navigation::Page;
use argui::{
    core::Point,
    runtime::{Context, Entity, Render},
    ui::{
        Element, EventType, ScrollRequest, UiEvent, UiEventKind, VirtualAlignment, VirtualList,
        length,
    },
    widgets::{List, ListState, Table, TableColumn, VList, shadcn},
};

pub(crate) struct DataDemo {
    page: Page,
    selection: ListState,
    heights: VirtualList,
    offset: f32,
}

impl Default for DataDemo {
    fn default() -> Self {
        Self {
            page: Page::List,
            selection: ListState::default(),
            heights: VirtualList::variable(10_000, 40.0, 320.0),
            offset: 0.0,
        }
    }
}

pub(crate) fn render(
    entity: &Entity<DataDemo>,
    page: Page,
    cx: &mut Context<crate::WidgetGallery>,
) -> Element {
    if entity.read(|demo| demo.page != page) {
        entity.update(|demo, cx| {
            demo.page = page;
            demo.selection = ListState::default();
            cx.notify();
        });
    }
    cx.entity(entity)
}

impl DataDemo {
    fn count(&self) -> usize {
        if self.page == Page::VList { 10_000 } else { 8 }
    }

    fn event(&mut self, event: &UiEvent, cx: &mut Context<Self>) {
        if let UiEventKind::Scrolled { offset, .. } = event.kind {
            if event.target_key() == Some("data") {
                self.offset = offset.y;
                cx.notify();
            }
            return;
        }
        if let Some(state) = List::new("data", self.count())
            .selection(&self.selection, true)
            .action(event)
        {
            self.selection = state;
            let _ = event.prevent_default();
            event.stop_propagation();
            if let Some(active) = self.selection.active {
                cx.request_focus(format!("data::row::{active}"));
            }
            if self.page == Page::VList
                && let Some(active) = self.selection.active
            {
                self.offset =
                    self.heights
                        .scroll_to(active, VirtualAlignment::Nearest, self.offset);
                cx.scroll(ScrollRequest::offset("data", Point::new(0.0, self.offset)));
            }
            cx.notify();
        }
    }
}

impl Render for DataDemo {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = shadcn(cx.environment().primary);
        let theme = themes.resolve(cx.environment().color_scheme);
        let text_style = argui::text::TextStyle {
            color: theme.foreground,
            font_size: 14.0,
            line_height: 20.0,
            ..Default::default()
        };
        let text = |value: String| Element::text(value).text_style(text_style.clone());
        let row = |index: usize| {
            text(format!("Item {}", index + 1))
                .height(length(if index.is_multiple_of(3) { 64.0 } else { 36.0 }))
                .padding(argui::ui::Sides::length(8.0))
        };
        let content = match self.page {
            Page::VList => VList::variable("data", &self.heights, self.offset).build_list(
                self.count(),
                &self.selection,
                true,
                theme,
                row,
            ),
            Page::Table => Table::new(
                "data",
                [
                    TableColumn::new("Name", 180.0),
                    TableColumn::new("Value", 120.0),
                ],
                self.count(),
            )
            .selection(&self.selection, true)
            .build(theme, |index, column| {
                text(if column == 0 {
                    format!("Item {}", index + 1)
                } else {
                    format!("{}", index * 10)
                })
                .padding(argui::ui::Sides::length(8.0))
            }),
            _ => List::new("data", self.count())
                .selection(&self.selection, true)
                .build(theme, row),
        };
        Element::column([
            text("Click · Ctrl/Cmd to toggle · Shift to select a range · Arrow keys".into()),
            content
                .on(cx.listener(EventType::Click, Self::event))
                .on(cx.listener(EventType::Key, Self::event))
                .on(cx.listener(EventType::Scroll, Self::event)),
            text(format!("{} selected", self.selection.selected.len())),
        ])
        .gap(12.0)
    }
}
