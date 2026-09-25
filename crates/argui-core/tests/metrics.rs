#![cfg(feature = "metrics")]

use argui_core::{MetricKind, MetricTrace};

#[test]
fn nested_spans_and_gauges_keep_parent_and_units() {
    let trace = MetricTrace::new();
    {
        let _outer = trace.span("frame");
        {
            let _inner = trace.span("layout.measure");
            trace.gauge("layout.nodes", 42.0, "count");
            trace.gauge("ignored", f64::NAN, "count");
        }
    }
    let events = trace.events();
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].parent_id, None);
    assert_eq!(events[1].parent_id, Some(0));
    assert_eq!(events[2].parent_id, Some(1));
    assert_eq!(events[2].kind, MetricKind::Gauge { unit: "count" });
    assert_eq!(events[2].value, 42.0);
    assert!(events[0].value >= events[1].value);
}
