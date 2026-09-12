//! Compare ordinary and isolated fixed panels with the same rendered geometry.
use argui_core::{Color, Size};
use argui_layout::LayoutEngine;
use argui_text::TextEngine;
use argui_ui::{Axes, Element, Overflow, TreeUpdate, UiTree, length};
use web_time::Instant;

fn root(isolated: bool) -> Element {
    Element::row((0..40).map(|panel| {
        let content = Element::column((0..40).map(|row| {
            Element::container([])
                .keyed(row.to_string())
                .width(length(8.0))
                .height(length(8.0))
                .shrink(0.0)
                .background(Color::WHITE)
        }))
        .shrink(0.0);
        let wrapper = if isolated {
            Element::layout_boundary(content)
        } else {
            Element::column([content])
        };
        wrapper
            .keyed(panel.to_string())
            .width(length(30.0))
            .height(length(200.0))
            .shrink(0.0)
            .overflow(Axes {
                x: Overflow::Clip,
                y: Overflow::Clip,
            })
    }))
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let mode = args.get(1).map(String::as_str).unwrap_or("isolated");
    assert!(matches!(mode, "normal" | "isolated"));
    let frames: usize = args.get(2).and_then(|n| n.parse().ok()).unwrap_or(200);
    let mut ui = UiTree::new(root(mode == "isolated"));
    let mut engine = LayoutEngine::new();
    let mut text = TextEngine::from_embedded_fonts([], "sans", "sans", "mono");
    let viewport = Size::new(1280.0, 900.0);
    let mut output = engine.compute(&mut ui, &mut text, viewport).unwrap();
    let mut layouts = 0;
    let start = Instant::now();
    for step in 0..frames {
        let mut root = ui.root().clone();
        root.children[0].children[0].children[0].style.size.height =
            length(8.0 + (step % 2) as f32);
        if ui.update(root) == TreeUpdate::Layout {
            output = engine.compute(&mut ui, &mut text, viewport).unwrap();
            layouts += 1;
        }
        std::hint::black_box(&output);
    }
    let elapsed_ns = start.elapsed().as_nanos();
    let nodes = output.nodes.len();
    let checksum: f64 = output
        .nodes
        .iter()
        .map(|n| {
            f64::from(
                n.bounds.origin.x + n.bounds.origin.y + n.bounds.size.width + n.bounds.size.height,
            )
        })
        .sum();
    assert!(checksum.is_finite());
    println!(
        "{{\"case\":\"boundary\",\"frames\":{frames},\"nodes\":{nodes},\"layouts\":{layouts},\"elapsed_ns\":{elapsed_ns},\"checksum\":{checksum}}}"
    );
    if args.get(3).is_some_and(|arg| arg == "heap") {
        std::mem::forget((ui, engine, text, output));
    }
}
