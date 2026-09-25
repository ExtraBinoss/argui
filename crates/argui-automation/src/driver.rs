use std::time::Duration;

use argui_animation::Time;
use argui_core::{
    Key, KeyInput, KeyState, MetricTrace, Point, PointerButton, PointerEvent, PointerId,
    PointerPhase, ScrollDelta, Size,
};
use argui_host::{Host, Operation};
use argui_inspect::{AdapterRecord, FrameRecord, GpuFrameRecord, GpuPassRecord, TreeSnapshot};
use argui_layout::{LayoutEngine, LayoutOutput};
use argui_render::RenderProfile;
use argui_runtime::{Inspection, NativeHostDelivery};
use argui_text::TextEngine;
use argui_ui::{InteractionUpdate, NodeId, TreeUpdate, UiEventKind, UiTree};
use serde::Serialize;
use web_time::Instant;

const FONT: &[u8] = include_bytes!("../../../assets/fonts/NotoSans-Regular.ttf");

mod actions;
mod input;
mod viewport;
use input::parse_key;
pub use viewport::Viewport;

/// Everyday automation inputs sent through Argui's retained event APIs.
#[derive(Clone, Debug)]
pub enum Action {
    /// Click a keyed element or point; secondary clicks also dispatch context menu.
    Click { target: String, right: bool },
    /// Scroll at a keyed element or point in pixels or lines; positive y moves down.
    Scroll {
        target: String,
        x: f32,
        y: f32,
        lines: bool,
    },
    /// Replace an input's value using Argui's user-edit path.
    Fill { target: String, value: String },
    /// Press and release one normalized keyboard key.
    Key { value: String },
    /// Drag between keyed elements or points, retaining pointer capture.
    Drag { from: String, to: String },
}

/// One scene update measured by the engine, relative to driver creation.
#[derive(Clone, Debug, Serialize)]
pub struct StepFrame {
    /// Elapsed milliseconds when the frame completed.
    pub timestamp_ms: f64,
    /// Time between this frame and the previous frame.
    pub interval_ms: Option<f64>,
    /// CPU time spent updating the retained tree and layout.
    pub cpu_ms: f64,
    /// CPU time spent computing geometry and text layout.
    pub layout_ms: f64,
    /// CPU time spent producing paint output or applying scroll presentation.
    pub paint_ms: f64,
    /// Container query passes needed to settle this frame's layout.
    pub layout_passes: usize,
    /// CPU time spent submitting renderer work, when rendered.
    pub render_cpu_ms: Option<f64>,
    /// Clock time when renderer submission completed, when rendered.
    pub rendered_at_ms: Option<f64>,
}

/// Platform-neutral host, retained UI, layout, and input state for one test.
pub struct Driver {
    host: Host,
    tree: Option<UiTree>,
    layout: Option<LayoutOutput>,
    engine: LayoutEngine,
    text: TextEngine,
    viewport: Viewport,
    started: Instant,
    last_advance: Instant,
    last_frame: Option<Instant>,
    frames: Vec<StepFrame>,
    records: Vec<FrameRecord>,
    metrics: MetricTrace,
    deliveries: Vec<NativeHostDelivery>,
}

impl Driver {
    /// Creates a windowless driver for `viewport` using the built-in host ABI.
    ///
    /// # Errors
    /// Returns a schema or viewport error.
    pub fn new(viewport: Viewport) -> Result<Self, String> {
        viewport.physical_size()?;
        let started = Instant::now();
        Ok(Self {
            host: Host::with_builtins().map_err(|error| error.to_string())?,
            tree: None,
            layout: None,
            engine: LayoutEngine::new(),
            text: TextEngine::from_embedded_fonts([FONT], "Noto Sans", "Noto Sans", "Noto Sans"),
            viewport,
            started,
            last_advance: started,
            last_frame: None,
            frames: Vec::new(),
            records: Vec::new(),
            metrics: MetricTrace::new(),
            deliveries: Vec::new(),
        })
    }

    /// Returns the schema fingerprint expected by the application bridge.
    #[must_use]
    pub fn abi_hash(&self) -> u64 {
        self.host.abi_hash()
    }

    /// Returns the selected logical viewport.
    #[must_use]
    pub const fn viewport(&self) -> Viewport {
        self.viewport
    }

    /// Recomputes the mounted application for a new validated viewport.
    /// `viewport` controls logical layout and screenshot pixel size.
    ///
    /// # Errors
    /// Returns a viewport or layout error.
    pub fn set_viewport(&mut self, viewport: Viewport) -> Result<(), String> {
        viewport.physical_size()?;
        self.viewport = viewport;
        if self.tree.is_some() {
            self.refresh(Instant::now())?;
        }
        Ok(())
    }

