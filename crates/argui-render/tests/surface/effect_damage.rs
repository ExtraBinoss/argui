use super::*;

/// Exercises retained effect roots with built-in and custom bounded filters.
pub(super) fn exercise(
    renderer: &mut SurfaceRenderer,
    custom_effect: EffectId,
    render: &mut impl FnMut(&mut SurfaceRenderer, &DisplayList) -> Result<(), RendererError>,
) {
    let blur_first = backdrop_scene(80.0, Filter::Blur(6.0));
    render(renderer, &blur_first).unwrap();
    let blur_moved = backdrop_scene(96.0, Filter::Blur(6.0));
    render(renderer, &blur_moved).unwrap();
    assert_eq!(renderer.last_profile().damage.mode, DamageMode::Partial);
    assert!(renderer.last_profile().damage.damaged_pixels < 256 * 256);
    assert!(renderer.last_profile().damage.retained_bytes > 0);
    render(renderer, &blur_moved).unwrap();
    assert_eq!(renderer.last_profile().damage.mode, DamageMode::Reused);

    let custom = |x| {
        backdrop_scene(
            x,
            Filter::Effect(EffectInstance::new(
                custom_effect,
                std::iter::empty::<argui_paint::EffectArgument>(),
            )),
        )
    };
    render(renderer, &custom(80.0)).unwrap();
    render(renderer, &custom(96.0)).unwrap();
    assert_eq!(renderer.last_profile().damage.mode, DamageMode::Partial);
}

/// Builds a scene whose moving backdrop intersects one bounded effect layer.
fn backdrop_scene(x: f32, filter: Filter) -> DisplayList {
    let mut list = DisplayList::new();
    list.push_quad(colored_quad(x, Color::WHITE));
    list.begin_layer(
        LayerStyle::new(Rect::new(Point::new(64.0, 0.0), Size::new(128.0, 96.0))).backdrop(filter),
    );
    list.push_quad(Quad {
        bounds: Rect::new(Point::new(80.0, 16.0), Size::new(48.0, 48.0)),
        background: Some(Fill::Solid(Color::from_srgba8(40, 80, 180, 96))),
        border: Border::all(0.0, Color::TRANSPARENT),
        radii: CornerRadii::all(8.0),
        opacity: 1.0,
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
    });
    list.end_layer();
    list
}
