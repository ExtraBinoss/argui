#![cfg(target_os = "linux")]

#[path = "gpu_canvas/pipeline.rs"]
mod gpu_canvas;

use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
};

use argui_core::{Affine2D, Color, Point, Rect, Size};
use argui_paint::{
    Border, ClipChain, CornerRadii, DisplayList, EffectId, EffectInstance, Fill, Filter,
    GpuCanvasId, GpuCanvasPrimitive, ImageFit, ImageSampling, LayerStyle, ProfileDomain, Quad,
    RenderObjectId, VectorAsset, VectorId, VectorPrimitive,
};
use argui_render::{
    DamageMode, DamageTracking, EffectDefinition, EffectPassDefinition, EffectRegistry,
    GpuCanvasDeviceContext, GpuCanvasDiagnosticKind, GpuCanvasError, GpuCanvasFactory,
    GpuCanvasRegistration, GpuCanvasRegistry, GpuCanvasRenderContext, GpuCanvasRenderer,
    GpuCanvasRequirements, RenderStatus, RendererConfig, RendererError, SurfaceRenderer,
};
use argui_text::{PreparedText, TextEngine};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    platform::{pump_events::EventLoopExtPumpEvents, wayland::EventLoopBuilderExtWayland},
    window::{Window, WindowId},
};

const SHADER: &str = "fn argui_effect(uv: vec2<f32>, source: vec4<f32>, backdrop: vec4<f32>) -> vec4<f32> { return source * 0.5; }";
const NATIVE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);
const PASSES: &[EffectPassDefinition] = &[
    EffectPassDefinition::fragment("first", SHADER),
    EffectPassDefinition::fragment("second", SHADER),
];

const COMPUTE_SHADER: &str = r#"
@group(0) @binding(0) var<storage, read_write> output: array<vec4<f32>, 1>;
@compute @workgroup_size(1)
fn main() { output[0] = vec4<f32>(0.08, 0.6, 0.25, 0.8); }
"#;
const CANVAS_SHADER: &str = r#"
@group(0) @binding(0) var<storage, read> color: array<vec4<f32>, 1>;
@vertex
fn vertex(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
    return vec4<f32>(positions[index], 0.0, 1.0);
}
@fragment
fn fragment() -> @location(0) vec4<f32> { return color[0]; }
"#;

#[derive(Default)]
struct CanvasProbe {
    creates: AtomicUsize,
    renders: AtomicUsize,
    extent: AtomicU64,
    create_fail: AtomicBool,
    fail: AtomicBool,
}

struct ComputeFactory(Arc<CanvasProbe>);

struct RequiredFeatureFactory;

impl GpuCanvasFactory for RequiredFeatureFactory {
    fn requirements(&self) -> GpuCanvasRequirements {
        GpuCanvasRequirements::default()
            .required_features(wgpu::Features::TEXTURE_COMPRESSION_BC)
            .reason("validates shared-device capability rejection")
    }

    fn create(
        &self,
        _context: &GpuCanvasDeviceContext<'_>,
    ) -> Result<Box<dyn GpuCanvasRenderer>, GpuCanvasError> {
        Err(GpuCanvasError::new(
            "shared device validation should run before factory creation",
        ))
    }
}

impl GpuCanvasFactory for ComputeFactory {
    fn create(
        &self,
        context: &GpuCanvasDeviceContext<'_>,
    ) -> Result<Box<dyn GpuCanvasRenderer>, GpuCanvasError> {
        self.0.creates.fetch_add(1, Ordering::Relaxed);
        if self.0.create_fail.load(Ordering::Relaxed) {
            return Err(GpuCanvasError::new(
                "intentional integration-test factory failure",
            ));
        }
        Ok(Box::new(ComputeRenderer::new(context, self.0.clone())))
    }
}

struct ComputeRenderer {
    probe: Arc<CanvasProbe>,
    compute: wgpu::ComputePipeline,
    compute_group: wgpu::BindGroup,
    render: wgpu::RenderPipeline,
    render_group: wgpu::BindGroup,
    _color: wgpu::Buffer,
}

