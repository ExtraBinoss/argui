use std::sync::Arc;

use argui_core::Size;
use argui_layout::LayoutEngine;
use argui_paint::{Border, Color, ImageAsset, ImageId, QuadStyle};
use argui_text::{TextEngine, TextStyle};
use argui_ui::{
    AlignItems, Axes, Display, Element, Overflow, ScrollConfig, ScrollbarGutter,
    ScrollbarPartStyle, ScrollbarStyle, Sides, UiTree, evenly_sized_tracks, length,
};

const NOTO_SANS: &[u8] = include_bytes!("../../../assets/fonts/NotoSans-Regular.ttf");

fn text_engine() -> TextEngine {
    TextEngine::from_embedded_fonts([NOTO_SANS], "Noto Sans", "Noto Sans", "Noto Sans")
}

fn compute(root: Element, viewport: Size) -> (UiTree, argui_layout::LayoutOutput) {
    let mut tree = UiTree::new(root);
    let output = LayoutEngine::new()
        .compute(&mut tree, &mut text_engine(), viewport)
        .unwrap();
    (tree, output)
}

#[test]
fn block_is_the_default_and_stacks_normal_flow_children() {
    let (_, output) = compute(
        Element::container([
            Element::container([]).height(length(20.0)),
            Element::container([]).height(length(30.0)),
        ])
        .width(length(100.0)),
        Size::new(100.0, 100.0),
    );

    assert_eq!(output.nodes[1].bounds.origin.y, 0.0);
    assert_eq!(output.nodes[2].bounds.origin.y, 20.0);
    assert_eq!(output.nodes[0].bounds.size.height, 50.0);
}

#[test]
fn grid_tracks_and_gaps_are_resolved_by_taffy() {
    let cells = (0..4).map(|_| Element::container([]).height(length(20.0)));
    let (_, output) = compute(
        Element::grid(cells)
            .grid_template_columns(evenly_sized_tracks::<String>(2))
            .width(length(210.0))
            .column_gap(10.0)
            .row_gap(6.0),
        Size::new(210.0, 100.0),
    );

    assert_eq!(
        output.nodes[1].bounds.size.width, 100.0,
        "{:?}",
        output.nodes
    );
    assert_eq!(output.nodes[2].bounds.origin.x, 110.0);
    assert_eq!(output.nodes[3].bounds.origin.y, 26.0);
}

#[test]
fn border_box_keeps_authored_size_including_padding_and_border() {
    let (_, output) = compute(
        Element::container([])
            .width(length(100.0))
            .height(length(60.0))
            .padding(Sides::length(12.0))
            .border(Border::all(4.0, Color::WHITE)),
        Size::new(200.0, 100.0),
    );

    assert_eq!(output.nodes[0].bounds.size, Size::new(100.0, 60.0));
}

#[test]
fn text_content_box_excludes_border_and_padding() {
    let (_, output) = compute(
        Element::text("content")
            .width(length(100.0))
            .height(length(50.0))
            .padding(Sides::length(6.0))
            .border(Border::all(4.0, Color::WHITE)),
        Size::new(200.0, 100.0),
    );
    let block = &output.text.blocks()[0];

    assert_eq!(block.bounds.origin.x, 10.0);
    assert_eq!(block.bounds.origin.y, 10.0);
    assert_eq!(block.bounds.size, Size::new(80.0, 30.0));
}

#[test]
fn display_none_removes_layout_paint_and_hit_testing() {
    let (tree, output) = compute(
        Element::column([
            Element::container([])
                .display(Display::None)
                .width(length(80.0))
                .height(length(40.0)),
            Element::container([]).height(length(20.0)),
        ]),
        Size::new(100.0, 100.0),
    );

    assert_eq!(output.nodes[1].bounds.size, Size::default());
    assert_eq!(output.nodes[2].bounds.origin.y, 0.0);
    assert!(
        output
            .hit_regions
            .iter()
            .all(|region| region.node != tree.node_id_at(1).unwrap())
    );
    assert!(output.display_list.commands().is_empty());
}

#[test]
fn registered_images_supply_intrinsic_size_and_aspect_ratio() {
    let id = ImageId(90);
    let image = ImageAsset::rgba8(id, 40, 20, Arc::<[u8]>::from(vec![255; 40 * 20 * 4])).unwrap();
    let mut tree = UiTree::new(Element::image(id).width(length(80.0)));
    let mut layout = LayoutEngine::new();
    layout.set_assets(&[image], &[]);
    let output = layout
        .compute(&mut tree, &mut text_engine(), Size::new(200.0, 100.0))
        .unwrap();

    assert_eq!(output.nodes[0].bounds.size, Size::new(80.0, 40.0));
}

#[test]
fn image_assets_arriving_after_layout_invalidate_measurement_but_repeats_are_stable() {
    let id = ImageId(91);
    let image = ImageAsset::rgba8(id, 40, 20, Arc::<[u8]>::from(vec![255; 40 * 20 * 4])).unwrap();
    let mut ui = UiTree::new(Element::image(id).width(length(80.0)));
    let mut engine = LayoutEngine::new();
    let mut text = text_engine();
    let initial = engine
        .compute(&mut ui, &mut text, Size::new(200.0, 100.0))
        .unwrap();
    assert_eq!(initial.nodes[0].bounds.size.height, 0.0);
    for _ in 0..2 {
        engine.set_assets(std::slice::from_ref(&image), &[]);
        let loaded = engine
            .compute(&mut ui, &mut text, Size::new(200.0, 100.0))
            .unwrap();
        assert_eq!(loaded.nodes[0].bounds.size, Size::new(80.0, 40.0));
    }
}

