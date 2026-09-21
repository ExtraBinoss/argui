use argui_core::{Color, Point};
use argui_inspect::{StyleLength, StyleProperty, StyleUnit, StyleValue};
use argui_layout::LayoutOutput;
use argui_paint::{
    BilinearGradient, ColorInterpolation, ConicGradient, Fill, GradientStop, RadialGradient,
};
use argui_runtime::Inspection;
use argui_ui::{Axes, Dimension, EffectScope, Element, Overflow, UiTree};

/// The inspector keeps less common fills distinguishable in its public snapshot.
#[test]
fn snapshots_name_conic_and_bilinear_backgrounds() {
    let stops = [
        GradientStop::new(0.0, Color::BLACK),
        GradientStop::new(1.0, Color::WHITE),
    ];
    let radial = RadialGradient::new(
        Point::default(),
        Point::new(1.0, 1.0),
        ColorInterpolation::Srgb,
        stops,
    )
    .unwrap();
    let conic = ConicGradient::with_stops(
        Point::new(0.5, 0.5),
        30.0,
        ColorInterpolation::Srgb,
        radial.stops,
    );
    let mut element = Element::container([]);
    for (fill, expected) in [
        (Fill::Conic(conic), "conic gradient · 2 stops"),
        (
            Fill::Bilinear(BilinearGradient::new(
                [Color::WHITE; 4],
                ColorInterpolation::Srgb,
            )),
            "bilinear gradient · 4 corners",
        ),
    ] {
        element.paint.quad.background = Some(fill);
        let snapshot =
            Inspection::snapshot(&UiTree::new(element.clone()), &LayoutOutput::default());
        let background = snapshot.nodes[0]
            .properties
            .iter()
            .find(|entry| entry.property == StyleProperty::Background)
            .unwrap();
        assert_eq!(background.value, StyleValue::Summary(expected.into()));
        assert!(background.authored);
    }
}

#[test]
fn default_and_authored_style_snapshots_preserve_units_and_layer_identity() {
    use argui_core::Rect;
    use argui_paint::LayerStyle;

    let default = Inspection::snapshot(
        &UiTree::new(Element::container([])),
        &LayoutOutput::default(),
    );
    assert_eq!(default.nodes[0].properties.len(), StyleProperty::ALL.len());
    assert!(
        default.nodes[0]
            .properties
            .iter()
            .all(|entry| !entry.authored)
    );
    for property in [StyleProperty::Width, StyleProperty::Height] {
        let value = default.nodes[0]
            .properties
            .iter()
            .find(|entry| entry.property == property)
            .unwrap();
        assert_eq!(value.value, StyleValue::Length(StyleLength::default()));
    }
    for property in [StyleProperty::Layer, StyleProperty::Effects] {
        let value = default.nodes[0]
            .properties
            .iter()
            .find(|entry| entry.property == property)
            .unwrap();
        assert_eq!(value.value, StyleValue::Summary("none".into()));
    }

    let styled = Element::container([])
        .width(Dimension::percent(0.6))
        .height(Dimension::length(42.0))
        .overflow(Axes {
            x: Overflow::Auto,
            y: Overflow::Hidden,
        })
        .layer(LayerStyle::new(Rect::default()))
        .effect(EffectScope::Content, LayerStyle::new(Rect::default()));
    let snapshot = Inspection::snapshot(&UiTree::new(styled), &LayoutOutput::default());
    let properties = &snapshot.nodes[0].properties;
    let get = |property| {
        properties
            .iter()
            .find(|entry| entry.property == property)
            .unwrap()
    };
    assert_eq!(
        get(StyleProperty::Width).value,
        StyleValue::Length(StyleLength {
            value: 0.6,
            unit: StyleUnit::Percent,
        })
    );
    assert_eq!(
        get(StyleProperty::Height).value,
        StyleValue::Length(StyleLength {
            value: 42.0,
            unit: StyleUnit::Px,
        })
    );
    for property in [
        StyleProperty::Width,
        StyleProperty::Height,
        StyleProperty::Overflow,
        StyleProperty::Layer,
        StyleProperty::Effects,
    ] {
        assert!(get(property).authored);
    }
    assert!(matches!(
        &get(StyleProperty::Layer).value,
        StyleValue::Parameters(_)
    ));
    assert!(matches!(
        &get(StyleProperty::Effects).value,
        StyleValue::Parameters(_)
    ));
}
