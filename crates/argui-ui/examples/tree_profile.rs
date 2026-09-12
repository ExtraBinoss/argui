//! Headless retained-tree benchmark. See docs/optimizations.md for the protocol.
use std::{hint::black_box, mem::size_of};
use web_time::Instant;

use argui_core::Color;
use argui_ui::{Element, ElementKind, TextSelectionStyle, UiEventKind, UiTree};

fn subtree(leaves: usize, fanout: usize) -> Element {
    if leaves == 1 {
        return Element::text("Benchmark leaf");
    }
    let chunk = leaves.div_ceil(fanout);
    Element::column(
        (0..leaves)
            .step_by(chunk)
            .map(|start| subtree(chunk.min(leaves - start), fanout).keyed(start.to_string())),
    )
}

fn rss_kib() -> usize {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|status| {
            status.lines().find_map(|line| {
                line.strip_prefix("VmRSS:")?
                    .split_whitespace()
                    .next()?
                    .parse()
                    .ok()
            })
        })
        .unwrap_or_default()
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let leaves: usize = args
        .get(1)
        .and_then(|value| value.parse().ok())
        .unwrap_or(10_000);
    let fanout = args
        .get(2)
        .and_then(|value| value.parse().ok())
        .unwrap_or(100);
    assert!(leaves > 0 && fanout > 1);
    let root = subtree(leaves, fanout);
    let start = Instant::now();
    let mut tree = UiTree::new(root);
    let build_ns = start.elapsed().as_nanos();
    let nodes = tree.node_ids().len();
    let rss = rss_kib();
    if args.get(3).is_some_and(|mode| mode == "heap") {
        println!("{{\"nodes\":{nodes},\"rss_kib\":{rss}}}");
        // Keep the retained graph alive at exit so Valgrind DHAT's te-g is its
        // live heap, excluding freed construction scratch space. This mode exits
        // immediately; the operating system reclaims the process's memory.
        std::mem::forget(tree);
        return;
    }
    let ids = tree.node_ids().to_vec();
    let start = Instant::now();
    for _ in 0..10 {
        for node in &ids {
            black_box(tree.parent_of(black_box(*node)));
        }
    }
    let parent_ns = start.elapsed().as_nanos() / (nodes * 10) as u128;
    let start = Instant::now();
    for _ in 0..10 {
        for node in &ids {
            black_box(tree.resolved_selection_style(black_box(*node)));
            black_box(tree.resolved_user_select(black_box(*node)));
        }
    }
    let selection_ns = start.elapsed().as_nanos() / (nodes * 10) as u128;
    let start = Instant::now();
    for i in 0..1000 {
        let node = ids[(i * 7919) % nodes];
        black_box(tree.event_deliveries(black_box(node), UiEventKind::Focused));
    }
    let event_ns = start.elapsed().as_nanos() / 1000;
    let start = Instant::now();
    for i in 0..30 {
        let color = if i % 2 == 0 {
            Color::WHITE
        } else {
            Color::BLACK
        };
        black_box(tree.update(tree.root().clone().background(color)));
    }
    let paint_update_ns = start.elapsed().as_nanos() / 30;
    println!(
        "{{\"leaves\":{leaves},\"fanout\":{fanout},\"nodes\":{nodes},\"element_bytes\":{},\"kind_bytes\":{},\"selection_bytes\":{},\"rss_kib\":{rss},\"build_ns\":{build_ns},\"parent_ns\":{parent_ns},\"selection_ns\":{selection_ns},\"event_ns\":{event_ns},\"paint_update_ns\":{paint_update_ns}}}",
        std::mem::size_of_val(&**tree.root()),
        size_of::<ElementKind>(),
        size_of::<TextSelectionStyle>()
    );
}
