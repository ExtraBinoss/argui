use std::collections::{HashMap, VecDeque};

use argui_core::{Point, PointerEvent, PointerId, PointerPhase};

use crate::NodeId;

mod config;
pub use config::{
    GestureCapture, GestureDelivery, GestureEvent, GestureKind, GesturePhase, GestureSet, PanAxis,
    PanGesture, PinchGesture, RotationGesture, TapGesture,
};

#[derive(Clone, Debug)]
struct Contact {
    pointer: PointerId,
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
    /// Processes a pointer event and returns any gesture events it produces.
    ///
    /// # Arguments
    ///
    /// * `event` — pointer event to feed to the recognizers.
    /// * `hit` — hit target and its enabled gesture set for a press, if any.
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

    /// Cancels active gestures and returns the resulting cancellation events.
    pub fn cancel_all(&mut self) -> Vec<GestureEvent> {
        let mut output = Vec::new();
        for contact in self.contacts.values() {
            if let Some(pan) = contact.gestures.pan.filter(|_| contact.pan_started) {
                output.push(pan_event(
                    contact,
                    pan,
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
        let pan_started = gestures.pan.is_some_and(|pan| pan.threshold <= 0.0);
        self.contacts.insert(
            event.id,
            Contact {
                pointer: event.id,
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
        if let Some(pan) = gestures.pan.filter(|_| pan_started) {
            vec![pan_event(
                &self.contacts[&event.id],
                pan,
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
        let Some(pan) = contact.gestures.pan else {
            return Vec::new();
        };
        if magnitude(axis_point(total, pan.axis)) <= pan.threshold {
            return Vec::new();
        }
        let phase = if contact.pan_started {
            GesturePhase::Changed
        } else {
            contact.pan_started = true;
            GesturePhase::Started
        };
        vec![gesture_event(
            contact.target,
            event.id,
            phase,
            pan.delivery,
            GestureKind::Pan {
                position: event.position,
                delta: axis_point(delta, pan.axis),
                total: axis_point(total, pan.axis),
                velocity: axis_point(velocity(contact), pan.axis),
            },
        )]
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
        if let Some(pan) = contact.gestures.pan.filter(|_| contact.pan_started) {
            output.push(pan_event(
                &contact,
                pan,
                if cancelled {
                    GesturePhase::Cancelled
                } else {
                    GesturePhase::Ended
                },
                Point::default(),
            ));
        } else if !cancelled
            && contact.gestures.tap.is_some_and(|tap| {
                magnitude(difference(contact.last, contact.start)) < tap.max_distance
                    && event.timestamp.saturating_sub(contact.started_at) <= tap.max_duration
            })
        {
            output.push(gesture_event(
                contact.target,
                event.id,
                GesturePhase::Ended,
                GestureDelivery::Immediate,
                GestureKind::Tap {
                    position: event.position,
                },
            ));
        }
        output
    }
}

fn update_pair(pair: &mut Pair, contacts: &HashMap<PointerId, Contact>) -> Vec<GestureEvent> {
    let first = &contacts[&pair.ids[0]];
    let second = &contacts[&pair.ids[1]];
    let mut output = Vec::new();
    let scale = distance(first.last, second.last) / pair.distance;
    if let Some(pinch) = pair.gestures.pinch
        && (scale - 1.0).abs() >= pinch.threshold
    {
        let phase = if pair.pinch_started {
            GesturePhase::Changed
        } else {
            pair.pinch_started = true;
            GesturePhase::Started
        };
        output.push(gesture_event(
            pair.target,
            pair.ids[0],
            phase,
            pinch.delivery,
            GestureKind::Pinch { scale },
        ));
    }
    let radians = normalized_angle(angle(first.last, second.last) - pair.angle);
    if let Some(rotation) = pair.gestures.rotation
        && radians.abs() >= rotation.threshold_radians
    {
        let phase = if pair.rotation_started {
            GesturePhase::Changed
        } else {
            pair.rotation_started = true;
            GesturePhase::Started
        };
        output.push(gesture_event(
            pair.target,
            pair.ids[0],
            phase,
            rotation.delivery,
            GestureKind::Rotation { radians },
        ));
    }
    output
}

fn finish_pair(pair: &Pair, phase: GesturePhase, output: &mut Vec<GestureEvent>) {
    if let Some(pinch) = pair.gestures.pinch.filter(|_| pair.pinch_started) {
        output.push(gesture_event(
            pair.target,
            pair.ids[0],
            phase,
            pinch.delivery,
            GestureKind::Pinch { scale: 1.0 },
        ));
    }
    if let Some(rotation) = pair.gestures.rotation.filter(|_| pair.rotation_started) {
        output.push(gesture_event(
            pair.target,
            pair.ids[0],
            phase,
            rotation.delivery,
            GestureKind::Rotation { radians: 0.0 },
        ));
    }
}

fn pan_event(
    contact: &Contact,
    pan: PanGesture,
    phase: GesturePhase,
    delta: Point,
) -> GestureEvent {
    gesture_event(
        contact.target,
        contact.pointer,
        phase,
        pan.delivery,
        GestureKind::Pan {
            position: contact.last,
            delta: axis_point(delta, pan.axis),
            total: axis_point(difference(contact.last, contact.start), pan.axis),
            velocity: axis_point(velocity(contact), pan.axis),
        },
    )
}

fn gesture_event(
    target: NodeId,
    pointer: PointerId,
    phase: GesturePhase,
    delivery: GestureDelivery,
    kind: GestureKind,
) -> GestureEvent {
    GestureEvent {
        target,
        pointer,
        phase,
        kind,
        delivery,
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

fn axis_point(point: Point, axis: PanAxis) -> Point {
    match axis {
        PanAxis::Horizontal => Point::new(point.x, 0.0),
        PanAxis::Vertical => Point::new(0.0, point.y),
        PanAxis::Both => point,
    }
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
