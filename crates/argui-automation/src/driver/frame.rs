//! Scene updates follow the same layout, scroll, paint, and composite routes as the runtime.

use super::*;
use argui_core::Size;
use argui_ui::InteractionUpdate;

impl Driver {
    /// Recomputes geometry after a host commit or a layout invalidation.
    /// `began` starts the measured update interval.
    ///
    /// # Errors
    /// Returns a layout error from the retained engine.
    pub(super) fn refresh(&mut self, began: Instant) -> Result<(), String> {
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
        self.record_frame(
            began,
            layout_started.duration_since(began),
            profile.layout,
            profile.paint,
            profile.passes,
        );
        Ok(())
    }

    /// Applies a non-layout animation or input update to the retained scene.
    /// `update` selects the native invalidation route; `text_input` refreshes
    /// editor geometry; `began` starts the measured update interval.
    ///
    /// # Errors
    /// Returns a scroll geometry or fallback layout error.
    pub(super) fn present(
        &mut self,
        update: TreeUpdate,
        text_input: bool,
        began: Instant,
    ) -> Result<(), String> {
        if update == TreeUpdate::Layout {
            return self.refresh(began);
        }
        if !text_input && matches!(update, TreeUpdate::None | TreeUpdate::Semantics) {
            return Ok(());
        }
        let (Some(tree), Some(output)) = (&mut self.tree, &mut self.layout) else {
            return self.refresh(began);
        };
        let _frame = self.metrics.span("frame.update");
        let paint_started = Instant::now();
        if text_input {
            let _paint = self.metrics.span("paint.text_input");
            self.engine.update_text_inputs(tree, &mut self.text, output);
        } else {
            match update {
                TreeUpdate::Scroll => {
                    let _paint = self.metrics.span("paint.scroll");
                    self.engine
                        .apply_scroll_with_text(tree, &mut self.text, output)
                        .map_err(|error| error.to_string())?;
                }
                TreeUpdate::Paint => {
                    let _paint = self.metrics.span("paint.generate");
                    self.engine.repaint(tree, output);
                }
                TreeUpdate::Composite => {
                    let _paint = self.metrics.span("paint.composite");
                    if !self.engine.composite(tree, output) {
                        self.engine.repaint(tree, output);
                    }
                }
                _ => unreachable!("layout and nonvisual updates returned above"),
            }
        }
        let paint = paint_started.elapsed();
        self.metrics.gauge(
            "paint.commands",
            output.display_list.commands().len() as f64,
            "count",
        );
        self.record_frame(began, Duration::ZERO, Duration::ZERO, paint, 0);
        Ok(())
    }

    /// Queues callbacks and applies the strongest native invalidation.
    /// `update` carries input effects and their layout or presentation flags.
    ///
    /// # Errors
    /// Returns a layout or scroll geometry error.
    pub(super) fn apply(&mut self, update: InteractionUpdate) -> Result<(), String> {
        let began = Instant::now();
        let invalidation = if update.layout_changed {
            TreeUpdate::Layout
        } else if update.scroll_changed {
            TreeUpdate::Scroll
        } else if update.paint_changed {
            TreeUpdate::Paint
        } else if update.composite_changed {
            TreeUpdate::Composite
        } else {
            TreeUpdate::None
        };
        self.queue(update.events);
        self.present(invalidation, update.text_input_changed, began)
    }

    /// Records one native update using `began` and its separate phase durations.
    /// `passes` is zero when retained geometry was reused.
    fn record_frame(
        &mut self,
        began: Instant,
        model: Duration,
        layout: Duration,
        paint: Duration,
        passes: usize,
    ) {
        let now = Instant::now();
        let interval = self
            .last_frame
            .map(|previous| now.duration_since(previous).as_secs_f64() * 1000.0);
        self.last_frame = Some(now);
        self.frames.push(StepFrame {
            timestamp_ms: self.metrics.now_ms(),
            interval_ms: interval,
            cpu_ms: now.duration_since(began).as_secs_f64() * 1000.0,
            layout_ms: layout.as_secs_f64() * 1000.0,
            paint_ms: paint.as_secs_f64() * 1000.0,
            layout_passes: passes,
            render_cpu_ms: None,
            rendered_at_ms: None,
        });
        self.metrics.gauge("layout.passes", passes as f64, "count");
        self.records.push(FrameRecord {
            interval: interval
                .map(|value| Duration::from_secs_f64(value / 1000.0))
                .unwrap_or_default(),
            model,
            layout,
            paint,
            ..FrameRecord::default()
        });
    }
}
