use std::collections::{HashMap, VecDeque};

use argui_core::{Point, PointerEvent, PointerId, PointerPhase};

use crate::NodeId;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GestureSet(u8);

impl GestureSet {
    const TAP: u8 = 1;
    const PAN: u8 = 2;
    const PINCH: u8 = 4;
    const ROTATION: u8 = 8;
    const PAN_IMMEDIATE: u8 = 16;

    pub const NONE: Self = Self(0);
    pub const ALL: Self = Self(Self::TAP | Self::PAN | Self::PINCH | Self::ROTATION);

    #[must_use]
    pub const fn tap(mut self) -> Self {
        self.0 |= Self::TAP;
        self
    }

    #[must_use]
    pub const fn pan(mut self) -> Self {
        self.0 |= Self::PAN;
        self
    }

    #[must_use]
    pub const fn pan_immediate(mut self) -> Self {
        self.0 |= Self::PAN | Self::PAN_IMMEDIATE;
        self
    }

    #[must_use]
    pub const fn pinch(mut self) -> Self {
        self.0 |= Self::PINCH;
        self
    }

    #[must_use]
    pub const fn rotation(mut self) -> Self {
        self.0 |= Self::ROTATION;
        self
    }

    const fn contains(self, flag: u8) -> bool {
        self.0 & flag != 0
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GesturePhase {
    Started,
    Changed,
    Ended,
    Cancelled,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GestureKind {
    Tap {
        position: Point,
    },
    Pan {
        position: Point,
        delta: Point,
        total: Point,
        velocity: Point,
    },
    Pinch {
        scale: f32,
    },
    Rotation {
        radians: f32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GestureEvent {
    pub target: NodeId,
    pub phase: GesturePhase,
    pub kind: GestureKind,
}

#[derive(Clone, Debug)]
struct Contact {
    target: NodeId,
    gestures: GestureSet,
    start: Point,
    last: Point,
    started_at: std::time::Duration,
    pan_started: bool,
    history: VecDeque<(std::time::Duration, Point)>,
}

#[derive(Clone, Debug)]
struct Pair {
    ids: [PointerId; 2],
    target: NodeId,
    gestures: GestureSet,
    distance: f32,
    angle: f32,
    pinch_started: bool,
    rotation_started: bool,
}

#[derive(Clone, Debug, Default)]
pub struct GestureArena {
    contacts: HashMap<PointerId, Contact>,
    pair: Option<Pair>,
}

impl GestureArena {
    pub fn update(
        &mut self,
        event: PointerEvent,
        hit: Option<(NodeId, GestureSet)>,
    ) -> Vec<GestureEvent> {
        match event.phase {
            PointerPhase::Pressed => self.pressed(event, hit),
            PointerPhase::Moved => self.moved(event),
            PointerPhase::Released => self.released(event, false),
            PointerPhase::Cancelled => self.released(event, true),
            PointerPhase::Entered | PointerPhase::Left => Vec::new(),
        }
    }

    pub fn cancel_all(&mut self) -> Vec<GestureEvent> {
        let mut output = Vec::new();
        for contact in self.contacts.values() {
            if contact.pan_started {
                output.push(pan_event(
                    contact,
                    GesturePhase::Cancelled,
                    Point::default(),
                ));
            }
        }
        if let Some(pair) = self.pair.take() {
            finish_pair(&pair, GesturePhase::Cancelled, &mut output);
        }
        self.contacts.clear();
        output
    }

    fn pressed(
        &mut self,
        event: PointerEvent,
        hit: Option<(NodeId, GestureSet)>,
    ) -> Vec<GestureEvent> {
        let Some((target, gestures)) = hit else {
            return Vec::new();
        };
        let mut history = VecDeque::new();
        history.push_back((event.timestamp, event.position));
        let pan_started = gestures.contains(GestureSet::PAN_IMMEDIATE);
        self.contacts.insert(
            event.id,
            Contact {
                target,
                gestures,
                start: event.position,
                last: event.position,
                started_at: event.timestamp,
                pan_started,
                history,
            },
        );
        let same_target = self
            .contacts
            .iter()
            .filter(|(_, contact)| contact.target == target)
            .map(|(id, _)| *id)
            .take(2)
            .collect::<Vec<_>>();
        if same_target.len() == 2 {
            let first = &self.contacts[&same_target[0]];
            let second = &self.contacts[&same_target[1]];
            self.pair = Some(Pair {
                ids: [same_target[0], same_target[1]],
                target,
                gestures,
                distance: distance(first.last, second.last).max(f32::EPSILON),
                angle: angle(first.last, second.last),
                pinch_started: false,
                rotation_started: false,
            });
        }
        if pan_started {
            vec![pan_event(
                &self.contacts[&event.id],
                GesturePhase::Started,
                Point::default(),
            )]
        } else {
            Vec::new()
        }
    }

    fn moved(&mut self, event: PointerEvent) -> Vec<GestureEvent> {
        let Some(contact) = self.contacts.get_mut(&event.id) else {
            return Vec::new();
        };
        let delta = difference(event.position, contact.last);
        contact.last = event.position;
        push_history(contact, event.timestamp, event.position);
        if let Some(pair) = &mut self.pair
            && pair.ids.contains(&event.id)
        {
            return update_pair(pair, &self.contacts);
        }
        let total = difference(event.position, contact.start);
        let threshold = if contact.gestures.contains(GestureSet::TAP) {
            8.0
        } else {
            0.0
        };
        if !contact.gestures.contains(GestureSet::PAN) || magnitude(total) <= threshold {
            return Vec::new();
        }
        let phase = if contact.pan_started {
            GesturePhase::Changed
        } else {
            contact.pan_started = true;
            GesturePhase::Started
        };
        vec![GestureEvent {
            target: contact.target,
            phase,
            kind: GestureKind::Pan {
                position: event.position,
                delta,
                total,
                velocity: velocity(contact),
            },
        }]
    }

    fn released(&mut self, event: PointerEvent, cancelled: bool) -> Vec<GestureEvent> {
        let mut output = Vec::new();
        if let Some(pair) = self.pair.take() {
            if pair.ids.contains(&event.id) {
                finish_pair(
                    &pair,
                    if cancelled {
                        GesturePhase::Cancelled
                    } else {
                        GesturePhase::Ended
                    },
                    &mut output,
                );
            } else {
                self.pair = Some(pair);
            }
        }
        let Some(mut contact) = self.contacts.remove(&event.id) else {
            return output;
        };
        contact.last = event.position;
        push_history(&mut contact, event.timestamp, event.position);
        if contact.pan_started {
            output.push(pan_event(
                &contact,
                if cancelled {
                    GesturePhase::Cancelled
                } else {
                    GesturePhase::Ended
                },
                Point::default(),
            ));
        } else if !cancelled
            && contact.gestures.contains(GestureSet::TAP)
            && magnitude(difference(contact.last, contact.start)) < 8.0
            && event.timestamp.saturating_sub(contact.started_at)
                <= std::time::Duration::from_millis(500)
        {
            output.push(GestureEvent {
                target: contact.target,
                phase: GesturePhase::Ended,
                kind: GestureKind::Tap {
                    position: event.position,
                },
            });
        }
        output
    }
}

fn update_pair(pair: &mut Pair, contacts: &HashMap<PointerId, Contact>) -> Vec<GestureEvent> {
    let first = &contacts[&pair.ids[0]];
    let second = &contacts[&pair.ids[1]];
    let mut output = Vec::new();
    let scale = distance(first.last, second.last) / pair.distance;
    if pair.gestures.contains(GestureSet::PINCH) && (scale - 1.0).abs() >= 0.02 {
        let phase = if pair.pinch_started {
            GesturePhase::Changed
        } else {
            pair.pinch_started = true;
            GesturePhase::Started
        };
        output.push(GestureEvent {
            target: pair.target,
            phase,
            kind: GestureKind::Pinch { scale },
        });
    }
    let radians = normalized_angle(angle(first.last, second.last) - pair.angle);
    if pair.gestures.contains(GestureSet::ROTATION) && radians.abs() >= 2.0_f32.to_radians() {
        let phase = if pair.rotation_started {
            GesturePhase::Changed
        } else {
            pair.rotation_started = true;
            GesturePhase::Started
        };
        output.push(GestureEvent {
            target: pair.target,
            phase,
            kind: GestureKind::Rotation { radians },
        });
    }
    output
}

fn finish_pair(pair: &Pair, phase: GesturePhase, output: &mut Vec<GestureEvent>) {
    if pair.pinch_started {
        output.push(GestureEvent {
            target: pair.target,
            phase,
            kind: GestureKind::Pinch { scale: 1.0 },
        });
    }
    if pair.rotation_started {
        output.push(GestureEvent {
            target: pair.target,
            phase,
            kind: GestureKind::Rotation { radians: 0.0 },
        });
    }
}

fn pan_event(contact: &Contact, phase: GesturePhase, delta: Point) -> GestureEvent {
    GestureEvent {
        target: contact.target,
        phase,
        kind: GestureKind::Pan {
            position: contact.last,
            delta,
            total: difference(contact.last, contact.start),
            velocity: velocity(contact),
        },
    }
}

fn push_history(contact: &mut Contact, timestamp: std::time::Duration, point: Point) {
    contact.history.push_back((timestamp, point));
    while contact.history.len() > 6
        || contact.history.front().is_some_and(|(time, _)| {
            timestamp.saturating_sub(*time) > std::time::Duration::from_millis(100)
        })
    {
        contact.history.pop_front();
    }
}

fn velocity(contact: &Contact) -> Point {
    let (start_time, start) = contact.history[0];
    let (end_time, end) = contact.history[contact.history.len() - 1];
    let seconds = end_time.saturating_sub(start_time).as_secs_f32();
    if seconds <= f32::EPSILON {
        Point::default()
    } else {
        let delta = difference(end, start);
        Point::new(delta.x / seconds, delta.y / seconds)
    }
}

fn difference(a: Point, b: Point) -> Point {
    Point::new(a.x - b.x, a.y - b.y)
}

fn magnitude(point: Point) -> f32 {
    point.x.hypot(point.y)
}

fn distance(a: Point, b: Point) -> f32 {
    magnitude(difference(a, b))
}

fn angle(a: Point, b: Point) -> f32 {
    (b.y - a.y).atan2(b.x - a.x)
}

fn normalized_angle(mut value: f32) -> f32 {
    if value > std::f32::consts::PI {
        value -= std::f32::consts::TAU;
    } else if value < -std::f32::consts::PI {
        value += std::f32::consts::TAU;
    }
    value
}

#[cfg(test)]
mod tests {
    use super::normalized_angle;

    #[test]
    fn angles_wrap_in_both_directions() {
        assert!(normalized_angle(std::f32::consts::TAU + 0.2) < 0.21);
        assert!(normalized_angle(-std::f32::consts::TAU - 0.2) > -0.21);
    }
}