impl ComputeRenderer {
    fn new(context: &GpuCanvasDeviceContext<'_>, probe: Arc<CanvasProbe>) -> Self {
        let device = context.device();
        let color = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("argui-test-canvas-color"),
            size: 16,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let compute_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("argui-test-canvas-compute-layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let render_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("argui-test-canvas-render-layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let group = |label, layout: &wgpu::BindGroupLayout| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(label),
                layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: color.as_entire_binding(),
                }],
            })
        };
        let compute_group = group("argui-test-canvas-compute-group", &compute_layout);
        let render_group = group("argui-test-canvas-render-group", &render_layout);
        let compute_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("argui-test-canvas-compute"),
            source: wgpu::ShaderSource::Wgsl(COMPUTE_SHADER.into()),
        });
        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("argui-test-canvas-compute-pipeline-layout"),
                bind_group_layouts: &[Some(&compute_layout)],
                immediate_size: 0,
            });
        let compute = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("argui-test-canvas-compute-pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_module,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        let render_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("argui-test-canvas-render"),
            source: wgpu::ShaderSource::Wgsl(CANVAS_SHADER.into()),
        });
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("argui-test-canvas-render-pipeline-layout"),
                bind_group_layouts: &[Some(&render_layout)],
                immediate_size: 0,
            });
        let render = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("argui-test-canvas-render-pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &render_module,
                entry_point: Some("vertex"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &render_module,
                entry_point: Some("fragment"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: context.target_format(),
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        Self {
            probe,
            compute,
            compute_group,
            render,
            render_group,
            _color: color,
        }
    }
}

impl GpuCanvasRenderer for ComputeRenderer {
    fn render(&mut self, context: &mut GpuCanvasRenderContext<'_>) -> Result<(), GpuCanvasError> {
        self.probe.renders.fetch_add(1, Ordering::Relaxed);
        let [width, height] = context.physical_extent();
        self.probe.extent.store(
            (u64::from(width) << 32) | u64::from(height),
            Ordering::Relaxed,
        );
        if self.probe.fail.load(Ordering::Relaxed) {
            return Err(GpuCanvasError::new("intentional integration-test failure"));
        }
        let (encoder, target) = context.encoder_and_target();
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            pass.set_pipeline(&self.compute);
            pass.set_bind_group(0, &self.compute_group, &[]);
            pass.dispatch_workgroups(1, 1, 1);
        }
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("argui-test-canvas-render-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        pass.set_pipeline(&self.render);
        pass.set_bind_group(0, &self.render_group, &[]);
        pass.draw(0..3, 0..1);
        Ok(())
    }
}

#[test]
#[ignore = "Native surface integration: requires a dedicated Wayland test display"]
fn native_surface_grows_its_atlas_recovers_from_capacity_and_uses_custom_effects() {
    struct TestApp(Option<Arc<Window>>, bool);
    impl ApplicationHandler for TestApp {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            let window = Arc::new(
                event_loop
                    .create_window(
                        Window::default_attributes()
                            .with_title("Argui renderer tests")
                            .with_inner_size(winit::dpi::PhysicalSize::new(256, 256)),
                    )
                    .unwrap(),
            );
            window.request_redraw();
            self.0 = Some(window);
        }

        fn window_event(&mut self, _: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
            self.1 |= matches!(event, WindowEvent::RedrawRequested);
        }
    }
    let mut event_loop = EventLoop::builder()
        .with_wayland()
        .with_any_thread(true)
        .build()
        .unwrap();
    let mut app = TestApp(None, false);
    let deadline = web_time::Instant::now() + NATIVE_TIMEOUT;
    while !app.1 {
        event_loop.pump_app_events(Some(std::time::Duration::from_millis(16)), &mut app);
        assert!(
            web_time::Instant::now() < deadline,
            "window was not configured"
        );
    }
    let window = app.0.as_ref().unwrap().clone();
    exercise(window, || {
        event_loop.pump_app_events(Some(std::time::Duration::from_millis(16)), &mut app);
    });
}

