use argui_core::{Affine2D, Color, Point, Rect, Size};
use argui_paint::{
    Border, ClipChain, CompositorId, CompositorLayer, CornerRadii, DisplayList, Fill, LayerStyle,
    Quad, Shadow,
};
use argui_render::{RendererConfig, RendererError, SurfaceAlphaMode, SurfaceRenderer};
use argui_text::{PreparedText, TextEngine};

/// Creates a copyable transparent native-output target, failing explicit GPU gates.
fn renderer() -> Option<SurfaceRenderer> {
    match pollster::block_on(SurfaceRenderer::new_offscreen(
        88,
        64,
        RendererConfig::default()
            .clear_color(Color::TRANSPARENT)
            .surface_alpha(SurfaceAlphaMode::Transparent),
    )) {
        Ok(renderer) => Some(renderer),
        Err(RendererError::AdapterRequest(error))
            if std::env::var("ARGUI_NATIVE_TESTS").as_deref() != Ok("1") =>
        {
            eprintln!("skipping native-alpha readback without a GPU adapter: {error}");
            None
        }
        Err(error) => panic!("transparent output: {error}"),
    }
}

/// Builds a dark, rounded grey border with an optional shadow under one retained fade.
fn scene(opacity: f32, shadow: bool) -> DisplayList {
    let mut list = DisplayList::new();
    list.begin_compositor(CompositorLayer::new(
        CompositorId::new(10),
        Rect::new(Point::default(), Size::new(88.0, 64.0)),
        Affine2D::IDENTITY,
        Affine2D::IDENTITY,
        opacity,
    ));
    let bounds = Rect::new(Point::new(16.0, 12.0), Size::new(56.0, 36.0));
    if shadow {
        list.begin_layer(LayerStyle::new(bounds).shadow(Shadow::drop(
            [0.0, 3.0],
            5.0,
            Color::from_srgba8(0, 0, 0, 128),
        )));
    }
    list.push_quad(Quad {
        bounds,
        background: Some(Fill::Solid(Color::from_srgba8(43, 43, 46, 255))),
        border: Border::all(1.0, Color::from_srgba8(84, 84, 86, 255)),
        radii: CornerRadii::all(7.0),
        opacity: 1.0,
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
    });
    if shadow {
        list.end_layer();
    }
    list.end_compositor();
    list
}

/// Renders actual GPU primitives and returns the encoded native RGBA bytes.
fn render(renderer: &mut SurfaceRenderer, opacity: f32, shadow: bool) -> Vec<u8> {
    renderer
        .render_ui(
            &mut TextEngine::new(),
            &PreparedText::default(),
            &scene(opacity, shadow),
            1.0,
        )
        .unwrap();
    renderer.read_offscreen_rgba().unwrap()
}

/// Reads one pixel from the 88-pixel-wide native output.
fn pixel(bytes: &[u8], x: usize, y: usize) -> [u8; 4] {
    bytes[(y * 88 + x) * 4..(y * 88 + x + 1) * 4]
        .try_into()
        .unwrap()
}

#[test]
fn transparent_rounded_corners_preserve_the_authored_grey_instead_of_turning_white() {
    let Some(mut renderer) = renderer() else {
        return;
    };
    let bytes = render(&mut renderer, 1.0, false);
    let mut partial = 0;
    for y in 12..19 {
        for x in 16..23 {
            let rgba = pixel(&bytes, x, y);
            if (24..230).contains(&rgba[3]) {
                partial += 1;
                for channel in &rgba[..3] {
                    let straight = f32::from(*channel) * 255.0 / f32::from(rgba[3]);
                    assert!(
                        (35.0..=92.0).contains(&straight),
                        "corner ({x}, {y}) is brighter than its grey border: {rgba:?}"
                    );
                }
            }
        }
    }
    assert!(
        partial >= 4,
        "rounded corners must contain antialiasing pixels"
    );
    assert_eq!(pixel(&bytes, 44, 12), [84, 84, 86, 255]);
    assert_eq!(pixel(&bytes, 44, 30), [43, 43, 46, 255]);
    assert_eq!(pixel(&bytes, 16, 12), [0; 4]);
}

#[test]
fn the_panel_border_and_shadow_share_the_same_native_fade() {
    let Some(mut renderer) = renderer() else {
        return;
    };
    let full = render(&mut renderer, 1.0, true);
    let shadow = (48..61)
        .flat_map(|y| (20..68).map(move |x| (x, y)))
        .find(|&(x, y)| pixel(&full, x, y)[3] >= 16)
        .expect("visible shadow");
    for opacity in [0.0, 0.25, 0.5, 0.8, 1.0] {
        let faded = render(&mut renderer, opacity, true);
        for (label, (x, y)) in [
            ("panel", (44, 30)),
            ("border", (44, 12)),
            ("shadow", shadow),
        ] {
            let reference = pixel(&full, x, y);
            let actual = pixel(&faded, x, y);
            for channel in 0..4 {
                let expected = f32::from(reference[channel]) * opacity;
                assert!(
                    (f32::from(actual[channel]) - expected).abs() <= 3.0,
                    "{label} at opacity {opacity}: {actual:?}, reference {reference:?}"
                );
            }
        }
    }
    assert!(
        render(&mut renderer, 0.0, true)
            .iter()
            .all(|byte| *byte == 0)
    );
}

#[test]
fn first_frame_readiness_wakes_after_gpu_completion_without_blocking_the_caller() {
    let Some(mut renderer) = renderer() else {
        return;
    };
    render(&mut renderer, 0.25, true);
    let (ready, completion) = std::sync::mpsc::channel();
    renderer.after_frame_ready(move || ready.send(()).unwrap());
    completion
        .recv_timeout(std::time::Duration::from_secs(5))
        .expect("GPU readiness callback");
}
