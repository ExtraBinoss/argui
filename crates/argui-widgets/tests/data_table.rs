#[path = "data_table/model.rs"]
mod model;
use argui_core::{Color, ColorScheme, Key, KeyInput, KeyState, Modifiers};
use argui_ui::{Element, UiEvent, UiEventKind, UiTree, VirtualList};
use argui_widgets::{DataTable, DataTableAction, shadcn};

#[test]
fn hundred_thousand_rows_build_a_bounded_window_and_keep_the_active_cell() {
    let mut model = model::model(100_000);
    model.active = Some(model::address(99_999));
    let heights = VirtualList::fixed(100_000, 32.0, 320.0);
    let table = DataTable::new("table", &model, &heights, 0.0);
    let themes = shadcn(Color::WHITE);
    let tree = UiTree::new(table.build(themes.resolve(ColorScheme::Light)));
    assert!(tree.node_ids().len() < 300);
    assert!(tree.semantic_diagnostics().is_empty());
    let semantic = tree.semantic_tree(&[], 1.0);
    assert!(
        semantic
            .nodes
            .iter()
            .any(|node| node.semantics.relations.active_descendant.is_some())
    );
}

#[test]
fn grid_navigation_and_cell_editing_are_scoped_to_the_table() {
    let mut model = model::model(100);
    let heights = VirtualList::fixed(100, 32.0, 320.0);
    let id = UiTree::new(Element::container([])).node_ids()[0];
    let event = |target: &str, key, modifiers| {
        UiEvent::new(
            id,
            Some(target.into()),
            UiEventKind::KeyInput(KeyInput {
                key,
                state: KeyState::Pressed,
                repeat: false,
                text: None,
                modifiers,
            }),
        )
    };
    let action = DataTable::new("table", &model, &heights, 0.0)
        .action(&event(
            "table",
            Key::End,
            Modifiers {
                control: true,
                ..Default::default()
            },
        ))
        .unwrap();
    model.apply(action).unwrap();
    assert_eq!(model.active, Some(model::address(99)));
    let table = DataTable::new("table", &model, &heights, 0.0);
    assert!(
        table
            .action(&event("other", Key::Home, Modifiers::default()))
            .is_none()
    );
    let action = table
        .action(&event("table", Key::Enter, Modifiers::default()))
        .unwrap();
    assert!(matches!(action, DataTableAction::BeginEdit(_)));
    model.apply(action).unwrap();
    assert_eq!(model.edit().unwrap().address, model::address(99));
}

#[test]
fn horizontal_scroll_keeps_header_and_cells_aligned_after_column_resize() {
    use argui_core::{Point, ScrollDelta, Size};
    use argui_layout::LayoutEngine;
    use argui_text::TextEngine;
    use argui_widgets::{DataColumn, DataRow, DataTableModel, TableColumn};
    let columns = ["one", "two"].map(|id| {
        DataColumn::new(id, TableColumn::new(id, 240.0), |value: &i32| {
            value.to_string()
        })
    });
    let mut model = DataTableModel::new(
        (0..100)
            .map(|i| DataRow {
                id: i.to_string(),
                value: i,
            })
            .collect(),
        columns.into(),
    )
    .unwrap();
    let heights = VirtualList::fixed(100, 32.0, 160.0);
    let themes = shadcn(Color::WHITE);
    let theme = themes.resolve(ColorScheme::Light);
    let mut layout = LayoutEngine::new();
    let mut text = TextEngine::from_embedded_fonts(
        [include_bytes!("../../argui-web-demo/assets/fonts/NotoSans-Regular.ttf").as_slice()],
        "Noto Sans",
        "Noto Sans",
        "Noto Sans",
    );
    for width in [240.0, 320.0] {
        model.resize("one", width).unwrap();
        let table = DataTable::new("table", &model, &heights, 0.0);
        let mut ui = UiTree::new(table.build(theme));
        let mut output = layout
            .compute(&mut ui, &mut text, Size::new(300.0, 300.0))
            .unwrap();
        let keys: std::collections::HashMap<_, _> = ui
            .node_ids()
            .iter()
            .enumerate()
            .filter_map(|(i, id)| Some((*id, ui.element_at(i)?.key.clone()?)))
            .collect();
        let bounds = |output: &argui_layout::LayoutOutput, key: &str| {
            output
                .nodes
                .iter()
                .find(|node| keys.get(&node.node).map(String::as_str) == Some(key))
                .unwrap()
                .bounds
        };
        let cell_key = table.cell_key(&argui_widgets::CellAddress {
            row: "0".into(),
            column: "two".into(),
        });
        let initial = bounds(&output, &cell_key).origin.x;
        let horizontal = output
            .scroll_regions
            .iter()
            .find(|region| keys.get(&region.node).map(String::as_str) == Some("table::horizontal"))
            .unwrap();
        let point = Point::new(
            horizontal.bounds.origin.x + 10.0,
            horizontal.bounds.origin.y + 10.0,
        );
        ui.scroll(
            point,
            ScrollDelta::Pixels(Point::new(-100.0, 0.0)),
            &output.scroll_regions,
        );
        layout.apply_scroll(&ui, &mut output).unwrap();
        let moved = output
            .nodes
            .iter()
            .find(|node| keys.get(&node.node).map(String::as_str) == Some(cell_key.as_str()))
            .unwrap()
            .bounds
            .origin
            .x;
        let header = output
            .nodes
            .iter()
            .find(|node| keys.get(&node.node).map(String::as_str) == Some("table::sort::two"))
            .unwrap()
            .bounds
            .origin
            .x;
        assert!(moved < initial, "table must actually scroll");
        assert_eq!(moved, header);
    }
}

