use argui_core::{Affine2D, Point, Rect, Size};
use argui_text::{PreparedGlyph, TextBlock, TextEngine, TextScene};

const FONT: &[u8] = include_bytes!("../../../assets/fonts/NotoSans-Regular.ttf");

/// Creates a deterministic engine and a multiline scene with fractional baselines.
fn fixture() -> (TextEngine, TextScene) {
    let engine = TextEngine::from_embedded_fonts([FONT], "Noto Sans", "Noto Sans", "Noto Sans");
    let scene = TextScene::new().with(
        TextBlock::new(
            "AV ffi\ne\u{301} x",
            Rect::new(Point::new(10.0, 20.0), Size::new(300.0, 80.0)),
        )
        .size(17.0)
        .line_height(22.5)
        .clip(Rect::new(
            Point::new(-200.0, -200.0),
            Size::new(1000.0, 1000.0),
        )),
    );
    (engine, scene)
}

/// Checks raster identity and physical position without coupling to private retained fields.
fn same_raster(actual: &PreparedGlyph, expected: &PreparedGlyph) {
    assert_eq!(actual.key, expected.key);
    assert_eq!((actual.x, actual.y), (expected.x, expected.y));
    assert_eq!(actual.color, expected.color);
    assert_eq!((actual.start, actual.end), (expected.start, expected.end));
}

/// Fractional and negative scroll positions must match a freshly prepared scene.
#[test]
fn reposition_matches_full_origin_preparation_at_every_dpi() {
    let (mut engine, scene) = fixture();
    for scale in [1.0, 1.25, 1.5, 2.0] {
        for fraction in [0.0, 0.25, 0.5, 0.75] {
            let mut prepared = engine.prepare(&scene, scale);
            let mut moved = scene.clone();
            let origin = Point::new(-30.0 + fraction, -100.0 + fraction);
            moved.blocks_mut()[0].bounds.origin = origin;
            prepared.reposition_block(0, origin, moved.blocks()[0].clip);
            let fresh = engine.prepare(&moved, scale);
            assert_eq!(prepared, fresh);
        }
    }
    assert_eq!(engine.stats().layout_misses, 1);
}

/// Supported affine transforms must reraster at the target size and subpixel origin.
#[test]
fn translation_and_uniform_scale_match_target_dpi() {
    let (mut engine, scene) = fixture();
    for dpi in [1.0, 1.25, 1.5, 2.0] {
        let prepared = engine.prepare(&scene, dpi);
        for factor in [0.75, 1.0, 1.5, 2.0] {
            let transform = Affine2D {
                matrix: [factor, 0.0, 0.0, factor],
                translation: Point::new(0.25, -60.5),
            };
            let target_scale = dpi * factor;
            let mut moved = scene.clone();
            moved.blocks_mut()[0].bounds.origin.x += 0.25 / target_scale;
            moved.blocks_mut()[0].bounds.origin.y -= 60.5 / target_scale;
            let fresh = engine.prepare(&moved, target_scale);
            for (glyph, expected) in prepared.glyphs.iter().zip(&fresh.glyphs) {
                let transformed = glyph.transform_for_raster(transform).unwrap();
                same_raster(&transformed, expected);
                assert_eq!(transformed.clip, glyph.clip);
            }
        }
    }
}

/// Unsupported or invalid transforms leave the renderer its geometric fallback path.
#[test]
fn rejects_transforms_that_cannot_use_axis_aligned_rasters() {
    let (mut engine, scene) = fixture();
    let glyph = engine.prepare(&scene, 1.0).glyphs[0];
    for matrix in [
        [-1.0, 0.0, 0.0, -1.0],
        [0.0, 0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0, 2.0],
        [1.0, 0.1, 0.0, 1.0],
        [1.0, 0.0, 0.1, 1.0],
        [f32::NAN, 0.0, 0.0, 1.0],
    ] {
        assert!(
            glyph
                .transform_for_raster(Affine2D {
                    matrix,
                    translation: Point::new(0.0, 0.0)
                })
                .is_none()
        );
    }
    assert!(
        glyph
            .transform_for_raster(Affine2D::translation(f32::INFINITY, 0.0))
            .is_none()
    );
    assert_eq!(glyph.transform_for_raster(Affine2D::IDENTITY), Some(glyph));
}
