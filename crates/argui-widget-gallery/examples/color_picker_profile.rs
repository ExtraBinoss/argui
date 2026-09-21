//! Bounded CPU replay of real gallery/DevTools color gestures, without GPU or runtime probes.
use argui::{
    core::{Point, PointerId, Rect, Size},
    layout::{LayoutEngine, LayoutOutput},
    runtime::{
        Entity, Inspection, InspectionCache, LayoutBounds, LayoutSnapshot, Mount, WindowEnvironment,
    },
    text::TextEngine,
    ui::{
        ClickEvent, GestureDelivery, GestureEvent, GestureKind, GesturePhase, TreeUpdate, UiEvent,
        UiEventKind, UiTree,
    },
};
use argui_devtools::DevtoolsHost;
use argui_widget_gallery::WidgetGallery;
use web_time::Instant;

struct Replay {
    host: Mount<DevtoolsHost<WidgetGallery>>,
    tree: UiTree,
    engine: LayoutEngine,
    text: TextEngine,
    output: LayoutOutput,
    inspection: InspectionCache,
    environment: WindowEnvironment,
}

impl Replay {
    fn new(open: bool) -> Self {
        let host = Entity::new(DevtoolsHost::new(WidgetGallery::default()).open(open))
            .mount()
            .unwrap();
        let environment = WindowEnvironment {
            reduced_motion: true,
            ..Default::default()
        };
        let mut tree = UiTree::new(host.render(environment.clone()).unwrap());
        tree.set_reduced_motion(true);
        let mut engine = LayoutEngine::new();
        const FONT: &[u8] =
            include_bytes!("../../argui-web-demo/assets/fonts/NotoSans-Regular.ttf");
        let mut text =
            TextEngine::from_embedded_fonts([FONT], "Noto Sans", "Noto Sans", "Noto Sans");
        let output = engine
            .compute(&mut tree, &mut text, Size::new(1220.0, 1000.0))
            .unwrap();
        let mut replay = Self {
            host,
            tree,
            engine,
            text,
            output,
            inspection: InspectionCache::default(),
            environment,
        };
        replay.layout();
        replay.publish();
        replay.click("nav::color-picker");
        replay
    }

    fn layout(&self) {
        self.host
            .layout_changed(&LayoutSnapshot {
                viewport: self.output.viewport,
                nodes: self
                    .output
                    .nodes
                    .iter()
                    .map(|node| LayoutBounds {
                        node: node.node,
                        key: self.tree.key(node.node).map(str::to_owned),
                        retained_identity: self
                            .tree
                            .element_for(node.node)
                            .and_then(|element| element.source_identity().cloned()),
                        bounds: node.bounds,
                    })
                    .collect(),
            })
            .unwrap();
    }

    fn publish(&mut self) {
        let inspector = self.host.read(|host| host.inspector());
        if inspector.enabled()
            && let Some(snapshot) = self.inspection.snapshot(&self.tree, &self.output)
        {
            inspector.publish_tree(snapshot);
        }
    }

    fn bounds(&self, key: &str) -> Rect {
        self.output
            .nodes
            .iter()
            .find(|node| self.tree.key(node.node) == Some(key))
            .unwrap()
            .bounds
    }

    fn dispatch(&mut self, key: &str, kind: UiEventKind) {
        let node = self
            .tree
            .node_ids()
            .iter()
            .copied()
            .find(|id| self.tree.key(*id) == Some(key))
            .unwrap_or_else(|| panic!("missing event target {key}"));
        for event in self.tree.event_deliveries(node, kind) {
            if event.should_dispatch() {
                self.host.dispatch_event(&event).unwrap();
            }
        }
    }

    fn click(&mut self, key: &str) {
        self.dispatch(key, UiEventKind::Click(ClickEvent::accessibility()));
        self.frame();
    }

    fn gesture(&mut self, key: &str, phase: GesturePhase, position: Point) {
        self.dispatch(
            key,
            UiEventKind::Gesture(GestureEvent {
                target: self.tree.node_ids()[0],
                pointer: PointerId::MOUSE,
                phase,
                delivery: GestureDelivery::FrameCoalesced,
                kind: GestureKind::Pan {
                    position,
                    delta: Point::default(),
                    total: Point::default(),
                    velocity: Point::default(),
                },
            }),
        );
    }