fn exercise(window: Arc<Window>, mut pump: impl FnMut()) {
    let effect = EffectId::new("test.half");
    let unused = EffectId::new("test.unused");
    let registry = EffectRegistry::new([
        EffectDefinition::new(effect, &[], PASSES),
        EffectDefinition::new(unused, &[], PASSES),
    ])
    .unwrap();
    let probe = Arc::new(CanvasProbe::default());
    let canvas_registration =
        GpuCanvasRegistration::new("test.compute-canvas", ComputeFactory(probe.clone()));
    let canvas_id = canvas_registration.id();
    let retry_probe = Arc::new(CanvasProbe::default());
    retry_probe.create_fail.store(true, Ordering::Relaxed);
    let retry_registration =
        GpuCanvasRegistration::new("test.creation-retry", ComputeFactory(retry_probe.clone()));
    let retry_id = retry_registration.id();
    let canvas_registry =
        GpuCanvasRegistry::new([canvas_registration, retry_registration]).unwrap();
    let size = window.inner_size();
    let mut config = RendererConfig::default()
        .profiling(true)
        .effects(registry)
        .gpu_canvas_cache_bytes(16 * 1024)
        .gpu_canvases(canvas_registry);
    // This test submits many frames without application work between them. An
    // automatic non-vsync mode avoids depending on compositor frame throttling.
    config.present_mode = wgpu::PresentMode::AutoNoVsync;
    let mut renderer = pollster::block_on(SurfaceRenderer::new(
        window.clone(),
        size.width,
        size.height,
        config,
    ))
    .unwrap();
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
    let mut render = |renderer: &mut SurfaceRenderer, list: &DisplayList| {
        render(renderer, list, &window, &mut pump)
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
    render(&mut renderer, &small).unwrap();
    assert_eq!(
        renderer.last_profile().vector_atlas.allocated_bytes,
        2048 * 2048 * 4
    );

    for id in [effect, effect, unused] {
        let mut list = DisplayList::new();
        list.begin_layer(LayerStyle::new(bounds(32.0)).filter(Filter::Effect(
            EffectInstance::new(id, std::iter::empty::<argui_paint::EffectArgument>()),
        )));
        list.push_vector(vector(assets[0].id, 32.0));
        list.end_layer();
        render(&mut renderer, &list).unwrap();
        assert!(renderer.last_profile().effects.filter_passes >= 2);
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
        Err(RendererError::MissingEffect("test.missing"))
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
}

fn canvas_list(id: GpuCanvasId, revision: u64, size: Size, slot: u32, effect: bool) -> DisplayList {
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

fn canvas_primitive(id: GpuCanvasId, revision: u64, size: Size, slot: u32) -> GpuCanvasPrimitive {
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

fn render(
    renderer: &mut SurfaceRenderer,
    list: &DisplayList,
    window: &Window,
    pump: &mut impl FnMut(),
) -> Result<(), RendererError> {
    let mut text = TextEngine::from_embedded_fonts([], "sans-serif", "serif", "monospace");
    let deadline = web_time::Instant::now() + NATIVE_TIMEOUT;
    loop {
        // A Wayland surface needs configure/frame callbacks between submissions.
        window.request_redraw();
        pump();
        let status =
            renderer.render_ui_notified(&mut text, &PreparedText::default(), list, 1.0, || {
                window.pre_present_notify()
            })?;
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

fn asset() -> VectorAsset {
    VectorAsset {
        id: VectorId::fresh(),
        size: Size::new(16.0, 16.0),
        svg: Arc::from(br#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"><rect width="16" height="16" fill="red"/></svg>"#.as_slice()),
        tintable: false,
    }
}

fn bounds(size: f32) -> Rect {
    Rect::new(Point::default(), Size::new(size, size))
}

fn colored_quad(x: f32, color: Color) -> Quad {
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

fn vector(id: VectorId, size: f32) -> VectorPrimitive {
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

fn vectors(assets: &[VectorAsset], size: f32) -> DisplayList {
    let mut list = DisplayList::new();
    for asset in assets {
        list.push_vector(vector(asset.id, size));
    }
    list
}
