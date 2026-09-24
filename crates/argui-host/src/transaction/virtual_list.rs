//! Retains virtual-list measurements across atomic host commits.

use argui_schema::{SchemaValue, builtin};
use argui_ui::VirtualList;

use super::HostNode;

/// Builds a list from validated-looking host properties, retaining prior measurements.
///
/// `node` carries current properties and the previous list; `reset` discards
/// measurements after a structural data-version change. Returns `None` for
/// incomplete or invalid input so the schema adapter reports the canonical
/// property error. Count edits copy shared measurements before mutation.
pub(super) fn retained(node: &HostNode, reset: bool) -> Option<VirtualList> {
    if node.native_type != builtin::VIRTUAL_WINDOW {
        return None;
    }
    let count = match node.properties.get(&builtin::ITEM_COUNT)? {
        SchemaValue::Int(value) => usize::try_from(*value).ok()?,
        _ => return None,
    };
    let estimate = match node.properties.get(&builtin::ROW_HEIGHT)? {
        SchemaValue::Float(value) if value.is_finite() && *value > 0.0 => *value,
        _ => return None,
    };
    let horizontal = matches!(
        node.properties.get(&builtin::VIRTUAL_HORIZONTAL),
        Some(SchemaValue::Bool(true))
    );
    let variable = matches!(
        node.properties.get(&builtin::VARIABLE_HEIGHT),
        Some(SchemaValue::Bool(true))
    );
    let viewport_property = if horizontal {
        builtin::VIRTUAL_VIEWPORT_WIDTH
    } else {
        builtin::VIEWPORT_HEIGHT
    };
    let viewport = match node.properties.get(&viewport_property) {
        Some(SchemaValue::Float(value)) if value.is_finite() && *value >= 0.0 => *value,
        None => 0.0,
        _ => return None,
    };
    let overscan = match node.properties.get(&builtin::OVERSCAN) {
        Some(SchemaValue::Int(value)) => usize::try_from(*value).ok()?,
        None => 3,
        _ => return None,
    };
    let mut list = match node.virtual_list.as_ref().filter(|_| !reset) {
        Some(previous)
            if previous.is_variable() == variable
                && previous.is_horizontal() == horizontal
                && previous.estimated_extent() == estimate =>
        {
            if previous.item_count() == count {
                previous.clone()
            } else {
                let mut next = previous.detached();
                let old_count = next.item_count();
                if count > old_count {
                    next.insert(old_count, count - old_count);
                } else {
                    next.remove(count..old_count);
                }
                next
            }
        }
        _ if variable => VirtualList::variable(count, estimate, viewport),
        _ => VirtualList::fixed(count, estimate, viewport),
    };
    if horizontal {
        list = list.horizontal();
    }
    Some(list.with_viewport(viewport).overscan(overscan))
}
