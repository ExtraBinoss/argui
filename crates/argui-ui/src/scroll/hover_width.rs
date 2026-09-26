use std::collections::HashMap;

use crate::NodeId;

use super::ScrollState;

/// Retains only scrollbars whose hovered width is visible or changing.
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct HoverWidths {
    entries: HashMap<NodeId, HoverWidth>,
}

#[derive(Clone, Debug, PartialEq)]
struct HoverWidth {
    from: f32,
    current: f32,
    target: f32,
    elapsed: f32,
    duration: f32,
}

impl HoverWidths {
    /// Sets the desired hover state and returns whether geometry changed immediately.
    ///
    /// * `node` — scrollbar owner.
    /// * `hovered` — whether its thumb or track is under the pointer.
    /// * `duration` — animation duration in seconds.
    /// * `reduced_motion` — applies the target without intermediate frames.
    pub(super) fn retarget(
        &mut self,
        node: NodeId,
        hovered: bool,
        duration: f32,
        reduced_motion: bool,
    ) -> bool {
        let target = f32::from(hovered);
        let entry = self.entries.entry(node).or_insert(HoverWidth {
            from: 0.0,
            current: 0.0,
            target: 0.0,
            elapsed: 0.0,
            duration: 0.0,
        });
        if entry.target == target && !reduced_motion {
            return false;
        }
        entry.from = entry.current;
        entry.target = target;
        entry.elapsed = 0.0;
        entry.duration = if reduced_motion || entry.from == target {
            0.0
        } else {
            duration.max(0.0)
        };
        if entry.duration == 0.0 {
            entry.current = target;
        }
        let changed = entry.current != entry.from;
        if entry.current == 0.0 && entry.target == 0.0 {
            self.entries.remove(&node);
        }
        changed
    }

    /// Returns the presented hover fraction for a scrollbar.
    ///
    /// * `node` — scrollbar owner.
    pub(super) fn fraction(&self, node: NodeId) -> f32 {
        self.entries.get(&node).map_or(0.0, |entry| entry.current)
    }

    /// Returns the last configured duration for `node`, when its width state exists.
    pub(super) fn duration(&self, node: NodeId) -> Option<f32> {
        self.entries.get(&node).map(|entry| entry.duration)
    }

    /// Advances width interpolation and returns whether any scrollbar changed.
    ///
    /// * `elapsed` — nonnegative time since the prior display frame, in seconds.
    pub(super) fn advance(&mut self, elapsed: f32) -> bool {
        let mut changed = false;
        self.entries.retain(|_, entry| {
            if entry.elapsed < entry.duration {
                entry.elapsed = (entry.elapsed + elapsed.max(0.0)).min(entry.duration);
                let progress = entry.elapsed / entry.duration;
                let eased = 1.0 - (1.0 - progress).powi(3);
                let next = entry.from + (entry.target - entry.from) * eased;
                changed |= next != entry.current;
                entry.current = next;
            }
            entry.target != 0.0 || entry.elapsed < entry.duration
        });
        changed
    }

    /// Returns whether any width needs another display frame.
    pub(super) fn is_active(&self) -> bool {
        self.entries
            .values()
            .any(|entry| entry.elapsed < entry.duration)
    }

    /// Finishes active width transitions and returns whether geometry changed.
    pub(super) fn finish(&mut self) -> bool {
        let mut changed = false;
        self.entries.retain(|_, entry| {
            changed |= entry.current != entry.target;
            entry.current = entry.target;
            entry.elapsed = entry.duration;
            entry.target != 0.0
        });
        changed
    }

    /// Drops animation state for removed scrollbar owners.
    ///
    /// * `ids` — retained node identities.
    pub(super) fn retain(&mut self, ids: &[NodeId]) {
        self.entries.retain(|node, _| ids.contains(node));
    }
}

impl ScrollState {
    /// Advances animated thumb widths by `elapsed` seconds; returns whether geometry changed.
    pub fn advance_hover_widths(&mut self, elapsed: f32) -> bool {
        self.hover_widths.advance(elapsed)
    }

    /// Returns whether another width animation frame is needed.
    pub fn hover_width_active(&self) -> bool {
        self.hover_widths.is_active()
    }

    /// Resolves the current width between `base` and `hovered` for `node`.
    pub fn hover_width(&self, node: NodeId, base: f32, hovered: f32) -> f32 {
        base + (hovered - base) * self.hover_widths.fraction(node)
    }

    /// Finishes width animations; returns whether any geometry changed.
    pub fn finish_hover_widths(&mut self) -> bool {
        self.hover_widths.finish()
    }
}
