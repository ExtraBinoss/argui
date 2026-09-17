use crate::{List, ListState, WidgetTheme};
use argui_ui::{Element, Role, Semantics, ValueHandler, length};

#[derive(Clone, Debug)]
pub struct TableColumn {
    pub label: String,
    pub width: f32,
}

impl TableColumn {
    /// Creates a column with display `label` and preferred width in logical pixels.
    ///
    /// # Panics
    ///
    /// Panics if `width` is not finite and positive.
    #[must_use]
    pub fn new(label: impl Into<String>, width: f32) -> Self {
        assert!(width.is_finite() && width > 0.0);
        Self {
            label: label.into(),
            width,
        }
    }
}

/// A table with shared column widths and controlled row selection.
#[derive(Clone, Debug)]
pub struct Table<'a> {
    list: List<'a>,
    columns: Vec<TableColumn>,
    count: usize,
}

impl<'a> Table<'a> {
    /// Creates a table from `columns` and rows in display order.
    /// `key` identifies the table, `label` names it accessibly, and `collection` supplies its rows.
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        columns: impl IntoIterator<Item = TableColumn>,
        collection: &'a crate::Collection,
    ) -> Self {
        Self {
            list: List::new(key, collection),
            columns: columns.into_iter().collect(),
            count: collection.len(),
        }
    }

    #[must_use]
    /// Enables controlled row selection using `state`; `multiple` allows selecting more than one row.
    pub fn selection(mut self, state: &'a ListState, multiple: bool) -> Self {
        self.list = self.list.selection(state, multiple);
        self
    }

    /// Adds a handler that receives the stable id of a selected row.
    #[must_use]
    pub fn on_select(mut self, handler: ValueHandler<String>) -> Self {
        self.list = self.list.on_select(handler);
        self
    }

    /// Adds a handler that receives the stable id of an activated row.
    #[must_use]
    pub fn on_activate(mut self, handler: ValueHandler<String>) -> Self {
        self.list = self.list.on_activate(handler);
        self
    }

    #[must_use]
    /// Sets the accessible name of the table; `label` is announced to assistive technology.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.list = self.list.label(label);
        self
    }

    #[must_use]
    /// Returns updated selection state when `event` selects a row.
    pub fn action(&self, event: &argui_ui::UiEvent) -> Option<ListState> {
        self.list.action(event)
    }

    #[must_use]
    /// Builds the table using `theme` and `row` to render each row.
    ///
    /// # Panics
    ///
    /// Panics if the list builder does not provide expected semantics. `cell` builds the content for each cell.
    pub fn build(
        &self,
        theme: &WidgetTheme,
        mut cell: impl FnMut(usize, usize) -> Element,
    ) -> Element {
        let header = Element::row(self.columns.iter().map(|column| {
            Element::text(column.label.as_str())
                .text_style(argui_text::TextStyle {
                    color: theme.muted_foreground,
                    font_size: 14.0,
                    line_height: 20.0,
                    ..Default::default()
                })
                .padding(argui_ui::Sides::length(8.0))
                .width(length(column.width))
                .shrink(0.0)
                .semantics(Semantics::new(Role::ColumnHeader).label(&column.label))
        }))
        .semantics(Semantics::new(Role::Row));
        let rows = (0..self.count).map(|index| {
            let row = Element::row(self.columns.iter().enumerate().map(|(col, column)| {
                let cell = cell(index, col);
                let semantics = crate::list::content_semantics(Role::Cell, &cell);
                cell.width(length(column.width))
                    .shrink(0.0)
                    .semantics(semantics)
            }));
            let mut row = self.list.row(index, row, theme);
            row.semantics.as_mut().expect("list row semantics").role = Role::Row;
            row
        });
        let mut root = self
            .list
            .root(Element::column(std::iter::once(header).chain(rows)));
        root.semantics.as_mut().expect("list semantics").role = Role::Grid;
        root
    }
}
