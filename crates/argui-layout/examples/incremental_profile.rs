//! Short, opt-in workloads for retained updates; no runtime instrumentation.
use argui_core::{Color, Size};
use argui_layout::LayoutEngine;
use argui_text::TextEngine;
use argui_ui::{Element, TreeUpdate, UiTree, length};
use std::hint::black_box;
use web_time::Instant;

fn leaf(i: usize) -> Element {
    Element::container([])
        .keyed(i.to_string())
        .width(length(8.0))
        .height(length(8.0))
        .shrink(0.0)
        .background(Color::WHITE)
}

fn root(case: &str) -> Element {
    match case {
        "batch" => {
            let mut root = Element::row((0..200).map(leaf));
            for _ in 0..64 {
                root = Element::column([root]);
            }
            root
        }
        "wide" => Element::row((0..2000).map(leaf)),
        _ => Element::row((0..40).map(|i| {
            Element::column((0..40).map(leaf))
                .keyed(i.to_string())
                .width(length(30.0))
                .height(length(400.0))
                .shrink(0.0)
        })),
    }
}

fn change(root: &Element, case: &str, step: usize) -> Element {
    let mut next = root.clone();
    let height = length(8.0 + (step % 2) as f32);
    match case {
        "paint" => {
            next.children[0].children[0] =
                next.children[0].children[0]
                    .clone()
                    .background(if step.is_multiple_of(2) {
                        Color::BLACK
                    } else {
                        Color::WHITE
                    });
        }
        "batch" => {
            let mut branch = &mut next;
            for _ in 0..64 {
                branch = &mut branch.children[0];
            }
            for child in &mut branch.children {
                child.style.size.height = height;
            }
        }
        "structure" => {
            next.children.rotate_left(1);
            next.children[0].children.push(leaf(10_000 + step));
            next.children[0].children.remove(0);
        }
        "wide" => next.children[0].style.size.height = height,
        "panel" => next.children[0].children[0].style.size.height = height,
        _ => panic!("unknown workload: {case}"),
    }
    next
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let case = args.get(1).map(String::as_str).unwrap_or("paint");
    let frames = args.get(2).and_then(|n| n.parse().ok()).unwrap_or(100);
    let mut ui = UiTree::new(root(case));
    let mut engine = LayoutEngine::new();
    let mut text = TextEngine::from_embedded_fonts([], "sans", "sans", "mono");
    let viewport = Size::new(1280.0, 900.0);
    let mut output = engine.compute(&mut ui, &mut text, viewport).unwrap();
    let mut layouts = 0;
    let start = Instant::now();
    for step in 0..frames {
        match ui.update(change(ui.root(), case, step)) {
            TreeUpdate::Layout => {
                output = engine.compute(&mut ui, &mut text, viewport).unwrap();
                layouts += 1;
            }
            _ => {
                engine.repaint(&ui, &mut output);
            }
        }
        black_box(&output);
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
        "{{\"case\":\"{case}\",\"frames\":{frames},\"nodes\":{nodes},\"layouts\":{layouts},\"elapsed_ns\":{elapsed_ns},\"checksum\":{checksum}}}"
    );
}