#[test]
fn nested_image_and_vector_dimensions_follow_replacement_and_removal() {
    use argui_paint::{VectorAsset, VectorId};
    let image_id = ImageId(92);
    let vector_id = VectorId(93);
    let root = Element::column([
        Element::image(image_id).keyed("image").width(length(80.0)),
        Element::vector(vector_id)
            .keyed("vector")
            .width(length(80.0)),
        Element::image(image_id)
            .keyed("authored")
            .width(length(80.0))
            .aspect_ratio(1.0),
    ]);
    let mut ui = UiTree::new(root);
    let ids = ui.node_ids().to_vec();
    let mut engine = LayoutEngine::new();
    let mut text = text_engine();
    for dimensions in [None, Some((40, 20)), Some((20, 40)), None, Some((40, 20))] {
        let (images, vectors, expected) = if let Some((width, height)) = dimensions {
            let image = ImageAsset::rgba8(
                image_id,
                width,
                height,
                Arc::<[u8]>::from(vec![255; (width * height * 4) as usize]),
            )
            .unwrap();
            let svg = format!(
                "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\"><rect width=\"{width}\" height=\"{height}\"/></svg>"
            );
            let vector = VectorAsset {
                id: vector_id,
                size: Size::new(width as f32, height as f32),
                svg: Arc::from(svg.into_bytes()),
                tintable: false,
            };
            (
                vec![image],
                vec![vector],
                80.0 * height as f32 / width as f32,
            )
        } else {
            (vec![], vec![], 0.0)
        };
        for _ in 0..2 {
            engine.set_assets(&images, &vectors);
            let output = engine
                .compute(&mut ui, &mut text, Size::new(300.0, 600.0))
                .unwrap();
            assert_eq!(output.nodes[1].bounds.size, Size::new(80.0, expected));
            assert_eq!(output.nodes[2].bounds.size, Size::new(80.0, expected));
            assert_eq!(output.nodes[3].bounds.size, Size::new(80.0, 80.0));
            assert_eq!(ui.node_ids(), ids);
        }
    }
}

#[test]
fn text_baselines_align_across_different_font_sizes() {
    let small = TextStyle {
        font_size: 12.0,
        line_height: 16.0,
        ..TextStyle::default()
    };
    let large = TextStyle {
        font_size: 28.0,
        line_height: 34.0,
        ..TextStyle::default()
    };
    let mut metrics = text_engine();
    let small_baseline = metrics
        .measure_layout("small", &small, None)
        .first_baseline
        .unwrap();
    let large_baseline = metrics
        .measure_layout("Large", &large, None)
        .first_baseline
        .unwrap();
    let root = Element::row([
        Element::text("small").text_style(small),
        Element::text("Large").text_style(large),
    ])
    .align_items(AlignItems::BASELINE);
    assert_eq!(root.style.align_items, Some(AlignItems::BASELINE));
    let (_, output) = compute(root, Size::new(300.0, 80.0));

    let first = output.nodes[1].bounds.origin.y + small_baseline;
    let second = output.nodes[2].bounds.origin.y + large_baseline;
    assert!(
        (first - second).abs() < 0.3,
        "{first} != {second}; {:?}",
        output.nodes
    );
}

#[test]
fn percentage_padding_resolves_against_container_width() {
    let (_, output) = compute(
        Element::container([Element::container([]).height(length(10.0))])
            .width(length(200.0))
            .padding(Sides::percent(0.1)),
        Size::new(200.0, 100.0),
    );

    assert_eq!(output.nodes[1].bounds.origin.x, 20.0);
    assert_eq!(output.nodes[1].bounds.origin.y, 20.0);
    assert_eq!(output.nodes[1].bounds.size.width, 160.0);
    assert_eq!(output.nodes[0].bounds.size.height, 50.0);
    assert_eq!(output.nodes[0].bounds.size.width, 200.0);
}

#[test]
fn stable_scrollbar_gutter_uses_the_authored_scrollbar_geometry() {
    let scrollbar = ScrollbarStyle::new(
        ScrollbarPartStyle::new(QuadStyle::default()),
        ScrollbarPartStyle::new(QuadStyle::default()),
    )
    .width(8.0)
    .insets(Sides {
        left: 2.0,
        right: 5.0,
        top: 2.0,
        bottom: 3.0,
    });
    let (_, output) = compute(
        Element::container([Element::container([]).height(length(140.0))])
            .width(length(100.0))
            .height(length(80.0))
            .overflow(Axes {
                x: Overflow::Hidden,
                y: Overflow::Auto,
            })
            .scrollbar_gutter(ScrollbarGutter::Stable)
            .scroll_config(ScrollConfig::default().scrollbar(scrollbar)),
        Size::new(100.0, 80.0),
    );

    assert_eq!(output.nodes[1].bounds.size.width, 87.0);
}
