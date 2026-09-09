use crate::{Collection, CollectionItem, ListState, TableColumn, WidgetTheme};
use argui_ui::{Element, SortDirection};
use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    num::NonZeroUsize,
};

type Compare<R> = dyn Fn(&R, &R) -> Ordering;
type Filter<R> = dyn Fn(&R, &str) -> bool;
type Render<R> = dyn Fn(&R, &WidgetTheme) -> Element;
type Validate<R> = dyn Fn(&R, &str) -> Result<(), String>;
type Editor = dyn Fn(&str, &str, &WidgetTheme) -> Element;

pub struct DataColumn<R> {
    pub id: String,
    pub presentation: TableColumn,
    pub minimum_width: f32,
    pub maximum_width: f32,
    pub value: Box<dyn Fn(&R) -> String>,
    pub compare: Option<Box<Compare<R>>>,
    pub filter: Option<Box<Filter<R>>>,
    pub render: Option<Box<Render<R>>>,
    pub validate: Option<Box<Validate<R>>>,
    pub editor: Option<Box<Editor>>,
}

impl<R> DataColumn<R> {
    pub fn new(
        id: impl Into<String>,
        presentation: TableColumn,
        value: impl Fn(&R) -> String + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            presentation,
            minimum_width: 1.0,
            maximum_width: f32::MAX,
            value: Box::new(value),
            compare: None,
            filter: None,
            render: None,
            validate: None,
            editor: None,
        }
    }
}

