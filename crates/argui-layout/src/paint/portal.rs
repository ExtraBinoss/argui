use argui_ui::{Element, NodeId, UiTree, WindowLayer};

use crate::{LayoutOutput, engine::NodeMap};

use super::{PaintCache, PaintContext, ScrollPaintUpdate, paint_node};

pub(super) fn paint(
    root: &NodeMap,
    elements: &[&Element],
    ui: &UiTree,
    output: &mut LayoutOutput,
    context: &PaintContext,
    cache: &mut PaintCache,
    scroll_updates: &mut Vec<ScrollPaintUpdate>,
) {
    output.native_surfaces.clear();
    let mut portals = Vec::new();
    collect(root, elements, &mut portals);
    portals.sort_by_key(|node| {
        let element = elements[node.index];
        (
            element
                .portal
                .as_ref()
                .map_or(WindowLayer::Content, |portal| portal.layer),
            element.z_index,
        )
    });

    let mut painter = PortalPainter {
        elements,
        ui,
        output,
        context,
        cache,
        scroll_updates,
    };
    painter.layer(&portals, WindowLayer::Background);
    painter.node(root, None);
    for layer in [
        WindowLayer::Content,
        WindowLayer::Floating,
        WindowLayer::Popover,
        WindowLayer::Modal,
        WindowLayer::Debug,
    ] {
        painter.layer(&portals, layer);
    }
    for surface in &mut painter.output.native_surfaces {
        surface.localize();
    }
}

struct PortalPainter<'a> {
    elements: &'a [&'a Element],
    ui: &'a UiTree,
    output: &'a mut LayoutOutput,
    context: &'a PaintContext,
    cache: &'a mut PaintCache,
    scroll_updates: &'a mut Vec<ScrollPaintUpdate>,
}

impl PortalPainter<'_> {
    fn layer(&mut self, portals: &[&NodeMap], layer: WindowLayer) {
        for portal in portals {
            if self.elements[portal.index]
                .portal
                .as_ref()
                .is_some_and(|candidate| candidate.layer == layer)
            {
                self.node(portal, Some(portal.node));
            }
        }
    }

    fn node(&mut self, node: &NodeMap, active_portal: Option<NodeId>) {
        if active_portal.is_some()
            && !self
                .output
                .portals
                .iter()
                .any(|portal| portal.node == node.node)
        {
            return;
        }
        let mut context = self.context.clone();
        context.active_portal = active_portal;
        let owner = self.ui.native_portal_owner(node.node);
        let native = owner.and_then(|owner| {
            self.ui
                .native_portal_bounds(owner)
                .map(|bounds| (owner, bounds))
        });
        let main_commands = native.map(|(_, bounds)| {
            context.clip_bounds = bounds;
            context.clips =
                argui_paint::ClipChain::from_regions(vec![argui_paint::ClipRegion::new(
                    bounds,
                    argui_core::Affine2D::IDENTITY,
                )]);
            std::mem::take(&mut self.output.display_list)
        });
        paint_node(
            node,
            self.elements,
            self.ui,
            self.output,
            &context,
            self.cache,
            self.scroll_updates,
        );
        if let Some((owner, bounds)) = native {
            let commands = std::mem::replace(&mut self.output.display_list, main_commands.unwrap());
            if let Some(surface) = self
                .output
                .native_surfaces
                .iter_mut()
                .find(|surface| surface.node == owner)
            {
                surface
                    .display_list
                    .extend(commands.commands().iter().cloned());
            } else {
                self.output.native_surfaces.push(crate::NativeSurfacePaint {
                    node: owner,
                    bounds,
                    display_list: commands,
                });
            }
        }
    }
}

fn collect<'a>(node: &'a NodeMap, elements: &[&Element], portals: &mut Vec<&'a NodeMap>) {
    if node.style.display == argui_ui::Display::None {
        return;
    }
    if elements[node.index].portal.is_some() {
        portals.push(node);
    }
    for child in &node.children {
        collect(child, elements, portals);
    }
}
