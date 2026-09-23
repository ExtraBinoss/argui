use argui_animation::{Duration as AnimationDuration, Frame, Time};
use argui_core::{Point, Rect, Size};
use argui_layout::{LayoutEngine, LayoutOutput};
use argui_platform::WindowKey;
use argui_runtime::{
    AppCommand, AppEvent, AppModel, AppUpdate, Entity, LayoutBounds, LayoutSnapshot, Render,
    SingleWindowModel, ViewUpdate, WindowEnvironment,
};
use argui_text::TextEngine;
use argui_ui::{
    ClipboardRequest, InteractionUpdate, ScrollAlignment, ScrollRequest, ScrollTarget, TreeUpdate,
    UiTree,
};

use crate::TestError;

const DEFAULT_SETTLE_LIMIT: usize = 32;

/// Headless retained application with deterministic layout and input delivery.
pub struct TestApp<A: Render> {
    pub(crate) entity: Entity<A>,
    pub(crate) model: SingleWindowModel<A>,
    pub(crate) window: WindowKey,
    pub(crate) environment: WindowEnvironment,
    pub(crate) viewport: Size,
    pub(crate) ui: Option<UiTree>,
    pub(crate) layout_engine: LayoutEngine,
    pub(crate) text_engine: TextEngine,
    pub(crate) layout: Option<LayoutOutput>,
    pub(crate) clipboard: Option<String>,
    pub(crate) commands: Vec<AppCommand>,
    pub(crate) scroll_requests: Vec<ScrollRequest>,
    pub(crate) now: Time,
    pub(crate) pending: bool,
    pub(crate) settle_limit: usize,
    #[cfg(feature = "tasks")]
    pub(crate) tasks: argui_runtime::tasks::TaskRuntime,
}

impl<A: Render> TestApp<A> {
    /// Creates and settles a headless application at a 1024 by 768 logical viewport.
    ///
    /// # Panics
    ///
    /// Panics when initial layout or stabilization fails. Use [`TestApp::try_new`]
    /// to handle the failure explicitly.
    #[must_use]
    pub fn new(app: A) -> Self {
        Self::try_new(app).unwrap_or_else(|error| panic!("{error}"))
    }

    /// Creates and settles a headless application, returning initialization failures.
    ///
    /// # Errors
    ///
    /// Returns a layout or non-settling diagnostic.
    pub fn try_new(app: A) -> Result<Self, TestError> {
        let entity = Entity::new(app);
        Self::from_entity_in_window(entity, WindowKey::main())
    }

    /// Creates an independent headless window presentation for an existing entity.
    ///
    /// `entity` supplies shared application state and `window` identifies this
    /// presentation. Each call owns independent focus, layout, handler slots, and
    /// presentation-scoped resources.
    ///
    /// # Errors
    /// Returns a layout, closed-scope, or non-settling diagnostic.
    pub fn from_entity_in_window(entity: Entity<A>, window: WindowKey) -> Result<Self, TestError> {
        let model = SingleWindowModel::from_entity(entity.clone())
            .map_err(|_| TestError::ClosedPresentation)?
            .window_key(window.clone());
        #[cfg(feature = "tasks")]
        let tasks = {
            #[cfg(not(target_arch = "wasm32"))]
            let tasks = argui_runtime::tasks::TaskRuntime::new_paused(|| {});
            #[cfg(target_arch = "wasm32")]
            let tasks = argui_runtime::tasks::TaskRuntime::new(|| {});
            entity.set_task_runtime(tasks.clone());
            tasks
        };
        let mut test = Self {
            entity,
            model,
            window,
            environment: WindowEnvironment::default(),
            viewport: Size::new(1024.0, 768.0),
            ui: None,
            layout_engine: LayoutEngine::new(),
            text_engine: TextEngine::from_embedded_fonts(
                [
                    epaint_default_fonts::UBUNTU_LIGHT,
                    epaint_default_fonts::HACK_REGULAR,
                    epaint_default_fonts::NOTO_EMOJI_REGULAR,
                ],
                "Ubuntu",
                "Ubuntu",
                "Hack",
            ),
            layout: None,
            clipboard: None,
            commands: Vec::new(),
            scroll_requests: Vec::new(),
            now: Time::ZERO,
            pending: true,
            settle_limit: DEFAULT_SETTLE_LIMIT,
            #[cfg(feature = "tasks")]
            tasks,
        };
        test.settle()?;
        Ok(test)
    }

