use argui_core::{Affine2D, Color, Point, Rect, Size};
use argui_paint::{
    Border, ClipChain, ClipRegion, CompositorId, CompositorLayer, CornerRadii, DisplayList, Fill,
    GpuCanvasId, GpuCanvasPrimitive, ImageFit, ImageId, ImagePrimitive, ImageSampling, LayerStyle,
    ProfileDomain, Quad, RenderObjectId, VectorId, VectorPrimitive,
};
use argui_render::{DamagePlan, DamageRegion, DamageSnapshot, DamageTracking};
use argui_text::{PreparedText, TextBlock, TextDecoration, TextEngine, TextScene, UnderlineStyle};

const NOTO_SANS: &[u8] = include_bytes!("../../argui-web-demo/assets/fonts/NotoSans-Regular.ttf");

#[test]
fn empty_damage_is_unchanged_and_disabled_tracking_is_full() {
    assert_eq!(
        DamagePlan::resolve([], [800, 600], DamageTracking::enabled()),
        DamagePlan::Unchanged
    );
    assert_eq!(
        DamagePlan::resolve(
            [DamageRegion::new(10, 10, 20, 20)],
            [800, 600],
            DamageTracking::disabled(),
        ),
        DamagePlan::Full
    );
}

#[test]
fn touching_regions_merge_before_thresholds_are_applied() {
    let plan = DamagePlan::resolve(
        [
            DamageRegion::new(0, 0, 64, 64),
            DamageRegion::new(64, 0, 32, 64),
        ],
        [256, 256],
        DamageTracking::enabled().max_regions(1),
    );
    assert_eq!(
        plan,
        DamagePlan::Partial(vec![DamageRegion::new(0, 0, 96, 64)])
    );
}

#[test]
fn region_count_area_and_viewport_clipping_select_full_frames() {
    assert_eq!(
        DamagePlan::resolve(
            [
                DamageRegion::new(0, 0, 16, 16),
                DamageRegion::new(96, 96, 16, 16),
            ],
            [128, 128],
            DamageTracking::enabled().max_regions(1),
        ),
        DamagePlan::Full
    );
    assert_eq!(
        DamagePlan::resolve(
            [DamageRegion::new(0, 0, 100, 100)],
            [100, 100],
            DamageTracking::enabled().max_area_ratio(0.25),
        ),
        DamagePlan::Full
    );
    assert_eq!(
        DamagePlan::resolve(
            [DamageRegion::new(90, 90, 40, 40)],
            [100, 100],
            DamageTracking::enabled(),
        ),
        DamagePlan::Partial(vec![DamageRegion::new(64, 64, 36, 36)])
    );
}

#[test]
fn scene_snapshots_find_moved_and_inserted_primitives() {
    let mut previous = DisplayList::new();
    previous.push_quad(quad(8.0, 8.0, 16.0));
    previous.push_quad(quad(160.0, 8.0, 16.0));
    let mut moved = DisplayList::new();
    moved.push_quad(quad(48.0, 8.0, 16.0));
    moved.push_quad(quad(160.0, 8.0, 16.0));
    let previous = snapshot(&previous, [256, 128], 1.0);
    let moved_snapshot = snapshot(&moved, [256, 128], 1.0);
    assert_eq!(
        previous.compare(&moved_snapshot, DamageTracking::enabled()),
        DamagePlan::Partial(vec![DamageRegion::new(0, 0, 96, 32)])
    );

    let mut inserted = DisplayList::new();
    inserted.push_quad(quad(48.0, 8.0, 16.0));
    inserted.push_quad(quad(96.0, 8.0, 16.0));
    inserted.push_quad(quad(160.0, 8.0, 16.0));
    assert_eq!(
        moved_snapshot.compare(
            &snapshot(&inserted, [256, 128], 1.0),
            DamageTracking::enabled()
        ),
        DamagePlan::Partial(vec![DamageRegion::new(64, 0, 64, 32)])
    );
    assert_eq!(
        snapshot(&inserted, [256, 128], 1.0).compare(&moved_snapshot, DamageTracking::enabled()),
        DamagePlan::Partial(vec![DamageRegion::new(64, 0, 64, 32)])
    );
}

#[test]
fn scene_snapshots_handle_identity_and_incompatible_viewports() {
    let mut list = DisplayList::new();
    list.push_quad(quad(8.0, 8.0, 16.0));
    let baseline = snapshot(&list, [256, 128], 1.0);
    assert_eq!(
        baseline.compare(&baseline.clone(), DamageTracking::enabled()),
        DamagePlan::Unchanged
    );
    assert_eq!(
        baseline.compare(&snapshot(&list, [512, 256], 2.0), DamageTracking::enabled()),
        DamagePlan::Full
    );
}

