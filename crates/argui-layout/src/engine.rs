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
use taffy::{NodeId, TaffyTree};

mod compute;

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
    pub hit_regions: Vec<HitRegion>,
    pub scroll_regions: Vec<ScrollRegion>,
    pub text_inputs: Vec<TextInputRegion>,
    pub text_regions: Vec<TextRegion>,
    pub display_list: DisplayList,
    pub text: TextScene,
    pub portals: Vec<PortalLayout>,
    pub paint_stats: PaintStats,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PortalLayout {
    pub node: UiNodeId,
    pub layer: argui_ui::WindowLayer,
    pub anchor: Option<String>,
    pub requested: Option<argui_ui::Placement>,
    pub resolved: Option<argui_ui::Placement>,
    pub bounds: Rect,
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
    pub(crate) index: usize,
    pub(crate) node: UiNodeId,
    pub(crate) id: NodeId,
    pub(crate) kind: ElementKind,
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
}

#[derive(Debug)]
pub struct LayoutEngine {
    tree: TaffyTree<usize>,
    root: Option<NodeMap>,
    revision: Option<u64>,
    paint_cache: paint::PaintCache,
    nodes_by_index: Vec<NodeId>,
    assets: AssetMetrics,
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self {
            tree: TaffyTree::new(),
            root: None,
            revision: None,
            paint_cache: paint::PaintCache::default(),
            nodes_by_index: Vec::new(),
            assets: AssetMetrics::default(),
        }
    }
}

impl LayoutEngine {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_assets(&mut self, images: &[ImageAsset], vectors: &[VectorAsset]) {
        if !self.assets.update(images, vectors) {
            return;
        }
        if let Some(root) = &self.root {
            self.tree
                .mark_dirty(root.id)
                .expect("the retained layout root belongs to its Taffy tree");
        }
    }

    pub fn apply_scroll(
        &mut self,
        ui: &UiTree,
        output: &mut LayoutOutput,
    ) -> Result<(), LayoutError> {
        let elements = flattened(ui.root());
        let root = self.root.as_ref().ok_or(LayoutError::MissingRoot)?;
        output.scroll_regions.clear();
        apply_scroll_layout(
            &self.tree,
            root,
            &elements,
            ui,
            Point::default(),
            Some(output.viewport),
            output,
        )?;
        crate::overlay::resolve(&self.tree, root, &elements, ui, output)?;
        self.repaint(ui, output);
        Ok(())
    }

    pub fn repaint(&mut self, ui: &UiTree, output: &mut LayoutOutput) {
        paint::repaint(self.root.as_ref(), ui, output, &mut self.paint_cache);
    }

    pub fn update_text_inputs(
        &mut self,
        ui: &mut UiTree,
        text_engine: &mut TextEngine,
        output: &mut LayoutOutput,
    ) {
        input::update(ui, text_engine, output);
        self.repaint(ui, output);
    }

    fn rebuild(&mut self, ui: &UiTree) -> Result<(), LayoutError> {
        self.tree = TaffyTree::new();
        let mut next_index = 0;
        self.root = Some(build_node(
            &mut self.tree,
            &self.assets,
            ui,
            ui.root(),
            &mut next_index,
        )?);
        self.rebuild_node_index();
        self.revision = Some(ui.revision());
        Ok(())
    }

