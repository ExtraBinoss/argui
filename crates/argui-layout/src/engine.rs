use crate::layout_tree::LayoutTree;
use crate::{
    LayoutError, TextInputRegion, TextRegion, assets::AssetMetrics, input, paint, scroll,
    style::taffy_style,
};
use argui_core::{Point, Rect, Size};
use argui_paint::{DisplayList, ImageAsset, VectorAsset};
use argui_text::{TextBlock, TextEngine, TextScene};
use argui_ui::{
    Element, ElementKind, HitRegion, LayoutStyle, NodeId as UiNodeId, ScrollRegion, UiTree,
};
use std::collections::HashMap;
use taffy::NodeId;

mod compute;
mod output;

mod storage;
pub use storage::LayoutStorage;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LayoutNode {
    pub index: usize,
    pub node: UiNodeId,
    pub bounds: Rect,
    pub layout_bounds: Rect,
    pub clip: Option<Rect>,
    pub text_index: Option<usize>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LayoutOutput {
    pub viewport: Rect,
    pub nodes: Vec<LayoutNode>,
    pub semantic_bounds: Vec<(UiNodeId, Rect)>,
    pub hit_regions: Vec<HitRegion>,
    pub scroll_regions: Vec<ScrollRegion>,
    pub text_inputs: Vec<TextInputRegion>,
    pub text_regions: Vec<TextRegion>,
    pub display_list: DisplayList,
    pub text: TextScene,
    pub portals: Vec<PortalLayout>,
    pub native_surfaces: Vec<crate::NativeSurfacePaint>,
    pub desktop_backdrops: Vec<crate::DesktopBackdropRegion>,
    pub paint_stats: PaintStats,
    pub virtualization_changed: bool,
    /// Native virtual-list measurement and bounded-window listener deliveries.
    pub virtual_events: Vec<argui_ui::UiEvent>,
    pub(crate) compositor_owners: HashMap<UiNodeId, argui_paint::CompositorId>,
    pub(crate) composite_geometry: crate::composite::CompositeGeometry,
    pub(crate) input_sources: HashMap<UiNodeId, argui_text::TextContent>,
    pub(crate) input_windows: HashMap<UiNodeId, std::ops::Range<f32>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PortalLayout {
    pub node: UiNodeId,
    pub layer: argui_ui::WindowLayer,
    pub anchor: Option<String>,
    pub requested: Option<argui_ui::Placement>,
    pub resolved: Option<argui_ui::Placement>,
    pub bounds: Rect,
    pub desired_size: Size,
    pub available_size: Size,
    pub constrained_width: bool,
    pub constrained_height: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PaintStats {
    pub visited_subtrees: usize,
    pub reused_subtrees: usize,
    pub reused_commands: usize,
}

#[derive(Debug)]
pub(crate) struct NodeMap {
    pub(crate) custom_state: Option<std::rc::Rc<argui_ui::CustomState>>,
    pub(crate) index: usize,
    pub(crate) node: UiNodeId,
    pub(crate) id: NodeId,
    pub(crate) style: LayoutStyle,
    pub(crate) element: Element,
    pub(crate) subtree_len: usize,
    pub(crate) children: Vec<Self>,
}

#[derive(Clone, Copy)]
struct Placement {
    layout_parent: Point,
    translation: Point,
    clip: Option<Rect>,
    sticky_container: Option<Rect>,
}

#[derive(Clone, Copy)]
struct ScrollPlacement {
    translation: Point,
    clip: Option<Rect>,
    sticky_container: Option<Rect>,
}

#[derive(Debug)]
pub struct LayoutEngine {
    tree: LayoutTree,
    root: Option<NodeMap>,
    revision: Option<u64>,
    paint_cache: paint::PaintCache,
    nodes_by_index: Vec<NodeId>,
    assets: AssetMetrics,
    scroll_anchors: Vec<crate::anchor::ScrollAnchor>,
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self {
            tree: LayoutTree::new(),
            root: None,
            revision: None,
            paint_cache: paint::PaintCache::default(),
            nodes_by_index: Vec::new(),
            assets: AssetMetrics::default(),
            scroll_anchors: Vec::new(),
        }
    }
}

impl LayoutEngine {
    /// Samples retained extension counters on demand, with no frame-time traversal.
    pub fn custom_stats(&self) -> Vec<crate::CustomElementStats> {
        crate::custom::stats(self.root.as_ref())
    }

    /// Live retained layout nodes, including nodes currently hidden by display style.
    #[must_use]
    pub fn retained_node_count(&self) -> usize {
        self.tree.len()
    }
    /// Creates an engine with empty retained layout and paint state.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Replaces intrinsic image and vector assets used during layout.
    ///
    /// * `images` — image assets available to image elements.
    /// * `vectors` — vector assets available to vector elements.
    pub fn set_assets(&mut self, images: &[ImageAsset], vectors: &[VectorAsset]) {
        if !self.assets.update(images, vectors) {
            return;
        }
        // Intrinsic dimensions also supply derived styles (such as aspect ratio).
        // Invalidating measurement alone leaves those styles and child caches stale.
        self.root = None;
        self.revision = None;
    }

    fn rebuild(&mut self, ui: &UiTree) -> Result<(), LayoutError> {
        self.tree = LayoutTree::new();
        self.tree
            .reserve_nodes(ui.node_ids().len() + ui.layout_root_indices().len());
        let mut next_index = 0;
        self.root = Some(build_node(
            &mut self.tree,
            &self.assets,
            ui,
            ui.root(),
            &mut next_index,
        )?);
        self.tree.compact();
        self.rebuild_node_index();
        self.revision = Some(ui.revision());
        Ok(())
    }

    fn sync_or_rebuild(&mut self, ui: &UiTree) -> Result<(), LayoutError> {
        self.paint_cache.retain(ui);
        let Some(root) = self.root.take() else {
            return self.rebuild(ui);
        };
        self.tree
            .reserve_nodes(ui.node_ids().len() + ui.layout_root_indices().len());
        self.root = Some(crate::reconcile::sync(
            &mut self.tree,
            root,
            &self.assets,
            ui,
        )?);
        self.tree.compact();
        self.rebuild_node_index();
        self.revision = Some(ui.revision());
        Ok(())
    }

    fn rebuild_node_index(&mut self) {
        self.nodes_by_index.clear();
        if let Some(root) = &self.root {
            collect_node_ids(root, &mut self.nodes_by_index);
        }
    }
}

fn build_node(
    tree: &mut LayoutTree,
    assets: &AssetMetrics,
    ui: &UiTree,
    element: &Element,
    next_index: &mut usize,
) -> Result<NodeMap, LayoutError> {
    let index = *next_index;
    *next_index += 1;
    let node = ui
        .node_id_at(index)
        .ok_or(LayoutError::MissingNodeIdentity(index))?;
    let children = element
        .children
        .iter()
        .map(|child| build_node(tree, assets, ui, child, next_index))
        .collect::<Result<Vec<_>, _>>()?;
    let resolved_style =
        assets.layout_style(ui.resolved_layout_style(node, element), &element.kind);
    let style = taffy_style(&resolved_style);
    let id = match element.kind {
        ElementKind::Text { .. }
        | ElementKind::TextEditor { .. }
        | ElementKind::GpuCanvas(_)
        | ElementKind::Image { .. }
        | ElementKind::Vector { .. } => tree.new_leaf_with_context(style, index)?,
        ElementKind::Custom(_) | ElementKind::Container => {
            let child_ids = crate::overlay::layout_children(&children);
            tree.new_with_children(style, &child_ids, element.layout_boundary)?
        }
    };
    Ok(NodeMap {
        custom_state: crate::custom::install(tree, id, &element.kind, None)?,
        index,
        node,
        id,
        style: resolved_style,
        element: element.clone(),
        subtree_len: 1 + children
            .iter()
            .map(|child| child.subtree_len)
            .sum::<usize>(),
        children,
    })
}

fn collect_node_ids(node: &NodeMap, output: &mut Vec<NodeId>) {
    if output.len() == node.index {
        output.push(node.id);
    } else if let Some(slot) = output.get_mut(node.index) {
        *slot = node.id;
    }
    for child in &node.children {
        collect_node_ids(child, output);
    }
}

fn collect_layout(
    tree: &LayoutTree,
    node: &NodeMap,
    elements: &[&Element],
    ui: &UiTree,
    text_engine: &mut TextEngine,
    placement: Placement,
    output: &mut LayoutOutput,
) -> Result<(), LayoutError> {
    let layout = tree.layout(node.id)?;
    let layout_origin = Point::new(
        placement.layout_parent.x + layout.location.x,
        placement.layout_parent.y + layout.location.y,
    );
    let normal_origin = Point::new(
        layout_origin.x - placement.translation.x,
        layout_origin.y - placement.translation.y,
    );
    let size = Size::new(layout.size.width, layout.size.height);
    let origin = sticky_origin(node, normal_origin, size, placement.sticky_container);
    let sticky_delta = Point::new(origin.x - normal_origin.x, origin.y - normal_origin.y);
    let bounds = Rect::new(origin, size);
    let layout_bounds = Rect::new(
        layout_origin,
        Size::new(layout.size.width, layout.size.height),
    );
    let element = elements[node.index];
    let mut text_index = None;
    let mut text_scroll = None;
    if let Some((content, style)) = crate::text::content(ui, node.node, element) {
        let editor = matches!(element.kind, ElementKind::TextEditor { .. });
        let gutter = if editor && node.style.scrollbar_gutter == argui_ui::ScrollbarGutter::Stable {
            node.style.scrollbar_width.max(0.0)
        } else {
            0.0
        };
        let text_bounds = Rect::new(
            Point::new(
                origin.x + layout.border.left + layout.padding.left,
                origin.y + layout.border.top + layout.padding.top,
            ),
            Size::new(
                (layout.size.width
                    - layout.border.left
                    - layout.border.right
                    - layout.padding.left
                    - layout.padding.right
                    - if node.style.overflow.y.scrolls() {
                        gutter
                    } else {
                        0.0
                    })
                .max(0.0),
                (layout.size.height
                    - layout.border.top
                    - layout.border.bottom
                    - layout.padding.top
                    - layout.padding.bottom
                    - if node.style.overflow.x.scrolls() {
                        gutter
                    } else {
                        0.0
                    })
                .max(0.0),
            ),
        );
        let text_clip = crate::text::clip(node, element, placement.clip, bounds);
        let text_clip = text_clip.unwrap_or_default();
        let mut block = TextBlock::new(content.clone(), text_bounds);
        block.clip = text_clip;
        block.style = style.into_owned();
        if let Some((region, scroll, paint, text_window)) = input::prepare(
            ui,
            node.node,
            element,
            &block.content,
            text_engine,
            input::InputPlacement {
                text: text_bounds,
                hit: bounds,
                clip: text_clip,
                scroll_x: ui.scroll_offset(node.node).x,
                scroll_y: ui.scroll_offset(node.node).y,
            },
        ) {
            input::position_input_block(&mut block, text_bounds, scroll, paint, &content, &region);
            output.input_sources.insert(node.node, content.clone());
            output.input_windows.insert(node.node, text_window);
            text_scroll = Some((region.scroll_content_size(), scroll));
            output.text_inputs.push(region);
        }
        output.text.push(block);
        text_index = Some(output.text.blocks().len() - 1);
        if matches!(element.kind, ElementKind::Text { .. })
            && ui.resolved_user_select(node.node) != argui_ui::UserSelect::None
        {
            output.text_regions.push(TextRegion {
                node: node.node,
                text_index: text_index.unwrap(),
                text_len: output.text.blocks()[text_index.unwrap()]
                    .content
                    .as_str()
                    .len(),
                origin: text_bounds.origin,
                layout: text_engine.layout_text(
                    &output.text.blocks()[text_index.unwrap()].content,
                    &output.text.blocks()[text_index.unwrap()].style,
                    text_bounds.size,
                ),
                transform: argui_core::Affine2D::IDENTITY,
                clips: argui_paint::ClipChain::from_regions([argui_paint::ClipRegion::new(
                    text_clip,
                    argui_core::Affine2D::IDENTITY,
                )]),
                style: ui.resolved_selection_style(node.node),
                highlight: ui.resolved_selection_highlight(node.node),
                interaction_order: 0,
            });
        }
    }
    output.nodes.push(LayoutNode {
        index: node.index,
        node: node.node,
        bounds,
        layout_bounds,
        clip: placement.clip,
        text_index,
    });
    let child_clip = scroll::clipped(node, placement.clip, bounds);
    let scroll_config = scroll::config(node, element);
    if let Some(config) = scroll_config.clone()
        && let Some(region_clip) = placement.clip.and_then(|clip| clip.intersection(bounds))
    {
        let (content, offset) = match text_scroll {
            Some(metrics) => metrics,
            None => (
                scroll::content_size(tree, node)?,
                ui.scroll_offset(node.node),
            ),
        };
        output.scroll_regions.push(scroll::region(
            node.node,
            bounds,
            region_clip,
            content,
            ui.resolved_scroll_config(node.node, &config),
            offset,
        ));
    }
    let scroll = ui.scroll_offset(node.node);
    let child_translation = Point::new(
        placement.translation.x + scroll.x - sticky_delta.x,
        placement.translation.y + scroll.y - sticky_delta.y,
    );
    let sticky_container = scroll_config.map_or(placement.sticky_container, |_| Some(bounds));
    for child in &node.children {
        collect_layout(
            tree,
            child,
            elements,
            ui,
            text_engine,
            Placement {
                layout_parent: layout_origin,
                translation: child_translation,
                clip: child_clip,
                sticky_container,
            },
            output,
        )?;
    }
    Ok(())
}

fn apply_scroll_layout(
    tree: &LayoutTree,
    node: &NodeMap,
    elements: &[&Element],
    ui: &UiTree,
    placement: ScrollPlacement,
    output: &mut LayoutOutput,
) -> Result<(), LayoutError> {
    let layout_bounds = output.nodes[node.index].layout_bounds;
    let previous_bounds = output.nodes[node.index].bounds;
    let normal_origin = Point::new(
        layout_bounds.origin.x - placement.translation.x,
        layout_bounds.origin.y - placement.translation.y,
    );
    let origin = sticky_origin(
        node,
        normal_origin,
        layout_bounds.size,
        placement.sticky_container,
    );
    let sticky_delta = Point::new(origin.x - normal_origin.x, origin.y - normal_origin.y);
    let bounds = Rect::new(origin, layout_bounds.size);
    let element = elements[node.index];
    let text_index = output.nodes[node.index].text_index;
    output.nodes[node.index].bounds = bounds;
    output.nodes[node.index].clip = placement.clip;
    if let Some(text_index) = text_index {
        let delta = Point::new(
            bounds.origin.x - previous_bounds.origin.x,
            bounds.origin.y - previous_bounds.origin.y,
        );
        let text_clip =
            crate::text::clip(node, element, placement.clip, bounds).unwrap_or_default();
        let mut content_delta = Point::default();
        if let Some(region) = output
            .text_inputs
            .iter_mut()
            .find(|region| region.node == node.node)
        {
            region.translate(delta, text_clip);
            if matches!(element.kind, ElementKind::TextEditor { .. }) {
                let offset = ui.scroll_offset(node.node);
                content_delta = Point::new(region.scroll_x - offset.x, region.scroll_y - offset.y);
                region.scroll_to(offset);
            }
        }
        let block = &mut output.text.blocks_mut()[text_index];
        block.bounds.origin.x += delta.x;
        block.bounds.origin.y += delta.y;
        block.bounds.origin.x += content_delta.x;
        block.bounds.origin.y += content_delta.y;
        block.clip = text_clip;
        if let Some(region) = output
            .text_regions
            .iter_mut()
            .find(|region| region.node == node.node)
        {
            region.translate(delta);
        }
    }
    let child_clip = scroll::clipped(node, placement.clip, bounds);
    let scroll_config = scroll::config(node, element);
    if let Some(config) = scroll_config.clone()
        && let Some(region_clip) = placement.clip.and_then(|clip| clip.intersection(bounds))
    {
        let (content, offset) = output
            .text_inputs
            .iter()
            .find(|region| region.node == node.node)
            .map_or_else(
                || {
                    scroll::content_size(tree, node)
                        .map(|content| (content, ui.scroll_offset(node.node)))
                },
                |region| {
                    Ok((
                        region.scroll_content_size(),
                        Point::new(region.scroll_x, region.scroll_y),
                    ))
                },
            )?;
        output.scroll_regions.push(scroll::region(
            node.node,
            bounds,
            region_clip,
            content,
            ui.resolved_scroll_config(node.node, &config),
            offset,
        ));
    }
    let scroll = ui.scroll_offset(node.node);
    let child_translation = Point::new(
        placement.translation.x + scroll.x - sticky_delta.x,
        placement.translation.y + scroll.y - sticky_delta.y,
    );
    let sticky_container = scroll_config.map_or(placement.sticky_container, |_| Some(bounds));
    for child in &node.children {
        apply_scroll_layout(
            tree,
            child,
            elements,
            ui,
            ScrollPlacement {
                translation: child_translation,
                clip: child_clip,
                sticky_container,
            },
            output,
        )?;
    }
    Ok(())
}

fn sticky_origin(node: &NodeMap, origin: Point, size: Size, container: Option<Rect>) -> Point {
    if node.style.position != argui_ui::Position::Sticky {
        return origin;
    }
    let Some(container) = container else {
        return origin;
    };
    let resolve = |value: argui_ui::LengthPercentageAuto, context: f32| {
        value.resolve_to_option(context, |_, _| 0.0)
    };
    let mut result = origin;
    if let Some(top) = resolve(node.style.inset.top, container.size.height) {
        result.y = result.y.max(container.origin.y + top);
    }
    if let Some(bottom) = resolve(node.style.inset.bottom, container.size.height) {
        result.y = result
            .y
            .min(container.origin.y + container.size.height - bottom - size.height);
    }
    if let Some(left) = resolve(node.style.inset.left, container.size.width) {
        result.x = result.x.max(container.origin.x + left);
    }
    if let Some(right) = resolve(node.style.inset.right, container.size.width) {
        result.x = result
            .x
            .min(container.origin.x + container.size.width - right - size.width);
    }
    result
}

pub(crate) fn flattened(root: &Element) -> Vec<&Element> {
    fn visit<'a>(element: &'a Element, output: &mut Vec<&'a Element>) {
        output.push(element);
        for child in &element.children {
            visit(child, output);
        }
    }
    let mut output = Vec::new();
    visit(root, &mut output);
    output
}