    /// Returns the retained application entity for explicit state inspection.
    #[must_use]
    pub const fn entity(&self) -> &Entity<A> {
        &self.entity
    }

    /// Returns the application-window identity used for model events.
    #[must_use]
    pub const fn window_key(&self) -> &WindowKey {
        &self.window
    }

    /// Returns the logical platform and theme environment used by the current window.
    #[must_use]
    pub const fn environment(&self) -> &WindowEnvironment {
        &self.environment
    }

    /// Changes the maximum render/effect iterations allowed during stabilization.
    pub fn set_settle_limit(&mut self, limit: usize) {
        self.settle_limit = limit.max(1);
    }

    /// Renders, reconciles, lays out, delivers layout effects, and repeats to quiescence.
    ///
    /// # Errors
    ///
    /// Returns a layout error or a bounded non-settling diagnostic.
    pub fn settle(&mut self) -> Result<(), TestError> {
        for _ in 0..self.settle_limit {
            #[cfg(all(feature = "tasks", not(target_arch = "wasm32")))]
            {
                self.tasks
                    .advance_time(std::time::Duration::ZERO)
                    .map_err(|error| TestError::Task {
                        message: error.to_string(),
                    })?;
                let update = self.model.tasks_ready(&self.window);
                self.record_app_update(update);
            }
            self.pending = false;
            let root = self
                .model
                .view(&self.window, self.environment.clone())
                .expect("single-window test model must provide its main view");
            let tree_update = match &mut self.ui {
                Some(ui) => ui.update(root),
                None => {
                    self.ui = Some(UiTree::new(root));
                    TreeUpdate::Layout
                }
            };
            let needs_layout = self.layout.is_none()
                || tree_update == TreeUpdate::Layout
                || self.ui().layout_dirty();
            if needs_layout {
                self.compute_layout()?;
                let update = self
                    .model
                    .layout_changed(&self.window, &self.layout_snapshot());
                self.record_app_update(update);
            }
            self.refresh_interaction_observations();
            let effects = self.apply_model_effects()?;
            if !self.pending && !effects {
                return Ok(());
            }
        }
        Err(TestError::DidNotSettle {
            limit: self.settle_limit,
            tree: self.compact_dump(),
        })
    }

    pub(crate) fn ui(&self) -> &UiTree {
        self.ui.as_ref().expect("test application is settled")
    }

    pub(crate) fn ui_mut(&mut self) -> &mut UiTree {
        self.ui.as_mut().expect("test application is settled")
    }

    pub(crate) fn output(&self) -> &LayoutOutput {
        self.layout.as_ref().expect("test application is settled")
    }

    fn compute_layout(&mut self) -> Result<(), TestError> {
        let output = self
            .layout_engine
            .compute(
                self.ui.as_mut().expect("UI created before layout"),
                &mut self.text_engine,
                self.viewport,
            )
            .map_err(|error| TestError::Layout {
                message: error.to_string(),
            })?;
        self.ui_mut().mark_layout_clean();
        self.layout = Some(output);
        Ok(())
    }

    fn layout_snapshot(&self) -> LayoutSnapshot {
        LayoutSnapshot {
            viewport: self.output().viewport,
            nodes: self
                .output()
                .nodes
                .iter()
                .map(|layout| LayoutBounds {
                    node: layout.node,
                    key: self.ui().key(layout.node).map(str::to_owned),
                    retained_identity: self
                        .ui()
                        .element_for(layout.node)
                        .and_then(|element| element.source_identity().cloned()),
                    bounds: layout.bounds,
                })
                .collect(),
        }
    }

