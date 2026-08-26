use argui_core::{Point, Rect, Size};
use argui_paint::{Border, ClipBehavior, Color, DisplayList, Fill, Quad};
use argui_text::{TextBlock, TextEngine, TextScene};
use argui_ui::{
    Align, Direction, Edges, Element, ElementKind, HitRegion, Justify, LayoutStyle, Length,
    NodeId as UiNodeId, UiTree, Wrap,
};
use taffy::{
    AlignItems, AvailableSpace, Dimension, FlexDirection, FlexWrap, JustifyContent,
    LengthPercentage, LengthPercentageAuto, NodeId, Style, TaffyTree, compute_leaf_layout,
    geometry::{Rect as TaffyRect, Size as TaffySize},
};

use crate::LayoutError;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LayoutNode {
    pub index: usize,
    pub node: UiNodeId,
    pub bounds: Rect,
    pub clip: Option<Rect>,
    pub text_index: Option<usize>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LayoutOutput {
    pub nodes: Vec<LayoutNode>,
    pub hit_regions: Vec<HitRegion>,
    pub display_list: DisplayList,
    pub text: TextScene,
}

#[derive(Debug)]
struct NodeMap {
    index: usize,
    node: UiNodeId,
    id: NodeId,
    children: Vec<Self>,
}

#[derive(Debug)]
pub struct LayoutEngine {
    tree: TaffyTree<usize>,
    root: Option<NodeMap>,
    revision: Option<u64>,
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self {
            tree: TaffyTree::new(),
            root: None,
            revision: None,
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
            self.rebuild(ui)?;
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
                        let ElementKind::Text { content, style } = &elements[index].kind else {
                            return TaffySize::ZERO;
                        };
                        let width = known.width.or_else(|| available.width.into_option());
                        let measured = text_engine.measure(content, style, width);
                        TaffySize {
                            width: known.width.unwrap_or(measured.width),
                            height: known.height.unwrap_or(measured.height),
                        }
                    },
                )
            },
        )?;

        let mut output = LayoutOutput::default();
        collect_layout(
            &self.tree,
            root,
            &elements,
            Point::default(),
            Some(Rect::new(Point::default(), viewport)),
            &mut output,
        )?;
        self.repaint(ui, &mut output);
        ui.mark_layout_clean();
        Ok(output)
    }

    pub fn repaint(&self, ui: &UiTree, output: &mut LayoutOutput) {
        let elements = flattened(ui.root());
        output.display_list.clear();
        for node in &output.nodes {
            let element = elements[node.index];
            let style = ui.resolved_quad(node.node, element);
            if style.is_visible()
                && let Some(clip) = node.clip.and_then(|clip| clip.intersection(node.bounds))
            {
                output.display_list.push_quad(Quad {
                    bounds: node.bounds,
                    background: match style.background {
                        Some(Fill::Solid(color)) => color,
                        None => Color::TRANSPARENT,
                    },
                    border: style.border.unwrap_or(Border::all(0.0, Color::TRANSPARENT)),
                    radii: style.radii,
                    opacity: style.opacity,
                    clip,
                });
            }
            if let Some(text_index) = node.text_index {
                output.display_list.push_text(text_index);
            }
        }
    }

    fn rebuild(&mut self, ui: &UiTree) -> Result<(), LayoutError> {
        self.tree = TaffyTree::new();
        let mut next_index = 0;
        self.root = Some(build_node(&mut self.tree, ui, ui.root(), &mut next_index)?);
        self.revision = Some(ui.revision());
        Ok(())
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
    let style = taffy_style(&element.style);
    let id = match element.kind {
        ElementKind::Text { .. } => tree.new_leaf_with_context(style, index)?,
        ElementKind::Container => {
            let child_ids = children.iter().map(|child| child.id).collect::<Vec<_>>();
            tree.new_with_children(style, &child_ids)?
        }
    };
    Ok(NodeMap {
        index,
        node,
        id,
        children,
    })
}

fn collect_layout(
    tree: &TaffyTree<usize>,
    node: &NodeMap,
    elements: &[&Element],
    parent: Point,
    clip: Option<Rect>,
    output: &mut LayoutOutput,
) -> Result<(), LayoutError> {
    let layout = tree.layout(node.id)?;
    let origin = Point::new(parent.x + layout.location.x, parent.y + layout.location.y);
    let bounds = Rect::new(origin, Size::new(layout.size.width, layout.size.height));
    let element = elements[node.index];
    if let Some(interaction) = element.interaction
        && interaction.enabled
        && let Some(region_clip) = clip.and_then(|clip| clip.intersection(bounds))
    {
        output.hit_regions.push(HitRegion {
            node: node.node,
            bounds,
            clip: region_clip,
            focusable: interaction.focusable,
        });
    }
    let mut text_index = None;
    if let ElementKind::Text { content, style } = &element.kind {
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
            ClipBehavior::None => clip,
            ClipBehavior::Bounds => clip.and_then(|clip| clip.intersection(bounds)),
        };
        if let Some(text_clip) = text_clip
            && text_clip.intersection(text_bounds).is_some()
        {
            let mut block = TextBlock::new(content, text_bounds);
            block.clip = text_clip;
            block.style = style.clone();
            output.text.push(block);
            text_index = Some(output.text.blocks().len() - 1);
        }
    }
    output.nodes.push(LayoutNode {
        index: node.index,
        node: node.node,
        bounds,
        clip,
        text_index,
    });
    let child_clip = match element.paint.clip {
        ClipBehavior::None => clip,
        ClipBehavior::Bounds => clip.and_then(|clip| clip.intersection(bounds)),
    };
    for child in &node.children {
        collect_layout(tree, child, elements, origin, child_clip, output)?;
    }
    Ok(())
}

fn flattened(root: &Element) -> Vec<&Element> {
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

fn taffy_style(style: &LayoutStyle) -> Style {
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