    /// Applies a normal `argui-host` operation batch and refreshes retained layout.
    /// `operations` are the application's native transactions.
    ///
    /// # Errors
    /// Returns a host, tree, or layout error.
    pub fn commit(&mut self, operations: &[Operation]) -> Result<(), String> {
        let commit = self.metrics.span("model.commit");
        let began = Instant::now();
        let result = self
            .host
            .commit(operations)
            .map_err(|error| error.to_string())?;
        drop(commit);
        if !result.visible_changed {
            return Ok(());
        }
        let reconcile = self.metrics.span("tree.reconcile");
        let root = self.host.root_element();
        self.tree = match (self.tree.take(), root) {
            (Some(mut tree), Some(root)) => {
                tree.update(root);
                Some(tree)
            }
            (_, Some(root)) => Some(UiTree::new(root)),
            (_, None) => None,
        };
        drop(reconcile);
        self.refresh(began)
    }

    /// Returns an inspector snapshot of the current retained UI.
    ///
    /// # Errors
    /// Returns an error while the application has no mounted root.
    pub fn snapshot(&self) -> Result<TreeSnapshot, String> {
        let (Some(tree), Some(layout)) = (&self.tree, &self.layout) else {
            return Err("application has not mounted a visible root".into());
        };
        Ok(Inspection::snapshot(tree, layout))
    }

    /// Checks whether visible inspected text contains `expected`.
    ///
    /// # Errors
    /// Returns the visible text summaries when the assertion fails.
    pub fn expect_text(&self, expected: &str) -> Result<(), String> {
        let snapshot = self.snapshot()?;
        if snapshot.nodes.iter().any(|node| {
            node.visible
                && node.kind == "text"
                && node
                    .summary
                    .as_deref()
                    .is_some_and(|text| text.contains(expected))
        }) {
            return Ok(());
        }
        let visible = snapshot
            .nodes
            .iter()
            .filter(|node| node.visible && node.kind == "text")
            .filter_map(|node| node.summary.as_deref())
            .collect::<Vec<_>>()
            .join(" | ");
        Err(format!(
            "expected visible text {expected:?}; found: {visible}"
        ))
    }

    /// Returns the center of a keyed node or explicit `x,y` coordinate.
    ///
    /// # Errors
    /// Returns an error for a missing, invisible, or non-finite target.
    pub fn point(&self, target: &str) -> Result<Point, String> {
        if let Some((x, y)) = target.split_once(',') {
            if let (Ok(x), Ok(y)) = (x.parse::<f32>(), y.parse::<f32>())
                && x.is_finite()
                && y.is_finite()
            {
                return Ok(Point::new(x, y));
            }
            return Err(format!("invalid coordinate target {target:?}; use x,y"));
        }
        let node = self.find_node(target)?;
        let layout = self.layout.as_ref().expect("mounted node has layout");
        let bounds = layout
            .nodes
            .iter()
            .find(|item| item.node == node)
            .ok_or_else(|| format!("{target:?} has no layout bounds"))?
            .bounds;
        Ok(Point::new(
            bounds.origin.x + bounds.size.width / 2.0,
            bounds.origin.y + bounds.size.height / 2.0,
        ))
    }

    /// Drains UI events mapped to still-live application callbacks.
    /// Returns events in their original dispatch order.
    pub fn take_deliveries(&mut self) -> Vec<NativeHostDelivery> {
        std::mem::take(&mut self.deliveries)
    }

    /// Returns all recorded frame measurements.
    #[must_use]
    pub fn frames(&self) -> &[StepFrame] {
        &self.frames
    }

    /// Returns the engine's raw frame records for performance summaries.
    #[must_use]
    pub fn frame_records(&self) -> &[FrameRecord] {
        &self.records
    }

    /// Returns the shared trace handle for actions, scene work, and report output.
    #[must_use]
    pub fn metrics(&self) -> MetricTrace {
        self.metrics.clone()
    }

    /// Returns the latest layout's scene data and mutable text engine for rendering.
    ///
    /// # Errors
    /// Returns an error before the application mounts.
    pub fn scene(&mut self) -> Result<(&LayoutOutput, &mut TextEngine), String> {
        Ok((
            self.layout.as_ref().ok_or("application has no layout")?,
            &mut self.text,
        ))
    }

