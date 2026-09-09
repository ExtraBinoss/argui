use argui_ui::{CheckedState, SortDirection};
use argui_widgets::{
    CellAddress, DataColumn, DataPage, DataRow, DataSort, DataTableAction, DataTableModel,
    TableColumn,
};

pub fn model(count: usize) -> DataTableModel<i32> {
    let mut column = DataColumn::new("value", TableColumn::new("Value", 100.0), |row: &i32| {
        row.to_string()
    });
    column.compare = Some(Box::new(i32::cmp));
    column.validate = Some(Box::new(|_, draft| {
        draft
            .parse::<i32>()
            .map(|_| ())
            .map_err(|_| "Enter an integer".into())
    }));
    column.minimum_width = 50.0;
    column.maximum_width = 200.0;
    DataTableModel::new(
        (0..count)
            .map(|i| DataRow {
                id: format!("row-{i}"),
                value: i as i32,
            })
            .collect(),
        vec![column],
    )
    .unwrap()
}
pub fn address(row: usize) -> CellAddress {
    CellAddress {
        row: format!("row-{row}"),
        column: "value".into(),
    }
}

#[test]
fn filters_sort_then_paginate_and_page_selection_has_mixed_state() {
    let mut model = model(100);
    model.set_filter("value", "1".into()).unwrap();
    assert_eq!(model.filtered_count(), 19);
    model
        .set_sort(vec![DataSort {
            column: "value".into(),
            direction: SortDirection::Descending,
        }])
        .unwrap();
    model.set_page(Some(DataPage {
        index: 1,
        size: 3.try_into().unwrap(),
    }));
    assert_eq!(model.row(0).unwrap().value, 61);
    assert_eq!(model.row(2).unwrap().value, 41);
    assert_eq!(model.page_checked(), CheckedState::Unchecked);
    let mut selection = model.selection().clone();
    selection.selected.insert("row-61".into());
    model.set_selection(selection);
    assert_eq!(model.page_checked(), CheckedState::Mixed);
    model.select_page(true);
    assert_eq!(model.page_checked(), CheckedState::Checked);
    model.set_page(Some(DataPage {
        index: 0,
        size: 3.try_into().unwrap(),
    }));
    assert_eq!(model.page_checked(), CheckedState::Unchecked);
    assert_eq!(model.selection().selected.len(), 3);
    model.select_page(false);
    assert_eq!(model.selection().selected.len(), 3);
    model.set_filter("value", String::new()).unwrap();
    assert_eq!(model.filtered_count(), 100);
}

#[test]
fn editing_validates_and_emits_a_request_without_mutating_source_data() {
    let mut model = model(3);
    assert!(model.begin_edit(address(1)));
    model
        .apply(DataTableAction::Draft("invalid".into()))
        .unwrap();
    assert!(model.commit_edit().is_none());
    assert_eq!(
        model.edit().unwrap().error.as_deref(),
        Some("Enter an integer")
    );
    model.apply(DataTableAction::Draft("42".into())).unwrap();
    let commit = model
        .apply(DataTableAction::CommitEdit { next: Some(true) })
        .unwrap()
        .unwrap();
    assert_eq!(commit.address, address(1));
    assert_eq!(commit.original, "1");
    assert_eq!(commit.value, "42");
    assert_eq!(model.row(1).unwrap().value, 1);
    assert_eq!(model.active, Some(address(2)));
    assert!(model.edit().is_none());
}

#[test]
fn sort_keeps_edit_identity_and_external_changes_or_filtering_cancel_it() {
    let mut model = model(3);
    model.begin_edit(address(1));
    model
        .set_sort(vec![DataSort {
            column: "value".into(),
            direction: SortDirection::Descending,
        }])
        .unwrap();
    assert_eq!(model.edit().unwrap().address, address(1));
    model
        .replace_rows(vec![DataRow {
            id: "row-1".into(),
            value: 9,
        }])
        .unwrap();
    assert!(model.edit().is_none());
    model.begin_edit(address(1));
    model.set_filter("value", "absent".into()).unwrap();
    assert!(model.edit().is_none());
    assert!(model.active.is_none());
    assert!(!model.begin_edit(address(1)));
}

