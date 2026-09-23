use std::collections::HashMap;

use argui_accessibility::SemanticTree;
use argui_core::{Point, Rect, Size};

/// Decides whether an accessibility snapshot needs rebuilding after a frame.
///
/// `presentation_only` means neither the retained tree nor semantic state changed.
/// `focus_unchanged` says focus processing did not alter the semantic focus.
/// `bounds` contains current logical rectangles keyed by retained node ID.
/// `scale` converts those rectangles to the snapshot's physical coordinates.
/// Returns true whenever semantic content, focus, or a semantic node's bounds
/// may have changed; decorative bounds are ignored.
pub(crate) fn needs_semantic_sync(
    snapshot: &SemanticTree,
    bounds: impl IntoIterator<Item = (u64, Rect)>,
    scale: f32,
    presentation_only: bool,
    focus_unchanged: bool,
) -> bool {
    if !presentation_only || !focus_unchanged {
        return true;
    }
    let bounds = bounds.into_iter().collect::<HashMap<_, _>>();
    snapshot.nodes.iter().any(|node| {
        let rect = bounds.get(&node.id.get()).copied().unwrap_or_default();
        node.bounds
            != Rect::new(
                Point::new(rect.origin.x * scale, rect.origin.y * scale),
                Size::new(rect.size.width * scale, rect.size.height * scale),
            )
    })
}
