//! Real gallery actions, without a window or instrumentation in the runtime.
use argui::{
    core::{Point, Size},
    layout::LayoutEngine,
    runtime::{Entity, WindowEnvironment},
    text::TextEngine,
    ui::{ClickEvent, TreeUpdate, UiEventKind, UiTree},
};
use argui_widget_gallery::WidgetGallery;
use web_time::Instant;

fn dispatch(app: &Entity<WidgetGallery>, ui: &mut UiTree, key: &str, event: UiEventKind) {
    let target = ui
        .node_ids()
        .iter()
        .copied()
        .find(|id| ui.key(*id) == Some(key))
        .unwrap();
    for event in ui.event_deliveries(target, event) {
        if event.should_dispatch() {
            app.dispatch_event(&event);
        }
    }
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let page = args.get(1).map(String::as_str).unwrap_or("toggle");
    let frames: usize = args.get(2).and_then(|n| n.parse().ok()).unwrap_or(100);
    let key = match page {
        "toggle" => "bold",
        "accordion" => "faq::item::keyboard::trigger",
        "sidebar" => "workspace::toggle",
        "message-scroller" => "chat-add",
        "vlist" | "table" => "data",
        "data-table" => "grid::rows",
        _ => panic!("unknown page: {page}"),
    };
    let scrolling = matches!(page, "vlist" | "table" | "data-table");
    let app = Entity::new(WidgetGallery::default());
    let environment = WindowEnvironment {
        reduced_motion: true,
        ..Default::default()
    };
    let mut ui = UiTree::new(app.render_in(environment.clone()));
    dispatch(
        &app,
        &mut ui,
        &format!("nav::{page}"),
        UiEventKind::Click(ClickEvent::accessibility()),
    );
    ui.update(app.render_in(environment.clone()));
    ui.set_reduced_motion(true);
    let mut engine = LayoutEngine::new();
    const FONT: &[u8] = include_bytes!("../../argui-web-demo/assets/fonts/NotoSans-Regular.ttf");
    let mut text = TextEngine::from_embedded_fonts([FONT], "Noto Sans", "Noto Sans", "Noto Sans");
    let viewport = Size::new(1280.0, 900.0);
    let mut output = engine.compute(&mut ui, &mut text, viewport).unwrap();
    let mut layouts = 0;
    let start = Instant::now();
    for step in 0..frames {
        let event = if scrolling {
            let offset = Point::new(0.0, step as f32 * 4.0);
            let target = ui
                .node_ids()
                .iter()
                .copied()
                .find(|id| ui.key(*id) == Some(key))
                .unwrap();
            ui.set_scroll_offset(target, offset);
            UiEventKind::Scrolled {
                delta: Point::new(0.0, 4.0),
                offset,
            }
        } else {
            UiEventKind::Click(ClickEvent::accessibility())
        };
        dispatch(&app, &mut ui, key, event);
        match ui.update(app.render_in(environment.clone())) {
            TreeUpdate::Layout => {
                output = engine.compute(&mut ui, &mut text, viewport).unwrap();
                layouts += 1;
            }
            _ if scrolling => engine.apply_scroll(&ui, &mut output).unwrap(),
            _ => {
                engine.repaint(&ui, &mut output);
            }
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
    assert!(checksum.is_finite() && nodes > 100);
    println!(
        "{{\"case\":\"{page}\",\"frames\":{frames},\"nodes\":{nodes},\"layouts\":{layouts},\"elapsed_ns\":{elapsed_ns},\"checksum\":{checksum}}}"
    );
    if args.get(3).is_some_and(|arg| arg == "heap") {
        std::mem::forget((app, ui, engine, text, output));
    }
}