    /// Adds the measured renderer CPU time to the newest frame.
    /// `duration` is the elapsed time for scene submission.
    pub fn record_render(&mut self, duration: Duration) {
        if let Some(frame) = self.frames.last_mut() {
            frame.render_cpu_ms = Some(duration.as_secs_f64() * 1000.0);
            frame.rendered_at_ms = Some(self.metrics.now_ms());
        }
        if let Some(record) = self.records.last_mut() {
            record.render_cpu = duration;
        }
    }

    /// Attaches measured renderer work to the newest frame.
    /// `profile` contains CPU submission, workload counters, adapter identity,
    /// and optional GPU timestamps from the completed offscreen render.
    pub fn record_render_profile(&mut self, profile: &RenderProfile) {
        let Some(record) = self.records.last_mut() else {
            return;
        };
        record.passes = profile.draw_batches;
        record.damaged_pixels = profile.damage.damaged_pixels;
        record.text_atlas_bytes = profile.text_atlas.allocated_bytes;
        record.text_atlas_entries = profile.text_atlas.entries;
        record.text_atlas_hits = profile.text_atlas.hits_this_frame;
        record.text_raster_requests = profile.text_atlas.raster_requests_this_frame;
        record.text_upload_bytes = profile.text_atlas.uploaded_bytes_this_frame;
        record.vector_atlas_bytes = profile.vector_atlas.allocated_bytes;
        record.vector_atlas_entries = profile.vector_atlas.entries;
        record.vector_atlas_hits = profile.vector_atlas.hits_this_frame;
        record.vector_rasterizations = profile.vector_atlas.rasterizations_this_frame;
        record.adapter = AdapterRecord {
            name: profile.adapter.name.clone(),
            backend: profile.adapter.backend.clone(),
            device_type: profile.adapter.device_type.clone(),
            timestamp_queries: profile.adapter.timestamp_queries,
            ..AdapterRecord::default()
        };
        record.gpu = profile.gpu.as_ref().map(|gpu| GpuFrameRecord {
            sequence: gpu.frame,
            total: gpu.total,
            passes: gpu
                .passes
                .iter()
                .map(|pass| GpuPassRecord {
                    label: pass.label.clone(),
                    start: pass.start,
                    duration: pass.duration,
                    pixels: pass.pixels,
                    object_domain: pass.object.map(|object| format!("{:?}", object.domain)),
                    object_id: pass.object.map(|object| object.value),
                })
                .collect(),
        });
        self.metrics
            .gauge("render.draw_batches", profile.draw_batches as f64, "count");
        self.metrics.gauge(
            "render.damaged_pixels",
            profile.damage.damaged_pixels as f64,
            "pixels",
        );
        self.metrics.gauge(
            "render.text_atlas_bytes",
            profile.text_atlas.allocated_bytes as f64,
            "bytes",
        );
        self.metrics.gauge(
            "render.text_upload_bytes",
            profile.text_atlas.uploaded_bytes_this_frame as f64,
            "bytes",
        );
        self.metrics.gauge(
            "render.vector_rasterizations",
            profile.vector_atlas.rasterizations_this_frame as f64,
            "count",
        );
        if let Some(gpu) = &profile.gpu {
            self.metrics
                .gauge("render.gpu_time", gpu.total.as_secs_f64() * 1000.0, "ms");
        }
    }

