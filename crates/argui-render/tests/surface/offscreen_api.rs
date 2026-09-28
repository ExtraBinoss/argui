use argui_core::{Affine2D, Color, Point, Rect, Size};
use argui_paint::{Border, ClipChain, ClipRegion, CornerRadii, DisplayList, Fill, Quad};
use argui_render::{RenderStatus, RendererConfig, RendererError, SurfaceRenderer};
use argui_text::{PreparedText, TextEngine};

#[test]
fn full_scene_renders_to_a_copyable_windowless_texture() {
    let mut renderer = match pollster::block_on(SurfaceRenderer::new_offscreen(
        48,
        32,
        RendererConfig::default().profiling(true),
    )) {
        Ok(renderer) => renderer,
        Err(RendererError::AdapterRequest(_)) => return,
        Err(error) => panic!("offscreen renderer: {error}"),
    };
    let mut scene = DisplayList::new();
    scene.push_quad(Quad {
        bounds: Rect::new(Point::new(4.0, 4.0), Size::new(20.0, 20.0)),
        background: Some(Fill::Solid(Color::from_hex("#ff0000").unwrap())),
        border: Border::all(0.0, Color::TRANSPARENT),
        radii: CornerRadii::default(),
        opacity: 1.0,
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
    });
    renderer
        .render_ui(
            &mut TextEngine::new(),
            &PreparedText::default(),
            &scene,
            1.0,
        )
        .unwrap();
    let pixels = renderer.read_offscreen_rgba().unwrap();
    if renderer.last_profile().adapter.timestamp_queries {
        assert!(renderer.last_profile().gpu.is_some());
    }
    assert_eq!(pixels.len(), 48 * 32 * 4);
    assert!(pixels.as_chunks::<4>().0.iter().any(|pixel| {
        u16::from(pixel[0]) > u16::from(pixel[1]) + 50
            && u16::from(pixel[0]) > u16::from(pixel[2]) + 50
    }));
    assert!(renderer.resize(32, 16));
    renderer
        .render_ui(
            &mut TextEngine::new(),
            &PreparedText::default(),
            &scene,
            1.0,
        )
        .unwrap();
    assert_eq!(renderer.read_offscreen_rgba().unwrap().len(), 32 * 16 * 4);
}

/// Checks one-pixel opaque borders and rounded clipping on the physical pixel grid.
///
/// # Panics
/// Panics on a rendering failure or incorrect pixel coverage. A missing adapter
/// also fails when native checks are explicitly enabled with `ARGUI_NATIVE_TESTS=1`.
#[test]
fn opaque_rounded_borders_and_clips_keep_grid_aligned_edges_solid() {
    let mut renderer = match pollster::block_on(SurfaceRenderer::new_offscreen(
        64,
        48,
        RendererConfig::default().clear_color(Color::TRANSPARENT),
    )) {
        Ok(renderer) => renderer,
        Err(RendererError::AdapterRequest(error))
            if std::env::var("ARGUI_NATIVE_TESTS").as_deref() != Ok("1") =>
        {
            eprintln!("skipping rounded-border readback without a GPU adapter: {error}");
            return;
        }
        Err(error) => panic!("offscreen rounded-border renderer: {error}"),
    };
    let bounds = Rect::new(Point::new(8.0, 8.0), Size::new(48.0, 32.0));
    let radii = CornerRadii::all(7.0);
    let clips = ClipChain::from_regions([ClipRegion::rounded(bounds, Affine2D::IDENTITY, radii)]);
    let rounded = Quad {
        bounds,
        background: Some(Fill::Solid(Color::from_hex("#4080ff").unwrap())),
        border: Border::all(1.0, Color::WHITE),
        radii,
        opacity: 1.0,
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
    };
    let clipped = Quad {
        clips: clips.clone(),
        ..rounded.clone()
    };
    let clipped_background = Quad {
        bounds: Rect::new(Point::default(), Size::new(64.0, 48.0)),
        background: Some(Fill::Solid(Color::WHITE)),
        border: Border::all(0.0, Color::TRANSPARENT),
        radii: CornerRadii::default(),
        clips,
        ..rounded.clone()
    };
    let mut text = TextEngine::from_embedded_fonts([], "sans-serif", "serif", "monospace");
    for (name, quad, edge) in [
        ("rounded border", rounded, [255; 4]),
        ("rounded border with identical clip", clipped, [255; 4]),
        (
            "opaque background behind clip",
            clipped_background,
            [255; 4],
        ),
    ] {
        let mut scene = DisplayList::new();
        scene.push_quad(quad);
        assert_eq!(
            renderer
                .render_ui(&mut text, &PreparedText::default(), &scene, 1.0)
                .unwrap(),
            RenderStatus::Presented,
            "{name} must submit a fresh offscreen frame",
        );
        let pixels = renderer.read_offscreen_rgba().unwrap();
        assert_rounded_coverage(name, &pixels, edge);
    }
}

/// Reads `(x, y)` from tightly packed RGBA8 data with a fixed 64-pixel row width.
///
/// # Panics
/// Panics if `pixels` does not contain the requested pixel.
fn rgba_at(pixels: &[u8], x: usize, y: usize) -> [u8; 4] {
    let offset = (y * 64 + x) * 4;
    pixels[offset..offset + 4].try_into().unwrap()
}

/// Checks the radius-seven shape spanning `(8, 8)` to `(56, 40)` in `pixels`.
/// `edge` is the expected opaque color at the four straight-edge midpoints;
/// `name` identifies the scene in failed assertions.
///
/// # Panics
/// Panics on an incorrect buffer size, edge, interior, exterior, or corner pixel.
fn assert_rounded_coverage(name: &str, pixels: &[u8], edge: [u8; 4]) {
    assert_eq!(pixels.len(), 64 * 48 * 4, "{name}: RGBA8 buffer size");
    for (x, y) in [(32, 8), (32, 39), (8, 24), (55, 24)] {
        assert_eq!(
            rgba_at(pixels, x, y),
            edge,
            "{name}: opaque edge ({x}, {y})"
        );
    }
    let interior = rgba_at(pixels, 32, 24);
    assert_eq!(
        [interior[2], interior[3]],
        [255; 2],
        "{name}: opaque interior"
    );
    for (x, y) in [
        (32, 7),
        (32, 40),
        (7, 24),
        (56, 24),
        (8, 8),
        (55, 8),
        (8, 39),
        (55, 39),
    ] {
        assert_eq!(
            rgba_at(pixels, x, y),
            [0; 4],
            "{name}: transparent exterior ({x}, {y})",
        );
    }
    for (x, y) in [(8, 8), (49, 8), (8, 33), (49, 33)] {
        assert!(
            (y..y + 7).any(|row| {
                (x..x + 7).any(|column| {
                    let alpha = rgba_at(pixels, column, row)[3];
                    alpha > 0 && alpha < 255
                })
            }),
            "{name}: corner ({x}, {y}) must retain partial antialiasing coverage",
        );
    }
    // Every source color has a full blue channel. After blending onto clear
    // transparent pixels, sRGB-encoded blue cannot be darker than linear alpha.
    for (index, pixel) in pixels.chunks_exact(4).enumerate() {
        assert!(
            pixel[2] >= pixel[3],
            "{name}: black contamination at ({}, {}): {pixel:?}",
            index % 64,
            index / 64,
        );
    }
}
