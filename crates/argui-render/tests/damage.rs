use argui_core::{Affine2D, Color, Point, Rect, Size};
use argui_paint::{
    Border, ClipChain, ClipRegion, CompositorId, CompositorLayer, CornerRadii, DisplayList,
    EffectId, EffectInstance, Fill, Filter, GpuCanvasId, GpuCanvasPrimitive, ImageFit, ImageId,
    ImagePrimitive, ImageSampling, LayerStyle, ProfileDomain, Quad, RenderObjectId, Shadow,
    VectorId, VectorPrimitive,
};
use argui_render::{
    DamagePlan, DamageRegion, DamageSnapshot, DamageTracking, EffectDamage, EffectDefinition,
    EffectPassDefinition, EffectRegistry,
};
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

/// Empty or fully offscreen damage is discarded before threshold evaluation.
#[test]
fn zero_extent_and_offscreen_regions_leave_retained_pixels_unchanged() {
    assert_eq!(DamageRegion::new(u32::MAX - 4, 0, 20, 1).right(), u32::MAX);
    assert_eq!(DamageRegion::new(0, u32::MAX - 4, 1, 20).bottom(), u32::MAX);
    assert_eq!(
        DamagePlan::resolve(
            [
                DamageRegion::new(0, 0, 0, 20),
                DamageRegion::new(0, 0, 20, 0),
                DamageRegion::new(500, 500, 20, 20),
            ],
            [256, 256],
            DamageTracking::enabled(),
        ),
        DamagePlan::Unchanged
    );
}

/// A bridge region transitively coalesces neighbors even when inserted last.
#[test]
fn bridge_region_merges_separated_tiles_transitively() {
    assert_eq!(
        DamagePlan::resolve(
            [
                DamageRegion::new(0, 0, 32, 32),
                DamageRegion::new(64, 0, 32, 32),
                DamageRegion::new(32, 0, 32, 32),
            ],
            [256, 256],
            DamageTracking::enabled().max_regions(1),
        ),
        DamagePlan::Partial(vec![DamageRegion::new(0, 0, 96, 32)])
    );
}

/// Non-touching tiles remain separate in either horizontal or vertical order.
#[test]
fn separated_tiles_do_not_merge_on_any_axis_or_insertion_order() {
    let horizontal = [
        DamageRegion::new(0, 0, 32, 32),
        DamageRegion::new(96, 0, 32, 32),
    ];
    let vertical = [
        DamageRegion::new(0, 0, 32, 32),
        DamageRegion::new(0, 96, 32, 32),
    ];
    for pair in [
        horizontal,
        [horizontal[1], horizontal[0]],
        vertical,
        [vertical[1], vertical[0]],
    ] {
        let DamagePlan::Partial(regions) =
            DamagePlan::resolve(pair, [256, 256], DamageTracking::enabled())
        else {
            panic!("separated damage must remain partial");
        };
        assert_eq!(regions.len(), 2);
    }
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
        baseline.compare(&baseline.clone(), DamageTracking::disabled()),
        DamagePlan::Full
    );
    assert_eq!(
        baseline.compare(&snapshot(&list, [512, 256], 2.0), DamageTracking::enabled()),
        DamagePlan::Full
    );
    assert_eq!(
        baseline.compare(&snapshot(&list, [256, 128], 2.0), DamageTracking::enabled()),
        DamagePlan::Full
    );
}

/// Changes wholly outside the viewport do not dirty retained pixels.
#[test]
fn offscreen_scene_changes_remain_unchanged_until_they_enter_view() {
    let mut old = DisplayList::new();
    old.push_quad(quad(-100.0, -100.0, 16.0));
    let mut moved = DisplayList::new();
    moved.push_quad(quad(-80.0, -80.0, 16.0));
    let old = snapshot(&old, [128, 128], 1.0);
    let moved = snapshot(&moved, [128, 128], 1.0);
    assert_eq!(
        old.compare(&moved, DamageTracking::enabled()),
        DamagePlan::Unchanged
    );

    let mut above = DisplayList::new();
    above.push_quad(quad(8.0, -100.0, 16.0));
    let mut still_above = DisplayList::new();
    still_above.push_quad(quad(8.0, -80.0, 16.0));
    assert_eq!(
        snapshot(&above, [128, 128], 1.0).compare(
            &snapshot(&still_above, [128, 128], 1.0),
            DamageTracking::enabled(),
        ),
        DamagePlan::Unchanged
    );

    let mut visible = DisplayList::new();
    visible.push_quad(quad(8.0, 8.0, 16.0));
    assert_eq!(
        moved.compare(
            &snapshot(&visible, [128, 128], 1.0),
            DamageTracking::enabled()
        ),
        DamagePlan::Partial(vec![DamageRegion::new(0, 0, 32, 32)])
    );
}

