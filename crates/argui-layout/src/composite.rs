use std::{collections::HashMap, mem::size_of};

use argui_core::{Affine2D, Point, Rect};
use argui_paint::{ClipChain, CompositorId, CompositorPatch, DisplayCommand, DisplayList};
use argui_ui::{HitRegion, NodeId, ScrollRegion, UiTree};

use crate::{DesktopBackdropRegion, LayoutOutput, TextRegion, engine::LayoutEngine};

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct CompositeGeometry {
    owners: HashMap<NodeId, CompositorId>,
    nodes: HashMap<CompositorId, usize>,
    carets: HashMap<CompositorId, usize>,
    hit_regions: Vec<HitRegion>,
    semantic_bounds: Vec<(NodeId, argui_core::Rect)>,
    text_regions: Vec<TextRegion>,
    scroll_regions: Vec<ScrollRegion>,
    desktop_backdrops: Vec<DesktopBackdropRegion>,
}

impl CompositeGeometry {
    /// Captures the painted presentation baseline and compositor ownership map.
    pub(crate) fn capture(output: &LayoutOutput) -> Self {
        Self {
            owners: output.compositor_owners.clone(),
            nodes: output
                .nodes
                .iter()
                .enumerate()
                .filter_map(|(index, node)| {
                    let id = CompositorId::new(node.node.get());
                    (output.compositor_owners.get(&node.node) == Some(&id)).then_some((id, index))
                })
                .collect(),
            carets: output
                .text_inputs
                .iter()
                .enumerate()
                .filter_map(|(index, region)| {
                    (region.caret.is_some()
                        && region
                            .caret_style
                            .animation
                            .as_ref()
                            .is_some_and(argui_ui::CaretAnimation::supports_composition))
                    .then_some((crate::input::caret_compositor_id(region.node), index))
                })
                .collect(),
            hit_regions: output.hit_regions.clone(),
            semantic_bounds: output.semantic_bounds.clone(),
            text_regions: output.text_regions.clone(),
            scroll_regions: output.scroll_regions.clone(),
            desktop_backdrops: output.desktop_backdrops.clone(),
        }
    }

    /// Returns retained compositor baseline capacity in bytes, excluding shared clip storage.
    fn storage_bytes(&self) -> usize {
        self.owners.capacity() * size_of::<(NodeId, CompositorId)>()
            + self.nodes.capacity() * size_of::<(CompositorId, usize)>()
            + self.carets.capacity() * size_of::<(CompositorId, usize)>()
            + self.hit_regions.capacity() * size_of::<HitRegion>()
            + self.semantic_bounds.capacity() * size_of::<(NodeId, argui_core::Rect)>()
            + self.text_regions.capacity() * size_of::<TextRegion>()
            + self.scroll_regions.capacity() * size_of::<ScrollRegion>()
            + self.desktop_backdrops.capacity() * size_of::<DesktopBackdropRegion>()
    }
}

impl LayoutOutput {
    #[doc(hidden)]
    /// Returns the retained compositor baseline's allocated capacity in bytes.
    #[must_use]
    pub fn compositor_storage_bytes(&self) -> usize {
        self.composite_geometry.storage_bytes()
    }
}

impl LayoutEngine {
    /// Applies transform and group-opacity changes to retained paint output.
    ///
    /// This path does not rebuild layout, paint primitives, or shaped text. It
    /// updates compositor commands and presentation geometry from the baseline
    /// captured by the latest paint pass.
    ///
    /// * `ui` — retained tree containing the current animated presentation values.
    /// * `output` — previously painted output receiving lightweight patches.
    ///
    /// Returns `false` when a retained layer cannot represent the requested
    /// transform, in which case the caller must fall back to a normal repaint.
    pub fn composite(&mut self, ui: &UiTree, output: &mut LayoutOutput) -> bool {
        let (patches, mut deltas) =
            match compositor_patches(ui, output, &output.display_list, output.viewport) {
                Some(value) => value,
                None => return false,
            };
        let mut native_patches = Vec::with_capacity(output.native_surfaces.len());
        for surface in &output.native_surfaces {
            let bounds = Rect::new(Point::new(0.0, 0.0), surface.bounds.size);
            let Some((patches, surface_deltas)) =
                compositor_patches(ui, output, &surface.display_list, bounds)
            else {
                return false;
            };
            let origin = surface.bounds.origin;
            let to_global = Affine2D::translation(origin.x, origin.y);
            let to_surface = Affine2D::translation(-origin.x, -origin.y);
            deltas.extend(
                surface_deltas
                    .into_iter()
                    .map(|(id, delta)| (id, to_global * delta * to_surface)),
            );
            native_patches.push(patches);
        }
        output.display_list.apply_compositor_patches(&patches);
        for (surface, patches) in output.native_surfaces.iter_mut().zip(native_patches) {
            surface.display_list.apply_compositor_patches(&patches);
        }
        apply_geometry(output, &deltas);
        true
    }
}

