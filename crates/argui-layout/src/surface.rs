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
                    }
                    DisplayCommand::EndLayer => {}
                }
                command
            })
            .collect::<Vec<_>>();
        self.display_list.clear();
        self.display_list.extend(commands);
    }
}

impl crate::LayoutOutput {
    /// Resolve an anchored portal against an OS work area, keeping the tree's logical coordinates.
    #[must_use]
    pub fn native_portal_placement(
        &self,
        ui: &argui_ui::UiTree,
        node: NodeId,
        work_area: Rect,
    ) -> Option<Rect> {
        let metadata = self.portals.iter().find(|portal| portal.node == node)?;
        let element = ui.element_for(node)?;
        let (anchor, placement) = match &element.portal.as_ref()?.target {
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
                (bounds, anchor.placement)
            }
            argui_ui::PortalTarget::Rect { bounds, placement } => (*bounds, *placement),
            _ => return None,
        };
        let placed = placement
            .place(
                work_area,
                anchor,
                metadata.desired_size,
                ui.resolved_layout_style(node, element).writing_direction,
            )
            .bounds;
        (placed.size.width > 0.0 && placed.size.height > 0.0).then_some(placed)
    }
}
