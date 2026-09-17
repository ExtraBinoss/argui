use std::ops::Range;

use argui_paint::{DisplayCommand, DisplayList, ImageId, ImageSampling};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DrawKind {
    Quad,
    Text,
    Image(ImageId, ImageSampling),
    GpuCanvas(u32),
    Vector,
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
    let mut gpu_canvas = 0;
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
            DisplayCommand::GpuCanvas(_) => {
                let instances = gpu_canvas..gpu_canvas + 1;
                let kind = DrawKind::GpuCanvas(gpu_canvas);
                gpu_canvas += 1;
                Some((kind, instances))
            }
            DisplayCommand::Vector(_) => {
                let instances = vector..vector + 1;
                vector += 1;
                Some((DrawKind::Vector, instances))
            }
            DisplayCommand::BeginLayer(_)
            | DisplayCommand::EndLayer
            | DisplayCommand::BeginCompositor(_)
            | DisplayCommand::EndCompositor => None,
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
