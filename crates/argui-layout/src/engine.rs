use argui_core::{Point, Rect, Size};
use argui_paint::{Border, ClipBehavior, Color, DisplayList, Fill, Quad};
use argui_text::{TextBlock, TextEngine, TextScene};
use argui_ui::{
    Align, Direction, Edges, Element, ElementKind, Justify, LayoutStyle, Length, UiTree,
};
use taffy::{
    AlignItems, AvailableSpace, Dimension, FlexDirection, JustifyContent, LengthPercentage,
    LengthPercentageAuto, NodeId, Style, TaffyTree, compute_leaf_layout,
    geometry::{Rect as TaffyRect, Size as TaffySize},
};

use crate::LayoutError;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LayoutNode {
    pub index: usize,
    pub bounds: Rect,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LayoutOutput {
    pub nodes: Vec<LayoutNode>,
    pub display_list: DisplayList,
    pub text: TextScene,
}

#[derive(Debug)]
struct NodeMap {
    index: usize,
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
        ui.mark_layout_clean();
        Ok(output)
    }

    fn rebuild(&mut self, ui: &UiTree) -> Result<(), LayoutError> {
        self.tree = TaffyTree::new();
        let mut next_index = 0;
        self.root = Some(build_node(&mut self.tree, ui.root(), &mut next_index)?);
        self.revision = Some(ui.revision());
        Ok(())
    }
}

fn build_node(
    tree: &mut TaffyTree<usize>,
    element: &Element,
    next_index: &mut usize,
) -> Result<NodeMap, LayoutError> {
    let index = *next_index;
    *next_index += 1;
    let children = element
        .children
        .iter()
        .map(|child| build_node(tree, child, next_index))
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
    output.nodes.push(LayoutNode {
        index: node.index,
        bounds,
    });
    let element = elements[node.index];
    if element.paint.is_visible()
        && let Some(clip) = clip.and_then(|clip| clip.intersection(bounds))
    {
        output.display_list.push_quad(Quad {
            bounds,
            background: match element.paint.background {
                Some(Fill::Solid(color)) => color,
                None => Color::TRANSPARENT,
            },
            border: element
                .paint
                .border
                .unwrap_or(Border::all(0.0, Color::TRANSPARENT)),
            radii: element.paint.radii,
            opacity: element.paint.opacity,
            clip,
        });
    }
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
        if let Some(text_clip) = clip.and_then(|clip| clip.intersection(text_bounds)) {
            let mut block = TextBlock::new(content, text_bounds);
            block.clip = text_clip;
            block.style = style.clone();
            output.text.push(block);
            output
                .display_list
                .push_text(output.text.blocks().len() - 1);
        }
    }
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
