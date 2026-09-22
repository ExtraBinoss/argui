use std::sync::Arc;

use argui_core::{Affine2D, Color, Point, Rect, Size};
use argui_paint::{
    Border, ClipChain, CornerRadii, DisplayList, Fill, Filter, GpuCanvasId, GpuCanvasPrimitive,
    ImageFit, ImageSampling, LayerStyle, ProfileDomain, Quad, RenderObjectId, VectorAsset,
    VectorId, VectorPrimitive,
};
use argui_render::{RenderStatus, RendererError, SurfaceRenderer};
use argui_text::{PreparedText, TextEngine};
use winit::window::Window;

const NATIVE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

pub(super) type FrameResult = Result<RenderStatus, RendererError>;

pub(super) type FramePump<'a> =
    dyn FnMut(&mut dyn FnMut() -> FrameResult) -> Option<FrameResult> + 'a;

pub(super) fn canvas_list(
    id: GpuCanvasId,
    revision: u64,
    size: Size,
    slot: u32,
    effect: bool,
) -> DisplayList {
    let mut list = DisplayList::new();
    list.push_quad(Quad {
        bounds: bounds(size.width),
        background: Some(Fill::Solid(Color::BLACK)),
        border: Border::all(0.0, Color::TRANSPARENT),
        radii: CornerRadii::default(),
        opacity: 1.0,
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
    });
    if effect {
        list.begin_layer(LayerStyle::new(bounds(size.width)).filter(Filter::Blur(2.0)));
    }
    list.push_gpu_canvas(canvas_primitive(id, revision, size, slot));
    if effect {
        list.end_layer();
    }
    list.push_quad(Quad {
        bounds: Rect::new(Point::new(8.0, 8.0), Size::new(10.0, 10.0)),
        background: Some(Fill::Solid(Color::WHITE)),
        border: Border::all(0.0, Color::TRANSPARENT),
        radii: CornerRadii::all(2.0),
        opacity: 0.5,
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
    });
    list
}

pub(super) fn canvas_primitive(
    id: GpuCanvasId,
    revision: u64,
    size: Size,
    slot: u32,
) -> GpuCanvasPrimitive {
    GpuCanvasPrimitive {
        canvas: id,
        object: RenderObjectId::new(ProfileDomain::Ui, 777),
        slot,
        bounds: Rect::new(Point::new(0.0, 0.0), size),
        content_revision: revision,
        resolution_scale: 1.0,
        sampling: if revision.is_multiple_of(2) {
            ImageSampling::Nearest
        } else {
            ImageSampling::Linear
        },
        opacity: 0.9,
        radii: CornerRadii::all(3.0),
        transform: Affine2D::IDENTITY,
        clips: ClipChain::from_regions([argui_paint::ClipRegion::new(
            Rect::new(Point::default(), size),
            Affine2D::IDENTITY,
        )]),
    }
}

pub(super) fn render(
    renderer: &mut SurfaceRenderer,
    list: &DisplayList,
    window: &Window,
    pump: &mut FramePump<'_>,
) -> Result<(), RendererError> {
    render_at_scale(renderer, list, window, pump, 1.0)
}

/// Presents `list` with `renderer` in `window` at logical-to-physical `scale_factor`.
///
/// `pump` runs the render callback on a window redraw. Returns after a frame is
/// presented.
///
/// # Errors
/// Returns an error if frame acquisition or rendering fails.
pub(super) fn render_at_scale(
    renderer: &mut SurfaceRenderer,
    list: &DisplayList,
    window: &Window,
    pump: &mut FramePump<'_>,
    scale_factor: f32,
) -> Result<(), RendererError> {
    let mut text = TextEngine::from_embedded_fonts([], "sans-serif", "serif", "monospace");
    let deadline = web_time::Instant::now() + NATIVE_TIMEOUT;
    loop {
        window.request_redraw();
        let Some(status) = pump(&mut || {
            renderer.render_ui_notified(
                &mut text,
                &PreparedText::default(),
                list,
                scale_factor,
                || {},
            )
        }) else {
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
            "surface did not present within {NATIVE_TIMEOUT:?}: {} commands, atlas {:?}, size {:?}",
            list.commands().len(),
            renderer.last_profile().vector_atlas,
            window.inner_size(),
        );
    }
}

pub(super) fn asset() -> VectorAsset {
    VectorAsset {
        id: VectorId::fresh(),
        size: Size::new(16.0, 16.0),
        svg: Arc::from(br#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"><rect width="16" height="16" fill="red"/></svg>"#.as_slice()),
        tintable: false,
    }
}

pub(super) fn bounds(size: f32) -> Rect {
    Rect::new(Point::default(), Size::new(size, size))
}

pub(super) fn colored_quad(x: f32, color: Color) -> Quad {
    Quad {
        bounds: Rect::new(Point::new(x, 8.0), Size::new(16.0, 16.0)),
        background: Some(Fill::Solid(color)),
        border: Border::all(0.0, Color::TRANSPARENT),
        radii: CornerRadii::default(),
        opacity: 1.0,
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
    }
}

pub(super) fn vector(id: VectorId, size: f32) -> VectorPrimitive {
    VectorPrimitive {
        vector: id,
        bounds: bounds(size),
        fit: ImageFit::Contain,
        color: Color::WHITE,
        opacity: 1.0,
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
    }
}

pub(super) fn vectors(assets: &[VectorAsset], size: f32) -> DisplayList {
    let mut list = DisplayList::new();
    for asset in assets {
        list.push_vector(vector(asset.id, size));
    }
    list
}
