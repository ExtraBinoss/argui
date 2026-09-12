use argui_core::{Color, ColorScheme, Size};
use argui_layout::LayoutEngine;
use argui_text::TextEngine;
use argui_ui::{ClickEvent, UiEvent, UiEventKind, UiTree};
use argui_widgets::{Chart, ChartError, ChartKind, ChartSeries, shadcn};

#[test]
fn chart_scales_negative_and_positive_values_and_rejects_invalid_geometry() {
    let mut chart = Chart::new(
        "chart",
        "Revenue",
        ["Jan".into(), "Feb".into(), "Mar".into()],
        [ChartSeries {
            label: "Sales".into(),
            values: vec![-10.0, 0.0, 30.0],
            color: Color::BLACK,
        }],
    );
    assert_eq!(chart.domain(), Ok((-10.0, 30.0)));
    for kind in [ChartKind::Bar, ChartKind::Line] {
        chart.kind = kind;
        let mut tree = UiTree::new(
            chart
                .build(shadcn(Color::BLACK).resolve(ColorScheme::Light))
                .unwrap(),
        );
        let output = LayoutEngine::new()
            .compute(&mut tree, &mut TextEngine::new(), Size::new(600.0, 400.0))
            .unwrap();
        assert!(
            output
                .nodes
                .iter()
                .all(|node| node.bounds.origin.y.is_finite())
        );
        let semantic = tree.semantic_tree(&output.semantic_bounds, 1.0);
        assert_eq!(
            semantic
                .nodes
                .iter()
                .filter(|node| node.semantics.focus_policy.is_tab_stop())
                .count(),
            3
        );
        let node = tree.node_ids()[0];
        assert_eq!(
            chart.action(&UiEvent::new(
                node,
                Some("chart::point::0::2".into()),
                UiEventKind::Click(ClickEvent::accessibility())
            )),
            Some((0, 2))
        );
        for key in [
            "other",
            "chart::point::x::2",
            "chart::point::0::x",
            "chart::point::0::9",
            "chart::point::9::0",
            "chart::point::0",
        ] {
            assert!(
                chart
                    .action(&UiEvent::new(
                        node,
                        Some(key.into()),
                        UiEventKind::Click(ClickEvent::accessibility())
                    ))
                    .is_none()
            );
        }
    }
    chart.series[0].values = vec![0.0; 3];
    assert_eq!(chart.domain(), Ok((0.0, 1.0)));
    chart.series[0].values[0] = f64::NAN;
    assert_eq!(chart.domain(), Err(ChartError::InvalidValue));
    chart.series[0].values = vec![f64::MIN, f64::MAX, 0.0];
    assert_eq!(chart.domain(), Err(ChartError::InvalidValue));
    chart.series[0].values.clear();
    assert_eq!(chart.domain(), Err(ChartError::MismatchedSeries));
    chart.height = 0.0;
    assert_eq!(chart.domain(), Err(ChartError::InvalidSize));
    chart.height = 240.0;
    chart.width = f32::NAN;
    assert_eq!(chart.domain(), Err(ChartError::InvalidSize));
    let empty = Chart::new("empty", "Empty", [], []);
    assert!(
        empty
            .build(shadcn(Color::BLACK).resolve(ColorScheme::Light))
            .is_ok()
    );
}
