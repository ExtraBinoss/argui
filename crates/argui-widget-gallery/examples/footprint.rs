//! Native release workload for external CPU/RSS sampling, including the real GPU renderer.
//! Usage: cargo run -p argui-widget-gallery --release --all-features --example footprint -- data-table scroll
use argui::runtime::tasks::{self, TaskSlot};
use argui::{
    core::Point,
    platform::{ApplicationConfig, ApplicationIdentity, WindowConfig},
    render::{EffectRegistry, RendererConfig},
    runtime::{Context, Entity, Render, SingleWindowModel, run_application_with_text_engine},
    text::TextEngine,
    ui::{ClickEvent, Element, ScrollRequest, UiEventKind, UiTree},
};
use argui_widget_gallery::WidgetGallery;
use std::time::Duration;
use web_time::Instant;

struct Workload {
    gallery: Entity<WidgetGallery>,
    scroll: Option<&'static str>,
    started: Instant,
    tick: TaskSlot,
    input: UiTree,
}

impl Render for Workload {
    fn image_assets(&self) -> Vec<argui::paint::ImageAsset> {
        self.gallery.read(Render::image_assets)
    }

    fn vector_assets(&self) -> Vec<argui::paint::VectorAsset> {
        self.gallery.read(Render::vector_assets)
    }

    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        cx.entity(&self.gallery)
    }
    fn layout_changed(&mut self, _: &argui::runtime::LayoutSnapshot, cx: &mut Context<Self>) {
        if self.scroll.is_some() && !self.tick.is_running() {
            self.schedule(cx);
        }
    }
}

impl Workload {
    fn schedule(&mut self, cx: &mut Context<Self>) {
        cx.spawn_latest(
            &mut self.tick,
            tasks::sleep(Duration::from_millis(16)),
            |workload, _, cx| {
                if let Some(key) = workload.scroll {
                    let elapsed = (workload.started.elapsed().as_secs_f32() - 3.0).max(0.0);
                    let offset = Point::new(0.0, elapsed * 960.0);
                    workload.input.update(workload.gallery.render());
                    let target = workload
                        .input
                        .node_ids()
                        .iter()
                        .copied()
                        .find(|node| workload.input.key(*node) == Some(key))
                        .expect("scroll viewport");
                    for event in workload.input.event_deliveries(
                        target,
                        UiEventKind::Scrolled {
                            delta: Point::new(0.0, 16.0),
                            offset,
                        },
                    ) {
                        if event.should_dispatch() {
                            workload.gallery.dispatch_event(&event);
                        }
                    }
                    cx.scroll(ScrollRequest::offset(key, offset));
                    workload.schedule(cx);
                }
            },
        )
        .expect("workload timer");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let page = args.next().unwrap_or_else(|| "table".into());
    let scrolling = args.next().as_deref() == Some("scroll");
    let scroll_key = match page.as_str() {
        "vlist" | "table" => "data",
        "data-table" => "grid::rows",
        _ => return Err("expected table, vlist, or data-table".into()),
    };
    let gallery = Entity::new(WidgetGallery::default());
    let mut tree = UiTree::new(gallery.render());
    let key = format!("nav::{page}");
    let target = tree
        .node_ids()
        .iter()
        .copied()
        .find(|node| tree.key(*node) == Some(key.as_str()))
        .expect("gallery navigation entry");
    for event in tree.event_deliveries(target, UiEventKind::Click(ClickEvent::accessibility())) {
        if event.should_dispatch() {
            gallery.dispatch_event(&event);
        }
    }
    let text = TextEngine::from_embedded_fonts(
        [include_bytes!("../../argui-web-demo/assets/fonts/NotoSans-Regular.ttf").as_slice()],
        "Noto Sans",
        "Noto Sans",
        "Noto Sans",
    );
    run_application_with_text_engine(
        ApplicationConfig::new(
            ApplicationIdentity::development("Argui footprint measurement"),
            WindowConfig {
                title: format!("Argui footprint · {page}"),
                width: 1220.0,
                height: 780.0,
                ..WindowConfig::default()
            },
        ),
        argui_devtools::configure_renderer(RendererConfig::default().effects(
            EffectRegistry::new(argui_effects::registry()?.definitions().iter().cloned())?,
        ))?,
        text,
        argui_devtools::DevtoolsApp::new(SingleWindowModel::new(
            argui::widgets::SelectionHost::new(Workload {
                gallery,
                scroll: scrolling.then_some(scroll_key),
                started: Instant::now(),
                tick: TaskSlot::default(),
                input: tree,
            }),
        )),
        |event| {
            if let argui::runtime::RuntimeEvent::Window {
                event:
                    argui::runtime::WindowRuntimeEvent::RendererFailed(message)
                    | argui::runtime::WindowRuntimeEvent::LayoutFailed(message),
                ..
            } = event
            {
                eprintln!("{message}");
            }
        },
    )?;
    Ok(())
}
