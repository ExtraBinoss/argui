use argui_core::{Affine2D, Color, Point, Rect, Size};
use argui_paint::{
    Border, ClipChain, CompositorId, CompositorLayer, CornerRadii, DisplayList, EffectId,
    EffectInstance, Fill, Filter, GpuCanvasId, GpuCanvasPrimitive, ImageFit, ImageId,
    ImagePrimitive, ImageSampling, LayerMask, LayerStyle, ProfileDomain, Quad, Refraction,
    RenderObjectId, Shadow, VectorId, VectorPrimitive,
};
use argui_render::analyze_display_list;

fn bounds() -> Rect {
    Rect::new(Point::new(10.0, 12.0), Size::new(100.0, 80.0))
}

fn quad() -> Quad {
    Quad {
        bounds: bounds(),
        background: Some(Fill::Solid(Color::WHITE)),
        border: Border::all(0.0, Color::TRANSPARENT),
        radii: CornerRadii::default(),
        opacity: 1.0,
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
    }
}

fn image() -> ImagePrimitive {
    ImagePrimitive {
        bounds: bounds(),
        image: ImageId(7),
        fit: ImageFit::Contain,
        sampling: ImageSampling::Linear,
        opacity: 1.0,
        radii: CornerRadii::default(),
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
    }
}

fn vector() -> VectorPrimitive {
    VectorPrimitive {
        vector: VectorId(9),
        bounds: bounds(),
        fit: ImageFit::Contain,
        color: Color::WHITE,
        opacity: 1.0,
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
    }
}

fn canvas(revision: u64) -> GpuCanvasPrimitive {
    GpuCanvasPrimitive {
        canvas: GpuCanvasId::fresh(),
        object: RenderObjectId::new(ProfileDomain::Ui, 17),
        slot: 0,
        bounds: bounds(),
        content_revision: revision,
        resolution_scale: 1.0,
        sampling: ImageSampling::Linear,
        opacity: 0.75,
        radii: CornerRadii::all(4.0),
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
    }
}

#[test]
fn analysis_counts_merged_draws_nested_layers_filters_and_custom_effects() {
    let custom = EffectInstance::new(
        EffectId::new("test.effect"),
        [("amount", argui_paint::EffectValue::F32(1.0))],
    );
    let filters = [
        Filter::Blur(0.0),
        Filter::Brightness(1.1),
        Filter::Contrast(0.9),
        Filter::Saturation(0.8),
        Filter::HueRotate(0.2),
        Filter::Opacity(0.7),
        Filter::ColorMatrix([1.0; 20]),
        Filter::Blur(4.0),
        Filter::Refraction(Refraction {
            strength: 2.0,
            chromatic_aberration: 0.4,
            edge: 0.6,
        }),
        Filter::Effect(custom.clone()),
    ];
    let layer = filters
        .into_iter()
        .fold(LayerStyle::new(bounds()), LayerStyle::filter)
        .backdrop(Filter::Effect(custom))
        .shadow(Shadow::drop([2.0, 3.0], 5.0, Color::BLACK))
        .mask(LayerMask::Bounds);
    let mut list = DisplayList::new();
    list.push_quad(quad());
    list.push_quad(quad());
    list.push_text(0);
    list.push_text(3);
    list.push_image(image());
    list.push_image(image());
    list.push_vector(vector());
    list.begin_layer(layer);
    list.push_quad(quad());
    list.begin_layer(LayerStyle::new(bounds()).opacity(0.0));
    list.push_quad(quad());
    list.end_layer();
    list.end_layer();

    let visible_commands = [4..11, 11..12];
    let analysis = analyze_display_list(&list, &visible_commands, [320.0, 240.0], 2.0, 41).unwrap();
    assert_eq!(analysis.stats.layers, 1);
    assert_eq!(analysis.stats.offscreen_layers, 1);
    assert_eq!(analysis.stats.draw_batches, 5);
    assert_eq!(analysis.stats.filter_passes, 7);
    assert!(analysis.stats.offscreen_pixels > 0);
    assert_eq!(
        analysis.custom_effects,
        [EffectId::new("test.effect"), EffectId::new("test.effect")]
    );
}

#[test]
fn analysis_handles_empty_clipped_and_malformed_display_lists() {
    let empty = analyze_display_list(&DisplayList::new(), &[], [0.0, 0.0], 1.0, 0).unwrap();
    assert_eq!(empty.stats, Default::default());

    let mut clipped = DisplayList::new();
    clipped.begin_layer(
        LayerStyle::new(Rect::new(Point::new(500.0, 500.0), Size::new(20.0, 20.0)))
            .filter(Filter::Blur(2.0)),
    );
    clipped.push_quad(quad());
    clipped.end_layer();
    let clipped = analyze_display_list(&clipped, &[], [100.0, 100.0], 1.0, 0).unwrap();
    assert_eq!(clipped.stats.offscreen_pixels, 0);
    assert_eq!(clipped.stats.draw_batches, 1);

    let mut unexpected_end = DisplayList::new();
    unexpected_end.end_layer();
    assert!(analyze_display_list(&unexpected_end, &[], [1.0, 1.0], 1.0, 0).is_err());
    let mut unclosed = DisplayList::new();
    unclosed.begin_layer(LayerStyle::new(bounds()));
    assert!(analyze_display_list(&unclosed, &[], [1.0, 1.0], 1.0, 0).is_err());
}

#[test]
fn canvas_draws_keep_display_order_and_participate_in_effect_layers() {
    let mut list = DisplayList::new();
    list.push_quad(quad());
    list.push_gpu_canvas(canvas(1));
    list.push_quad(quad());
    list.begin_layer(LayerStyle::new(bounds()).filter(Filter::Blur(3.0)));
    list.push_gpu_canvas(canvas(2));
    list.end_layer();

    let analysis = analyze_display_list(&list, &[], [320.0, 240.0], 1.0, 19).unwrap();
    assert_eq!(analysis.stats.draw_batches, 4);
    assert_eq!(analysis.stats.offscreen_layers, 1);
    assert_eq!(analysis.stats.filter_passes, 1);
}

#[test]
fn identity_compositor_layers_are_pre_promoted_for_future_animation_frames() {
    let mut list = DisplayList::new();
    list.begin_compositor(CompositorLayer::new(
        CompositorId::new(9),
        bounds(),
        Affine2D::IDENTITY,
        Affine2D::IDENTITY,
        1.0,
    ));
    list.push_quad(quad());
    list.end_compositor();

    let analysis = analyze_display_list(&list, &[], [320.0, 240.0], 1.0, 4).unwrap();
    assert_eq!(analysis.stats.layers, 1);
    assert_eq!(analysis.stats.offscreen_layers, 1);
    assert_eq!(analysis.stats.draw_batches, 1);
}
