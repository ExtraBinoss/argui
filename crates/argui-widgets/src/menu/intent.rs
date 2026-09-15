use argui_core::{Point, Rect};
use std::time::Duration;

/// A single cancellable submenu deadline. The owner supplies time and current popup bounds.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MenuIntent {
    pending: Option<(String, Duration)>,
    origin: Option<Point>,
}

impl MenuIntent {
    /// Schedules activation of `id` after `delay`, recording pointer position and monotonic time `now`.
    pub fn schedule(
        &mut self,
        id: impl Into<String>,
        pointer: Point,
        now: Duration,
        delay: Duration,
    ) {
        self.pending = now.checked_add(delay).map(|deadline| (id.into(), deadline));
        self.origin = Some(pointer);
    }
    /// Cancels the pending submenu change and clears its pointer origin.
    pub fn cancel(&mut self) {
        self.pending = None;
        self.origin = None;
    }
    /// Returns the pending activation deadline, if any.
    pub fn next_deadline(&self) -> Option<Duration> {
        self.pending.as_ref().map(|(_, deadline)| *deadline)
    }
    /// Takes the scheduled item id when its deadline is no later than `now`.
    pub fn take_due(&mut self, now: Duration) -> Option<String> {
        if self.next_deadline().is_none_or(|deadline| deadline > now) {
            return None;
        }
        self.origin = None;
        self.pending.take().map(|(id, _)| id)
    }
    /// Keep a pending sibling change delayed while moving through the triangle
    /// toward the open submenu. Works with either left- or right-opening menus.
    /// `point` is the new pointer position, `submenu` its current bounds, and `now`/`delay`
    /// provide the clock and extension interval.
    /// Returns whether the pointer is inside the corridor.
    pub fn pointer_moved(
        &mut self,
        point: Point,
        submenu: Rect,
        now: Duration,
        delay: Duration,
    ) -> bool {
        if submenu.contains(point) {
            self.cancel();
            return false;
        }
        let Some(origin) = self.origin else {
            return false;
        };
        let left = submenu.origin.x;
        let right = left + submenu.size.width;
        let edge = if origin.x < left { left } else { right };
        let a = Point::new(edge, submenu.origin.y);
        let b = Point::new(edge, submenu.origin.y + submenu.size.height);
        let cross =
            |a: Point, b: Point, c: Point| (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
        let sides = [
            cross(origin, a, point),
            cross(a, b, point),
            cross(b, origin, point),
        ];
        let inside = cross(origin, a, b) != 0.0
            && (sides.iter().all(|v| *v >= 0.0) || sides.iter().all(|v| *v <= 0.0));
        if inside
            && let Some((_, deadline)) = &mut self.pending
            && let Some(next) = now.checked_add(delay)
        {
            *deadline = next;
        }
        self.origin = Some(point);
        inside
    }
}

impl super::Menu {
    /// Schedule a menu entry change on pointer entry. Advance the one returned deadline
    /// through the owning component's task scheduler and call `hover_response` when due.
    /// Returns whether a hover deadline was scheduled; `delay`, `event`, `intent`, and `now` describe the pending hover action and timing.
    pub fn schedule_hover(
        &self,
        event: &argui_ui::UiEvent,
        intent: &mut MenuIntent,
        now: Duration,
        delay: Duration,
    ) -> bool {
        let argui_ui::UiEventKind::Pointer(pointer) = &event.kind else {
            return false;
        };
        if pointer.phase != argui_core::PointerPhase::Entered || !self.open {
            return false;
        }
        let Some(id) = event
            .target_key()
            .and_then(|key| key.strip_prefix(&format!("{}::item::", self.key)))
        else {
            return false;
        };
        if self.hover_response(id).is_none() {
            return false;
        }
        intent.schedule(id, pointer.position, now, delay);
        true
    }

    /// Returns the submenu response to apply for hovered item `id`.
    pub fn hover_response(&self, id: &str) -> Option<super::MenuResponse> {
        use super::{MenuItem, MenuItemKind, MenuResponse};
        fn find<'a>(
            items: &'a [MenuItem],
            id: &str,
            path: &mut Vec<String>,
        ) -> Option<&'a MenuItem> {
            for item in items {
                if item.id == id {
                    return Some(item);
                }
                match &item.kind {
                    MenuItemKind::Submenu(children) => {
                        path.push(item.id.clone());
                        if let Some(found) = find(children, id, path) {
                            return Some(found);
                        }
                        path.pop();
                    }
                    MenuItemKind::Group(children) => {
                        if let Some(found) = find(children, id, path) {
                            return Some(found);
                        }
                    }
                    _ => {}
                }
            }
            None
        }
        if !self.open {
            return None;
        }
        let mut path = Vec::new();
        let item = find(&self.items, id, &mut path)?;
        if !item.enabled() || !self.path.starts_with(&path) {
            return None;
        }
        let focus = if let MenuItemKind::Submenu(children) = &item.kind {
            let first = super::item::level(children)
                .into_iter()
                .find(|item| item.enabled())?;
            path.push(item.id.clone());
            self.item_key(&first.id)
        } else {
            self.item_key(&item.id)
        };
        Some(MenuResponse::Submenu {
            path,
            focus: focus.into(),
        })
    }
}
