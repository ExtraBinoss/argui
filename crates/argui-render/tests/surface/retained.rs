//! Pixel regressions for separated damage on translucent retained roots.

use argui_core::{Affine2D, Color, Point, Rect, Size};
use argui_paint::{Border, ClipChain, CornerRadii, DisplayList, Fill, Filter, LayerStyle, Quad};
use argui_render::{DamageMode, DamageTracking, RendererConfig, SurfaceAlphaMode, SurfaceRenderer};
use argui_text::{PreparedText, TextEngine};

/// Builds separated animated patches with unchanged translucent pixels between them.
fn scene(step: usize, effect: bool) -> DisplayList {
    let mut list = DisplayList::new();
    let quad = |x, y, width, height, color| Quad {
        bounds: Rect::new(Point::new(x, y), Size::new(width, height)),
        background: Some(Fill::Solid(color)),
        border: Border::all(0.0, Color::TRANSPARENT),
        radii: CornerRadii::all(0.0),
        opacity: 1.0,
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
    };
    list.push_quad(quad(
        0.0,
        0.0,
        256.0,
        192.0,
        Color::from_srgba8(40, 60, 80, 96),
    ));
    let color = if step % 2 == 0 {
        Color::WHITE
    } else {
        Color::BLACK
    };
    for (x, y) in [(8.0, 8.0), (208.0, 144.0)] {
        list.push_quad(quad(x, y, 12.0, 12.0, color));
    }
    if effect {
        let bounds = Rect::new(Point::new(96.0, 64.0), Size::new(48.0, 32.0));
        list.begin_layer(LayerStyle::new(bounds).filter(Filter::Brightness(0.8)));
        list.push_quad(quad(
            96.0,
            64.0,
            48.0,
            32.0,
            Color::from_srgba8(60, 90, 140, 128),
        ));
        list.end_layer();
    }
    list
}

/// Compares every incremental native pixel with a fully cleared reference frame.
fn compare_frames(effect: bool) {
    let config = RendererConfig::default()
        .profiling(true)
        .clear_color(Color::TRANSPARENT)
        .surface_alpha(SurfaceAlphaMode::Transparent);
    let mut renderer =
        pollster::block_on(SurfaceRenderer::new_offscreen(256, 192, config.clone())).unwrap();
    let mut fresh = pollster::block_on(SurfaceRenderer::new_offscreen(
        256,
        192,
        config.damage_tracking(DamageTracking::disabled()),
    ))
    .unwrap();
    let mut text = TextEngine::new();
    for step in 0..7 {
        let list = scene(step, effect);
        renderer
            .render_ui(&mut text, &PreparedText::default(), &list, 1.0)
            .unwrap();
        let mode = renderer.last_profile().damage.mode;
        if step >= 2 {
            assert_eq!(mode, DamageMode::Partial);
        }
        let incremental = renderer.read_offscreen_rgba().unwrap();
        fresh
            .render_ui(&mut text, &PreparedText::default(), &list, 1.0)
            .unwrap();
        let reference = fresh.read_offscreen_rgba().unwrap();
        for (index, (&actual, &expected)) in incremental.iter().zip(&reference).enumerate() {
            assert!(
                actual.abs_diff(expected) <= 1,
                "effect={effect} step={step} mode={mode:?} pixel=({}, {}) channel={} incremental={actual} fresh={expected}",
                index / 4 % 256,
                index / 4 / 256,
                index % 4
            );
        }
    }
}

#[test]
fn separate_damage_does_not_accumulate_alpha_between_plain_regions() {
    compare_frames(false);
}

#[test]
fn separate_damage_does_not_accumulate_alpha_between_effect_regions() {
    compare_frames(true);
}

#[test]
fn unchanged_transparent_frames_reuse_the_same_pixels_without_alpha_accumulation() {
    let config = RendererConfig::default()
        .profiling(true)
        .clear_color(Color::TRANSPARENT)
        .surface_alpha(SurfaceAlphaMode::Transparent);
    let mut renderer =
        pollster::block_on(SurfaceRenderer::new_offscreen(256, 192, config)).unwrap();
    let mut text = TextEngine::new();
    for step in 0..2 {
        renderer
            .render_ui(
                &mut text,
                &PreparedText::default(),
                &scene(step, false),
                1.0,
            )
            .unwrap();
    }
    let expected = renderer.read_offscreen_rgba().unwrap();
    for _ in 0..4 {
        renderer
            .render_ui(&mut text, &PreparedText::default(), &scene(1, false), 1.0)
            .unwrap();
        assert_eq!(renderer.last_profile().damage.mode, DamageMode::Reused);
        assert_eq!(renderer.read_offscreen_rgba().unwrap(), expected);
    }
}