    pub(crate) fn dispatch_interaction(
        &mut self,
        update: InteractionUpdate,
    ) -> Result<(), TestError> {
        self.refresh_interaction_observations();
        if let Some(request) = update.clipboard {
            self.apply_clipboard_request(request)?;
        }
        self.pending |= update.layout_changed || update.text_input_changed;
        for event in update.events {
            if !event.should_dispatch() {
                continue;
            }
            let update = self.model.update(&AppEvent::Ui {
                window: self.window.clone(),
                event,
            });
            self.record_app_update(update);
        }
        self.apply_model_effects()?;
        Ok(())
    }

    /// Publishes UI state before handlers run and schedules watched visual changes.
    fn refresh_interaction_observations(&mut self) {
        let Some(ui) = self.ui.as_ref() else {
            return;
        };
        let regions = self
            .layout
            .as_ref()
            .map_or(&[][..], |layout| layout.hit_regions.as_slice());
        let scroll_regions = self
            .layout
            .as_ref()
            .map_or(&[][..], |layout| layout.scroll_regions.as_slice());
        let snapshot = self.layout_snapshot();
        let changed = self.model.refresh_interaction_observations(
            ui,
            regions,
            scroll_regions,
            None,
            Some(&snapshot),
        );
        self.pending |= changed;
    }

    pub(crate) fn record_app_update(&mut self, update: AppUpdate) {
        self.pending |= update
            .windows
            .iter()
            .any(|window| window.window == self.window && window.update != ViewUpdate::None);
        self.commands.extend(update.commands);
    }

    fn apply_model_effects(&mut self) -> Result<bool, TestError> {
        let mut changed = false;
        loop {
            let commands = self.model.take_ui_commands(&self.window);
            let focus = self.model.take_focus_request(&self.window);
            let selection = self.model.take_text_selection_request(&self.window);
            let clipboard = self.model.take_clipboard_request(&self.window);
            let scroll = self.model.take_scroll_request(&self.window);
            let theme = self.model.take_theme_request(&self.window);
            if commands.is_empty()
                && focus.is_none()
                && selection.is_none()
                && clipboard.is_none()
                && scroll.is_none()
                && theme.is_none()
            {
                break;
            }
            changed = true;
            let mut interaction = InteractionUpdate::default();
            let hit_regions = self.output().hit_regions.clone();
            if let Some(request) = focus {
                interaction.merge(self.ui_mut().sync_focus(&hit_regions, Some(request)));
            }
            if let Some(request) = selection {
                interaction.merge(self.ui_mut().select_text(request));
            }
            for command in commands {
                interaction.merge(self.ui_mut().apply_command(command));
            }
            if let Some(request) = clipboard {
                self.apply_clipboard_request(request)?;
            }
            if let Some(request) = scroll {
                self.pending |= self.apply_scroll_request(&request);
                self.scroll_requests.push(request);
            }
            if let Some(request) = theme {
                if let Some(color_scheme) = request.color_scheme {
                    self.environment.color_scheme = color_scheme;
                }
                if let Some(primary) = request.primary {
                    self.environment.primary = primary;
                }
                self.pending = true;
            }
            if !interaction.events.is_empty()
                || interaction.layout_changed
                || interaction.text_input_changed
            {
                self.dispatch_interaction(interaction)?;
            }
        }
        Ok(changed)
    }

    fn apply_clipboard_request(&mut self, request: ClipboardRequest) -> Result<(), TestError> {
        match request {
            ClipboardRequest::Write(value) => self.clipboard = Some(value),
            ClipboardRequest::Read { target } => {
                if let Some(value) = self.clipboard.clone() {
                    let update = self.ui_mut().paste_text(target, &value);
                    self.dispatch_interaction(update)?;
                }
            }
        }
        Ok(())
    }

