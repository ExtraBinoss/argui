use argui_core::{Affine2D, Point, Rect, Size};
use argui_ui::{
    EffectId, EffectInstance, EffectValue, Filter, LayerStyle, ScrollEffect, ScrollMetric,
    ScrollMetrics,
};

fn metrics() -> ScrollMetrics {
    ScrollMetrics {
        viewport: Rect::new(Point::new(10.0, 20.0), Size::new(100.0, 80.0)),
        transform: Affine2D::IDENTITY,
        offset: Point::new(20.0, 30.0),
        max_offset: Point::new(100.0, 60.0),
    }
}

fn effect() -> ScrollEffect {
    ScrollEffect::new(
        LayerStyle::new(Rect::default()).filter(Filter::Effect(EffectInstance::new(
            EffectId::new("test.scroll"),
            [("value", EffectValue::Vec4([0.0; 4]))],
        ))),
    )
}

#[test]
fn metrics_clamp_bounce_and_invalid_values_without_losing_end_distance() {
    let mut m = metrics();
    assert_eq!(m.remaining(), [20.0, 30.0, 80.0, 30.0]);
    assert_eq!(
        m.value(ScrollMetric::Offset),
        EffectValue::Vec2([20.0, 30.0])
    );
    assert_eq!(
        m.value(ScrollMetric::Progress),
        EffectValue::Vec2([0.2, 0.5])
    );
    assert_eq!(
        m.value(ScrollMetric::ViewportWidth),
        EffectValue::LogicalPixels(100.0)
    );
    assert_eq!(
        m.value(ScrollMetric::ViewportHeight),
        EffectValue::LogicalPixels(80.0)
    );
    m.offset = Point::new(-12.0, 90.0);
    assert_eq!(m.remaining(), [0.0, 60.0, 100.0, 0.0]);
    m.offset = Point::new(f32::NAN, f32::INFINITY);
    m.max_offset = Point::new(-1.0, f32::NAN);
    assert_eq!(m.remaining(), [0.0; 4]);
    assert_eq!(m.value(ScrollMetric::Progress), EffectValue::Vec2([0.0; 2]));
}

#[test]
fn edge_ramp_is_continuous_and_respects_threshold_and_strength() {
    let mut m = metrics();
    m.offset = Point::new(4.0, 8.0);
    let source = ScrollMetric::Edges {
        strengths: [1.0, 0.5, 0.0, f32::NAN],
        threshold: 4.0,
        ramp: 8.0,
    };
    assert_eq!(m.value(source), EffectValue::Vec4([0.0, 0.25, 0.0, 0.0]));
    assert_eq!(
        m.value(ScrollMetric::Edges {
            strengths: [1.0; 4],
            threshold: f32::NAN,
            ramp: -1.0
        }),
        EffectValue::Vec4([1.0; 4])
    );
    m.offset = Point::default();
    assert_eq!(
        m.value(ScrollMetric::Edges {
            strengths: [1.0; 4],
            threshold: 0.0,
            ramp: 0.0
        }),
        EffectValue::Vec4([0.0, 0.0, 1.0, 1.0])
    );
}

#[test]
fn local_mapping_survives_translation_rotation_and_nonuniform_scale() {
    let mut m = metrics();
    for transform in [
        Affine2D::IDENTITY,
        Affine2D {
            matrix: [0.0, 2.0, -3.0, 0.0],
            translation: Point::new(40.0, 15.0),
        },
    ] {
        m.transform = transform;
        let bounds = transform.transform_rect(m.viewport);
        let global = transform.transform_point(Point::new(35.0, 80.0));
        let normalized = [
            (global.x - bounds.origin.x) / bounds.size.width,
            (global.y - bounds.origin.y) / bounds.size.height,
            1.0,
        ];
        for (source, expected) in [(ScrollMetric::LocalX, 0.25), (ScrollMetric::LocalY, 0.75)] {
            let EffectValue::Vec3(row) = m.value(source) else {
                panic!()
            };
            let value: f32 = row.into_iter().zip(normalized).map(|(a, b)| a * b).sum();
            assert!((value - expected).abs() < 0.00001);
        }
    }
}

#[test]
fn bindings_replace_by_target_and_do_not_mutate_authored_parameters() {
    let effect = effect()
        .bind(
            0,
            "value",
            ScrollMetric::Edges {
                strengths: [1.0; 4],
                threshold: 0.0,
                ramp: 1.0,
            },
        )
        .bind(0, "value", ScrollMetric::Remaining);
    let layer = effect.resolve(metrics()).unwrap();
    let Filter::Effect(resolved) = &layer.filters[0] else {
        panic!()
    };
    assert_eq!(
        resolved.parameters[0].value,
        EffectValue::Vec4([20.0, 30.0, 80.0, 30.0])
    );
    assert_eq!(layer.bounds, metrics().viewport);
    let Filter::Effect(authored) = &effect.layer.filters[0] else {
        panic!()
    };
    assert_eq!(authored.parameters[0].value, EffectValue::Vec4([0.0; 4]));
}

#[test]
fn no_overflow_disabled_edges_zero_viewport_and_singular_transform_skip_layers() {
    let effect = effect().bind(
        0,
        "value",
        ScrollMetric::Edges {
            strengths: [0.0; 4],
            threshold: 0.0,
            ramp: 12.0,
        },
    );
    assert!(
        effect.resolve(metrics()).is_some(),
        "zero bindings must not disable arbitrary filters"
    );
    let effect = effect.when_edges([0.0; 4], 0.0, 12.0);
    assert!(effect.resolve(metrics()).is_none());
    let plain = ScrollEffect::new(LayerStyle::new(Rect::default()).opacity(0.5));
    let mut m = metrics();
    m.max_offset = Point::default();
    assert!(plain.resolve(m).is_none());
    m = metrics();
    m.viewport.size.width = 0.0;
    assert!(plain.resolve(m).is_none());
    m = metrics();
    m.viewport.size.height = 0.0;
    assert!(plain.resolve(m).is_none());
    m = metrics();
    m.transform.matrix = [0.0; 4];
    assert!(plain.resolve(m).is_none());
    assert!(matches!(
        m.value(ScrollMetric::LocalX),
        EffectValue::Vec3(_)
    ));
}

#[test]
#[should_panic(expected = "custom filter")]
fn binding_rejects_non_custom_filter() {
    let _ = ScrollEffect::new(LayerStyle::new(Rect::default()).filter(Filter::Blur(2.0))).bind(
        0,
        "value",
        ScrollMetric::Progress,
    );
}

#[test]
#[should_panic(expected = "existing parameter")]
fn binding_rejects_missing_parameter() {
    let _ = effect().bind(0, "missing", ScrollMetric::Progress);
}

#[test]
#[should_panic(expected = "type mismatch")]
fn binding_rejects_wrong_type() {
    let _ = effect().bind(0, "value", ScrollMetric::Progress);
}
