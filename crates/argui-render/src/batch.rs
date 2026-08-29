use std::ops::Range;

use argui_paint::{DisplayCommand, DisplayList, ImageId, ImageSampling, VectorId};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DrawKind {
    Quad,
    Text,
    Image(ImageId, ImageSampling),
    Vector(VectorId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DrawBatch {
    pub kind: DrawKind,
    pub instances: Range<u32>,
}

pub(crate) fn build_batches(
    display_list: &DisplayList,
    text_ranges: &[Range<u32>],
    batches: &mut Vec<DrawBatch>,
) {
    batches.clear();
    let mut quad = 0;
    let mut image = 0;
    let mut vector = 0;
    for command in display_list.commands() {
        let draw = match command {
            DisplayCommand::Quad(_) => {
                let instances = quad..quad + 1;
                quad += 1;
                Some((DrawKind::Quad, instances))
            }
            DisplayCommand::Text { block, .. } => Some((
                DrawKind::Text,
                text_ranges.get(*block).cloned().unwrap_or(0..0),
            )),
            DisplayCommand::Image(item) => {
                let instances = image..image + 1;
                image += 1;
                Some((DrawKind::Image(item.image, item.sampling), instances))
            }
            DisplayCommand::Vector(item) => {
                let instances = vector..vector + 1;
                vector += 1;
                Some((DrawKind::Vector(item.vector), instances))
            }
            DisplayCommand::BeginLayer(_) | DisplayCommand::EndLayer => None,
        };
        let Some((kind, instances)) = draw else {
            continue;
        };
        if instances.start >= instances.end {
            continue;
        }
        if let Some(previous) = batches.last_mut()
            && previous.kind == kind
            && previous.instances.end == instances.start
        {
            previous.instances.end = instances.end;
        } else {
            batches.push(DrawBatch { kind, instances });
        }
    }
}

#[cfg(test)]
mod tests {
    use argui_core::{Affine2D, Rect};
    use argui_paint::{
        Border, ClipChain, Color, CornerRadii, DisplayList, Fill, Quad, VectorId, VectorPrimitive,
    };

    use super::{DrawBatch, DrawKind, build_batches};

    fn quad() -> Quad {
        let bounds = Default::default();
        Quad {
            bounds,
            background: Some(Fill::Solid(Color::WHITE)),
            border: Border::all(0.0, Color::TRANSPARENT),
            radii: CornerRadii::default(),
            opacity: 1.0,
            transform: Affine2D::IDENTITY,
            clips: ClipChain::default(),
        }
    }

    #[test]
    fn batches_only_merge_adjacent_compatible_draws() {
        let mut list = DisplayList::new();
        list.push_quad(quad());
        list.push_quad(quad());
        list.push_text(0);
        list.push_text(1);
        list.push_quad(quad());
        for id in [7, 7, 8] {
            list.push_vector(VectorPrimitive {
                vector: VectorId(id),
                bounds: Rect::default(),
                progress: 0.0,
                opacity: 1.0,
                transform: Affine2D::IDENTITY,
                clips: ClipChain::default(),
            });
        }

        let mut batches = Vec::new();
        build_batches(&list, &[0..4, 4..7], &mut batches);
        assert_eq!(
            batches,
            [
                DrawBatch {
                    kind: DrawKind::Quad,
                    instances: 0..2,
                },
                DrawBatch {
                    kind: DrawKind::Text,
                    instances: 0..7,
                },
                DrawBatch {
                    kind: DrawKind::Quad,
                    instances: 2..3,
                },
                DrawBatch {
                    kind: DrawKind::Vector(VectorId(7)),
                    instances: 0..2,
                },
                DrawBatch {
                    kind: DrawKind::Vector(VectorId(8)),
                    instances: 2..3,
                },
            ]
        );
    }

    #[test]
    fn layers_and_missing_text_ranges_do_not_create_empty_batches() {
        let mut list = DisplayList::new();
        list.begin_layer(argui_paint::LayerStyle::new(Rect::default()));
        list.push_text(9);
        list.end_layer();
        let mut batches = vec![DrawBatch {
            kind: DrawKind::Quad,
            instances: 0..1,
        }];
        build_batches(&list, &[], &mut batches);
        assert!(batches.is_empty());
    }
}
