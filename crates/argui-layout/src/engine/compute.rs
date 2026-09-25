use crate::layout_tree::LayoutTree;
#[cfg(feature = "metrics")]
use argui_core::MetricTrace;
use argui_core::{Point, Rect, Size};
use argui_text::TextEngine;
use argui_ui::{ElementKind, TreeUpdate, UiTree};
#[cfg(feature = "metrics")]
use std::time::Duration;
use taffy::{
    AvailableSpace, Dimension, LengthPercentageAuto, NodeId, compute_leaf_layout,
    geometry::Size as TaffySize, tree::LayoutOutput as TaffyLayoutOutput,
};
#[cfg(feature = "metrics")]
use web_time::Instant;

use crate::{
    LayoutError,
    assets::{AssetMetrics, resolve_intrinsic},
    overlay::PortalConstraint,
    style::taffy_style,
};

use super::{LayoutEngine, LayoutOutput, Placement, collect_layout, flattened};

const MAX_CONTAINER_QUERY_PASSES: usize = 4;

/// CPU time spent preparing geometry and producing paint output for one scene.
#[cfg(feature = "metrics")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LayoutProfile {
    /// Time spent reconciling, measuring, and placing the layout tree.
    pub layout: Duration,
    /// Time spent producing paint output or updating its scroll presentation.
    pub paint: Duration,
    /// Number of layout passes required for container query convergence.
    pub passes: usize,
}

impl LayoutEngine {
    /// Computes node geometry, text input regions, hit regions, and paint output.
    ///
    /// * `ui` — retained UI tree to lay out.
    /// * `text_engine` — text shaper used for intrinsic text and input geometry.
    /// * `viewport` — available viewport size in logical pixels.
    ///
    /// # Errors
    ///
    /// Returns layout, custom-element, identity, or Taffy errors; also returns
    /// [`LayoutError::NonConvergentContainerQueries`] if responsive queries do not settle.
    pub fn compute(
        &mut self,
        ui: &mut UiTree,
        text_engine: &mut TextEngine,
        viewport: Size,
    ) -> Result<LayoutOutput, LayoutError> {
        #[cfg(feature = "metrics")]
        {
            self.compute_profiled(ui, text_engine, viewport)
                .map(|(output, _)| output)
        }
        #[cfg(not(feature = "metrics"))]
        {
            let mut history = Vec::new();
            loop {
                let (mut output, query_update, sizes) =
                    self.compute_pass(ui, text_engine, viewport)?;
                if query_update != TreeUpdate::Layout {
                    if query_update == TreeUpdate::Scroll {
                        self.apply_scroll(ui, &mut output)?;
                    } else {
                        self.repaint(ui, &mut output);
                    }
                    let root = self.root.as_ref().ok_or(LayoutError::MissingRoot)?;
                    self.scroll_anchors = crate::anchor::capture(root, &output, ui);
                    ui.mark_layout_clean();
                    return Ok(output);
                }
                if history.contains(&sizes) || history.len() + 1 >= MAX_CONTAINER_QUERY_PASSES {
                    return Err(LayoutError::NonConvergentContainerQueries);
                }
                history.push(sizes);
            }
        }
    }

    /// Computes a scene and returns separate geometry and paint CPU timings.
    /// `ui` is the retained tree, `text_engine` shapes text, and `viewport` is
    /// the available logical size. The timings include all container query passes.
    ///
    /// # Errors
    /// Returns the same layout and container query errors as [`Self::compute`].
    #[cfg(feature = "metrics")]
    pub fn compute_profiled(
        &mut self,
        ui: &mut UiTree,
        text_engine: &mut TextEngine,
        viewport: Size,
    ) -> Result<(LayoutOutput, LayoutProfile), LayoutError> {
        self.compute_inner(ui, text_engine, viewport, None)
    }

    /// Computes a scene while writing nested phase spans to `trace`.
    /// `ui`, `text_engine`, and `viewport` have the same meaning as in
    /// [`Self::compute`]. Returns the scene and top-level timing totals.
    ///
    /// # Errors
    /// Returns the same layout and container query errors as [`Self::compute`].
    #[cfg(feature = "metrics")]
    pub fn compute_traced(
        &mut self,
        ui: &mut UiTree,
        text_engine: &mut TextEngine,
        viewport: Size,
        trace: &MetricTrace,
    ) -> Result<(LayoutOutput, LayoutProfile), LayoutError> {
        self.compute_inner(ui, text_engine, viewport, Some(trace))
    }

