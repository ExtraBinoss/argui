use argui_core::Point;

use crate::{ElasticScroll, NodeId, ScrollRegion, ScrollbarVisibility};

#[derive(Clone, Copy, Debug, PartialEq)]
struct Overscroll {
    node: NodeId,
    displacement: Point,
    velocity: Point,
    config: ElasticScroll,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ScrollbarActivity {
    node: NodeId,
    idle: f32,
    opacity: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ScrollPhysicsState {
    overscroll: Vec<Overscroll>,
    activity: Vec<ScrollbarActivity>,
}

impl ScrollPhysicsState {
    pub fn visual_offset(&self, node: NodeId, offset: Point) -> Point {
        self.overscroll
            .iter()
            .find(|entry| entry.node == node)
            .map_or(offset, |entry| {
                Point::new(
                    offset.x + entry.displacement.x,
                    offset.y + entry.displacement.y,
                )
            })
    }

    pub fn clear_overscroll(&mut self, node: NodeId) {
        self.overscroll.retain(|entry| entry.node != node);
    }

    pub fn push_overscroll(&mut self, node: NodeId, excess: Point, config: ElasticScroll) -> Point {
        let index = self
            .overscroll
            .iter()
            .position(|entry| entry.node == node)
            .unwrap_or_else(|| {
                self.overscroll.push(Overscroll {
                    node,
                    displacement: Point::default(),
                    velocity: Point::default(),
                    config,
                });
                self.overscroll.len() - 1
            });
        let entry = &mut self.overscroll[index];
        let previous = entry.displacement;
        let resistance = non_negative(config.resistance);
        let limit = non_negative(config.limit);
        entry.config = ElasticScroll {
            resistance,
            limit,
            spring: non_negative(config.spring),
            damping: non_negative(config.damping),
        };
        entry.displacement.x = (entry.displacement.x + excess.x * resistance).clamp(-limit, limit);
        entry.displacement.y = (entry.displacement.y + excess.y * resistance).clamp(-limit, limit);
        Point::new(
            entry.displacement.x - previous.x,
            entry.displacement.y - previous.y,
        )
    }

    pub fn advance_overscroll(&mut self, elapsed: f32) -> Vec<(NodeId, Point)> {
        let elapsed = if elapsed.is_finite() {
            elapsed.clamp(1.0 / 240.0, 1.0 / 20.0)
        } else {
            0.0
        };
        let mut changes = Vec::new();
        for entry in &mut self.overscroll {
            let previous = entry.displacement;
            let acceleration = Point::new(
                -entry.config.spring * entry.displacement.x
                    - entry.config.damping * entry.velocity.x,
                -entry.config.spring * entry.displacement.y
                    - entry.config.damping * entry.velocity.y,
            );
            entry.velocity.x += acceleration.x * elapsed;
            entry.velocity.y += acceleration.y * elapsed;
            entry.displacement.x += entry.velocity.x * elapsed;
            entry.displacement.y += entry.velocity.y * elapsed;
            settle_axis(&mut entry.displacement.x, &mut entry.velocity.x);
            settle_axis(&mut entry.displacement.y, &mut entry.velocity.y);
            let delta = Point::new(
                entry.displacement.x - previous.x,
                entry.displacement.y - previous.y,
            );
            if delta != Point::default() {
                changes.push((entry.node, delta));
            }
        }
        self.overscroll
            .retain(|entry| entry.displacement != Point::default());
        changes
    }

    pub fn overscroll_active(&self) -> bool {
        !self.overscroll.is_empty()
    }

    pub fn scrollbar_opacity(&self, node: NodeId, visibility: ScrollbarVisibility) -> f32 {
        match visibility {
            ScrollbarVisibility::Always => 1.0,
            ScrollbarVisibility::Hidden => 0.0,
            ScrollbarVisibility::Auto => self
                .activity
                .iter()
                .find_map(|entry| (entry.node == node).then_some(entry.opacity))
                .unwrap_or(0.0),
        }
    }

    pub fn activate_scrollbar(&mut self, node: NodeId, visibility: Option<ScrollbarVisibility>) {
        if visibility != Some(ScrollbarVisibility::Auto) {
            return;
        }
        if let Some(entry) = self.activity.iter_mut().find(|entry| entry.node == node) {
            entry.idle = 0.0;
            entry.opacity = 1.0;
        } else {
            self.activity.push(ScrollbarActivity {
                node,
                idle: 0.0,
                opacity: 1.0,
            });
        }
    }

    pub fn advance_scrollbars(
        &mut self,
        elapsed: f32,
        regions: &[ScrollRegion],
        held: impl Fn(NodeId) -> bool,
    ) -> bool {
        let elapsed = if elapsed.is_finite() {
            elapsed.max(0.0)
        } else {
            0.0
        };
        self.activity.retain(|entry| {
            regions.iter().any(|region| {
                region.node == entry.node
                    && region
                        .config
                        .scrollbar
                        .as_ref()
                        .is_some_and(|style| style.visibility == ScrollbarVisibility::Auto)
            })
        });
        let mut changed = false;
        for entry in &mut self.activity {
            let style = regions
                .iter()
                .find(|region| region.node == entry.node)
                .and_then(|region| region.config.scrollbar.as_ref())
                .expect("activity was retained only for an automatic scrollbar");
            if held(entry.node) {
                entry.idle = 0.0;
                changed |= entry.opacity != 1.0;
                entry.opacity = 1.0;
                continue;
            }
            entry.idle += elapsed;
            if entry.idle <= style.hide_delay.as_secs_f64() as f32 {
                continue;
            }
            let duration = style.fade_duration.as_secs_f64() as f32;
            let next = if duration <= f32::EPSILON {
                0.0
            } else {
                (entry.opacity - elapsed / duration).max(0.0)
            };
            changed |= next != entry.opacity;
            entry.opacity = next;
        }
        self.activity.retain(|entry| entry.opacity > 0.0);
        changed
    }

    pub fn scrollbar_activity_active(&self, held: impl Fn(NodeId) -> bool) -> bool {
        self.activity.iter().any(|entry| !held(entry.node))
    }

    pub fn retain(&mut self, ids: &[NodeId]) {
        self.overscroll.retain(|entry| ids.contains(&entry.node));
        self.activity.retain(|entry| ids.contains(&entry.node));
    }
}

fn settle_axis(displacement: &mut f32, velocity: &mut f32) {
    if displacement.abs() < 0.05 && velocity.abs() < 0.5 {
        *displacement = 0.0;
        *velocity = 0.0;
    }
}

fn non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}
