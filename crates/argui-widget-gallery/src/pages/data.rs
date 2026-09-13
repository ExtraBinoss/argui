use crate::navigation::Page;
use argui::{
    core::Point,
    runtime::{Context, Entity, Render},
    ui::{
        Element, EventType, ScrollRequest, UiEvent, UiEventKind, VirtualAlignment, VirtualList,
        VirtualWindow, length,
    },
    widgets::{Collection, CollectionItem, List, ListState, Table, TableColumn, VList, shadcn},
};

pub(crate) struct DataDemo {
    page: Page,
    items: Collection,
    selection: ListState,
    heights: VirtualList,
    offset: f32,
    window: Option<VirtualWindow>,
}

impl Default for DataDemo {
    fn default() -> Self {
        Self {
            page: Page::List,
            items: items(8),
            selection: ListState::default(),
            heights: VirtualList::variable(10_000, 40.0, 320.0).overscan(8),
            offset: 0.0,
            window: None,
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
            demo.items = items(if page == Page::VList { 10_000 } else { 8 });
            demo.selection = ListState::default();
            demo.offset = 0.0;
            cx.notify();
        });
    }
    cx.entity(entity)
}

impl DataDemo {
    fn event(&mut self, event: &UiEvent, cx: &mut Context<Self>) {
        if let UiEventKind::Scrolled { offset, .. } = event.kind {
            if event.target_key() == Some("data") {
                self.offset = offset.y;
                if self.page == Page::VList
                    && self.window.as_ref() != Some(&self.heights.window(self.offset))
                {
                    cx.notify();
                }
            }
            return;
        }
        if let Some(state) = List::new("data", &self.items)
            .selection(&self.selection, true)
            .action(event)
        {
            self.selection = state;
            let _ = event.prevent_default();
            event.stop_propagation();
            cx.request_focus("data");
            if self.page == Page::VList
                && let Some(active) = self
                    .selection
                    .active
                    .as_deref()
                    .and_then(|id| self.items.index_of(id))
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
        let themes = shadcn(cx.environment());
        let theme = themes.resolve(cx.environment().color_scheme);
        self.window = (self.page == Page::VList).then(|| self.heights.window(self.offset));
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
                &self.items,
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
                &self.items,
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
            _ => List::new("data", &self.items)
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

fn items(count: usize) -> Collection {
    Collection::new(
        (0..count).map(|i| CollectionItem::new(i.to_string(), format!("Item {}", i + 1))),
    )
    .expect("unique item identities")
}