#[test]
fn snapshots_cover_every_command_and_prepared_text_changes() {
    let canvas = GpuCanvasId::fresh();
    let list = mixed_list(canvas);
    let baseline = snapshot(&list, [512, 256], 1.0);
    assert_eq!(
        baseline.compare(&snapshot(&list, [512, 256], 1.0), DamageTracking::enabled()),
        DamagePlan::Unchanged
    );

    let mut engine =
        TextEngine::from_embedded_fonts([NOTO_SANS], "Noto Sans", "Noto Sans", "Noto Sans");
    let prepared = |engine: &mut TextEngine, value: &str| {
        let mut block = TextBlock::new(
            value,
            Rect::new(Point::new(16.0, 16.0), Size::new(120.0, 32.0)),
        );
        block.style.decoration = TextDecoration {
            underline: UnderlineStyle::Single,
            strikethrough: true,
            ..TextDecoration::default()
        };
        engine.prepare(&TextScene::new().with(block), 1.0)
    };
    let mut text_list = DisplayList::new();
    text_list.push_text_transformed(
        0,
        Affine2D::translation(4.0, 2.0),
        ClipChain::from_regions([ClipRegion::new(
            Rect::new(Point::default(), Size::new(160.0, 48.0)),
            Affine2D::IDENTITY,
        )]),
    );
    let text_bounds = [Some(Rect::new(
        Point::new(16.0, 16.0),
        Size::new(32.0, 24.0),
    ))];
    let first = DamageSnapshot::capture(
        &text_list,
        &prepared(&mut engine, "Alpha"),
        &text_bounds,
        [256, 128],
        1.0,
    );
    let second = DamageSnapshot::capture(
        &text_list,
        &prepared(&mut engine, "Beta"),
        &text_bounds,
        [256, 128],
        1.0,
    );
    assert!(matches!(
        first.compare(&second, DamageTracking::enabled()),
        DamagePlan::Partial(_)
    ));
}

fn snapshot(list: &DisplayList, viewport: [u32; 2], scale: f32) -> DamageSnapshot {
    DamageSnapshot::capture(list, &PreparedText::default(), &[], viewport, scale)
}

fn quad(x: f32, y: f32, size: f32) -> Quad {
    Quad {
        bounds: Rect::new(Point::new(x, y), Size::new(size, size)),
        background: Some(Fill::Solid(Color::WHITE)),
        border: Border::all(0.0, Color::TRANSPARENT),
        radii: CornerRadii::default(),
        opacity: 1.0,
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
    }
}

fn mixed_list(canvas: GpuCanvasId) -> DisplayList {
    let bounds = Rect::new(Point::new(8.0, 8.0), Size::new(32.0, 24.0));
    let clips = ClipChain::from_regions([ClipRegion::new(
        Rect::new(Point::new(4.0, 4.0), Size::new(80.0, 64.0)),
        Affine2D::IDENTITY,
    )]);
    let mut list = DisplayList::new();
    list.push_quad(quad(8.0, 8.0, 16.0));
    let mut fully_clipped = quad(8.0, 8.0, 16.0);
    fully_clipped.clips = ClipChain::from_regions([ClipRegion::new(
        Rect::new(Point::new(400.0, 200.0), Size::new(8.0, 8.0)),
        Affine2D::IDENTITY,
    )]);
    list.push_quad(fully_clipped);
    list.push_image(ImagePrimitive {
        bounds,
        image: ImageId(1),
        fit: ImageFit::Contain,
        sampling: ImageSampling::Linear,
        opacity: 1.0,
        radii: CornerRadii::default(),
        transform: Affine2D::translation(2.0, 0.0),
        clips: clips.clone(),
    });
    list.push_vector(VectorPrimitive {
        vector: VectorId(2),
        bounds,
        fit: ImageFit::Contain,
        color: Color::WHITE,
        opacity: 1.0,
        transform: Affine2D::IDENTITY,
        clips: clips.clone(),
    });
    list.push_gpu_canvas(GpuCanvasPrimitive {
        canvas,
        object: RenderObjectId::new(ProfileDomain::Ui, 3),
        slot: 0,
        bounds,
        content_revision: 4,
        resolution_scale: 1.0,
        sampling: ImageSampling::Nearest,
        opacity: 1.0,
        radii: CornerRadii::default(),
        transform: Affine2D::IDENTITY,
        clips,
    });
    list.begin_layer(LayerStyle::new(bounds));
    list.push_quad(quad(12.0, 12.0, 8.0));
    list.end_layer();
    list.begin_compositor(CompositorLayer::new(
        CompositorId::new(5),
        bounds,
        Affine2D::IDENTITY,
        Affine2D::IDENTITY,
        1.0,
    ));
    list.push_quad(quad(20.0, 12.0, 8.0));
    list.end_compositor();
    list
}
