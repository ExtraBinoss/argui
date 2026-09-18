use argui_core::{Affine2D, Point, Rect, Size};
use argui_paint::{
    Border, ClipChain, Color, CornerRadii, DisplayCommand, DisplayList, Fill, GpuCanvasId,
    GpuCanvasPrimitive, ImageSampling, LayerStyle, ProfileDomain, Quad, RenderObjectId,
};

#[test]
fn display_lists_preserve_cross_primitive_order() {
    let bounds = Rect::new(Point::default(), Size::new(100.0, 50.0));
    let quad = Quad {
        bounds,
        background: Some(Fill::Solid(Color::WHITE)),
        border: Border::all(0.0, Color::TRANSPARENT),
        radii: CornerRadii::default(),
        opacity: 1.0,
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
    };
    let mut list = DisplayList::new();
    list.push_quad(quad.clone());
    list.push_text(3);
    let canvas = GpuCanvasPrimitive {
        canvas: GpuCanvasId::fresh(),
        object: RenderObjectId::new(ProfileDomain::Ui, 9),
        slot: 2,
        bounds,
        content_revision: 7,
        resolution_scale: 1.5,
        sampling: ImageSampling::Nearest,
        opacity: 0.75,
        radii: CornerRadii::all(4.0),
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
    };
    list.push_gpu_canvas(canvas.clone());

    assert_eq!(list.quad_count(), 1);
    assert_eq!(list.gpu_canvas_count(), 1);
    assert_eq!(
        list.commands(),
        &[
            DisplayCommand::Quad(quad),
            DisplayCommand::Text {
                block: 3,
                transform: Affine2D::IDENTITY,
                clips: ClipChain::default(),
                backdrop: None,
            },
            DisplayCommand::GpuCanvas(canvas.clone()),
        ]
    );

    list.begin_layer(LayerStyle::new(Default::default()));
    list.end_layer();
    assert_eq!(list.quad_count(), 1);
    assert_eq!(list.gpu_canvas_count(), 1);

    list.extend([DisplayCommand::GpuCanvas(canvas)]);
    assert_eq!(list.gpu_canvas_count(), 2);

    list.clear();
    assert_eq!(list.quad_count(), 0);
    assert_eq!(list.gpu_canvas_count(), 0);
    assert!(list.commands().is_empty());
}

#[test]
fn gpu_canvas_ids_are_process_local_and_unique() {
    assert_ne!(GpuCanvasId::fresh(), GpuCanvasId::fresh());
}