/// Resolves compositor patches and global presentation deltas for one surface.
fn compositor_patches(
    ui: &UiTree,
    output: &LayoutOutput,
    display_list: &DisplayList,
    surface_bounds: Rect,
) -> Option<(Vec<CompositorPatch>, HashMap<CompositorId, Affine2D>)> {
    let mut patches = Vec::new();
    let mut deltas = HashMap::new();
    let mut stack = Vec::new();
    let mut boundaries = Vec::<(bool, Option<Rect>)>::new();
    for command in display_list.commands() {
        match command {
            DisplayCommand::BeginLayer(style) => boundaries.push((
                false,
                style.requires_offscreen().then(|| style.expanded_bounds()),
            )),
            DisplayCommand::BeginCompositor(layer) => {
                let (current_local, opacity) =
                    if let Some(index) = output.composite_geometry.nodes.get(&layer.id) {
                        let node = output.nodes.get(*index)?;
                        let element = ui.element_at(node.index)?;
                        (
                            ui.resolved_transform(node.node, element)
                                .affine(node.bounds, element.transform_origin),
                            if let Some(style) = &element.layer {
                                ui.resolved_layer(node.node, element, style).opacity
                            } else {
                                ui.resolved_layer(
                                    node.node,
                                    element,
                                    &argui_paint::LayerStyle::new(Default::default()),
                                )
                                .opacity
                            },
                        )
                    } else {
                        let region = output
                            .text_inputs
                            .get(*output.composite_geometry.carets.get(&layer.id)?)?;
                        let bounds = crate::input::visual_bounds(
                            region.caret?,
                            &region.caret_style.visual.primitives,
                        )?;
                        let frame = ui.resolved_caret_frame(region.node, &region.caret_style);
                        (
                            frame
                                .transform
                                .affine(bounds, argui_core::TransformOrigin::CENTER),
                            frame.opacity,
                        )
                    };
                let inverse = layer.base_transform.inverse()?;
                let local_delta = layer.base_parent * current_local * inverse;
                let inverse_delta = local_delta.inverse()?;
                let parent_delta = stack.last().map_or(Affine2D::IDENTITY, |(delta, _)| *delta);
                if local_delta != Affine2D::IDENTITY {
                    let retained_clip = boundaries
                        .iter()
                        .rev()
                        .find_map(|(_, bounds)| *bounds)
                        .map_or(Some(surface_bounds), |bounds| {
                            bounds.intersection(surface_bounds)
                        });
                    if !retained_source_covers(
                        retained_clip,
                        layer.bounds,
                        local_delta,
                        inverse_delta,
                    ) {
                        return None;
                    }
                }
                let global_delta = parent_delta * local_delta;
                patches.push(CompositorPatch::new(layer.id, local_delta, opacity));
                if global_delta != Affine2D::IDENTITY {
                    deltas.insert(layer.id, global_delta);
                }
                stack.push((global_delta, layer.bounds));
                boundaries.push((true, Some(layer.bounds)));
            }
            DisplayCommand::EndLayer => {
                if boundaries.pop()?.0 {
                    return None;
                }
            }
            DisplayCommand::EndCompositor => {
                if !boundaries.pop()?.0 {
                    return None;
                }
                stack.pop()?;
            }
            _ => {}
        }
    }
    (stack.is_empty() && boundaries.is_empty()).then_some((patches, deltas))
}