    fn frame(&mut self) -> (bool, [f64; 4]) {
        let start = Instant::now();
        let mut root = self.host.render(self.environment.clone()).unwrap();
        Inspection::apply_overrides(
            &mut root,
            &self.tree,
            &self.host.read(|host| host.inspector()),
        );
        let update = self.tree.update(root);
        let build = start.elapsed().as_secs_f64() * 1000.0;
        let start = Instant::now();
        if update == TreeUpdate::Layout {
            self.output = self
                .engine
                .compute(&mut self.tree, &mut self.text, self.output.viewport.size)
                .unwrap();
            self.layout();
        } else {
            self.engine.repaint(&self.tree, &mut self.output);
        }
        let layout = start.elapsed().as_secs_f64() * 1000.0;
        let start = Instant::now();
        std::hint::black_box(self.text.prepare(&self.output.text, 1.0));
        let shape = start.elapsed().as_secs_f64() * 1000.0;
        let start = Instant::now();
        self.publish();
        (
            update == TreeUpdate::Layout,
            [build, layout, shape, start.elapsed().as_secs_f64() * 1000.0],
        )
    }
}

fn main() {
    for case in [
        "gallery-pad",
        "gallery-hue",
        "devtools-property",
        "devtools-theme",
    ] {
        let mut replay = Replay::new(case.starts_with("devtools"));
        let key = match case {
            "devtools-property" => {
                let inspector = replay.host.read(|host| host.inspector());
                let id = inspector
                    .tree()
                    .nodes
                    .iter()
                    .find(|node| node.key.as_deref() == Some("color-preview-swatch"))
                    .unwrap()
                    .id
                    .0;
                replay
                    .host
                    .update(|host, cx| {
                        host.update(&UiEvent::new(
                            replay.tree.node_ids()[0],
                            Some(format!("__devtools-node-{id}")),
                            UiEventKind::Click(ClickEvent::accessibility()),
                        ));
                        cx.notify();
                    })
                    .unwrap();
                replay.frame();
                replay.click(&format!("__devtools-swatch-{id}-background"));
                format!("__devtools-color-{id}-background::pad")
            }
            "devtools-theme" => {
                replay.click("__devtools-theme");
                replay.click("__devtools-theme-color-primary");
                "__devtools-color-theme-primary::pad".into()
            }
            "gallery-hue" => "gallery-color::hue".into(),
            _ => "gallery-color::pad".into(),
        };
        let bounds = replay.bounds(&key);
        assert!(bounds.size.width > 0.0 && bounds.size.height > 0.0);
        replay.gesture(&key, GesturePhase::Started, bounds.origin);
        replay.frame();
        let mut samples = Vec::new();
        let mut phases = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
        let mut layouts = 0;
        for index in 0..340 {
            let x = 0.1 + (index % 89) as f32 / 110.0;
            let y = 0.1 + (index % 67) as f32 / 90.0;
            let start = Instant::now();
            replay.gesture(
                &key,
                GesturePhase::Changed,
                Point::new(
                    bounds.origin.x + bounds.size.width * x,
                    bounds.origin.y + bounds.size.height * y,
                ),
            );
            let (layout, times) = replay.frame();
            if index >= 40 {
                layouts += usize::from(layout);
                samples.push(start.elapsed().as_secs_f64() * 1000.0);
                for (samples, time) in phases.iter_mut().zip(times) {
                    samples.push(time);
                }
            }
        }
        samples.sort_by(f64::total_cmp);
        for phase in &mut phases {
            phase.sort_by(f64::total_cmp);
        }
        let memory = Inspection::memory(&replay.tree, &replay.engine, &replay.output);
        let bytes = memory.ui_index_bytes
            + memory.layout_metadata_bytes
            + memory.layout_cache_bytes
            + memory.layout_geometry_bytes
            + memory.layout_output_bytes
            + memory.paint_command_bytes;
        println!(
            "{{\"case\":\"{case}\",\"frames\":300,\"layouts\":{layouts},\"nodes\":{},\"storage_bytes\":{bytes},\"p50_ms\":{:.3},\"p95_ms\":{:.3},\"build_p95_ms\":{:.3},\"layout_p95_ms\":{:.3},\"shape_p95_ms\":{:.3},\"inspection_p95_ms\":{:.3}}}",
            memory.ui_nodes,
            samples[150],
            samples[285],
            phases[0][285],
            phases[1][285],
            phases[2][285],
            phases[3][285]
        );
    }
}
