//! Preparation and effective-clip regressions for the text renderer.

use super::*;

/// Keeps transformed text visible when its effective display clip replaces an empty
/// layout clip, then culls the same glyph once its physical bounds leave that clip.
#[test]
fn glyph_culling_uses_effective_display_clips_after_transform() {
    if std::env::var_os("ARGUI_NATIVE_TESTS").is_none() {
        return;
    }
    let mut fixture = Fixture::new();
    let background = Color::from_srgb8(240, 240, 240);
    let scene = TextScene::new().with(
        TextBlock::new(
            "VISIBLE",
            Rect::new(Point::default(), Size::new(200.0, 40.0)),
        )
        .color(Color::from_srgb8(20, 20, 20))
        .clip(Rect::new(Point::default(), Size::default())),
    );
    let clips = ClipChain::from_regions(vec![argui_paint::ClipRegion::new(
        Rect::new(Point::default(), Size::new(WIDTH as f32, HEIGHT as f32)),
        Affine2D::IDENTITY,
    )]);
    let mut list = DisplayList::new();
    list.push_text_with_backdrop(
        0,
        Affine2D::translation(24.0, 20.0),
        clips.clone(),
        Some(background),
    );
    let pixels = fixture.render(&scene, &list, background, 1.0);
    assert!(pixels.as_chunks::<4>().0.iter().any(|pixel| pixel[0] < 220));
    let mut hidden = DisplayList::new();
    hidden.push_text_with_backdrop(
        0,
        Affine2D::translation(10000.0, 20.0),
        clips,
        Some(background),
    );
    let prepared = fixture.engine.prepare(&scene, 1.0);
    let draw = fixture
        .gpu
        .prepare_ui(
            &fixture.device,
            &fixture.queue,
            &mut fixture.engine,
            &prepared,
            &hidden,
            1.0,
        )
        .unwrap();
    assert!(draw.all().is_empty());
    assert_eq!(fixture.gpu.stats().raster_requests_this_frame, 0);
    fixture.gpu.clear_frame_stats();
    assert_eq!(fixture.gpu.stats().hits_this_frame, 0);
    assert!(fixture.gpu.stats().entries > 0);
}

/// Retains glyph pixels in the widened antialiasing fringe of a scaled clip.
/// The clip starts at x=200, but its shader fringe reaches 15 physical pixels
/// outside that edge; this glyph ends before x=198 and must still be drawn.
#[test]
fn scaled_clip_antialiasing_fringe_is_not_culled() {
    if std::env::var_os("ARGUI_NATIVE_TESTS").is_none() {
        return;
    }
    let mut fixture = Fixture::new();
    let background = Color::from_srgb8(240, 240, 240);
    let scene = TextScene::new().with(
        TextBlock::new(
            "M",
            Rect::new(Point::new(184.0, 24.0), Size::new(40.0, 20.0)),
        )
        .size(8.0)
        .color(Color::from_srgb8(0, 0, 0)),
    );
    let prepared = fixture.engine.prepare(&scene, 1.0);
    let glyph = prepared.glyphs[0];
    let image = fixture.engine.rasterize(glyph.key).unwrap();
    assert!(glyph.x + image.left + (image.width as i32) < 198);
    let clips = ClipChain::from_regions(vec![argui_paint::ClipRegion::new(
        Rect::new(Point::new(10.0, 0.0), Size::new(20.0, 20.0)),
        Affine2D {
            matrix: [20.0, 0.0, 0.0, 20.0],
            translation: Point::default(),
        },
    )]);
    let mut list = DisplayList::new();
    list.push_text_with_backdrop(0, Affine2D::IDENTITY, clips, Some(background));
    let pixels = fixture.render(&scene, &list, background, 1.0);
    assert!(pixels.as_chunks::<4>().0.iter().any(|pixel| pixel[0] < 235));
}