    /// Calculates `ui` for `viewport`, using `text_engine`, and optionally
    /// writes nested timing spans to `trace`. Returns scene data and CPU totals.
    ///
    /// # Errors
    /// Returns layout or container query errors from the same path as `compute`.
    #[cfg(feature = "metrics")]
    fn compute_inner(
        &mut self,
        ui: &mut UiTree,
        text_engine: &mut TextEngine,
        viewport: Size,
        trace: Option<&MetricTrace>,
    ) -> Result<(LayoutOutput, LayoutProfile), LayoutError> {
        let _scene = trace.map(|trace| trace.span("scene.compute"));
        let mut history = Vec::new();
        let mut profile = LayoutProfile::default();
        loop {
            let _layout = trace.map(|trace| trace.span("layout.compute"));
            let layout_started = Instant::now();
            let (mut output, query_update, sizes) =
                self.compute_pass(ui, text_engine, viewport, trace)?;
            profile.layout += layout_started.elapsed();
            profile.passes += 1;
            drop(_layout);
            if query_update != TreeUpdate::Layout {
                let _paint = trace.map(|trace| trace.span("paint.generate"));
                let paint_started = Instant::now();
                if query_update == TreeUpdate::Scroll {
                    self.apply_scroll(ui, &mut output)?;
                } else {
                    self.repaint(ui, &mut output);
                }
                profile.paint += paint_started.elapsed();
                drop(_paint);
                let _finalize = trace.map(|trace| trace.span("layout.finalize"));
                let root = self.root.as_ref().ok_or(LayoutError::MissingRoot)?;
                self.scroll_anchors = crate::anchor::capture(root, &output, ui);
                ui.mark_layout_clean();
                return Ok((output, profile));
            }
            if history.contains(&sizes) || history.len() + 1 >= MAX_CONTAINER_QUERY_PASSES {
                return Err(LayoutError::NonConvergentContainerQueries);
            }
            history.push(sizes);
        }
    }

