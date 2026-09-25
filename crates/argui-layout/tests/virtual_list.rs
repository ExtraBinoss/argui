use argui_core::{Point, Size};
use argui_layout::{LayoutEngine, LayoutError};
use argui_text::TextEngine;
use argui_ui::{Element, TreeUpdate, UiTree, VirtualList, length};

const NOTO_SANS: &[u8] = include_bytes!("../../../assets/fonts/NotoSans-Regular.ttf");

fn text_engine() -> TextEngine {
    TextEngine::from_embedded_fonts([NOTO_SANS], "Noto Sans", "Noto Sans", "Noto Sans")
}

#[test]
fn variable_virtual_list_keeps_a_user_scroll_when_a_new_window_is_measured() {
    let list = VirtualList::variable(80, 176.0, 400.0).overscan(10);
    let view = |offset| {
        list.build("variable-list", offset, |index| {
            Element::container([])
                .keyed(format!("variable-{index}"))
                .height(length(80.0 + (index % 3) as f32 * 24.0))
        })
        .width(length(600.0))
    };
    let mut ui = UiTree::new(view(0.0));
    let viewport = ui.node_id_at(0).unwrap();
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    let mut output = layout
        .compute(&mut ui, &mut text, Size::new(600.0, 400.0))
        .unwrap();

    ui.set_scroll_offset(viewport, Point::new(0.0, 3_000.0));
    layout.apply_scroll(&ui, &mut output).unwrap();
    ui.update(view(3_000.0));
    output = layout
        .compute(&mut ui, &mut text, Size::new(600.0, 400.0))
        .unwrap();
    let before = ui.scroll_offset(viewport).y;

    let requested = before + 400.0;
    ui.set_scroll_offset(viewport, Point::new(0.0, requested));
    layout.apply_scroll(&ui, &mut output).unwrap();
    ui.update(view(requested));
    layout
        .compute(&mut ui, &mut text, Size::new(600.0, 400.0))
        .unwrap();

    assert!(
        ui.scroll_offset(viewport).y > before + 100.0,
        "offset stayed at {before}: {:?}",
        ui.scroll_offset(viewport),
    );
}

#[test]
fn changed_virtual_window_rebuilds_stale_scroll_layout() {
    let list = VirtualList::variable(80, 176.0, 400.0).overscan(10);
    let view = |offset| {
        list.build("variable-list", offset, |index| {
            Element::container([])
                .keyed(format!("variable-{index}"))
                .height(length(80.0 + (index % 3) as f32 * 24.0))
        })
        .width(length(600.0))
    };
    let mut ui = UiTree::new(view(0.0));
    let viewport = ui.node_id_at(0).unwrap();
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    let mut output = layout
        .compute(&mut ui, &mut text, Size::new(600.0, 400.0))
        .unwrap();

    ui.set_scroll_offset(viewport, Point::new(0.0, 3_000.0));
    assert_eq!(ui.update(view(3_000.0)), TreeUpdate::Layout);
    assert!(matches!(
        layout.apply_scroll(&ui, &mut output),
        Err(LayoutError::StaleOutput)
    ));
    assert!(
        layout
            .apply_scroll_with_text(&mut ui, &mut text, &mut output)
            .unwrap()
    );
    assert_eq!(output.nodes.len(), ui.node_ids().len());
}