#[test]
fn row_hover_stays_on_across_cells_and_editors_then_clears_without_a_frame() {
    use argui_core::{Point, Size};
    use argui_layout::LayoutEngine;
    use argui_text::TextEngine;
    use argui_widgets::{CellAddress, DataColumn, DataRow, DataTableModel, TableColumn};
    let themes = shadcn(Color::srgb(0.2, 0.5, 0.9));
    let mut model = DataTableModel::new(
        (0..3)
            .map(|i| DataRow {
                id: i.to_string(),
                value: i,
            })
            .collect(),
        ["left", "right"]
            .map(|id| {
                let mut column = DataColumn::new(id, TableColumn::new(id, 140.0), |value: &i32| {
                    value.to_string()
                });
                column.validate = Some(Box::new(|_, _| Ok(())));
                column
            })
            .into(),
    )
    .unwrap();
    let address = |row: usize, column: &str| CellAddress {
        row: row.to_string(),
        column: column.into(),
    };
    let heights = VirtualList::fixed(3, 52.0, 200.0);
    for editing in [false, true] {
        if editing {
            assert!(model.begin_edit(address(0, "right")));
        }
        for scheme in [ColorScheme::Light, ColorScheme::Dark] {
            let theme = themes.resolve(scheme);
            let table = DataTable::new("table", &model, &heights, 0.0);
            let mut tree = UiTree::new(table.build(theme));
            let output = LayoutEngine::new()
                .compute(&mut tree, &mut TextEngine::new(), Size::new(500.0, 400.0))
                .unwrap();
            let rows: Vec<_> = (0..3)
                .map(|row| {
                    let key = format!("table::row::{row}");
                    tree.node_ids()
                        .iter()
                        .enumerate()
                        .find(|(_, node)| tree.key(**node) == Some(&key))
                        .map(|(index, &id)| (index, id))
                        .unwrap()
                })
                .collect();
            for row in [0, 1, 2, 1, 0] {
                for column in ["left", "right"] {
                    let key = if editing && row == 0 && column == "right" {
                        table.editor_key(&address(row, column))
                    } else {
                        table.cell_key(&address(row, column))
                    };
                    let bounds = output
                        .hit_regions
                        .iter()
                        .find(|region| tree.key(region.node) == Some(&key))
                        .unwrap()
                        .bounds;
                    // Test both cell padding and its text/editor, which are separate hit targets.
                    for x in [2.0, bounds.size.width / 2.0] {
                        tree.pointer_moved(
                            Point::new(
                                bounds.origin.x + x,
                                bounds.origin.y + bounds.size.height / 2.0,
                            ),
                            &output.hit_regions,
                        );
                        for (candidate, (index, id)) in rows.iter().enumerate() {
                            assert_eq!(
                                tree.resolved_quad(*id, tree.element_at(*index).unwrap())
                                    .background,
                                Some(argui_paint::Fill::Solid(if row == candidate {
                                    theme.muted
                                } else {
                                    theme.card
                                })),
                                "row {candidate}, pointer over {key}, editing {editing}"
                            );
                        }
                    }
                }
            }
            tree.pointer_moved(Point::new(-10.0, -10.0), &output.hit_regions);
            for (index, id) in rows {
                assert_eq!(
                    tree.resolved_quad(id, tree.element_at(index).unwrap())
                        .background,
                    Some(argui_paint::Fill::Solid(theme.card))
                );
            }
        }
    }
}