    /// Advances retained animations, scroll physics, and coalesced gestures.
    /// Returns after recording any changed scene and queuing callback deliveries.
    ///
    /// # Errors
    /// Returns a layout error if an animated scene cannot be recomputed.
    pub fn advance(&mut self) -> Result<(), String> {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_advance).as_secs_f32();
        self.last_advance = now;
        let scroll_regions = self
            .layout
            .as_ref()
            .map(|layout| layout.scroll_regions.clone())
            .unwrap_or_default();
        let Some(tree) = self.tree.as_mut() else {
            return Ok(());
        };
        let time = Time::from_nanos(
            now.duration_since(self.started)
                .as_nanos()
                .min(u128::from(u64::MAX)) as u64,
        );
        let animation = tree.advance_animations(time);
        let gestures = tree.flush_gesture_frame();
        let scrolling = tree.advance_scroll_physics(elapsed, &scroll_regions);
        self.apply(gestures)?;
        self.apply(scrolling)?;
        if animation != TreeUpdate::None {
            self.refresh(now)?;
        }
        Ok(())
    }

    /// Recomputes layout after a host commit or input state change.
    fn refresh(&mut self, began: Instant) -> Result<(), String> {
        let _frame = self.metrics.span("frame.update");
        let layout_started = Instant::now();
        let profile = if let Some(tree) = &mut self.tree {
            let (output, profile) = self
                .engine
                .compute_traced(
                    tree,
                    &mut self.text,
                    Size::new(self.viewport.width as f32, self.viewport.height as f32),
                    &self.metrics,
                )
                .map_err(|error| error.to_string())?;
            let virtual_events = output.virtual_events.clone();
            self.metrics
                .gauge("scene.nodes", output.nodes.len() as f64, "count");
            self.metrics.gauge(
                "paint.commands",
                output.display_list.commands().len() as f64,
                "count",
            );
            self.metrics.gauge(
                "paint.reused_subtrees",
                output.paint_stats.reused_subtrees as f64,
                "count",
            );
            self.layout = Some(output);
            self.queue(virtual_events);
            profile
        } else {
            self.layout = None;
            Default::default()
        };
        let now = Instant::now();
        let interval = self
            .last_frame
            .map(|previous| now.duration_since(previous).as_secs_f64() * 1000.0);
        self.last_frame = Some(now);
        self.frames.push(StepFrame {
            timestamp_ms: self.metrics.now_ms(),
            interval_ms: interval,
            cpu_ms: now.duration_since(began).as_secs_f64() * 1000.0,
            layout_ms: profile.layout.as_secs_f64() * 1000.0,
            paint_ms: profile.paint.as_secs_f64() * 1000.0,
            layout_passes: profile.passes,
            render_cpu_ms: None,
            rendered_at_ms: None,
        });
        self.metrics
            .gauge("layout.passes", profile.passes as f64, "count");
        self.records.push(FrameRecord {
            interval: self
                .last_frame
                .and_then(|_| interval.map(|value| Duration::from_secs_f64(value / 1000.0)))
                .unwrap_or_default(),
            model: layout_started.duration_since(began),
            layout: profile.layout,
            paint: profile.paint,
            ..FrameRecord::default()
        });
        Ok(())
    }

    /// Converts an interaction update to callbacks and refreshes dirty paint/layout.
    fn apply(&mut self, update: InteractionUpdate) -> Result<(), String> {
        let began = Instant::now();
        let changed = update.layout_changed
            || update.paint_changed
            || update.scroll_changed
            || update.text_input_changed
            || update.composite_changed;
        self.queue(update.events);
        if changed {
            self.refresh(began)?;
        }
        Ok(())
    }

    /// Queues callback deliveries while their host listeners are still mounted.
    fn queue(&mut self, events: Vec<argui_ui::UiEvent>) {
        self.deliveries
            .extend(events.into_iter().filter_map(|event| {
                self.host
                    .callback_for(&event)
                    .map(|callback| NativeHostDelivery {
                        callback,
                        kind: event.kind,
                    })
            }));
    }

    /// Returns cloned hit regions for safe mutation of the retained tree.
    fn regions(&self) -> Result<Vec<argui_ui::HitRegion>, String> {
        Ok(self
            .layout
            .as_ref()
            .ok_or("application has no layout")?
            .hit_regions
            .clone())
    }

    /// Finds a currently mounted node with authored key `target`.
    fn find_node(&self, target: &str) -> Result<NodeId, String> {
        let tree = self
            .tree
            .as_ref()
            .ok_or("application has no mounted root")?;
        tree.resolve_node(&argui_ui::FocusTarget::Key(target.to_owned()))
            .ok_or_else(|| format!("no mounted element with id {target:?}"))
    }

    /// Returns the topmost hit node at `point`.
    fn hit_node(&self, point: Point) -> Result<NodeId, String> {
        self.layout
            .as_ref()
            .ok_or("application has no layout")?
            .hit_regions
            .iter()
            .rev()
            .find(|region| region.enabled && region.contains(point))
            .map(|region| region.node)
            .ok_or_else(|| "no interactive element at context-click point".into())
    }

    /// Sends one mouse phase through the canonical UI tree.
    fn pointer(
        &mut self,
        point: Point,
        phase: PointerPhase,
        button: Option<PointerButton>,
    ) -> Result<(), String> {
        let regions = self.regions()?;
        let event = PointerEvent {
            button: (phase != PointerPhase::Moved).then_some(button).flatten(),
            buttons: if matches!(phase, PointerPhase::Pressed | PointerPhase::Moved)
                && button.is_some()
            {
                if button == Some(PointerButton::Secondary) {
                    2
                } else {
                    1
                }
            } else {
                0
            },
            timestamp: self.started.elapsed(),
            ..PointerEvent::mouse(phase, point)
        };
        let update = self
            .tree
            .as_mut()
            .expect("mounted tree")
            .pointer_event(event, &regions);
        self.apply(update)
    }
}
