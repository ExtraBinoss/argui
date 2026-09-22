#![cfg(target_os = "linux")]

#[path = "gpu_canvas/pipeline.rs"]
mod gpu_canvas;

use std::{
    cell::Cell,
    sync::{Arc, atomic::Ordering},
};

use argui_core::{Affine2D, Color, Point, Rect, Size};
use argui_paint::{
    Border, ClipChain, CornerRadii, DisplayList, EffectId, EffectInstance, Fill, Filter,
    GpuCanvasId, GpuCanvasPrimitive, LayerStyle, Quad,
};
use argui_render::{
    DamageMode, DamageTracking, EffectDamage, EffectDefinition, EffectPassDefinition,
    EffectRegistry, GpuCanvasDiagnosticKind, GpuCanvasRegistration, GpuCanvasRegistry,
    RenderStatus, RendererConfig, RendererError, SurfaceRenderer,
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    platform::{
        pump_events::EventLoopExtPumpEvents, wayland::EventLoopBuilderExtWayland,
        x11::EventLoopBuilderExtX11,
    },
    window::{Window, WindowId},
};

#[path = "surface/effect_damage.rs"]
mod effect_damage;
#[path = "surface/composite.rs"]
mod headless;
#[path = "surface/api.rs"]
mod helpers;
use gpu_canvas::{CanvasProbe, ComputeFactory, RequiredFeatureFactory, RequiredLimitFactory};
use helpers::*;

const SHADER: &str = "fn argui_effect(uv: vec2<f32>, source: vec4<f32>, backdrop: vec4<f32>) -> vec4<f32> { return source * 0.5; }";
const NATIVE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);
fn effect_passes() -> [EffectPassDefinition; 2] {
    [
        EffectPassDefinition::fragment("first", SHADER),
        EffectPassDefinition::fragment("second", SHADER),
    ]
}

#[test]
#[ignore = "Native surface stress: set ARGUI_SURFACE_STRESS=1 on a working compositor"]
fn native_surface_grows_its_atlas_recovers_from_capacity_and_uses_custom_effects() {
    if std::env::var_os("ARGUI_SURFACE_STRESS").is_none() {
        return;
    }
    struct TestApp(Option<Arc<Window>>);
    impl ApplicationHandler for TestApp {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            let window = Arc::new(
                event_loop
                    .create_window(
                        Window::default_attributes()
                            .with_title("Argui renderer tests")
                            .with_inner_size(winit::dpi::PhysicalSize::new(256, 256))
                            .with_visible(false),
                    )
                    .unwrap(),
            );
            self.0 = Some(window);
        }

        fn window_event(&mut self, _: &ActiveEventLoop, _: WindowId, _: WindowEvent) {}
    }
    let mut builder = EventLoop::builder();
    if std::env::var("ARGUI_TEST_BACKEND").as_deref() == Ok("x11") {
        builder.with_x11();
        EventLoopBuilderExtX11::with_any_thread(&mut builder, true);
    } else {
        builder.with_wayland();
        EventLoopBuilderExtWayland::with_any_thread(&mut builder, true);
    }
    let mut event_loop = builder.build().unwrap();
    let mut app = TestApp(None);
    let deadline = web_time::Instant::now() + NATIVE_TIMEOUT;
    while app.0.is_none() {
        event_loop.pump_app_events(Some(std::time::Duration::from_millis(16)), &mut app);
        assert!(
            web_time::Instant::now() < deadline,
            "test window was not created"
        );
    }
    let window = app.0.as_ref().unwrap().clone();
    exercise(window.clone(), |render| {
        let mut frame = FrameEvent {
            window_id: window.id(),
            render,
            outcome: None,
        };
        event_loop.pump_app_events(Some(std::time::Duration::from_millis(16)), &mut frame);
        frame.outcome
    });
}

struct FrameEvent<'a> {
    window_id: WindowId,
    render: &'a mut dyn FnMut() -> helpers::FrameResult,
    outcome: Option<helpers::FrameResult>,
}

impl ApplicationHandler for FrameEvent<'_> {
    fn resumed(&mut self, _: &ActiveEventLoop) {}

    fn window_event(&mut self, _: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if id == self.window_id && matches!(event, WindowEvent::RedrawRequested) {
            self.outcome = Some((self.render)());
        }
    }
}

