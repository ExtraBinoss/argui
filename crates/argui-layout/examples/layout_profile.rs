//! Headless layout benchmark; no window, renderer, system fonts or GPU.
use std::hint::black_box;
use web_time::Instant;

use argui_core::{Color, Size};
use argui_layout::LayoutEngine;
use argui_text::TextEngine;
use argui_ui::{Element, UiTree, length};

fn root(count: usize) -> Element {
    Element::column((0..count.div_ceil(100)).map(|row| {
        Element::row((row * 100..((row + 1) * 100).min(count)).map(|i| {
            Element::container([])
                .keyed(i.to_string())
                .width(length(8.0))
                .height(length(8.0))
                .background(Color::WHITE)
        }))
        .keyed(row.to_string())
    }))
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let count = args
        .get(1)
        .and_then(|value| value.parse().ok())
        .unwrap_or(10_000);
    let mut ui = UiTree::new(root(count));
    let mut engine = LayoutEngine::new();
    let mut text = TextEngine::from_embedded_fonts([], "sans", "sans", "mono");
    let size = Size::new(1200.0, 1000.0);
    let start = Instant::now();
    let mut output = engine.compute(&mut ui, &mut text, size).unwrap();
    let first_ns = start.elapsed().as_nanos();
    if args.get(2).is_some_and(|mode| mode == "churn") {
        for _ in 0..5 {
            ui.update(root(100));
            output = engine.compute(&mut ui, &mut text, size).unwrap();
            black_box(&output);
            ui.update(root(count));
            output = engine.compute(&mut ui, &mut text, size).unwrap();
            black_box(&output);
        }
        ui.update(root(100));
        output = engine.compute(&mut ui, &mut text, size).unwrap();
    }
    let nodes = output.nodes.len();
    assert_eq!(engine.retained_node_count(), nodes);
    if args
        .get(2)
        .is_some_and(|mode| matches!(mode.as_str(), "heap" | "churn"))
    {
        println!("{{\"nodes\":{nodes}}}");
        // DHAT's at-exit counters measure these retained objects, not freed
        // construction scratch space. The process exits immediately afterwards.
        std::mem::forget((ui, engine, text, output));
        return;
    }
    let start = Instant::now();
    for _ in 0..20 {
        output = engine.compute(&mut ui, &mut text, size).unwrap();
        black_box(&output);
    }
    let cached_ns = start.elapsed().as_nanos() / 20;
    let start = Instant::now();
    for i in 0..20 {
        output = engine
            .compute(
                &mut ui,
                &mut text,
                Size::new(1200.0 + (i % 2) as f32, 1000.0),
            )
            .unwrap();
        black_box(&output);
    }
    let resize_ns = start.elapsed().as_nanos() / 20;
    let start = Instant::now();
    for _ in 0..20 {
        let mut next = ui.root().clone();
        next.children.rotate_left(1);
        ui.update(next);
        output = engine.compute(&mut ui, &mut text, size).unwrap();
        black_box(&output);
    }
    let reorder_ns = start.elapsed().as_nanos() / 20;
    println!(
        "{{\"leaves\":{count},\"nodes\":{nodes},\"first_ns\":{first_ns},\"cached_ns\":{cached_ns},\"resize_ns\":{resize_ns},\"reorder_ns\":{reorder_ns}}}"
    );
}