    /// Reconciles and measures one container query pass for `ui` in `viewport`.
    /// `text_engine` supplies text measurements and optional `trace` records
    /// subphases. Returns scene geometry, the query update, and node sizes.
    ///
    /// # Errors
    /// Returns validation, layout, or node identity errors.
    fn compute_pass(
        &mut self,
        ui: &mut UiTree,
        text_engine: &mut TextEngine,
        viewport: Size,
        #[cfg(feature = "metrics")] trace: Option<&MetricTrace>,
    ) -> Result<(LayoutOutput, TreeUpdate, Vec<Size>), LayoutError> {
        if self.revision != Some(ui.revision()) {
            #[cfg(feature = "metrics")]
            let _reconcile = trace.map(|trace| trace.span("layout.reconcile"));
            crate::custom::validate(ui.root())?;
            self.sync_or_rebuild(ui)?;
        }
        for index in ui.layout_animation_indices() {
            let Some(id) = self.nodes_by_index.get(index).copied() else {
                return Err(LayoutError::MissingNodeIdentity(index));
            };
            let Some(element) = ui.element_at(index) else {
                return Err(LayoutError::MissingNodeIdentity(index));
            };
            let node = ui
                .node_id_at(index)
                .ok_or(LayoutError::MissingNodeIdentity(index))?;
            let style = taffy_style(&ui.resolved_layout_style(node, element));
            if self.tree.style(id)? != &style {
                self.tree.set_style(id, style)?;
            }
        }
        let elements = flattened(ui.root());
        let root = self.root.as_ref().ok_or(LayoutError::MissingRoot)?;
        restore_root_styles(
            &mut self.tree,
            &self.assets,
            &elements,
            ui,
            &self.nodes_by_index,
        )?;
        let mut measurement = Measurement {
            assets: &self.assets,
            elements: &elements,
            ui,
            text_engine,
        };
        #[cfg(feature = "metrics")]
        let _measure = trace.map(|trace| trace.span("layout.measure"));
        measurement.compute(&mut self.tree, root.id, viewport, None)?;
        #[cfg(feature = "metrics")]
        drop(_measure);
        let viewport_rect = Rect::new(Point::default(), viewport);
        let mut desired_sizes = Vec::new();
        for &index in ui.layout_root_indices() {
            let element = elements[index];
            let node = self.nodes_by_index[index];
            if crate::overlay::detached(element) {
                #[cfg(feature = "metrics")]
                let _measure = trace.map(|trace| trace.span("layout.measure"));
                measurement.compute(&mut self.tree, node, viewport, None)?;
                let measured = self.tree.layout(node)?.size;
                desired_sizes.push((
                    ui.node_id_at(index),
                    Size::new(measured.width, measured.height),
                ));
                if let Some(constraint) = crate::overlay::constraint(
                    &self.tree,
                    root,
                    &elements,
                    ui,
                    viewport_rect,
                    node,
                )? {
                    let (node, size) = apply_constraint(&mut self.tree, constraint)?;
                    #[cfg(feature = "metrics")]
                    let _measure = trace.map(|trace| trace.span("layout.measure"));
                    measurement.compute(&mut self.tree, node, size, None)?;
                }
            }
            if element.layout_boundary {
                let work = self.tree.prepare_boundary(node)?;
                if work.dirty {
                    #[cfg(feature = "metrics")]
                    let _measure = trace.map(|trace| trace.span("layout.measure"));
                    measurement.compute(&mut self.tree, work.root, work.size, Some(work.input))?;
                } else if work.moved {
                    self.tree.round_root(work.root);
                }
                self.tree.finish_boundary(node)?;
            }
        }

        let mut output = LayoutOutput {
            viewport: viewport_rect,
            ..LayoutOutput::default()
        };
        #[cfg(feature = "metrics")]
        let _placement = trace.map(|trace| trace.span("layout.place"));
        collect_layout(
            &self.tree,
            root,
            &elements,
            ui,
            text_engine,
            Placement {
                layout_parent: Point::default(),
                translation: Point::default(),
                clip: Some(Rect::new(Point::default(), viewport)),
                sticky_container: None,
            },
            &mut output,
        )?;
        crate::overlay::resolve(&self.tree, root, &elements, ui, &mut output)?;
        for portal in &mut output.portals {
            if let Some((_, size)) = desired_sizes
                .iter()
                .find(|(node, _)| *node == Some(portal.node))
            {
                portal.desired_size = *size;
            }
        }
        drop(elements);
        output.virtualization_changed = crate::virtual_list::measure(root, &mut output, ui);
        let anchored = crate::anchor::apply(&self.scroll_anchors, ui, &output);
        for region in &output.text_inputs {
            ui.set_scroll_offset(region.node, Point::new(region.scroll_x, region.scroll_y));
        }
        ui.mark_text_input_layout_clean();
        let mut sizes = vec![Size::default(); output.nodes.len()];
        for node in &output.nodes {
            sizes[node.index] = node.bounds.size;
        }
        let update = if ui.has_container_queries() {
            ui.resolve_container_queries(&sizes)
        } else {
            TreeUpdate::None
        };
        let update = if (anchored || output.virtualization_changed) && update == TreeUpdate::None {
            TreeUpdate::Scroll
        } else {
            update
        };
        Ok((output, update, sizes))
    }
}

fn restore_root_styles(
    tree: &mut LayoutTree,
    assets: &AssetMetrics,
    elements: &[&argui_ui::Element],
    ui: &UiTree,
    nodes: &[NodeId],
) -> Result<(), LayoutError> {
    for &index in ui.layout_root_indices() {
        let element = elements[index];
        let node = ui
            .node_id_at(index)
            .ok_or(LayoutError::MissingNodeIdentity(index))?;
        let style = assets.layout_style(ui.resolved_layout_style(node, element), &element.kind);
        let mut hidden = style.display == taffy::Display::None;
        let mut parent = ui.parent_of(node);
        while let Some(node) = parent {
            if hidden {
                break;
            }
            let element = ui.element_for(node).expect("indexed ancestor");
            hidden = ui.resolved_layout_style(node, element).display == taffy::Display::None;
            parent = ui.parent_of(node);
        }
        if element.portal.is_some() {
            let mut style = taffy_style(&style);
            if hidden {
                style.display = taffy::Display::None;
            }
            tree.set_style(nodes[index], style)?;
        }
        if element.layout_boundary {
            tree.set_boundary_hidden(nodes[index], hidden);
        }
    }
    Ok(())
}