#[test]
fn bounded_effects_expand_only_intersecting_damage_to_layer_bounds() {
    let previous = effect_scene(104.0, Filter::Blur(8.0));
    let current = effect_scene(120.0, Filter::Blur(8.0));
    let plan = snapshot(&previous, [512, 256], 1.0).compare_with_effects(
        &snapshot(&current, [512, 256], 1.0),
        DamageTracking::enabled(),
        &EffectRegistry::default(),
    );
    let DamagePlan::Partial(regions) = plan else {
        panic!("bounded blur should preserve a partial plan");
    };
    assert!(
        regions
            .iter()
            .any(|region| region.x <= 96 && region.right() >= 224)
    );

    let halo_previous = effect_scene(56.0, Filter::Blur(8.0));
    let halo_current = effect_scene(64.0, Filter::Blur(8.0));
    let DamagePlan::Partial(regions) = snapshot(&halo_previous, [512, 256], 1.0)
        .compare_with_effects(
            &snapshot(&halo_current, [512, 256], 1.0),
            DamageTracking::enabled(),
            &EffectRegistry::default(),
        )
    else {
        panic!("the blur sampling halo should remain partial");
    };
    assert!(regions.iter().any(|region| region.right() >= 224));

    let previous = effect_scene(8.0, Filter::Blur(8.0));
    let current = effect_scene(24.0, Filter::Blur(8.0));
    let DamagePlan::Partial(regions) = snapshot(&previous, [512, 256], 1.0).compare_with_effects(
        &snapshot(&current, [512, 256], 1.0),
        DamageTracking::enabled(),
        &EffectRegistry::default(),
    ) else {
        panic!("unrelated damage should remain partial");
    };
    assert!(regions.iter().all(|region| region.right() < 224));

    let right_previous = effect_scene(240.0, Filter::Blur(8.0));
    let right_current = effect_scene(256.0, Filter::Blur(8.0));
    let DamagePlan::Partial(regions) = snapshot(&right_previous, [512, 256], 1.0)
        .compare_with_effects(
            &snapshot(&right_current, [512, 256], 1.0),
            DamageTracking::enabled(),
            &EffectRegistry::default(),
        )
    else {
        panic!("unrelated rightward damage should remain partial");
    };
    assert!(regions.iter().all(|region| region.x >= 224));
}

#[test]
fn custom_effect_damage_policy_is_conservative_by_default() {
    const WGSL: &str = r#"
fn argui_effect(_uv: vec2<f32>, source: vec4<f32>, _backdrop: vec4<f32>) -> vec4<f32> {
    return source;
}
"#;
    let id = EffectId::new("test.damage");
    let previous = effect_scene(
        104.0,
        Filter::Effect(EffectInstance::new(
            id.clone(),
            std::iter::empty::<argui_paint::EffectArgument>(),
        )),
    );
    let current = effect_scene(
        120.0,
        Filter::Effect(EffectInstance::new(
            id.clone(),
            std::iter::empty::<argui_paint::EffectArgument>(),
        )),
    );
    let previous = snapshot(&previous, [512, 256], 1.0);
    let current = snapshot(&current, [512, 256], 1.0);
    let passes = || [EffectPassDefinition::fragment("main", WGSL)];
    let unbounded = EffectRegistry::new([EffectDefinition::new(
        id.clone(),
        Vec::<argui_render::EffectParameter>::new(),
        passes(),
    )])
    .unwrap();
    assert_eq!(
        previous.compare_with_effects(&current, DamageTracking::enabled(), &unbounded),
        DamagePlan::Full
    );
    let bounded = EffectRegistry::new([EffectDefinition::new(
        id,
        Vec::<argui_render::EffectParameter>::new(),
        passes(),
    )
    .damage(EffectDamage::Bounded)])
    .unwrap();
    assert!(matches!(
        previous.compare_with_effects(&current, DamageTracking::enabled(), &bounded),
        DamagePlan::Partial(_)
    ));
    assert_eq!(
        current.compare_with_effects(&current, DamageTracking::enabled(), &unbounded),
        DamagePlan::Unchanged
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

fn effect_scene(moving_x: f32, filter: Filter) -> DisplayList {
    let mut list = DisplayList::new();
    list.push_quad(quad(moving_x, 24.0, 16.0));
    list.begin_layer(
        LayerStyle::new(Rect::new(Point::new(96.0, 8.0), Size::new(96.0, 96.0))).backdrop(filter),
    );
    list.push_quad(quad(112.0, 24.0, 16.0));
    list.end_layer();
    list
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
    let mut y_clipped = quad(8.0, 8.0, 16.0);
    y_clipped.clips = ClipChain::from_regions([ClipRegion::new(
        Rect::new(Point::new(8.0, 200.0), Size::new(16.0, 16.0)),
        Affine2D::IDENTITY,
    )]);
    list.push_quad(y_clipped);
    let outside = quad(600.0, 600.0, 16.0);
    list.push_quad(outside);
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
    let shadow = Shadow::drop([0.0, 2.0], 4.0, Color::BLACK);
    let styled_layer = LayerStyle::new(bounds)
        .filter(Filter::Blur(2.0))
        .shadow(shadow);
    list.begin_layer(styled_layer);
    list.push_quad(quad(14.0, 14.0, 8.0));
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