    fn sync_or_rebuild(&mut self, ui: &UiTree) -> Result<(), LayoutError> {
        let Some(root) = self.root.take() else {
            return self.rebuild(ui);
        };
        self.root = Some(crate::reconcile::sync(
            &mut self.tree,
            root,
            &self.assets,
            ui,
        )?);
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
    tree: &mut TaffyTree<usize>,
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
        | ElementKind::Image { .. }
        | ElementKind::Vector { .. } => tree.new_leaf_with_context(style, index)?,
        ElementKind::Container => {
            let child_ids = children.iter().map(|child| child.id).collect::<Vec<_>>();
            tree.new_with_children(style, &child_ids)?
        }
    };
    Ok(NodeMap {
        index,
        node,
        id,
        kind: element.kind.clone(),
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
    tree: &TaffyTree<usize>,
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
    let origin = Point::new(
        layout_origin.x - placement.translation.x,
        layout_origin.y - placement.translation.y,
    );
    let bounds = Rect::new(origin, Size::new(layout.size.width, layout.size.height));
    let layout_bounds = Rect::new(
        layout_origin,
        Size::new(layout.size.width, layout.size.height),
    );
    let element = elements[node.index];
    let mut text_index = None;
    let mut text_scroll = None;
    if let Some((content, style)) = crate::text::content(ui, node.node, element) {
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
                    - layout.padding.right)
                    .max(0.0),
                (layout.size.height
                    - layout.border.top
                    - layout.border.bottom
                    - layout.padding.top
                    - layout.padding.bottom)
                    .max(0.0),
            ),
        );
        let text_clip = crate::text::clip(node, element, placement.clip, bounds);
        let text_clip = text_clip.unwrap_or_default();
        let mut block = TextBlock::new(content, text_bounds);
        block.clip = text_clip;
        block.style = style.into_owned();
        if let Some((region, scroll)) = input::prepare(
            ui,
            node.node,
            element,
            text_engine,
            input::InputPlacement {
                text: text_bounds,
                hit: bounds,
                clip: text_clip,
                scroll_x: ui.scroll_offset(node.node).x,
                scroll_y: ui.scroll_offset(node.node).y,
            },
        ) {
            block.bounds.origin.x -= scroll.x;
            block.bounds.origin.y -= scroll.y;
            block.bounds.size.height = block.bounds.size.height.max(region.content_size.height);
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
    if let Some(config) = scroll::config(node, element)
        && let Some(region_clip) = placement.clip.and_then(|clip| clip.intersection(bounds))
    {
        let (content, offset) = match text_scroll {
            Some(metrics) => metrics,
            None => (content_size(tree, node)?, ui.scroll_offset(node.node)),
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
        placement.translation.x + scroll.x,
        placement.translation.y + scroll.y,
    );
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
            },
            output,
        )?;
    }
    Ok(())
}

fn apply_scroll_layout(
    tree: &TaffyTree<usize>,
    node: &NodeMap,
    elements: &[&Element],
    ui: &UiTree,
    translation: Point,
    clip: Option<Rect>,
    output: &mut LayoutOutput,
) -> Result<(), LayoutError> {
    let layout_bounds = output.nodes[node.index].layout_bounds;
    let previous_bounds = output.nodes[node.index].bounds;
    let bounds = Rect::new(
        Point::new(
            layout_bounds.origin.x - translation.x,
            layout_bounds.origin.y - translation.y,
        ),
        layout_bounds.size,
    );
    let element = elements[node.index];
    let text_index = output.nodes[node.index].text_index;
    output.nodes[node.index].bounds = bounds;
    output.nodes[node.index].clip = clip;
    if let Some(text_index) = text_index {
        let delta = Point::new(
            bounds.origin.x - previous_bounds.origin.x,
            bounds.origin.y - previous_bounds.origin.y,
        );
        let text_clip = crate::text::clip(node, element, clip, bounds).unwrap_or_default();
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
    let child_clip = scroll::clipped(node, clip, bounds);
    if let Some(config) = scroll::config(node, element)
        && let Some(region_clip) = clip.and_then(|clip| clip.intersection(bounds))
    {
        let (content, offset) = output
            .text_inputs
            .iter()
            .find(|region| region.node == node.node)
            .map_or_else(
                || content_size(tree, node).map(|content| (content, ui.scroll_offset(node.node))),
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
    let child_translation = Point::new(translation.x + scroll.x, translation.y + scroll.y);
    for child in &node.children {
        apply_scroll_layout(
            tree,
            child,
            elements,
            ui,
            child_translation,
            child_clip,
            output,
        )?;
    }
    Ok(())
}

pub(crate) fn content_size(tree: &TaffyTree<usize>, node: &NodeMap) -> Result<Size, LayoutError> {
    let layout = tree.layout(node.id)?;
    let mut size = Size::new(layout.size.width, layout.size.height);
    for child in &node.children {
        let child_layout = tree.layout(child.id)?;
        size.width = size
            .width
            .max(child_layout.location.x + child_layout.size.width + layout.padding.right);
        size.height = size
            .height
            .max(child_layout.location.y + child_layout.size.height + layout.padding.bottom);
    }
    Ok(size)
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

#[cfg(test)]
mod tests;