pub struct DataRow<R> {
    pub id: String,
    pub value: R,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataSort {
    pub column: String,
    pub direction: SortDirection,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DataPage {
    pub index: usize,
    pub size: NonZeroUsize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CellAddress {
    pub row: String,
    pub column: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CellEdit {
    pub address: CellAddress,
    pub original: String,
    pub draft: String,
    pub error: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CellCommit {
    pub address: CellAddress,
    pub original: String,
    pub value: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DataTableError {
    DuplicateRow(String),
    DuplicateColumn(String),
    UnknownColumn(String),
    InvalidWidth,
}

/// Retained data and transformation cache. Sorting/filtering run only when explicitly changed.
pub struct DataTableModel<R> {
    rows: Vec<DataRow<R>>,
    columns: Vec<DataColumn<R>>,
    row_indices: HashMap<String, usize>,
    column_indices: HashMap<String, usize>,
    order: Vec<usize>,
    collection: Collection,
    sorts: Vec<DataSort>,
    filters: HashMap<String, String>,
    hidden: HashSet<String>,
    page: Option<DataPage>,
    filtered_count: usize,
    selection: ListState,
    page_selected: usize,
    resizing: Option<(String, crate::SplitPane)>,
    pub active: Option<CellAddress>,
    pub(super) edit: Option<CellEdit>,
}

impl<R> DataTableModel<R> {
    pub fn new(rows: Vec<DataRow<R>>, columns: Vec<DataColumn<R>>) -> Result<Self, DataTableError> {
        let row_indices = index_rows(&rows)?;
        let mut column_indices = HashMap::new();
        for (index, column) in columns.iter().enumerate() {
            if !column.presentation.width.is_finite()
                || !column.minimum_width.is_finite()
                || !column.maximum_width.is_finite()
                || column.minimum_width <= 0.0
                || column.maximum_width < column.minimum_width
                || !(column.minimum_width..=column.maximum_width)
                    .contains(&column.presentation.width)
            {
                return Err(DataTableError::InvalidWidth);
            }
            if column_indices.insert(column.id.clone(), index).is_some() {
                return Err(DataTableError::DuplicateColumn(column.id.clone()));
            }
        }
        let mut model = Self {
            rows,
            columns,
            row_indices,
            column_indices,
            order: Vec::new(),
            collection: Collection::default(),
            sorts: Vec::new(),
            filters: HashMap::new(),
            hidden: HashSet::new(),
            page: None,
            filtered_count: 0,
            selection: ListState::default(),
            page_selected: 0,
            resizing: None,
            active: None,
            edit: None,
        };
        model.rebuild();
        Ok(model)
    }
    pub fn collection(&self) -> &Collection {
        &self.collection
    }
    pub fn visible_columns(&self) -> impl DoubleEndedIterator<Item = &DataColumn<R>> {
        self.columns
            .iter()
            .filter(|column| !self.hidden.contains(&column.id))
    }
    pub fn row(&self, index: usize) -> Option<&DataRow<R>> {
        self.order.get(index).map(|&index| &self.rows[index])
    }
    pub fn filtered_count(&self) -> usize {
        self.filtered_count
    }
    pub fn sort(&self) -> &[DataSort] {
        &self.sorts
    }
    pub fn set_sort(&mut self, sorts: Vec<DataSort>) -> Result<(), DataTableError> {
        for sort in &sorts {
            self.require_column(&sort.column)?;
        }
        self.sorts = sorts;
        self.rebuild();
        Ok(())
    }
    pub fn set_filter(&mut self, column: &str, query: String) -> Result<(), DataTableError> {
        self.require_column(column)?;
        if query.is_empty() {
            self.filters.remove(column);
        } else {
            self.filters.insert(column.into(), query);
        }
        self.rebuild();
        Ok(())
    }
    pub fn set_page(&mut self, page: Option<DataPage>) {
        self.page = page;
        self.rebuild();
    }
    pub fn set_hidden(&mut self, column: &str, hidden: bool) -> Result<(), DataTableError> {
        self.require_column(column)?;
        if hidden {
            self.hidden.insert(column.into());
        } else {
            self.hidden.remove(column);
        }
        if hidden
            && self
                .edit
                .as_ref()
                .is_some_and(|edit| edit.address.column == column)
        {
            self.edit = None;
        }
        if hidden
            && self
                .active
                .as_ref()
                .is_some_and(|active| active.column == column)
        {
            self.active = None;
        }
        Ok(())
    }
    pub fn resize(&mut self, column: &str, width: f32) -> Result<f32, DataTableError> {
        let index = self.require_column(column)?;
        let column = &mut self.columns[index];
        if !width.is_finite() {
            return Err(DataTableError::InvalidWidth);
        }
        column.presentation.width = width.clamp(column.minimum_width, column.maximum_width);
        let width = column.presentation.width;
        self.resizing = None;
        Ok(width)
    }
    pub fn replace_rows(&mut self, rows: Vec<DataRow<R>>) -> Result<(), DataTableError> {
        let indices = index_rows(&rows)?;
        self.rows = rows;
        self.row_indices = indices;
        self.selection
            .selected
            .retain(|id| self.row_indices.contains_key(id));
        self.rebuild();
        Ok(())
    }
    pub fn select_page(&mut self, selected: bool) {
        for row in self.collection.items() {
            if selected {
                self.selection.selected.insert(row.id.clone());
            } else {
                self.selection.selected.remove(&row.id);
            }
        }
        self.refresh_selected();
    }
    pub fn selection(&self) -> &ListState {
        &self.selection
    }
    pub fn set_selection(&mut self, mut selection: ListState) {
        selection
            .selected
            .retain(|id| self.row_indices.contains_key(id));
        self.selection = selection;
        self.refresh_selected();
    }
    fn refresh_selected(&mut self) {
        self.page_selected = self
            .selection
            .selected
            .iter()
            .filter(|id| self.collection.index_of(id).is_some())
            .count();
    }
    pub fn page_checked(&self) -> argui_ui::CheckedState {
        if self.page_selected == 0 {
            argui_ui::CheckedState::Unchecked
        } else if self.page_selected == self.collection.len() {
            argui_ui::CheckedState::Checked
        } else {
            argui_ui::CheckedState::Mixed
        }
    }
    pub fn edit(&self) -> Option<&CellEdit> {
        self.edit.as_ref()
    }

    pub fn begin_edit(&mut self, address: CellAddress) -> bool {
        let Some(&row) = self.row_indices.get(&address.row) else {
            return false;
        };
        let Some(&column) = self.column_indices.get(&address.column) else {
            return false;
        };
        if self.collection.index_of(&address.row).is_none()
            || self.hidden.contains(&address.column)
            || self.columns[column].validate.is_none()
        {
            return false;
        }
        let original = (self.columns[column].value)(&self.rows[row].value);
        self.active = Some(address.clone());
        self.edit = Some(CellEdit {
            address,
            draft: original.clone(),
            original,
            error: None,
        });
        true
    }
    /// Produces a controlled update request; persistence belongs to the caller.
    pub fn commit_edit(&mut self) -> Option<CellCommit> {
        let edit = self.edit.as_mut()?;
        let row = &self.rows[*self.row_indices.get(&edit.address.row)?].value;
        let column = &self.columns[*self.column_indices.get(&edit.address.column)?];
        if (column.value)(row) != edit.original {
            self.edit = None;
            return None;
        }
        if let Err(error) = column.validate.as_ref()?(row, &edit.draft) {
            edit.error = Some(error);
            return None;
        }
        let edit = self.edit.take()?;
        Some(CellCommit {
            address: edit.address,
            original: edit.original,
            value: edit.draft,
        })
    }
    pub fn apply(
        &mut self,
        action: super::DataTableAction,
    ) -> Result<Option<CellCommit>, DataTableError> {
        use super::DataTableAction;
        match action {
            DataTableAction::SelectRow(address, modifiers) => {
                if self.column_indices.contains_key(&address.column)
                    && !self.hidden.contains(&address.column)
                    && let Some(index) = self.collection.index_of(&address.row)
                {
                    self.selection
                        .select(index, &self.collection, true, modifiers);
                    self.refresh_selected();
                    self.edit = None;
                    self.active = Some(address);
                }
            }
            DataTableAction::Activate(address) => {
                if self.collection.index_of(&address.row).is_some()
                    && self.column_indices.contains_key(&address.column)
                    && !self.hidden.contains(&address.column)
                {
                    self.edit = None;
                    self.active = Some(address);
                }
            }
            DataTableAction::BeginEdit(address) => {
                self.begin_edit(address);
            }
            DataTableAction::Draft(value) => {
                if let Some(edit) = &mut self.edit {
                    edit.draft = value;
                    edit.error = None;
                }
            }
            DataTableAction::CancelEdit => self.edit = None,
            DataTableAction::CommitEdit { next } => {
                let Some(commit) = self.commit_edit() else {
                    return Ok(None);
                };
                if let Some(forward) = next {
                    let columns: Vec<_> = self
                        .visible_columns()
                        .map(|column| column.id.clone())
                        .collect();
                    if let Some(row) = self.collection.index_of(&commit.address.row)
                        && let Some(column) = columns
                            .iter()
                            .position(|column| *column == commit.address.column)
                    {
                        let cursor = row * columns.len() + column;
                        let target = if forward {
                            cursor.saturating_add(1)
                        } else {
                            cursor.saturating_sub(1)
                        };
                        if let Some(row) = self.row(target / columns.len()) {
                            self.active = Some(CellAddress {
                                row: row.id.clone(),
                                column: columns[target % columns.len()].clone(),
                            });
                        }
                    }
                }
                return Ok(Some(commit));
            }
            DataTableAction::ResizeColumn { column, event } => {
                let index = self.require_column(&column)?;
                if self.resizing.as_ref().is_none_or(|(id, _)| *id != column) {
                    let definition = &self.columns[index];
                    self.resizing = Some((
                        column,
                        crate::SplitPane::new(
                            event.target_key().unwrap_or_default(),
                            crate::SplitAxis::Horizontal,
                            definition.presentation.width,
                            definition.minimum_width,
                            definition.maximum_width,
                        ),
                    ));
                }
                if let Some((_, pane)) = &mut self.resizing {
                    pane.update(&event);
                    self.columns[index].presentation.width = pane.size;
                }
            }
            DataTableAction::Sort(sorts) => self.set_sort(sorts)?,
            DataTableAction::SelectPage(selected) => self.select_page(selected),
        }
        Ok(None)
    }

    fn require_column(&self, id: &str) -> Result<usize, DataTableError> {
        self.column_indices
            .get(id)
            .copied()
            .ok_or_else(|| DataTableError::UnknownColumn(id.into()))
    }
    fn rebuild(&mut self) {
        let mut order: Vec<_> = (0..self.rows.len())
            .filter(|&index| {
                self.filters.iter().all(|(id, query)| {
                    let column = &self.columns[self.column_indices[id]];
                    let row = &self.rows[index].value;
                    column.filter.as_ref().map_or_else(
                        || {
                            (column.value)(row)
                                .to_lowercase()
                                .contains(&query.to_lowercase())
                        },
                        |filter| filter(row, query),
                    )
                })
            })
            .collect();
        order.sort_by(|&a, &b| {
            for sort in &self.sorts {
                let column = &self.columns[self.column_indices[&sort.column]];
                let a = &self.rows[a].value;
                let b = &self.rows[b].value;
                let cmp = column.compare.as_ref().map_or_else(
                    || (column.value)(a).cmp(&(column.value)(b)),
                    |compare| compare(a, b),
                );
                let cmp = if sort.direction == SortDirection::Descending {
                    cmp.reverse()
                } else {
                    cmp
                };
                if cmp != Ordering::Equal {
                    return cmp;
                }
            }
            Ordering::Equal
        });
        self.filtered_count = order.len();
        if let Some(page) = self.page {
            let start = page.index.saturating_mul(page.size.get()).min(order.len());
            order = order[start..start.saturating_add(page.size.get()).min(order.len())].to_vec();
        }
        self.collection = Collection::new(
            order
                .iter()
                .map(|&index| CollectionItem::new(&self.rows[index].id, &self.rows[index].id)),
        )
        .expect("validated row identities");
        self.order = order;
        self.refresh_selected();
        if self
            .active
            .as_ref()
            .is_some_and(|active| self.collection.index_of(&active.row).is_none())
        {
            self.active = None;
        }
        if self.edit.as_ref().is_some_and(|edit| {
            self.collection.index_of(&edit.address.row).is_none()
                || self.row_indices.get(&edit.address.row).is_none_or(|&row| {
                    (self.columns[self.column_indices[&edit.address.column]].value)(
                        &self.rows[row].value,
                    ) != edit.original
                })
        }) {
            self.edit = None;
        }
    }
}
fn index_rows<R>(rows: &[DataRow<R>]) -> Result<HashMap<String, usize>, DataTableError> {
    let mut indices = HashMap::with_capacity(rows.len());
    for (index, row) in rows.iter().enumerate() {
        if indices.insert(row.id.clone(), index).is_some() {
            return Err(DataTableError::DuplicateRow(row.id.clone()));
        }
    }
    Ok(indices)
}
