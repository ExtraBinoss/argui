use argui_automation::summarize_metrics;
use argui_core::MetricTrace;

#[test]
fn report_keeps_nesting_and_aggregates_named_phases() {
    let trace = MetricTrace::new();
    {
        let _action = trace.span("automation.action");
        for _ in 0..2 {
            let _layout = trace.span("layout.compute");
        }
        trace.gauge("render.draw_batches", 3.0, "count");
    }
    let report = summarize_metrics(&trace);
    assert_eq!(report.phases["layout.compute"].count, 2);
    assert_eq!(report.events[1].parent_id, Some(0));
    assert_eq!(report.events[3].kind, "gauge");
    assert_eq!(report.events[3].unit, Some("count"));
}
