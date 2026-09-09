use argui_core::{Color, ColorScheme};
use argui_ui::{ClickEvent, Element, Role, UiEvent, UiEventKind, UiTree, length};
use argui_widgets::{Collection, CollectionItem, ListState, Table, TableColumn, shadcn};

#[test]
fn columns_align_headers_and_cells_and_rows_share_selection_behavior() {
    let items =
        Collection::new((0..3).map(|i| CollectionItem::new(i.to_string(), i.to_string()))).unwrap();
    let mut state = ListState::default();
    state.select(1, &items, true, Default::default());
    let table = Table::new(
        "table",
        [
            TableColumn::new("Name", 120.0),
            TableColumn::new("Count", 60.0),
        ],
        &items,
    )
    .label("Inventory")
    .selection(&state, true);
    let themes = shadcn(Color::WHITE);
    let built = table.build(themes.resolve(ColorScheme::Dark), |row, column| {
        Element::text(format!("{row}:{column}"))
    });
    assert_eq!(built.semantics.as_ref().unwrap().role, Role::Grid);
    assert_eq!(
        built.semantics.as_ref().unwrap().label.as_deref(),
        Some("Inventory")
    );
    assert_eq!(built.children.len(), 4);
    for row in &built.children {
        assert_eq!(row.semantics.as_ref().unwrap().role, Role::Row);
        assert_eq!(row.children[0].style.size.width, length(120.0));
        assert_eq!(row.children[1].style.size.width, length(60.0));
    }
    assert_eq!(
        built.children[0].children[0]
            .semantics
            .as_ref()
            .unwrap()
            .role,
        Role::ColumnHeader
    );
    assert_eq!(
        built.children[1].children[0]
            .semantics
            .as_ref()
            .unwrap()
            .role,
        Role::Cell
    );
    assert!(built.children[2].semantics.as_ref().unwrap().state.selected);
    let tree = UiTree::new(built);
    let event = UiEvent::new(
        tree.node_ids()[0],
        Some("table::row::2".into()),
        UiEventKind::Click(ClickEvent::accessibility()),
    );
    assert_eq!(table.action(&event).unwrap().selected, ["2".into()].into());
}

#[test]
#[should_panic]
fn columns_reject_nonfinite_widths() {
    let _ = TableColumn::new("Name", f32::NAN);
}
