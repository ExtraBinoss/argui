use argui_core::{Point, Rect};
use argui_paint::{ClipChain, DisplayCommand, DisplayList};
use argui_ui::NodeId;

/// A physical popup's commands, sharing text indices and logical node identities with its window.
#[derive(Clone, Debug, PartialEq)]
pub struct NativeSurfacePaint {
    pub node: NodeId,
    pub bounds: Rect,
    pub display_list: DisplayList,
}

impl NativeSurfacePaint {
    /// Translate drawing and clips together. Text glyphs can remain in the original scene.
    pub(crate) fn localize(&mut self) {
        let delta = Point::new(-self.bounds.origin.x, -self.bounds.origin.y);
        let translate = |transform: &mut argui_core::Affine2D| {
            transform.translation.x += delta.x;
            transform.translation.y += delta.y;
        };
        let clips = |clips: &mut ClipChain| {
            *clips = ClipChain::from_regions(
                clips
                    .regions()
                    .iter()
                    .map(|clip| {
                        let mut clip = *clip;
                        translate(&mut clip.transform);
                        clip
                    })
                    .collect::<Vec<_>>(),
            );
        };
        let commands = self
            .display_list
            .commands()
            .iter()
            .cloned()
            .map(|mut command| {
                match &mut command {
                    DisplayCommand::Quad(item) => {
                        translate(&mut item.transform);
                        clips(&mut item.clips);
                    }
                    DisplayCommand::Image(item) => {
                        translate(&mut item.transform);
                        clips(&mut item.clips);
                    }
                    DisplayCommand::GpuCanvas(item) => {
                        translate(&mut item.transform);
                        clips(&mut item.clips);
                    }
                    DisplayCommand::Vector(item) => {
                        translate(&mut item.transform);
                        clips(&mut item.clips);
                    }
                    DisplayCommand::Text {
                        transform,
                        clips: chain,
                        ..
                    } => {
                        translate(transform);
                        clips(chain);
                    }
                    DisplayCommand::BeginLayer(layer) => {
                        layer.bounds.origin.x += delta.x;
                        layer.bounds.origin.y += delta.y;
                        if let Some(clip) = &mut layer.clip {
                            clip.origin.x += delta.x;
                            clip.origin.y += delta.y;
                        }
                    }
                    DisplayCommand::BeginCompositor(layer) => {
                        layer.bounds.origin.x += delta.x;
                        layer.bounds.origin.y += delta.y;
                        translate(&mut layer.base_parent);
                        translate(&mut layer.base_transform);
                    }
                    DisplayCommand::EndLayer | DisplayCommand::EndCompositor => {}
                }
                command
            })
            .collect::<Vec<_>>();
        self.display_list.clear();
        self.display_list.extend(commands);
    }
}

impl crate::LayoutOutput {
    /// Resolve a portal for a native surface, keeping the tree's logical coordinates.
    ///
    /// * `ui` — retained UI tree containing the portal and anchor.
    /// * `node` — portal node to place.
    /// * `work_area` — available logical bounds of the native surface.
    ///
    /// Returns `None` when portal metadata, anchor geometry, or positive-size placement
    /// is unavailable.
    #[must_use]
    pub fn native_portal_placement(
        &self,
        ui: &argui_ui::UiTree,
        node: NodeId,
        work_area: Rect,
    ) -> Option<Rect> {
        let metadata = self.portals.iter().find(|portal| portal.node == node)?;
        let element = ui.element_for(node)?;
        let placed = match &element.portal.as_ref()?.target {
            argui_ui::PortalTarget::Anchor(anchor) => {
                let anchor_node = ui
                    .node_ids()
                    .iter()
                    .copied()
                    .find(|id| ui.key(*id) == Some(anchor.key.as_str()))?;
                let bounds = self
                    .nodes
                    .iter()
                    .find(|node| node.node == anchor_node)?
                    .bounds;
                anchor
                    .placement
                    .place(
                        work_area,
                        bounds,
                        metadata.desired_size,
                        ui.resolved_layout_style(node, element).writing_direction,
                    )
                    .bounds
            }
            argui_ui::PortalTarget::Rect { bounds, placement } => {
                placement
                    .place(
                        work_area,
                        *bounds,
                        metadata.desired_size,
                        ui.resolved_layout_style(node, element).writing_direction,
                    )
                    .bounds
            }
            argui_ui::PortalTarget::Viewport(placement) => {
                placement.place(self.viewport, metadata.desired_size)
            }
            _ => return None,
        };
        (placed.size.width > 0.0 && placed.size.height > 0.0).then_some(placed)
    }
}
