use argui_paint::{CornerRadii, Filter, LayerMask};
use argui_ui::Element;

#[test]
fn rounded_clip_is_geometric_not_an_offscreen_layer() {
    let element = Element::container([]).clip(CornerRadii::all(8.0));
    assert!(element.layer.is_none());
    assert_eq!(element.style.overflow.x, argui_ui::Overflow::Hidden);
    assert_eq!(element.style.overflow.y, argui_ui::Overflow::Hidden);
    assert_eq!(element.paint.quad.radii, CornerRadii::all(8.0));
}

#[test]
fn compositing_builders_preserve_other_layer_properties() {
    let element = Element::container([])
        .filter(Filter::Blur(2.0))
        .backdrop_filter(Filter::Blur(8.0))
        .mask(LayerMask::Rounded(CornerRadii::all(6.0)))
        .opacity(0.5)
        .filter(Filter::Blur(3.0));
    let layer = element.layer.as_ref().unwrap();
    assert_eq!(layer.filters, [Filter::Blur(2.0), Filter::Blur(3.0)]);
    assert_eq!(layer.backdrop_filters, [Filter::Blur(8.0)]);
    assert_eq!(layer.opacity, 0.5);
    assert_eq!(layer.mask, LayerMask::Rounded(CornerRadii::all(6.0)));
    assert_eq!(
        element.clone().opacity(2.0).layer.as_ref().unwrap().opacity,
        1.0
    );
    assert_eq!(
        element
            .clone()
            .opacity(-1.0)
            .layer
            .as_ref()
            .unwrap()
            .opacity,
        0.0
    );
    assert_eq!(
        element.opacity(f32::NAN).layer.as_ref().unwrap().opacity,
        1.0
    );
}
