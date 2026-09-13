use argui_core::{Color, Size};
use argui_layout::LayoutEngine;
use argui_runtime::Inspection;
use argui_text::TextEngine;
use argui_ui::{Element, UiTree, length};

#[test]
fn allocated_capacity_tracks_growth_buffer_reuse_and_reclamation_after_removal() {
    let view = |count| {
        Element::column((0..count).map(|index| {
            Element::container([])
                .keyed(format!("row-{index}"))
                .width(length(20.0))
                .height(length(2.0))
                .background(Color::WHITE)
        }))
    };
    let mut tree = UiTree::new(view(2));
    let mut engine = LayoutEngine::new();
    let mut text = TextEngine::new();
    let output = engine
        .compute(&mut tree, &mut text, Size::new(800.0, 600.0))
        .unwrap();
    let small = Inspection::memory(&tree, &engine, &output);
    assert_eq!(small.ui_nodes, 3);
    assert_eq!(small.layout_nodes, 3);
    assert!(small.ui_index_bytes > 0 && small.layout_cache_bytes > 0);
    assert!(small.layout_metadata_bytes > 0 && small.layout_geometry_bytes > 0);
    assert!(small.layout_output_bytes > 0 && small.paint_command_bytes > 0);
    tree.update(view(256));
    let mut output = engine
        .compute(&mut tree, &mut text, Size::new(800.0, 600.0))
        .unwrap();
    let large = Inspection::memory(&tree, &engine, &output);
    assert_eq!(large.ui_nodes, 257);
    assert_eq!(large.layout_nodes, 257);
    for (before, after) in [
        (small.ui_index_bytes, large.ui_index_bytes),
        (small.layout_metadata_bytes, large.layout_metadata_bytes),
        (small.layout_cache_bytes, large.layout_cache_bytes),
        (small.layout_geometry_bytes, large.layout_geometry_bytes),
        (small.layout_output_bytes, large.layout_output_bytes),
        (small.paint_command_bytes, large.paint_command_bytes),
    ] {
        assert!(after > before);
    }
    assert_eq!(Inspection::memory(&tree, &engine, &output), large);
    output.display_list.clear();
    assert_eq!(
        Inspection::memory(&tree, &engine, &output).paint_command_bytes,
        large.paint_command_bytes
    );
    tree.update(view(0));
    let output = engine
        .compute(&mut tree, &mut text, Size::new(800.0, 600.0))
        .unwrap();
    let removed = Inspection::memory(&tree, &engine, &output);
    assert_eq!(removed.ui_nodes, 1);
    assert_eq!(removed.layout_nodes, 1);
    assert!(
        removed.layout_cache_bytes < large.layout_cache_bytes,
        "layout removal reclaims excess dense cache capacity"
    );
}
