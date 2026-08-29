use argui_core::{Point, Rect, Size};
use argui_paint::{ClipBehavior, DisplayList};
use argui_text::{TextBlock, TextEngine, TextScene};
use argui_ui::{
    Align, Direction, Edges, Element, ElementKind, HitRegion, Justify, LayoutStyle, Length,
    NodeId as UiNodeId, Position, ScrollRegion, UiTree, Wrap,
};
use taffy::{
    AlignItems, AvailableSpace, Dimension, FlexDirection, FlexWrap, JustifyContent,
    LengthPercentage, LengthPercentageAuto, NodeId, Style, TaffyTree, compute_leaf_layout,
    geometry::{Rect as TaffyRect, Size as TaffySize},
};

use crate::{LayoutError, TextInputRegion, input, paint, scroll};

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
    pub display_list: DisplayList,
    pub text: TextScene,
    pub paint_stats: PaintStats,
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
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self {
            tree: TaffyTree::new(),
            root: None,
            revision: None,
            paint_cache: paint::PaintCache::default(),
            nodes_by_index: Vec::new(),
        }
    }
}

impl LayoutEngine {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn compute(
        &mut self,
        ui: &mut UiTree,
        text_engine: &mut TextEngine,
        viewport: Size,
    ) -> Result<LayoutOutput, LayoutError> {
        if self.revision != Some(ui.revision()) {
            self.sync_or_rebuild(ui)?;
        }
        for index in ui.layout_animation_indices() {
            let Some(id) = self.nodes_by_index.get(*index).copied() else {
                return Err(LayoutError::MissingNodeIdentity(*index));
            };
            let Some(element) = ui.element_at(*index) else {
                return Err(LayoutError::MissingNodeIdentity(*index));
            };
            let style = ui.resolved_layout_style(element);
            self.tree.set_style(id, taffy_style(&style))?;
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
                compute_leaf_layout(
                    inputs,
                    style,
                    |_, _| 0.0,
                    |known, available| {
                        let Some(index) = context.as_deref().copied() else {
                            return TaffySize::ZERO;
                        };
                        let Some(node) = ui.node_id_at(index) else {
                            return TaffySize::ZERO;
                        };
                        let Some((content, style)) =
                            crate::text::content(ui, node, elements[index])
                        else {
                            return TaffySize::ZERO;
                        };
                        let width = known.width.or_else(|| available.width.into_option());
                        let measured = text_engine.measure(&content, style, width);
                        TaffySize {
                            width: known.width.unwrap_or(measured.width),
                            height: known.height.unwrap_or(measured.height),
                        }
                    },
                )
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
        self.repaint(ui, &mut output);
        ui.mark_layout_clean();
        Ok(output)
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
        ui: &UiTree,
        text_engine: &mut TextEngine,
        output: &mut LayoutOutput,
    ) {
        input::update(ui, text_engine, output);
        self.repaint(ui, output);
    }

    fn rebuild(&mut self, ui: &UiTree) -> Result<(), LayoutError> {
        self.tree = TaffyTree::new();
        let mut next_index = 0;
        self.root = Some(build_node(&mut self.tree, ui, ui.root(), &mut next_index)?);
        self.rebuild_node_index();
        self.revision = Some(ui.revision());
        Ok(())
    }

    fn sync_or_rebuild(&mut self, ui: &UiTree) -> Result<(), LayoutError> {
        let Some(root) = self.root.take() else {
            return self.rebuild(ui);
        };
        self.root = Some(crate::reconcile::sync(&mut self.tree, root, ui)?);
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
        .map(|child| build_node(tree, ui, child, next_index))
        .collect::<Result<Vec<_>, _>>()?;
    let resolved_style = ui.resolved_layout_style(element);
    let style = taffy_style(&resolved_style);
    let id = match element.kind {
        ElementKind::Text { .. }
        | ElementKind::TextInput { .. }
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
    if let Some((content, style)) = crate::text::content(ui, node.node, element) {
        let text_bounds = Rect::new(
            Point::new(
                origin.x + layout.padding.left,
                origin.y + layout.padding.top,
            ),
            Size::new(
                (layout.size.width - layout.padding.left - layout.padding.right).max(0.0),
                (layout.size.height - layout.padding.top - layout.padding.bottom).max(0.0),
            ),
        );
        let text_clip = match element.paint.clip {
            ClipBehavior::None => placement.clip,
            ClipBehavior::Bounds => placement.clip.and_then(|clip| clip.intersection(bounds)),
        };
        let text_clip = text_clip.unwrap_or_default();
        let mut block = TextBlock::new(content, text_bounds);
        block.clip = text_clip;
        block.style = style.clone();
        if let Some((region, scroll_x)) = input::prepare(
            ui,
            node.node,
            element,
            text_engine,
            input::InputPlacement {
                text: text_bounds,
                hit: bounds,
                clip: text_clip,
                scroll_x: 0.0,
            },
        ) {
            block.bounds.origin.x -= scroll_x;
            output.text_inputs.push(region);
        }
        output.text.push(block);
        text_index = Some(output.text.blocks().len() - 1);
    }
    output.nodes.push(LayoutNode {
        index: node.index,
        node: node.node,
        bounds,
        layout_bounds,
        clip: placement.clip,
        text_index,
    });
    let child_clip = match element.paint.clip {
        ClipBehavior::None => placement.clip,
        ClipBehavior::Bounds => placement.clip.and_then(|clip| clip.intersection(bounds)),
    };
    if let Some(config) = element.scroll.clone()
        && let Some(region_clip) = placement.clip.and_then(|clip| clip.intersection(bounds))
    {
        let content = content_size(tree, node)?;
        output.scroll_regions.push(scroll::region(
            node.node,
            bounds,
            region_clip,
            content,
            config,
            ui.scroll_offset(node.node),
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
        let block = &mut output.text.blocks_mut()[text_index];
        let delta = Point::new(
            bounds.origin.x - previous_bounds.origin.x,
            bounds.origin.y - previous_bounds.origin.y,
        );
        block.bounds.origin.x += delta.x;
        block.bounds.origin.y += delta.y;
        block.clip = match element.paint.clip {
            ClipBehavior::None => clip,
            ClipBehavior::Bounds => clip.and_then(|clip| clip.intersection(bounds)),
        }
        .unwrap_or_default();
        if let Some(region) = output
            .text_inputs
            .iter_mut()
            .find(|region| region.node == node.node)
        {
            region.translate(delta, block.clip);
        }
    }
    let child_clip = match element.paint.clip {
        ClipBehavior::None => clip,
        ClipBehavior::Bounds => clip.and_then(|clip| clip.intersection(bounds)),
    };
    if let Some(config) = element.scroll.clone()
        && let Some(region_clip) = clip.and_then(|clip| clip.intersection(bounds))
    {
        let content = content_size(tree, node)?;
        output.scroll_regions.push(scroll::region(
            node.node,
            bounds,
            region_clip,
            content,
            config,
            ui.scroll_offset(node.node),
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

pub(crate) fn taffy_style(style: &LayoutStyle) -> Style {
    Style {
        size: TaffySize {
            width: dimension(style.width),
            height: dimension(style.height),
        },
        min_size: TaffySize {
            width: min_max_dimension(style.min_width),
            height: min_max_dimension(style.min_height),
        },
        max_size: TaffySize {
            width: min_max_dimension(style.max_width),
            height: min_max_dimension(style.max_height),
        },
        flex_direction: match style.direction {
            Direction::Row => FlexDirection::Row,
            Direction::Column => FlexDirection::Column,
        },
        flex_wrap: match style.wrap {
            Wrap::NoWrap => FlexWrap::NoWrap,
            Wrap::Wrap => FlexWrap::Wrap,
            Wrap::Reverse => FlexWrap::WrapReverse,
        },
        align_items: Some(match style.align {
            Align::Start => AlignItems::START,
            Align::Center => AlignItems::CENTER,
            Align::End => AlignItems::END,
            Align::Stretch => AlignItems::STRETCH,
        }),
        justify_content: Some(match style.justify {
            Justify::Start => JustifyContent::START,
            Justify::Center => JustifyContent::CENTER,
            Justify::End => JustifyContent::END,
            Justify::SpaceBetween => JustifyContent::SPACE_BETWEEN,
        }),
        position: match style.position {
            Position::Relative => taffy::Position::Relative,
            Position::Absolute => taffy::Position::Absolute,
        },
        inset: TaffyRect {
            left: min_max_dimension(style.inset.left),
            right: min_max_dimension(style.inset.right),
            top: min_max_dimension(style.inset.top),
            bottom: min_max_dimension(style.inset.bottom),
        },
        padding: padding(style.padding),
        gap: TaffySize {
            width: LengthPercentage::length(style.gap),
            height: LengthPercentage::length(style.gap),
        },
        flex_grow: style.grow,
        flex_shrink: style.shrink,
        ..Style::default()
    }
}

const fn dimension(value: Length) -> Dimension {
    match value {
        Length::Auto => Dimension::auto(),
        Length::Px(value) => Dimension::length(value),
        Length::Percent(value) => Dimension::percent(value),
    }
}

const fn min_max_dimension(value: Length) -> LengthPercentageAuto {
    match value {
        Length::Auto => LengthPercentageAuto::auto(),
        Length::Px(value) => LengthPercentageAuto::length(value),
        Length::Percent(value) => LengthPercentageAuto::percent(value),
    }
}

const fn padding(edges: Edges) -> TaffyRect<LengthPercentage> {
    TaffyRect {
        left: LengthPercentage::length(edges.left),
        right: LengthPercentage::length(edges.right),
        top: LengthPercentage::length(edges.top),
        bottom: LengthPercentage::length(edges.bottom),
    }
}

#[cfg(test)]
mod tests;
