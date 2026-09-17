use argui_paint::{GpuCanvasPrimitive, ProfileDomain, RenderObjectId};
use argui_ui::{Element, ElementKind, UiTree};

use crate::{LayoutNode, LayoutOutput};

use super::PaintContext;

/// Lowers one retained GPU-canvas leaf into the display list using `context` geometry.
pub(super) fn push(
    ui: &UiTree,
    element: &Element,
    node: LayoutNode,
    output: &mut LayoutOutput,
    context: &PaintContext,
) {
    let ElementKind::GpuCanvas(spec) = element.kind else {
        return;
    };
    let style = ui.resolved_quad(node.node, element);
    output.display_list.push_gpu_canvas(GpuCanvasPrimitive {
        canvas: spec.canvas(),
        object: RenderObjectId::new(ProfileDomain::Ui, node.node.get()),
        slot: 0,
        bounds: node.bounds,
        content_revision: spec.revision(),
        resolution_scale: spec.scale(),
        sampling: spec.image_sampling(),
        opacity: style.opacity,
        radii: style.radii,
        transform: context.transform,
        clips: context.clips.clone(),
    });
}
