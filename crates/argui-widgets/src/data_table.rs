use crate::{Button, Checkbox, Input, ToggleAction, ToggleBehavior, WidgetTheme};
use argui_core::{Key, KeyState};
use argui_paint::{Border, BorderWidths, CornerRadii, QuadStyle};
use argui_text::{TextStyle, TextWrap};
use argui_ui::{
    Element, FocusPolicy, GestureSet, GridPosition, Interaction, Role, SemanticState, Semantics,
    SortDirection, TapGesture, UiEvent, UiEventKind, VirtualList, length,
};

mod model;
pub use model::{
    CellAddress, CellCommit, CellEdit, DataColumn, DataPage, DataRow, DataSort, DataTableError,
    DataTableModel,
};

#[derive(Clone, Debug)]
pub enum DataTableAction {
    Activate(CellAddress),
    SelectRow(CellAddress, argui_core::Modifiers),
    BeginEdit(CellAddress),
    Draft(String),
    CancelEdit,
    CommitEdit { next: Option<bool> },
    Sort(Vec<DataSort>),
    SelectPage(bool),
    ResizeColumn { column: String, event: UiEvent },
}

pub struct DataTable<'a, R> {
    key: String,
    pub label: String,
    pub model: &'a DataTableModel<R>,
    pub heights: &'a VirtualList,
    pub offset: f32,
    pub select_page_label: String,
    icons: Option<&'a crate::WidgetAssets>,
}