fn apply_constraint(
    tree: &mut LayoutTree,
    constraint: PortalConstraint,
) -> Result<(NodeId, Size), LayoutError> {
    let (node, size) = match constraint {
        PortalConstraint::Bounds {
            node,
            size,
            anchor_width,
        } => {
            let mut style = tree.style(node)?.clone();
            style.max_size = TaffySize {
                width: LengthPercentageAuto::length(size.width),
                height: LengthPercentageAuto::length(size.height),
            };
            match anchor_width {
                argui_ui::AnchorWidth::Content => {}
                argui_ui::AnchorWidth::AtLeastAnchor => {
                    style.min_size.width = LengthPercentageAuto::length(size.width);
                }
                argui_ui::AnchorWidth::MatchAnchor => {
                    style.size.width = Dimension::length(size.width);
                }
            }
            tree.set_style(node, style)?;
            (node, size)
        }
        PortalConstraint::Fill { node, size } => {
            let mut style = tree.style(node)?.clone();
            style.size = TaffySize {
                width: Dimension::length(size.width),
                height: Dimension::length(size.height),
            };
            style.max_size = TaffySize {
                width: LengthPercentageAuto::length(size.width),
                height: LengthPercentageAuto::length(size.height),
            };
            tree.set_style(node, style)?;
            (node, size)
        }
    };
    Ok((node, size))
}

struct Measurement<'a> {
    assets: &'a AssetMetrics,
    elements: &'a [&'a argui_ui::Element],
    ui: &'a UiTree,
    text_engine: &'a mut TextEngine,
}

impl Measurement<'_> {
    fn compute(
        &mut self,
        tree: &mut LayoutTree,
        root: NodeId,
        available: Size,
        boundary_input: Option<taffy::tree::LayoutInput>,
    ) -> Result<(), LayoutError> {
        tree.compute_layout_with_measure(
            root,
            TaffySize {
                width: AvailableSpace::Definite(available.width),
                height: AvailableSpace::Definite(available.height),
            },
            boundary_input,
            |inputs, _, context, style| {
                let index = context;
                let intrinsic =
                    index.and_then(|index| self.assets.intrinsic(&self.elements[index].kind));
                let text = index.and_then(|index| {
                    let node = self.ui.node_id_at(index)?;
                    let (content, text_style) =
                        crate::text::content(self.ui, node, self.elements[index])?;
                    let width = inputs
                        .known_dimensions
                        .width
                        .or_else(|| inputs.available_space.width.into_option());
                    Some(
                        if matches!(self.elements[index].kind, ElementKind::TextEditor { .. }) {
                            self.text_engine
                                .measure_editor_content(&content, &text_style, width)
                        } else {
                            self.text_engine
                                .measure_content(&content, &text_style, width)
                        },
                    )
                });
                if index.is_some_and(|index| {
                    matches!(
                        self.elements[index].kind,
                        ElementKind::Text { .. } | ElementKind::TextEditor { .. }
                    )
                }) {
                    debug_assert!(
                        text.is_some_and(|measurement| measurement.first_baseline.is_some())
                    );
                }
                let baselines = text.map_or(taffy::tree::Baselines::NONE, |measurement| {
                    taffy::tree::Baselines {
                        first: measurement.first_baseline,
                        last: measurement.last_baseline,
                    }
                });
                let size = compute_leaf_layout(
                    inputs,
                    style,
                    |_, _| 0.0,
                    |known, _| {
                        if let Some(intrinsic) = intrinsic {
                            return resolve_intrinsic(known, intrinsic);
                        }
                        let Some(measured) = text else {
                            return TaffySize::ZERO;
                        };
                        TaffySize {
                            width: known.width.unwrap_or(measured.size.width),
                            height: known.height.unwrap_or(measured.size.height),
                        }
                    },
                );
                TaffyLayoutOutput { baselines, ..size }
            },
        )?;
        Ok(())
    }
}
