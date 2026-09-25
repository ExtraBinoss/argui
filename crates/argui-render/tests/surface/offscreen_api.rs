use argui_core::{Affine2D, Color, Point, Rect, Size};
use argui_paint::{Border, ClipChain, CornerRadii, DisplayList, Fill, Quad};
use argui_render::{RendererConfig, RendererError, SurfaceRenderer};
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