    fn apply_scroll_request(&mut self, request: &ScrollRequest) -> bool {
        let tracks = match &request.target {
            ScrollTarget::Offset { container, offset } => self
                .ui()
                .resolve_node(container)
                .and_then(|node| {
                    self.output()
                        .scroll_regions
                        .iter()
                        .find(|region| region.node == node)
                        .map(|region| (node, clamp_point(*offset, region.max_offset)))
                })
                .into_iter()
                .collect(),
            ScrollTarget::Rect { container, rect } => self
                .ui()
                .resolve_node(container)
                .and_then(|node| {
                    self.output()
                        .scroll_regions
                        .iter()
                        .find(|region| region.node == node)
                        .map(|region| {
                            (
                                node,
                                reveal_offset(
                                    self.ui().scroll_offset(node),
                                    region,
                                    *rect,
                                    request,
                                ),
                            )
                        })
                })
                .into_iter()
                .collect(),
            ScrollTarget::Element(target) => {
                let Some(node) = self.ui().resolve_node(target) else {
                    return false;
                };
                let Some(mut rect) = self
                    .output()
                    .nodes
                    .iter()
                    .find(|candidate| candidate.node == node)
                    .map(|candidate| candidate.bounds)
                else {
                    return false;
                };
                let mut tracks = Vec::new();
                let mut cursor = Some(node);
                while let Some(current) = cursor {
                    if let Some(region) = self
                        .output()
                        .scroll_regions
                        .iter()
                        .find(|region| region.node == current)
                    {
                        let from = self.ui().scroll_offset(current);
                        let to = reveal_offset(from, region, rect, request);
                        rect.origin.x -= to.x - from.x;
                        rect.origin.y -= to.y - from.y;
                        tracks.push((current, to));
                    }
                    cursor = self.ui().parent_of(current);
                }
                tracks
            }
        };
        let mut changed = false;
        for (node, offset) in tracks {
            changed |= self.ui_mut().set_scroll_offset(node, offset);
        }
        changed
    }

    pub(crate) fn frame(&mut self, elapsed: std::time::Duration) -> Result<(), TestError> {
        let elapsed = AnimationDuration::from(elapsed);
        self.now = self.now + elapsed;
        let update = self.model.animation_frame(
            &self.window,
            Frame {
                now: self.now,
                elapsed,
            },
        );
        self.record_app_update(update);
        let now = self.now;
        self.pending |= self.ui_mut().advance_animations(now) != TreeUpdate::None;
        self.settle()
    }

    pub(crate) fn point_in_viewport(&self, point: Point) -> bool {
        Rect::new(Point::default(), self.viewport).contains(point)
    }
}

fn reveal_offset(
    current: Point,
    region: &argui_ui::ScrollRegion,
    target: Rect,
    request: &ScrollRequest,
) -> Point {
    clamp_point(
        Point::new(
            align_axis(
                current.x,
                region.bounds.origin.x,
                region.bounds.origin.x + region.bounds.size.width,
                target.origin.x - request.margin.left,
                target.origin.x + target.size.width + request.margin.right,
                request.x,
            ),
            align_axis(
                current.y,
                region.bounds.origin.y,
                region.bounds.origin.y + region.bounds.size.height,
                target.origin.y - request.margin.top,
                target.origin.y + target.size.height + request.margin.bottom,
                request.y,
            ),
        ),
        region.max_offset,
    )
}

fn align_axis(
    current: f32,
    viewport_start: f32,
    viewport_end: f32,
    target_start: f32,
    target_end: f32,
    alignment: ScrollAlignment,
) -> f32 {
    current
        + match alignment {
            ScrollAlignment::Start => target_start - viewport_start,
            ScrollAlignment::Center => {
                (target_start + target_end - viewport_start - viewport_end) * 0.5
            }
            ScrollAlignment::End => target_end - viewport_end,
            ScrollAlignment::Nearest if target_start < viewport_start => {
                target_start - viewport_start
            }
            ScrollAlignment::Nearest if target_end > viewport_end => target_end - viewport_end,
            ScrollAlignment::Nearest => 0.0,
        }
}

fn clamp_point(point: Point, maximum: Point) -> Point {
    Point::new(point.x.clamp(0.0, maximum.x), point.y.clamp(0.0, maximum.y))
}