#[test]
fn columns_reject_invalid_requests_and_clamp_resizes() {
    let mut model = model(2);
    assert_eq!(model.resize("value", 10.0), Ok(50.0));
    assert_eq!(model.resize("value", 900.0), Ok(200.0));
    assert!(model.resize("value", f32::NAN).is_err());
    assert!(model.resize("missing", 80.0).is_err());
    assert!(model.set_filter("missing", "query".into()).is_err());
    model.begin_edit(address(1));
    model.set_hidden("value", true).unwrap();
    assert!(model.edit().is_none());
    assert_eq!(model.visible_columns().count(), 0);
    assert!(!model.begin_edit(address(1)));
    model.set_hidden("value", false).unwrap();
    assert_eq!(model.visible_columns().count(), 1);
}

#[test]
fn invalid_data_is_rejected_atomically_and_hidden_cells_cannot_be_activated() {
    use argui_widgets::DataTableError;
    let column = || {
        DataColumn::new("x", TableColumn::new("X", 100.0), |value: &i32| {
            value.to_string()
        })
    };
    for (width, min, max) in [
        (f32::NAN, 1.0, 100.0),
        (50.0, f32::NAN, 100.0),
        (50.0, 1.0, f32::INFINITY),
        (50.0, 0.0, 100.0),
        (50.0, 100.0, 10.0),
        (50.0, 60.0, 100.0),
    ] {
        let mut c = column();
        c.presentation.width = width;
        c.minimum_width = min;
        c.maximum_width = max;
        assert!(matches!(
            DataTableModel::new(vec![], vec![c]),
            Err(DataTableError::InvalidWidth)
        ));
    }
    assert!(matches!(
        DataTableModel::new(vec![], vec![column(), column()]),
        Err(DataTableError::DuplicateColumn(_))
    ));
    let duplicate_rows = || {
        vec![
            DataRow {
                id: "same".into(),
                value: 1,
            },
            DataRow {
                id: "same".into(),
                value: 2,
            },
        ]
    };
    assert!(matches!(
        DataTableModel::new(duplicate_rows(), vec![column()]),
        Err(DataTableError::DuplicateRow(_))
    ));
    let mut model = model(3);
    assert!(model.replace_rows(duplicate_rows()).is_err());
    assert_eq!(model.row(0).unwrap().value, 0);
    assert!(model.resize("value", f32::NAN).is_err());
    assert!(model.resize("missing", 42.0).is_err());
    assert!(model.set_filter("missing", "x".into()).is_err());
    assert!(
        model
            .set_sort(vec![DataSort {
                column: "missing".into(),
                direction: SortDirection::Ascending
            }])
            .is_err()
    );
    assert!(!model.begin_edit(CellAddress {
        row: "missing".into(),
        column: "value".into()
    }));
    assert!(!model.begin_edit(CellAddress {
        row: "row-0".into(),
        column: "missing".into()
    }));
    model.set_hidden("value", true).unwrap();
    assert!(!model.begin_edit(address(0)));
    for action in [
        DataTableAction::Activate(address(0)),
        DataTableAction::SelectRow(address(0), Default::default()),
    ] {
        model.apply(action).unwrap();
        assert_eq!(model.active, None);
    }
    model.set_hidden("value", false).unwrap();
    model.set_filter("value", "2".into()).unwrap();
    assert!(!model.begin_edit(address(0)));
    model
        .apply(DataTableAction::Draft("orphan".into()))
        .unwrap();
    assert_eq!(
        model
            .apply(DataTableAction::CommitEdit { next: None })
            .unwrap(),
        None
    );
}

#[test]
fn controlled_edits_move_both_directions_and_stop_at_table_edges() {
    let mut model = model(2);
    for (row, forward, expected) in [(0, true, 1), (1, true, 1), (1, false, 0), (0, false, 0)] {
        assert!(model.begin_edit(address(row)));
        let commit = model
            .apply(DataTableAction::CommitEdit {
                next: Some(forward),
            })
            .unwrap()
            .unwrap();
        assert_eq!(commit.address, address(row));
        assert_eq!(model.active, Some(address(expected)));
    }
    model.begin_edit(address(0));
    model.set_hidden("value", true).unwrap();
    assert_eq!(model.edit(), None);
    assert_eq!(model.active, None);
    model.set_hidden("value", false).unwrap();
    model.begin_edit(address(0));
    model
        .apply(DataTableAction::Draft("invalid".into()))
        .unwrap();
    assert_eq!(
        model
            .apply(DataTableAction::CommitEdit { next: Some(true) })
            .unwrap(),
        None
    );
    assert_eq!(model.active, Some(address(0)));
    assert!(model.edit().unwrap().error.is_some());
    model.apply(DataTableAction::CancelEdit).unwrap();
    assert!(model.edit().is_none());
}
