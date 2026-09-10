use crate::layout_tree::LayoutTree;
use argui_core::{Point, Rect, Size};
use argui_text::TextEngine;
use argui_ui::{ElementKind, TreeUpdate, UiTree};
use taffy::{
    AvailableSpace, Dimension, LengthPercentageAuto, NodeId, compute_leaf_layout,
    geometry::Size as TaffySize, tree::LayoutOutput as TaffyLayoutOutput,
};

use crate::{
    LayoutError,
    assets::{AssetMetrics, resolve_intrinsic},
    overlay::PortalConstraint,
    style::taffy_style,
};

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
                let root = self.root.as_ref().ok_or(LayoutError::MissingRoot)?;
                self.scroll_anchors = crate::anchor::capture(root, &output);
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
        restore_portal_styles(&mut self.tree, &self.assets, &elements, ui, root, false)?;
        compute_taffy(
            &mut self.tree,
            &self.assets,
            root.id,
            viewport,
            &elements,
            ui,
            text_engine,
        )?;
        let viewport_rect = Rect::new(Point::default(), viewport);
        for (index, element) in elements.iter().enumerate() {
            if !crate::overlay::detached(element) {
                continue;
            }
            let node = self.nodes_by_index[index];
            compute_taffy(
                &mut self.tree,
                &self.assets,
                node,
                viewport,
                &elements,
                ui,
                text_engine,
            )?;
            let Some(constraint) =
                crate::overlay::constraint(&self.tree, root, &elements, ui, viewport_rect, node)?
            else {
                continue;
            };
            let (node, size) = apply_constraint(&mut self.tree, constraint)?;
            compute_taffy(
                &mut self.tree,
                &self.assets,
                node,
                size,
                &elements,
                ui,
                text_engine,
            )?;
        }

        let mut output = LayoutOutput {
            viewport: viewport_rect,
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
                sticky_container: None,
            },
            &mut output,
        )?;
        crate::overlay::resolve(&self.tree, root, &elements, ui, &mut output)?;
        drop(elements);
        output.virtualization_changed = crate::virtual_list::measure(root, &output, ui);
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

fn restore_portal_styles(
    tree: &mut LayoutTree,
    assets: &AssetMetrics,
    elements: &[&argui_ui::Element],
    ui: &UiTree,
    map: &super::NodeMap,
    parent_hidden: bool,
) -> Result<(), LayoutError> {
    let element = elements[map.index];
    if element.portal.is_some() {
        let style = assets.layout_style(ui.resolved_layout_style(map.node, element), &element.kind);
        let mut style = taffy_style(&style);
        if parent_hidden {
            style.display = taffy::Display::None;
        }
        if tree.style(map.id)? != &style {
            tree.set_style(map.id, style)?;
        }
    }
    let hidden = parent_hidden || tree.style(map.id)?.display == taffy::Display::None;
    for child in &map.children {
        restore_portal_styles(tree, assets, elements, ui, child, hidden)?;
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

fn compute_taffy(
    tree: &mut LayoutTree,
    assets: &AssetMetrics,
    root: NodeId,
    available: Size,
    elements: &[&argui_ui::Element],
    ui: &UiTree,
    text_engine: &mut TextEngine,
) -> Result<(), LayoutError> {
    tree.compute_layout_with_measure(
        root,
        TaffySize {
            width: AvailableSpace::Definite(available.width),
            height: AvailableSpace::Definite(available.height),
        },
        |inputs, _, context, style| {
            let index = context.as_deref().copied();
            let intrinsic = index.and_then(|index| assets.intrinsic(&elements[index].kind));
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
                debug_assert!(text.is_some_and(|measurement| measurement.first_baseline.is_some()));
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