/// Returns whether the cached source contains every pixel exposed by `delta`.
fn retained_source_covers(
    retained_clip: Option<Rect>,
    source_bounds: Rect,
    delta: Affine2D,
    inverse_delta: Affine2D,
) -> bool {
    let Some(retained_clip) = retained_clip else {
        return true;
    };
    if contains_rect(retained_clip, source_bounds) {
        return true;
    }
    let Some(visible_output) = delta
        .transform_rect(source_bounds)
        .intersection(retained_clip)
    else {
        return true;
    };
    let Some(available_source) = source_bounds.intersection(retained_clip) else {
        return false;
    };
    contains_rect(
        available_source,
        inverse_delta.transform_rect(visible_output),
    )
}

/// Returns whether `outer` fully contains the axis-aligned `inner` rectangle.
fn contains_rect(outer: argui_core::Rect, inner: argui_core::Rect) -> bool {
    const EPSILON: f32 = 0.01;
    inner.origin.x >= outer.origin.x - EPSILON
        && inner.origin.y >= outer.origin.y - EPSILON
        && inner.origin.x + inner.size.width <= outer.origin.x + outer.size.width + EPSILON
        && inner.origin.y + inner.size.height <= outer.origin.y + outer.size.height + EPSILON
}

/// Restores baseline interaction geometry and applies the latest layer deltas.
fn apply_geometry(output: &mut LayoutOutput, deltas: &HashMap<CompositorId, Affine2D>) {
    let base = &output.composite_geometry;
    let owners = &base.owners;
    let mut clip_cache = HashMap::new();
    output.hit_regions.clone_from(&base.hit_regions);
    for region in &mut output.hit_regions {
        if let Some(delta) = delta_for(region.node, owners, deltas) {
            region.transform = delta * region.transform;
        }
        region.clips = patched_clips(&region.clips, deltas, &mut clip_cache);
    }
    output.semantic_bounds.clone_from(&base.semantic_bounds);
    for (node, bounds) in &mut output.semantic_bounds {
        if let Some(delta) = delta_for(*node, owners, deltas) {
            *bounds = delta.transform_rect(*bounds);
        }
    }
    output.text_regions.clone_from(&base.text_regions);
    for region in &mut output.text_regions {
        if let Some(delta) = delta_for(region.node, owners, deltas) {
            region.transform = delta * region.transform;
        }
        region.clips = patched_clips(&region.clips, deltas, &mut clip_cache);
    }
    output.scroll_regions.clone_from(&base.scroll_regions);
    for region in &mut output.scroll_regions {
        if let Some(delta) = delta_for(region.node, owners, deltas) {
            region.transform = delta * region.transform;
        }
        region.clips = patched_clips(&region.clips, deltas, &mut clip_cache);
    }
    output.desktop_backdrops.clone_from(&base.desktop_backdrops);
    for region in &mut output.desktop_backdrops {
        region.shape = patched_clips(&region.shape, deltas, &mut clip_cache);
    }
}

/// Returns the current presentation delta inherited by `node`.
fn delta_for(
    node: NodeId,
    owners: &HashMap<NodeId, CompositorId>,
    deltas: &HashMap<CompositorId, Affine2D>,
) -> Option<Affine2D> {
    owners
        .get(&node)
        .and_then(|owner| deltas.get(owner))
        .copied()
}

/// Returns a clip chain with retained-owner transforms patched once per shared chain.
fn patched_clips(
    clips: &ClipChain,
    deltas: &HashMap<CompositorId, Affine2D>,
    cache: &mut HashMap<(usize, usize), ClipChain>,
) -> ClipChain {
    let key = (clips.regions().as_ptr() as usize, clips.regions().len());
    if let Some(patched) = cache.get(&key) {
        return patched.clone();
    }
    if !clips.regions().iter().any(|clip| {
        clip.compositor
            .and_then(|id| deltas.get(&id))
            .is_some_and(|delta| *delta != Affine2D::IDENTITY)
    }) {
        let unchanged = clips.clone();
        cache.insert(key, unchanged.clone());
        return unchanged;
    }
    let patched = ClipChain::from_regions(
        clips
            .regions()
            .iter()
            .copied()
            .map(|mut clip| {
                if let Some(delta) = clip.compositor.and_then(|id| deltas.get(&id)) {
                    clip.transform = *delta * clip.transform;
                }
                clip
            })
            .collect::<Vec<_>>(),
    );
    cache.insert(key, patched.clone());
    patched
}
