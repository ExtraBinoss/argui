//! Optional, nested timing and numeric samples shared across engine crates.

use std::{cell::RefCell, rc::Rc};
use web_time::Instant;

/// The type and unit of one metric event.
#[derive(Clone, Debug, PartialEq)]
pub enum MetricKind {
    /// A completed wall-clock scope; `value` is elapsed milliseconds.
    Span,
    /// A point-in-time numeric sample with its unit.
    Gauge { unit: &'static str },
}

/// One nested timing span or numeric sample on a trace clock.
#[derive(Clone, Debug, PartialEq)]
pub struct MetricEvent {
    /// Stable index within this trace.
    pub id: usize,
    /// Enclosing scope, if any.
    pub parent_id: Option<usize>,
    /// Dot-separated phase or counter name.
    pub name: &'static str,
    /// The event's type and numeric unit.
    pub kind: MetricKind,
    /// Milliseconds since the trace began.
    pub started_ms: f64,
    /// Duration for spans, or sampled value for gauges.
    pub value: f64,
}

#[derive(Debug)]
struct TraceInner {
    started: Instant,
    events: Vec<MetricEvent>,
    active: Vec<usize>,
}

/// A cheap, optional trace handle shared by nested engine calls on one thread.
#[derive(Clone, Debug)]
pub struct MetricTrace(Rc<RefCell<TraceInner>>);

impl Default for MetricTrace {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricTrace {
    /// Starts a new monotonic trace clock with no events.
    #[must_use]
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(TraceInner {
            started: Instant::now(),
            events: Vec::new(),
            active: Vec::new(),
        })))
    }

    /// Returns elapsed milliseconds on this trace's monotonic clock.
    #[must_use]
    pub fn now_ms(&self) -> f64 {
        self.0.borrow().started.elapsed().as_secs_f64() * 1000.0
    }

    /// Opens a named wall-clock scope; dropping the returned guard closes it.
    /// `name` is a stable phase name for reports and aggregation.
    #[must_use]
    pub fn span(&self, name: &'static str) -> MetricSpan {
        let mut inner = self.0.borrow_mut();
        let id = inner.events.len();
        let started_ms = inner.started.elapsed().as_secs_f64() * 1000.0;
        let parent_id = inner.active.last().copied();
        inner.events.push(MetricEvent {
            id,
            parent_id,
            name,
            kind: MetricKind::Span,
            started_ms,
            value: 0.0,
        });
        inner.active.push(id);
        MetricSpan {
            trace: self.clone(),
            id,
        }
    }

    /// Records a point-in-time numeric value in `unit` under `name`.
    /// Non-finite values are ignored so reports remain valid JSON.
    pub fn gauge(&self, name: &'static str, value: f64, unit: &'static str) {
        if !value.is_finite() {
            return;
        }
        let mut inner = self.0.borrow_mut();
        let id = inner.events.len();
        let started_ms = inner.started.elapsed().as_secs_f64() * 1000.0;
        let parent_id = inner.active.last().copied();
        inner.events.push(MetricEvent {
            id,
            parent_id,
            name,
            kind: MetricKind::Gauge { unit },
            started_ms,
            value,
        });
    }

    /// Returns completed events in start order, including their parent IDs.
    #[must_use]
    pub fn events(&self) -> Vec<MetricEvent> {
        self.0.borrow().events.clone()
    }
}

/// Drop guard for a named CPU scope on a [`MetricTrace`].
#[derive(Debug)]
pub struct MetricSpan {
    trace: MetricTrace,
    id: usize,
}

impl MetricSpan {
    /// Returns this scope's stable ID for correlating external actions.
    #[must_use]
    pub const fn id(&self) -> usize {
        self.id
    }
}

impl Drop for MetricSpan {
    fn drop(&mut self) {
        let mut inner = self.trace.0.borrow_mut();
        let elapsed = inner.started.elapsed().as_secs_f64() * 1000.0;
        inner.events[self.id].value = elapsed - inner.events[self.id].started_ms;
        if let Some(position) = inner.active.iter().rposition(|&id| id == self.id) {
            inner.active.remove(position);
        }
    }
}
