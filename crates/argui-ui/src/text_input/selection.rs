use argui_core::{CaretAffinity, TextPosition};
use unicode_segmentation::UnicodeSegmentation;

use super::{EditResult, TextInputState, TextPrivacy, TextSelection, previous_grapheme};
use crate::SelectionGranularity;

impl TextInputState {
    pub(super) fn unit_range(
        &self,
        position: TextPosition,
        granularity: SelectionGranularity,
    ) -> (usize, usize) {
        let index = self.unmask_index(position.index);
        let range = match granularity {
            SelectionGranularity::Character => index..index,
            SelectionGranularity::Word if self.privacy == TextPrivacy::Password => {
                0..self.value.len()
            }
            SelectionGranularity::Word => {
                let index = if index == self.value.len() {
                    previous_grapheme(&self.value, index)
                } else {
                    index
                };
                self.value
                    .split_word_bound_indices()
                    .find_map(|(start, part)| {
                        (start <= index && index < start + part.len())
                            .then_some(start..start + part.len())
                    })
                    .unwrap_or(index..index)
            }
            SelectionGranularity::Line => {
                let mut range = argui_text::line_range(&self.value, index);
                if self.value[range.end..].starts_with('\n') {
                    range.end += 1;
                }
                range
            }
        };
        (range.start, range.end)
    }

    pub(super) fn select_range(&mut self, anchor: usize, cursor: usize) -> EditResult {
        self.select(TextSelection::Range {
            anchor: TextPosition::new(anchor, CaretAffinity::After),
            cursor: TextPosition::new(cursor, CaretAffinity::Before),
        })
    }

    pub fn goal_x(&self) -> Option<f32> {
        self.goal_x
    }

    pub fn move_vertically(
        &mut self,
        position: TextPosition,
        goal_x: f32,
        extend: bool,
    ) -> EditResult {
        let result = self.place_position(position, extend);
        self.goal_x = goal_x.is_finite().then_some(goal_x);
        result
    }
}
