use argui_core::{Point, Rect, Size};
use argui_text::TextEngine;
use argui_ui::{ElementKind, TreeUpdate, UiTree};
use taffy::{
    AvailableSpace, compute_leaf_layout, geometry::Size as TaffySize,
    tree::LayoutOutput as TaffyLayoutOutput,
};

use crate::{LayoutError, assets::resolve_intrinsic, style::taffy_style};

use super::{LayoutEngine, LayoutOutput, Placement, collect_layout, flattened};

const MAX_CONTAINER_QUERY_PASSES: usize = 4;

impl LayoutEngine {
    pub fn compute(
        &mut self,
        ui: &mut UiTree,
        text_engine: &mut TextEngine,
        viewport: Size,
    ) -> Result<LayoutOutput, LayoutError> {
        let mut history = Vec::new();
        loop {
            let (mut output, query_update, sizes) = self.compute_pass(ui, text_engine, viewport)?;
            if query_update != TreeUpdate::Layout {
                if query_update == TreeUpdate::Scroll {
                    self.apply_scroll(ui, &mut output)?;
                } else {
                    self.repaint(ui, &mut output);
                }
                ui.mark_layout_clean();
                return Ok(output);
            }
            if history.contains(&sizes) || history.len() + 1 >= MAX_CONTAINER_QUERY_PASSES {
                return Err(LayoutError::NonConvergentContainerQueries);
            }
            history.push(sizes);
        }
    }

    fn compute_pass(
        &mut self,
        ui: &mut UiTree,
        text_engine: &mut TextEngine,
        viewport: Size,
    ) -> Result<(LayoutOutput, TreeUpdate, Vec<Size>), LayoutError> {
        if self.revision != Some(ui.revision()) {
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
            self.tree
                .set_style(id, taffy_style(&ui.resolved_layout_style(node, element)))?;
        }
        let elements = flattened(ui.root());
        let root = self.root.as_ref().ok_or(LayoutError::MissingRoot)?;
        self.tree.compute_layout_with_measure(
            root.id,
            TaffySize {
                width: AvailableSpace::Definite(viewport.width),
                height: AvailableSpace::Definite(viewport.height),
            },
            |inputs, _, context, style| {
                let index = context.as_deref().copied();
                let intrinsic =
                    index.and_then(|index| self.assets.intrinsic(&elements[index].kind));
                let text = index.and_then(|index| {
                    let node = ui.node_id_at(index)?;
                    let (content, text_style) = crate::text::content(ui, node, elements[index])?;
                    let width = inputs
                        .known_dimensions
                        .width
                        .or_else(|| inputs.available_space.width.into_option());
                    Some(text_engine.measure_content(&content, &text_style, width))
                });
                if index.is_some_and(|index| {
                    matches!(
                        elements[index].kind,
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
                    |known, available| {
                        if let Some(intrinsic) = intrinsic {
                            return resolve_intrinsic(known, intrinsic);
                        }
                        let Some(measured) = text else {
                            return TaffySize::ZERO;
                        };
                        let _ = available;
                        TaffySize {
                            width: known.width.unwrap_or(measured.size.width),
                            height: known.height.unwrap_or(measured.size.height),
                        }
                    },
                );
                TaffyLayoutOutput { baselines, ..size }
            },
        )?;

        let mut output = LayoutOutput {
            viewport: Rect::new(Point::default(), viewport),
            ..LayoutOutput::default()
        };
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
            },
            &mut output,
        )?;
        crate::overlay::resolve(&self.tree, root, &elements, ui, &mut output)?;
        drop(elements);
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
        Ok((output, update, sizes))
    }
}