impl<'a, R> DataTable<'a, R> {
    pub fn new(
        key: impl Into<String>,
        model: &'a DataTableModel<R>,
        heights: &'a VirtualList,
        offset: f32,
    ) -> Self {
        let key = key.into();
        Self {
            label: key.clone(),
            key,
            model,
            heights,
            offset,
            select_page_label: "Select page".into(),
            icons: None,
        }
    }
    pub fn icons(mut self, icons: &'a crate::WidgetAssets) -> Self {
        self.icons = Some(icons);
        self
    }
    pub fn cell_key(&self, address: &CellAddress) -> String {
        self.address_key("cell", address)
    }
    pub fn editor_key(&self, address: &CellAddress) -> String {
        self.address_key("edit", address)
    }
    fn address_key(&self, part: &str, address: &CellAddress) -> String {
        format!(
            "{}::{part}::{}:{}{}",
            self.key,
            address.row.len(),
            address.row,
            address.column
        )
    }
    fn parse_address(&self, part: &str, key: &str) -> Option<CellAddress> {
        let encoded = key.strip_prefix(&format!("{}::{part}::", self.key))?;
        let (len, value) = encoded.split_once(':')?;
        let len = len.parse::<usize>().ok()?;
        let row = value.get(..len)?;
        let column = value.get(len..)?;
        if self.model.collection().index_of(row).is_none()
            || !self.model.visible_columns().any(|item| item.id == column)
        {
            return None;
        }
        Some(CellAddress {
            row: row.into(),
            column: column.into(),
        })
    }
    pub fn build(&self, theme: &WidgetTheme) -> Element {
        assert_eq!(
            self.heights.item_count(),
            self.model.collection().len(),
            "table heights must match the current page"
        );
        let columns: Vec<_> = self.model.visible_columns().collect();
        let row_border = Border {
            widths: BorderWidths {
                bottom: 1.0,
                ..BorderWidths::all(0.0)
            },
            color: theme.border,
        };
        let text = TextStyle {
            color: theme.foreground,
            font_size: 14.0,
            line_height: 20.0,
            wrap: TextWrap::None,
            ..Default::default()
        };
        let content_width = columns
            .iter()
            .map(|column| column.presentation.width)
            .sum::<f32>();
        let header = Element::row(columns.iter().map(|column| {
            let mut semantics =
                Semantics::new(Role::ColumnHeader).label(&column.presentation.label);
            semantics.sort = self
                .model
                .sort()
                .iter()
                .find(|sort| sort.column == column.id)
                .map(|sort| sort.direction);
            let mut style = theme.ghost_button();
            style.label.color = theme.muted_foreground;
            style.label.font_size = 13.0;
            style.layout.padding = argui_ui::sides(12.0, 0.0);
            let indicator = self
                .icons
                .map_or_else(
                    || {
                        Element::text(match semantics.sort {
                            Some(SortDirection::Ascending) => "^",
                            Some(SortDirection::Descending) => "v",
                            _ => ":",
                        })
                        .text_style(TextStyle {
                            color: theme.muted_foreground,
                            ..text.clone()
                        })
                    },
                    |icons| {
                        icons
                            .icon(
                                match semantics.sort {
                                    Some(SortDirection::Ascending) => crate::TablerIcon::ArrowUp,
                                    Some(SortDirection::Descending) => crate::TablerIcon::ArrowDown,
                                    _ => crate::TablerIcon::ArrowsSort,
                                },
                                14.0,
                            )
                            .vector_color(theme.muted_foreground)
                    },
                )
                .semantic_hidden(true);
            let button = Button::new(
                format!("{}::sort::{}", self.key, column.id),
                &column.presentation.label,
                style,
            )
            .trailing(indicator)
            .build()
            .justify_content(argui_ui::JustifyContent::SPACE_BETWEEN)
            .grow(1.0)
            .min_width(length(0.0));
            let mut separator = crate::SplitPane::new(
                format!("{}::resize::{}", self.key, column.id),
                crate::SplitAxis::Horizontal,
                column.presentation.width,
                column.minimum_width,
                column.maximum_width,
            )
            .separator(theme);
            if let Some(semantics) = &mut separator.semantics {
                semantics.label = Some(column.presentation.label.clone());
            }
            Element::row([button, separator])
                .width(length(column.presentation.width))
                .grow(1.0)
                .shrink(0.0)
                .semantics(semantics)
        }))
        .height(length(44.0))
        .shrink(0.0)
        .background(theme.muted)
        .border(row_border)
        .semantics(Semantics::new(Role::Row));
        let pinned = self
            .model
            .edit
            .as_ref()
            .map(|edit| &edit.address)
            .or(self.model.active.as_ref())
            .and_then(|address| self.model.collection().index_of(&address.row));
        let rows = self.heights.build_pinned(
            format!("{}::rows", self.key),
            self.offset,
            pinned,
            |index| {
                let row = self.model.row(index).expect("cached visible row");
                Element::row(columns.iter().enumerate().map(|(column_index, column)| {
                    let address = CellAddress {
                        row: row.id.clone(),
                        column: column.id.clone(),
                    };
                    let edit = self
                        .model
                        .edit
                        .as_ref()
                        .filter(|edit| edit.address == address);
                    let content = if let Some(edit) = edit {
                        let key = self.editor_key(&address);
                        let mut input = column.editor.as_ref().map_or_else(
                            || {
                                Input::new(&key, &edit.draft, "", theme.input())
                                    .invalid(edit.error.is_some())
                                    .label(&column.presentation.label)
                                    .build()
                            },
                            |editor| editor(&key, &edit.draft, theme),
                        );
                        if let Some(error) = &edit.error {
                            let error_key = format!("{key}::error");
                            input = input.described_by([error_key.clone()]);
                            Element::column([
                                input,
                                Element::text(error.as_str())
                                    .keyed(error_key)
                                    .semantics(Semantics::new(Role::Alert).label(error)),
                            ])
                        } else {
                            input
                        }
                    } else {
                        column.render.as_ref().map_or_else(
                            || Element::text((column.value)(&row.value)).text_style(text.clone()),
                            |render| render(&row.value, theme),
                        )
                    };
                    let mut semantics =
                        Semantics::new(Role::Cell).label((column.value)(&row.value));
                    semantics.grid = GridPosition {
                        row_index: Some(index as u32 + 2),
                        column_index: Some(column_index as u32 + 1),
                        ..Default::default()
                    };
                    let active = self.model.active.as_ref() == Some(&address);
                    let mut cell = Element::row([content])
                        .keyed(self.cell_key(&address))
                        .width(length(column.presentation.width))
                        .grow(1.0)
                        .shrink(0.0)
                        .min_width(length(0.0))
                        .padding(argui_ui::sides(12.0, 6.0))
                        .align_items(argui_ui::AlignItems::CENTER)
                        .interaction(
                            Interaction::default()
                                .focus_policy(FocusPolicy::None)
                                .gestures(GestureSet::default().tap(TapGesture::default())),
                        )
                        .semantics(semantics);
                    if active {
                        cell = cell.border(Border::all(1.0, theme.ring));
                    }
                    cell
                }))
                .min_height(length(self.heights.estimated_extent()))
                .background(if self.model.selection().selected.contains(&row.id) {
                    theme.secondary
                } else {
                    theme.card
                })
                .border(row_border)
                .interaction(Interaction::default())
                .when(
                    argui_ui::VisualState::Hovered,
                    argui_ui::StylePatch::from_quad(
                        QuadStyle::solid(theme.muted).border(row_border),
                    ),
                )
                .keyed(format!("{}::row::{}", self.key, row.id))
                .semantics(Semantics::new(Role::Row).state(SemanticState {
                    selected: self.model.selection().selected.contains(&row.id),
                    ..Default::default()
                }))
            },
        );
        let mut semantics = Semantics::new(Role::Grid).label(&self.label);
        semantics.grid = GridPosition {
            row_count: Some(self.model.collection().len() as u32 + 1),
            column_count: Some(columns.len() as u32),
            ..Default::default()
        };
        let rows = rows
            .shrink(0.0)
            .scroll_config(argui_ui::ScrollConfig::default().scrollbar(theme.scrollbar.clone()));
        let mut grid = Element::column([header, rows])
            .width(length(content_width))
            .min_width(argui_ui::percent(1.0))
            .shrink(0.0)
            .keyed(&self.key)
            .semantics(semantics)
            .interaction(Interaction::default().focus_policy(FocusPolicy::TabStop));
        if let Some(active) = &self.model.active {
            grid = grid.active_descendant(self.cell_key(active));
        }
        Element::column([
            Checkbox::new(
                format!("{}::select-page", self.key),
                &self.select_page_label,
                self.model.page_checked(),
            )
            .build(theme),
            Element::container([grid])
                .keyed(format!("{}::horizontal", self.key))
                .min_width(length(0.0))
                .width(argui_ui::percent(1.0))
                .border(Border::all(1.0, theme.border))
                .radius(CornerRadii::all(8.0))
                .background(theme.card)
                .overflow(argui_ui::Axes {
                    x: argui_ui::Overflow::Auto,
                    y: argui_ui::Overflow::Hidden,
                })
                .scroll_config(
                    argui_ui::ScrollConfig::default()
                        .axes(argui_ui::ScrollAxes::Horizontal)
                        .scrollbar(theme.scrollbar.clone()),
                ),
        ])
        .gap(12.0)
        .width(argui_ui::percent(1.0))
        .min_width(length(0.0))
        .semantic_scope()
    }
    pub fn action(&self, event: &UiEvent) -> Option<DataTableAction> {
        let key = event.target_key()?;
        if let Some(column) = key.strip_prefix(&format!("{}::resize::", self.key))
            && self.model.visible_columns().any(|item| item.id == column)
            && matches!(
                event.kind,
                UiEventKind::Gesture(_) | UiEventKind::KeyInput(_) | UiEventKind::Click(_)
            )
        {
            return Some(DataTableAction::ResizeColumn {
                column: column.into(),
                event: event.clone(),
            });
        }
        if let Some(ToggleAction::SetChecked(checked)) = ToggleBehavior::new(
            format!("{}::select-page", self.key),
            &self.select_page_label,
            Role::CheckBox,
            self.model.page_checked(),
        )
        .action(event)
        {
            return Some(DataTableAction::SelectPage(
                checked == argui_ui::CheckedState::Checked,
            ));
        }
        if let Some(column) = key.strip_prefix(&format!("{}::sort::", self.key))
            && matches!(event.kind, UiEventKind::Click(_))
            && self.model.visible_columns().any(|item| item.id == column)
        {
            let descending =
                self.model.sort().iter().any(|sort| {
                    sort.column == column && sort.direction == SortDirection::Ascending
                });
            let mut sorts = if let UiEventKind::Click(click) = &event.kind
                && click.modifiers().shift
            {
                self.model.sort().to_vec()
            } else {
                Vec::new()
            };
            sorts.retain(|sort| sort.column != column);
            sorts.push(DataSort {
                column: column.into(),
                direction: if descending {
                    SortDirection::Descending
                } else {
                    SortDirection::Ascending
                },
            });
            return Some(DataTableAction::Sort(sorts));
        }
        if let Some(address) = self.parse_address("edit", key) {
            if self
                .model
                .edit
                .as_ref()
                .is_none_or(|edit| edit.address != address)
            {
                return None;
            }
            return match &event.kind {
                UiEventKind::TextChanged(value) => Some(DataTableAction::Draft(value.clone())),
                UiEventKind::Submitted(_) => Some(DataTableAction::CommitEdit { next: None }),
                UiEventKind::KeyInput(input) if input.state == KeyState::Pressed => match input.key
                {
                    Key::Escape => Some(DataTableAction::CancelEdit),
                    Key::Tab => Some(DataTableAction::CommitEdit {
                        next: Some(!input.modifiers.shift),
                    }),
                    _ => None,
                },
                _ => None,
            };
        }
        let address = self.parse_address("cell", key);
        if let (Some(address), UiEventKind::Click(click)) = (&address, &event.kind) {
            return Some(if click.count == 2 {
                DataTableAction::BeginEdit(address.clone())
            } else {
                DataTableAction::SelectRow(address.clone(), click.modifiers())
            });
        }
        if key != self.key && address.is_none() {
            return None;
        }
        let UiEventKind::KeyInput(input) = &event.kind else {
            return None;
        };
        if input.state != KeyState::Pressed {
            return None;
        }
        let columns: Vec<_> = self.model.visible_columns().collect();
        if columns.is_empty() || self.model.collection().is_empty() {
            return None;
        }
        let active = self.model.active.as_ref().or(address.as_ref());
        let mut row = active
            .and_then(|active| self.model.collection().index_of(&active.row))
            .unwrap_or(0);
        let mut column = active
            .and_then(|active| columns.iter().position(|column| column.id == active.column))
            .unwrap_or(0);
        match input.key {
            Key::Enter => {
                return Some(DataTableAction::BeginEdit(CellAddress {
                    row: self.model.row(row)?.id.clone(),
                    column: columns[column].id.clone(),
                }));
            }
            Key::ArrowDown => row = (row + 1).min(self.model.collection().len() - 1),
            Key::ArrowUp => row = row.saturating_sub(1),
            Key::ArrowLeft => column = column.saturating_sub(1),
            Key::ArrowRight => column = (column + 1).min(columns.len() - 1),
            Key::Home => {
                column = 0;
                if input.modifiers.command() {
                    row = 0;
                }
            }
            Key::End => {
                column = columns.len() - 1;
                if input.modifiers.command() {
                    row = self.model.collection().len() - 1;
                }
            }
            Key::PageDown => {
                row = (row + self.heights.visible_range(self.offset).len().max(1))
                    .min(self.model.collection().len() - 1)
            }
            Key::PageUp => {
                row = row.saturating_sub(self.heights.visible_range(self.offset).len().max(1))
            }
            _ => return None,
        }
        Some(DataTableAction::Activate(CellAddress {
            row: self.model.row(row)?.id.clone(),
            column: columns[column].id.clone(),
        }))
    }
}