fn exercise(
    window: Arc<Window>,
    mut pump: impl FnMut(&mut dyn FnMut() -> helpers::FrameResult) -> Option<helpers::FrameResult>,
) {
    let effect = EffectId::new("test.half");
    let unused = EffectId::new("test.unused");
    let registry = EffectRegistry::new([
        EffectDefinition::new(
            effect.clone(),
            Vec::<argui_render::EffectParameter>::new(),
            effect_passes(),
        )
        .damage(EffectDamage::Bounded),
        EffectDefinition::new(
            unused.clone(),
            Vec::<argui_render::EffectParameter>::new(),
            effect_passes(),
        ),
    ])
    .unwrap();
    let probe = Arc::new(CanvasProbe::default());
    let canvas_registration =
        GpuCanvasRegistration::new("test.compute-canvas", ComputeFactory(probe.clone()));
    let canvas_id = canvas_registration.id();
    probe.canvas.store(canvas_id.get(), Ordering::Relaxed);
    let retry_probe = Arc::new(CanvasProbe::default());
    retry_probe.create_fail.store(true, Ordering::Relaxed);
    let retry_registration =
        GpuCanvasRegistration::new("test.creation-retry", ComputeFactory(retry_probe.clone()));
    let retry_id = retry_registration.id();
    retry_probe.canvas.store(retry_id.get(), Ordering::Relaxed);
    let canvas_registry =
        GpuCanvasRegistry::new([canvas_registration, retry_registration]).unwrap();
    let size = window.inner_size();
    let config = RendererConfig::default()
        .profiling(true)
        .effects(registry)
        .gpu_canvas_cache_bytes(16 * 1024)
        .gpu_canvases(canvas_registry);
    let mut renderer = pollster::block_on(SurfaceRenderer::new(
        window.clone(),
        size.width,
        size.height,
        config,
    ))
    .unwrap();
    window.set_visible(true);
    window.request_redraw();
    effect_damage::exercise_compositor(&mut renderer, &window, &mut pump);

    let scale_factor = Cell::new(1.0);
    let mut render = |renderer: &mut SurfaceRenderer, list: &DisplayList| {
        helpers::render_at_scale(renderer, list, &window, &mut pump, scale_factor.get())
    };
    render(&mut renderer, &DisplayList::new()).unwrap();
    assert_eq!(renderer.last_profile().damage.mode, DamageMode::Full);
    assert_eq!(
        renderer.last_profile().vector_atlas.allocated_bytes,
        256 * 256 * 4
    );

    let mut first = DisplayList::new();
    first.push_quad(colored_quad(8.0, Color::WHITE));
    render(&mut renderer, &first).unwrap();
    assert_eq!(renderer.last_profile().damage.mode, DamageMode::Seed);
    let mut moved = DisplayList::new();
    moved.push_quad(colored_quad(48.0, Color::WHITE));
    render(&mut renderer, &moved).unwrap();
    assert_eq!(renderer.last_profile().damage.mode, DamageMode::Partial);
    assert!(renderer.last_profile().damage.damaged_pixels < 256 * 256);
    render(&mut renderer, &moved).unwrap();
    assert_eq!(renderer.last_profile().damage.mode, DamageMode::Reused);
    renderer.set_damage_tracking(DamageTracking::enabled());
    render(&mut renderer, &moved).unwrap();
    assert_eq!(renderer.last_profile().damage.mode, DamageMode::Reused);
    renderer.set_damage_tracking(DamageTracking::disabled());
    render(&mut renderer, &moved).unwrap();
    assert_eq!(renderer.last_profile().damage.mode, DamageMode::Full);
    renderer.set_damage_tracking(DamageTracking::enabled());
    render(&mut renderer, &moved).unwrap();
    assert_eq!(renderer.last_profile().damage.mode, DamageMode::Full);
    let mut moved_again = DisplayList::new();
    moved_again.push_quad(colored_quad(80.0, Color::WHITE));
    render(&mut renderer, &moved_again).unwrap();
    assert_eq!(renderer.last_profile().damage.mode, DamageMode::Seed);

    effect_damage::exercise(&mut renderer, effect.clone(), &mut render);

    let assets: Vec<_> = (0..20).map(|_| asset()).collect();
    for asset in &assets {
        renderer.register_vector(asset).unwrap();
    }
    let small = vectors(&assets[..1], 32.0);
    render(&mut renderer, &small).unwrap();
    assert_eq!(
        renderer
            .last_profile()
            .vector_atlas
            .rasterizations_this_frame,
        1
    );
    let large = vectors(&assets[..1], 512.0);
    render(&mut renderer, &large).unwrap();
    assert_eq!(
        renderer.last_profile().vector_atlas.allocated_bytes,
        1024 * 1024 * 4
    );
    render(&mut renderer, &small).unwrap();
    render(&mut renderer, &small).unwrap();
    assert_eq!(
        renderer
            .last_profile()
            .vector_atlas
            .rasterizations_this_frame,
        0
    );
    assert_eq!(renderer.last_profile().vector_atlas.hits_this_frame, 1);

    assert!(matches!(
        render(&mut renderer, &vectors(&assets, 512.0)),
        Err(RendererError::VectorAtlasFull)
    ));
    // The expected preparation error discarded an acquired swapchain image.
    // Reconfigure the private-display surface before checking atlas recovery.
    assert!(renderer.resize(size.width + 1, size.height));
    assert!(renderer.resize(size.width, size.height));
    render(&mut renderer, &small).unwrap();
    assert_eq!(
        renderer.last_profile().vector_atlas.allocated_bytes,
        2048 * 2048 * 4
    );

    for id in [effect.clone(), effect.clone(), unused.clone()] {
        let mut list = DisplayList::new();
        list.begin_layer(LayerStyle::new(bounds(32.0)).filter(Filter::Effect(
            EffectInstance::new(
                id.clone(),
                std::iter::empty::<argui_paint::EffectArgument>(),
            ),
        )));
        list.push_vector(vector(assets[0].id, 32.0));
        list.end_layer();
        render(&mut renderer, &list).unwrap();
        assert!(renderer.last_profile().effects.filter_passes >= 2);
        if id == unused {
            assert_eq!(renderer.last_profile().damage.mode, DamageMode::Full);
        }
    }
    let mut missing = DisplayList::new();
    missing.begin_layer(
        LayerStyle::new(bounds(32.0)).filter(Filter::Effect(EffectInstance::new(
            EffectId::new("test.missing"),
            std::iter::empty::<argui_paint::EffectArgument>(),
        ))),
    );
    missing.push_vector(vector(assets[0].id, 32.0));
    missing.end_layer();
    assert!(matches!(
        render(&mut renderer, &missing),
        Err(RendererError::MissingEffect(id)) if id == EffectId::new("test.missing")
    ));

    let replacement = EffectRegistry::new([EffectDefinition::new(
        effect.clone(),
        Vec::<argui_render::EffectParameter>::new(),
        [EffectPassDefinition::fragment("replacement", SHADER)],
    )
    .with_revision(2)])
    .unwrap();
    renderer.replace_effect_registry(replacement).unwrap();
    let mut replaced = DisplayList::new();
    replaced.begin_layer(LayerStyle::new(bounds(32.0)).filter(Filter::Effect(
        EffectInstance::new(
            effect.clone(),
            std::iter::empty::<argui_paint::EffectArgument>(),
        ),
    )));
    replaced.push_vector(vector(assets[0].id, 32.0));
    replaced.end_layer();
    render(&mut renderer, &replaced).unwrap();
    assert!(renderer.last_profile().effects.filter_passes >= 1);
    renderer
        .replace_effect_registry(EffectRegistry::default())
        .unwrap();
    assert!(matches!(
        render(&mut renderer, &replaced),
        Err(RendererError::MissingEffect(id)) if id == effect
    ));

    gpu_canvas::exercise(
        &mut renderer,
        canvas_id,
        &probe,
        retry_id,
        &retry_probe,
        &window,
        &mut render,
    );

    let mut canvas_only = DisplayList::new();
    canvas_only.push_gpu_canvas(canvas_primitive(canvas_id, 9, Size::new(8.0, 6.0), 35));
    for invalid_scale in [0.0, f32::NAN] {
        scale_factor.set(invalid_scale);
        render(&mut renderer, &canvas_only).unwrap();
        assert!(
            renderer
                .take_gpu_canvas_diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.message.contains("invalid window scale factor"))
        );
    }
    scale_factor.set(1.0);
    assert_ne!(probe.generation.load(Ordering::Relaxed), 0);
    assert_eq!(
        probe.generation.load(Ordering::Relaxed),
        retry_probe.generation.load(Ordering::Relaxed)
    );

    // Additional WGPU surfaces on the same window can invalidate presentation,
    // so perform the rejected-device probes after all rendered frames.
    let required = GpuCanvasRegistration::new("test.required-feature", RequiredFeatureFactory);
    let incompatible = pollster::block_on(SurfaceRenderer::new_with_device(
        window.clone(),
        size.width,
        size.height,
        RendererConfig::default().gpu_canvases(GpuCanvasRegistry::new([required]).unwrap()),
        renderer.device_handle(),
    ));
    assert!(matches!(
        incompatible,
        Err(RendererError::IncompatibleGpuCanvasDevice { canvas, .. })
            if canvas == "test.required-feature"
    ));

    let required_limit = GpuCanvasRegistration::new("test.required-limit", RequiredLimitFactory);
    let limit_registry = GpuCanvasRegistry::new([required_limit]).unwrap();
    let incompatible_limit = pollster::block_on(SurfaceRenderer::new_with_device(
        window.clone(),
        size.width,
        size.height,
        RendererConfig::default().gpu_canvases(limit_registry.clone()),
        renderer.device_handle(),
    ));
    assert!(matches!(
        incompatible_limit,
        Err(RendererError::IncompatibleGpuCanvasDevice { canvas, message })
            if canvas == "test.required-limit" && message.contains("max_bind_groups")
    ));
    let unavailable_limit = pollster::block_on(SurfaceRenderer::new(
        window.clone(),
        size.width,
        size.height,
        RendererConfig::default().gpu_canvases(limit_registry),
    ));
    assert!(matches!(
        unavailable_limit,
        Err(RendererError::GpuCanvasCapability { canvas, message })
            if canvas == "test.required-limit" && message.contains("max_bind_groups")
    ));
}
