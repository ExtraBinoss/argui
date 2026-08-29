use std::ops::Range;

use argui_paint::{
    DisplayCommand, DisplayList, DisplayListError, EffectInstance, Filter, LayerStyle,
};

use crate::batch::{DrawBatch, DrawKind};
use crate::{effect_plan::plan_filters, target::PixelRegion};

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum EffectNode {
    Draw(DrawBatch),
    Layer(EffectLayer),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EffectLayer {
    pub style: LayerStyle,
    pub children: Vec<EffectNode>,
    pub region: Option<PixelRegion>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct EffectGraph {
    pub roots: Vec<EffectNode>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EffectGraphStats {
    pub layers: usize,
    pub offscreen_layers: usize,
    pub draw_batches: usize,
    pub filter_passes: usize,
    pub offscreen_pixels: u64,
    pub cached_layers: usize,
    pub damaged_pixels: u64,
}

impl EffectGraph {
    pub fn build(
        display_list: &DisplayList,
        text_ranges: &[Range<u32>],
        viewport: [f32; 2],
        scale_factor: f32,
    ) -> Result<Self, DisplayListError> {
        display_list.validate()?;
        let mut roots = Vec::new();
        let mut stack: Vec<EffectLayer> = Vec::new();
        let mut quad = 0_u32;
        let mut image = 0_u32;
        let mut vector = 0_u32;
        for command in display_list.commands() {
            match command {
                DisplayCommand::Quad(_) => {
                    push_draw(&mut roots, &mut stack, DrawKind::Quad, quad..quad + 1);
                    quad += 1;
                }
                DisplayCommand::Text { block, .. } => push_draw(
                    &mut roots,
                    &mut stack,
                    DrawKind::Text,
                    text_ranges.get(*block).cloned().unwrap_or(0..0),
                ),
                DisplayCommand::Image(item) => {
                    push_draw(
                        &mut roots,
                        &mut stack,
                        DrawKind::Image(item.image, item.sampling),
                        image..image + 1,
                    );
                    image += 1;
                }
                DisplayCommand::Vector(item) => {
                    push_draw(
                        &mut roots,
                        &mut stack,
                        DrawKind::Vector(item.vector),
                        vector..vector + 1,
                    );
                    vector += 1;
                }
                DisplayCommand::BeginLayer(style) => {
                    let style = style.scaled(scale_factor);
                    let viewport = PixelRegion::viewport(viewport[0] as u32, viewport[1] as u32);
                    let region = match stack.last() {
                        Some(parent) => parent.region.and_then(|parent| {
                            PixelRegion::from_rect(style.expanded_bounds(), parent)
                        }),
                        None => PixelRegion::from_rect(style.expanded_bounds(), viewport),
                    };
                    stack.push(EffectLayer {
                        region,
                        style,
                        children: Vec::new(),
                    });
                }
                DisplayCommand::EndLayer => {
                    let layer = stack.pop().expect("validated layer stack");
                    if layer.style.opacity > 0.0 {
                        push_node(&mut roots, &mut stack, EffectNode::Layer(layer));
                    }
                }
            }
        }
        Ok(Self { roots })
    }

    #[must_use]
    pub fn stats(&self) -> EffectGraphStats {
        fn visit(nodes: &[EffectNode], stats: &mut EffectGraphStats) {
            for node in nodes {
                match node {
                    EffectNode::Draw(_) => stats.draw_batches += 1,
                    EffectNode::Layer(layer) => {
                        stats.layers += 1;
                        stats.offscreen_layers += usize::from(layer.style.requires_offscreen());
                        stats.filter_passes += plan_filters(&layer.style.filters).len()
                            + plan_filters(&layer.style.backdrop_filters).len()
                            + layer.style.shadows.len() * 2;
                        if layer.style.requires_offscreen()
                            && let Some(region) = layer.region
                        {
                            stats.offscreen_pixels +=
                                u64::from(region.size[0]) * u64::from(region.size[1]);
                        }
                        visit(&layer.children, stats);
                    }
                }
            }
        }
        let mut stats = EffectGraphStats::default();
        visit(&self.roots, &mut stats);
        stats
    }

    #[must_use]
    pub fn needs_offscreen_root(&self) -> bool {
        self.stats().offscreen_layers != 0
    }

    pub fn effects(&self) -> Vec<&EffectInstance> {
        fn visit<'a>(nodes: &'a [EffectNode], effects: &mut Vec<&'a EffectInstance>) {
            for node in nodes {
                if let EffectNode::Layer(layer) = node {
                    for filter in layer
                        .style
                        .filters
                        .iter()
                        .chain(&layer.style.backdrop_filters)
                    {
                        if let Filter::Effect(effect) = filter {
                            effects.push(effect);
                        }
                    }
                    visit(&layer.children, effects);
                }
            }
        }
        let mut effects = Vec::new();
        visit(&self.roots, &mut effects);
        effects
    }
}

fn push_draw(
    roots: &mut Vec<EffectNode>,
    stack: &mut [EffectLayer],
    kind: DrawKind,
    instances: Range<u32>,
) {
    if instances.is_empty() {
        return;
    }
    let nodes = stack.last_mut().map_or(roots, |layer| &mut layer.children);
    if let Some(EffectNode::Draw(previous)) = nodes.last_mut()
        && previous.kind == kind
        && previous.instances.end == instances.start
    {
        previous.instances.end = instances.end;
    } else {
        nodes.push(EffectNode::Draw(DrawBatch { kind, instances }));
    }
}

fn push_node(roots: &mut Vec<EffectNode>, stack: &mut [EffectLayer], node: EffectNode) {
    stack
        .last_mut()
        .map_or(roots, |layer| &mut layer.children)
        .push(node);
}

#[cfg(test)]
mod tests {
    use argui_core::{Point, Rect, Size};
    use argui_paint::{EffectId, EffectInstance, EffectValue, Filter, LayerStyle};

    use super::{EffectGraph, EffectNode};

    #[test]
    fn nested_layers_preserve_draw_order_and_report_cost() {
        let bounds = Rect::new(Point::default(), Size::new(100.0, 100.0));
        let mut list = argui_paint::DisplayList::new();
        list.push_text(0);
        list.begin_layer(LayerStyle::new(bounds).filter(Filter::Blur(8.0)));
        list.push_text(1);
        list.begin_layer(LayerStyle::new(bounds).opacity(0.5));
        list.push_text(2);
        list.end_layer();
        list.end_layer();

        let graph = EffectGraph::build(&list, &[0..2, 2..4, 4..7], [800.0, 600.0], 1.0).unwrap();
        assert!(graph.needs_offscreen_root());
        assert_eq!(graph.stats().layers, 2);
        assert_eq!(graph.stats().offscreen_layers, 2);
        assert_eq!(graph.stats().draw_batches, 3);
        assert_eq!(graph.stats().filter_passes, 1);
        assert_eq!(graph.stats().offscreen_pixels, 32_768);
        assert!(matches!(graph.roots[1], EffectNode::Layer(_)));
    }

    #[test]
    fn graph_skips_empty_draws_and_collects_nested_custom_effects() {
        let bounds = Rect::new(Point::default(), Size::new(100.0, 100.0));
        let foreground = EffectInstance::new(
            EffectId::new("test.foreground"),
            [("amount", EffectValue::F32(0.25))],
        );
        let backdrop = EffectInstance::new(
            EffectId::new("test.backdrop"),
            [("amount", EffectValue::F32(0.75))],
        );
        let mut list = argui_paint::DisplayList::new();
        list.push_text(0);
        list.push_text(1);
        list.begin_layer(LayerStyle::new(bounds).filter(Filter::Effect(foreground.clone())));
        list.begin_layer(LayerStyle::new(bounds).backdrop(Filter::Effect(backdrop.clone())));
        list.push_text(2);
        list.end_layer();
        list.end_layer();

        let graph = EffectGraph::build(&list, &[0..2, 2..4, 0..0], [800.0, 600.0], 1.0).unwrap();
        assert_eq!(graph.stats().draw_batches, 1);
        assert_eq!(graph.effects(), vec![&foreground, &backdrop]);

        let plain =
            EffectGraph::build(&argui_paint::DisplayList::new(), &[], [800.0, 600.0], 1.0).unwrap();
        assert!(!plain.needs_offscreen_root());
        assert!(plain.effects().is_empty());
    }

    #[test]
    fn fully_transparent_layers_are_removed_before_gpu_planning() {
        let bounds = Rect::new(Point::default(), Size::new(100.0, 100.0));
        let mut list = argui_paint::DisplayList::new();
        list.begin_layer(
            LayerStyle::new(bounds)
                .opacity(0.0)
                .filter(Filter::Blur(8.0)),
        );
        list.push_text(0);
        list.end_layer();

        let ranges = std::iter::once(0..4).collect::<Vec<_>>();
        let graph = EffectGraph::build(&list, &ranges, [800.0, 600.0], 1.0).unwrap();
        assert!(graph.roots.is_empty());
        assert!(!graph.needs_offscreen_root());
    }

    #[test]
    fn children_of_a_clipped_out_layer_stay_clipped_out() {
        let outside = Rect::new(Point::new(500.0, 500.0), Size::new(40.0, 40.0));
        let inside = Rect::new(Point::new(10.0, 10.0), Size::new(20.0, 20.0));
        let mut list = argui_paint::DisplayList::new();
        list.begin_layer(LayerStyle::new(outside).opacity(0.5));
        list.begin_layer(LayerStyle::new(inside).opacity(0.5));
        list.end_layer();
        list.end_layer();
        let graph = EffectGraph::build(&list, &[], [100.0, 100.0], 1.0).unwrap();
        let EffectNode::Layer(parent) = &graph.roots[0] else {
            panic!("expected parent layer");
        };
        let EffectNode::Layer(child) = &parent.children[0] else {
            panic!("expected child layer");
        };
        assert_eq!(parent.region, None);
        assert_eq!(child.region, None);
    }
}
