use std::ops::Range;

use argui_paint::{DisplayCommand, DisplayList};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DrawKind {
    Quad,
    Text,
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
    for command in display_list.commands() {
        let draw = match command {
            DisplayCommand::Quad(_) => {
                let instances = quad..quad + 1;
                quad += 1;
                Some((DrawKind::Quad, instances))
            }
            DisplayCommand::Text(block) => Some((
                DrawKind::Text,
                text_ranges.get(*block).cloned().unwrap_or(0..0),
            )),
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
    use argui_paint::{Border, Color, CornerRadii, DisplayList, Quad};

    use super::{DrawBatch, DrawKind, build_batches};

    fn quad() -> Quad {
        let bounds = Default::default();
        Quad {
            bounds,
            background: Color::WHITE,
            border: Border::all(0.0, Color::TRANSPARENT),
            radii: CornerRadii::default(),
            opacity: 1.0,
            clip: bounds,
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
            ]
        );
    }
}
