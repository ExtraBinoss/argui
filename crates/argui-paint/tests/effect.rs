use argui_core::{Color, Point, Rect, Size};
use argui_paint::{
    BlendMode, CustomEffect, DisplayList, DisplayListError, Filter, LayerMask, LayerStyle,
    ShaderEffectId, Shadow,
};

fn bounds() -> Rect {
    Rect::new(Point::new(20.0, 30.0), Size::new(100.0, 50.0))
}

#[test]
fn effect_layers_are_explicit_and_compute_conservative_bounds() {
    let layer = LayerStyle::new(bounds())
        .opacity(0.8)
        .blend(BlendMode::Screen)
        .mask(LayerMask::Bounds)
        .filter(Filter::Blur(4.0))
        .shadow(Shadow::drop(
            [20.0, -2.0],
            8.0,
            Color::rgba(0.0, 0.0, 0.0, 0.5),
        ));

    assert!(layer.requires_offscreen());
    assert_eq!(layer.expanded_bounds().origin, Point::new(-24.0, -14.0));
    assert_eq!(layer.expanded_bounds().size, Size::new(188.0, 138.0));
    assert!(!LayerStyle::new(bounds()).requires_offscreen());
    assert_eq!(Filter::Contrast(2.0).expansion(), 0.0);

    let scaled = layer.scaled(1.5);
    assert_eq!(scaled.bounds.origin, Point::new(30.0, 45.0));
    assert_eq!(scaled.bounds.size, Size::new(150.0, 75.0));
    assert!(matches!(scaled.filters[0], Filter::Blur(6.0)));
    assert_eq!(scaled.shadows[0].offset, [30.0, -3.0]);

    let glow = Shadow::glow(20.0, Color::WHITE)
        .blur(24.0)
        .spread(3.0)
        .inset(true);
    assert_eq!(glow.offset, [0.0, 0.0]);
    assert_eq!(glow.blur, 24.0);
    assert_eq!(glow.spread, 3.0);
    assert!(glow.inset);
}

#[test]
fn every_layer_feature_independently_requests_offscreen_rendering() {
    let plain = LayerStyle::new(bounds());
    assert!(!plain.requires_offscreen());
    assert!(plain.clone().opacity(0.5).requires_offscreen());
    assert!(
        plain
            .clone()
            .blend(BlendMode::Multiply)
            .requires_offscreen()
    );
    assert!(
        plain
            .clone()
            .filter(Filter::Brightness(1.2))
            .requires_offscreen()
    );
    assert!(
        plain
            .clone()
            .backdrop(Filter::Saturation(1.2))
            .requires_offscreen()
    );
    assert!(
        plain
            .clone()
            .shadow(Shadow::glow(4.0, Color::WHITE))
            .requires_offscreen()
    );
    assert!(plain.mask(LayerMask::Bounds).requires_offscreen());

    let outer = LayerStyle::new(bounds()).filter(Filter::Custom(
        CustomEffect::new(ShaderEffectId(4), []).expansion(12.0),
    ));
    assert_eq!(outer.foreground_expansion(), 12.0);
    assert_eq!(outer.foreground_bounds().origin, Point::new(8.0, 18.0));
    assert_eq!(outer.scaled(2.0).foreground_expansion(), 24.0);
}

#[test]
fn custom_effect_pixel_parameters_follow_dpi_without_scaling_unitless_values() {
    let effect = Filter::Custom(
        CustomEffect::new(ShaderEffectId(8), [0.5, 12.0])
            .pixel_parameter(1)
            .expansion(8.0),
    );
    let Filter::Custom(scaled) = effect.scaled(2.0) else {
        panic!("expected a custom effect");
    };
    assert_eq!(scaled.parameters, [0.5, 24.0]);
    assert_eq!(scaled.expansion, 16.0);
    assert_eq!(scaled.pixel_parameters, 1 << 1);
}

#[test]
fn balanced_nested_layers_validate_without_gpu_state() {
    let layer = LayerStyle::new(bounds());
    let mut list = DisplayList::new();
    list.begin_layer(layer.clone());
    list.begin_layer(layer);
    list.end_layer();
    list.end_layer();
    assert_eq!(list.validate(), Ok(()));

    list.end_layer();
    assert_eq!(
        list.validate(),
        Err(DisplayListError::UnexpectedLayerEnd { command: 4 })
    );

    let mut open = DisplayList::new();
    open.begin_layer(LayerStyle::new(bounds()));
    assert_eq!(
        open.validate(),
        Err(DisplayListError::UnclosedLayers { count: 1 })
    );
    assert!(
        open.validate()
            .unwrap_err()
            .to_string()
            .contains("unclosed")
    );
}
