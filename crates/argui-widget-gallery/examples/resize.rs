//! Reproducible CPU resize workload; timings exclude the OS compositor and GPU.
use argui::{
    core::{Point, PointerId, Size},
    layout::LayoutEngine,
    runtime::{Entity, Inspection, InspectionCache},
    text::TextEngine,
    ui::{
        ClickEvent, Element, GestureDelivery, GestureEvent, GestureKind, GesturePhase, UiEventKind,
        UiTree,
    },
};
use argui_widget_gallery::WidgetGallery;
use web_time::Instant;

fn dispatch(gallery: &Entity<WidgetGallery>, tree: &mut UiTree, key: &str, kind: UiEventKind) {
    let target = tree
        .node_ids()
        .iter()
        .copied()
        .find(|node| tree.key(*node) == Some(key))
        .unwrap();
    for event in tree.event_deliveries(target, kind) {
        if event.should_dispatch() {
            gallery.dispatch_event(&event);
        }
    }
}

fn percentile(samples: &mut [f64], fraction: f64) -> f64 {
    samples.sort_by(f64::total_cmp);
    samples[((samples.len() - 1) as f64 * fraction).round() as usize]
}

fn main() {
    if std::env::args().any(|arg| arg == "--large-tree") {
        let mut tree = UiTree::new(Element::column((0..10_000).map(|index| {
            Element::container([])
                .keyed(index.to_string())
                .height(argui::ui::length(1.0))
        })));
        let mut layout = LayoutEngine::new();
        let output = layout
            .compute(&mut tree, &mut TextEngine::new(), Size::new(1220.0, 780.0))
            .unwrap();
        let mut samples = (0..30)
            .map(|_| {
                let start = Instant::now();
                std::hint::black_box(Inspection::snapshot(&tree, &output));
                start.elapsed().as_secs_f64() * 1000.0
            })
            .collect::<Vec<_>>();
        println!(
            "10k-node inspection p95={:.3} ms",
            percentile(&mut samples, 0.95)
        );
        let mut cache = InspectionCache::default();
        cache.snapshot(&tree, &output);
        let mut cached = (0..30)
            .map(|_| {
                let start = Instant::now();
                assert!(cache.snapshot(&tree, &output).is_none());
                start.elapsed().as_secs_f64() * 1000.0
            })
            .collect::<Vec<_>>();
        println!(
            "10k-node unchanged inspection p95={:.3} ms",
            percentile(&mut cached, 0.95)
        );
        return;
    }
    let gallery = Entity::new(WidgetGallery::default());
    let mut tree = UiTree::new(gallery.render());
    dispatch(
        &gallery,
        &mut tree,
        "nav::textarea",
        UiEventKind::Click(ClickEvent::accessibility()),
    );
    tree.update(gallery.render());
    let mut text = TextEngine::from_embedded_fonts(
        [include_bytes!("../../argui-web-demo/assets/fonts/NotoSans-Regular.ttf").as_slice()],
        "Noto Sans",
        "Noto Sans",
        "Noto Sans",
    );
    let mut layout = LayoutEngine::new();
    let target = UiTree::new(Element::container([])).node_id_at(0).unwrap();
    let gesture = |phase, total| {
        UiEventKind::Gesture(GestureEvent {
            target,
            pointer: PointerId::MOUSE,
            phase,
            delivery: GestureDelivery::FrameCoalesced,
            kind: GestureKind::Pan {
                position: total,
                delta: Point::default(),
                total,
                velocity: Point::default(),
            },
        })
    };
    dispatch(
        &gallery,
        &mut tree,
        "notes-resize",
        gesture(GesturePhase::Started, Point::default()),
    );
    let mut model = Vec::new();
    let mut geometry = Vec::new();
    let mut inspection = Vec::new();
    let mut total = Vec::new();
    for index in 0..300 {
        let frame = Instant::now();
        let position = Point::new((index as f32 * 0.09).sin() * 150.0, 0.0);
        dispatch(
            &gallery,
            &mut tree,
            "notes-resize",
            gesture(GesturePhase::Changed, position),
        );
        tree.update(gallery.render());
        let model_ms = frame.elapsed().as_secs_f64() * 1000.0;
        let started = Instant::now();
        let output = layout
            .compute(&mut tree, &mut text, Size::new(1220.0, 780.0))
            .unwrap();
        let layout_ms = started.elapsed().as_secs_f64() * 1000.0;
        let started = Instant::now();
        std::hint::black_box(Inspection::snapshot(&tree, &output));
        let inspection_ms = started.elapsed().as_secs_f64() * 1000.0;
        if index >= 60 {
            model.push(model_ms);
            geometry.push(layout_ms);
            inspection.push(inspection_ms);
            total.push(frame.elapsed().as_secs_f64() * 1000.0);
        }
    }
    for (label, mut samples) in [
        ("model/tree", model),
        ("layout/paint", geometry),
        ("inspection", inspection),
        ("CPU total", total),
    ] {
        let p95 = percentile(&mut samples, 0.95);
        let p99 = percentile(&mut samples, 0.99);
        println!("{label}: p95={p95:.3} ms p99={p99:.3} ms");
    }
}
