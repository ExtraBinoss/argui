use super::*;
use argui_paint::{CompositorId, CompositorLayer};

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
                custom_effect.clone(),
                std::iter::empty::<argui_paint::EffectArgument>(),
            )),
        )
    };
    render(renderer, &custom(80.0)).unwrap();
    render(renderer, &custom(96.0)).unwrap();
    assert_eq!(renderer.last_profile().damage.mode, DamageMode::Partial);
}

/// Verifies that moving an unrelated primitive preserves static effect layers.
///
/// `renderer` owns the cached GPU layers; `render` presents each test scene.
pub(super) fn exercise_unrelated_layer_cache(
    renderer: &mut SurfaceRenderer,
    render: &mut impl FnMut(&mut SurfaceRenderer, &DisplayList) -> Result<(), RendererError>,
) {
    let scene = |x, first_color| {
        let mut list = DisplayList::new();
        list.push_quad(colored_quad(x, Color::WHITE));
        for (index, left) in [8.0, 80.0].into_iter().enumerate() {
            let bounds = Rect::new(Point::new(left, 8.0), Size::new(48.0, 48.0));
            list.begin_layer(
                LayerStyle::new(bounds)
                    .filter(Filter::Brightness(0.9))
                    .profile(RenderObjectId::new(ProfileDomain::Ui, index as u64 + 100)),
            );
            list.push_quad(colored_quad(
                left,
                if index == 0 {
                    first_color
                } else {
                    Color::WHITE
                },
            ));
            list.end_layer();
        }
        list
    };
    render(renderer, &scene(8.0, Color::WHITE)).unwrap();
    render(renderer, &scene(90.0, Color::WHITE)).unwrap();
    assert_eq!(renderer.last_profile().effects.cached_layers, 2);
    assert_eq!(renderer.last_profile().effects.damaged_pixels, 0);
    render(renderer, &scene(8.0, Color::BLACK)).unwrap();
    assert_eq!(renderer.last_profile().effects.cached_layers, 1);
    assert!(renderer.last_profile().effects.damaged_pixels > 0);
}

/// Verifies that composition-only frames retain scene damage and cached layers.
pub(super) fn exercise_compositor(
    renderer: &mut SurfaceRenderer,
    window: &Window,
    pump: &mut FramePump<'_>,
) {
    let retained = compositor_scene(Affine2D::IDENTITY);
    render(renderer, &retained, window, pump).unwrap();
    let transformed = compositor_scene(Affine2D::translation(48.0, 0.0));
    render_composite(renderer, &transformed, window, pump).unwrap();
    assert_eq!(renderer.last_profile().damage.mode, DamageMode::Partial);
    assert!(renderer.last_profile().damage.damaged_pixels < 256 * 256);
    assert!(renderer.last_profile().effects.cached_layers >= 1);
    render_composite(renderer, &transformed, window, pump).unwrap();
    assert_eq!(renderer.last_profile().damage.mode, DamageMode::Reused);
    renderer.set_damage_tracking(DamageTracking::disabled());
    renderer.set_damage_tracking(DamageTracking::enabled());
}

/// Presents retained compositor changes until the Wayland surface accepts a frame.
fn render_composite(
    renderer: &mut SurfaceRenderer,
    list: &DisplayList,
    window: &Window,
    pump: &mut FramePump<'_>,
) -> Result<(), RendererError> {
    let deadline = web_time::Instant::now() + NATIVE_TIMEOUT;
    loop {
        window.request_redraw();
        let Some(status) = pump(&mut || renderer.render_composite_notified(list, 1.0, || {}))
        else {
            assert!(
                web_time::Instant::now() < deadline,
                "redraw callback did not arrive"
            );
            continue;
        };
        let status = status?;
        if status == RenderStatus::Presented {
            return Ok(());
        }
        assert_eq!(status, RenderStatus::Skipped);
        assert!(
            web_time::Instant::now() < deadline,
            "compositor surface did not present within {NATIVE_TIMEOUT:?}"
        );
    }
}

/// Builds a small retained layer at the supplied composition transform.
fn compositor_scene(transform: Affine2D) -> DisplayList {
    let bounds = Rect::new(Point::new(8.0, 8.0), Size::new(32.0, 32.0));
    let mut layer = CompositorLayer::new(
        CompositorId::new(41),
        bounds,
        Affine2D::IDENTITY,
        Affine2D::IDENTITY,
        1.0,
    );
    layer.update(transform, 1.0);
    let mut list = DisplayList::new();
    list.begin_compositor(layer);
    list.push_quad(colored_quad(8.0, Color::WHITE));
    list.end_compositor();
    list
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
