use super::*;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize};

use argui_paint::{ProfileDomain, RenderObjectId};
use argui_render::{
    GpuCanvasDeviceContext, GpuCanvasError, GpuCanvasFactory, GpuCanvasRenderContext,
    GpuCanvasRenderer, GpuCanvasRequirements, wgpu,
};

/// Exercises retained GPU-canvas callback, extent, diagnostics, effect and surface behavior.
pub(super) fn exercise(
    renderer: &mut SurfaceRenderer,
    canvas_id: GpuCanvasId,
    probe: &Arc<CanvasProbe>,
    retry_id: GpuCanvasId,
    retry_probe: &Arc<CanvasProbe>,
    window: &Arc<Window>,
    render: &mut impl FnMut(&mut SurfaceRenderer, &DisplayList) -> Result<(), RendererError>,
) {
    let first = canvas_list(canvas_id, 1, Size::new(48.0, 32.0), 0, false);
    render(renderer, &first).unwrap();
    assert_eq!(probe.creates.load(Ordering::Relaxed), 1);
    assert_eq!(probe.renders.load(Ordering::Relaxed), 1);
    assert_eq!(probe.extent.load(Ordering::Relaxed), (48_u64 << 32) | 32);
    assert_eq!(renderer.gpu_canvas_stats().renders_this_frame, 1);
    assert_eq!(renderer.gpu_canvas_stats().entries, 1);
    assert_eq!(renderer.gpu_canvas_stats().allocated_bytes, 48 * 32 * 4);

    render(renderer, &first).unwrap();
    assert_eq!(probe.renders.load(Ordering::Relaxed), 1);
    assert_eq!(renderer.gpu_canvas_stats().hits_this_frame, 1);

    let changed = canvas_list(canvas_id, 2, Size::new(48.0, 32.0), 0, false);
    render(renderer, &changed).unwrap();
    assert_eq!(probe.renders.load(Ordering::Relaxed), 2);

    let resized = canvas_list(canvas_id, 2, Size::new(64.0, 40.0), 0, false);
    render(renderer, &resized).unwrap();
    assert_eq!(probe.renders.load(Ordering::Relaxed), 3);
    assert_eq!(probe.extent.load(Ordering::Relaxed), (64_u64 << 32) | 40);

    let mut supersampled = canvas_primitive(canvas_id, 2, Size::new(32.0, 20.0), 1);
    supersampled.resolution_scale = 2.0;
    let mut supersampled_list = DisplayList::new();
    supersampled_list.push_gpu_canvas(supersampled);
    render(renderer, &supersampled_list).unwrap();
    assert_eq!(probe.extent.load(Ordering::Relaxed), (64_u64 << 32) | 40);
    assert_eq!(probe.last_slot.load(Ordering::Relaxed), 1);
    assert!(renderer.gpu_canvas_stats().allocated_bytes <= 16 * 1024);

    let before_lru = probe.renders.load(Ordering::Relaxed);
    render(renderer, &changed).unwrap();
    assert_eq!(probe.renders.load(Ordering::Relaxed), before_lru + 1);

    let before_empty = probe.renders.load(Ordering::Relaxed);
    render(
        renderer,
        &canvas_list(canvas_id, 2, Size::new(0.0, 0.0), 2, false),
    )
    .unwrap();
    assert_eq!(probe.renders.load(Ordering::Relaxed), before_empty);

    let mut invalid = canvas_primitive(canvas_id, 3, Size::new(32.0, 20.0), 3);
    invalid.resolution_scale = f32::NAN;
    assert!(failure_message(renderer, invalid, render).contains("invalid resolution scale"));

    let mut negative_scale = canvas_primitive(canvas_id, 3, Size::new(32.0, 20.0), 31);
    negative_scale.resolution_scale = -1.0;
    assert!(failure_message(renderer, negative_scale, render).contains("invalid resolution scale"));

    let mut invalid_extent = canvas_primitive(canvas_id, 3, Size::new(-1.0, 20.0), 4);
    assert!(failure_message(renderer, invalid_extent.clone(), render).contains("invalid logical"));
    invalid_extent.slot = 5;
    invalid_extent.bounds.size.width = f32::NAN;
    assert!(failure_message(renderer, invalid_extent, render).contains("invalid logical"));

    let invalid_height = canvas_primitive(canvas_id, 3, Size::new(20.0, f32::NAN), 32);
    assert!(failure_message(renderer, invalid_height, render).contains("invalid logical"));
    let negative_height = canvas_primitive(canvas_id, 3, Size::new(20.0, -1.0), 33);
    assert!(failure_message(renderer, negative_height, render).contains("invalid logical"));

    let before_zero_height = probe.renders.load(Ordering::Relaxed);
    render(
        renderer,
        &canvas_list(canvas_id, 3, Size::new(20.0, 0.0), 34, false),
    )
    .unwrap();
    assert_eq!(probe.renders.load(Ordering::Relaxed), before_zero_height);

    let mut overflow = canvas_primitive(canvas_id, 3, Size::new(32.0, 20.0), 6);
    overflow.resolution_scale = f32::MAX;
    assert!(failure_message(renderer, overflow, render).contains("cannot be represented"));

    let excessive = canvas_primitive(canvas_id, 3, Size::new(20_000.0, 1.0), 7);
    assert!(failure_message(renderer, excessive, render).contains("max_texture_dimension_2d"));
    let excessive_height = canvas_primitive(canvas_id, 3, Size::new(1.0, 20_000.0), 36);
    assert!(
        failure_message(renderer, excessive_height, render).contains("max_texture_dimension_2d")
    );

    let oversized = canvas_primitive(canvas_id, 3, Size::new(10_000.0, 10_000.0), 8);
    assert!(failure_message(renderer, oversized, render).contains("cache budget"));

    let missing = canvas_primitive(GpuCanvasId::fresh(), 1, Size::new(32.0, 20.0), 9);
    assert!(failure_message(renderer, missing, render).contains("missing registration"));

    let mut constrained = DisplayList::new();
    constrained.push_gpu_canvas(canvas_primitive(canvas_id, 3, Size::new(64.0, 40.0), 10));
    constrained.push_gpu_canvas(canvas_primitive(canvas_id, 3, Size::new(64.0, 40.0), 11));
    let before_constrained = probe.renders.load(Ordering::Relaxed);
    render(renderer, &constrained).unwrap();
    assert_eq!(
        probe.renders.load(Ordering::Relaxed),
        before_constrained + 1
    );
    assert_eq!(renderer.gpu_canvas_stats().failures_this_frame, 1);
    assert!(renderer.gpu_canvas_stats().allocated_bytes <= 16 * 1024);
    assert!(
        renderer.take_gpu_canvas_diagnostics()[0]
            .message
            .contains("visible canvases")
    );

    probe.fail.store(true, Ordering::Relaxed);
    let before_failure = probe.renders.load(Ordering::Relaxed);
    let failing = canvas_list(canvas_id, 3, Size::new(64.0, 40.0), 0, false);
    render(renderer, &failing).unwrap();
    assert_eq!(renderer.gpu_canvas_stats().failures_this_frame, 1);
    let failed = renderer.take_gpu_canvas_diagnostics();
    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0].kind, GpuCanvasDiagnosticKind::Failed);
    assert!(
        failed[0]
            .message
            .contains("intentional integration-test failure")
    );

    render(renderer, &failing).unwrap();
    assert!(renderer.take_gpu_canvas_diagnostics().is_empty());
    assert_eq!(probe.renders.load(Ordering::Relaxed), before_failure + 1);

    probe.fail.store(false, Ordering::Relaxed);
    let recovered = canvas_list(canvas_id, 4, Size::new(64.0, 40.0), 0, false);
    render(renderer, &recovered).unwrap();
    let recovered = renderer.take_gpu_canvas_diagnostics();
    assert_eq!(recovered.len(), 1);
    assert_eq!(recovered[0].kind, GpuCanvasDiagnosticKind::Recovered);

    let effected = canvas_list(canvas_id, 5, Size::new(64.0, 40.0), 0, true);
    render(renderer, &effected).unwrap();
    assert!(renderer.last_profile().effects.offscreen_layers >= 1);
    assert!(renderer.last_profile().effects.filter_passes >= 1);

    let mut duplicate = canvas_list(canvas_id, 6, Size::new(64.0, 40.0), 0, false);
    duplicate.push_gpu_canvas(canvas_primitive(canvas_id, 6, Size::new(64.0, 40.0), 0));
    render(renderer, &duplicate).unwrap();
    assert!(
        renderer
            .take_gpu_canvas_diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.message.contains("appears more than once"))
    );

    let resolved = canvas_list(canvas_id, 6, Size::new(64.0, 40.0), 0, false);
    let before_resolved = probe.renders.load(Ordering::Relaxed);
    render(renderer, &resolved).unwrap();
    assert_eq!(probe.renders.load(Ordering::Relaxed), before_resolved);
    assert_eq!(
        renderer.take_gpu_canvas_diagnostics()[0].kind,
        GpuCanvasDiagnosticKind::Recovered
    );

    let retained = canvas_list(canvas_id, 7, Size::new(64.0, 40.0), 0, false);
    render(renderer, &retained).unwrap();
    let before_recreate = probe.renders.load(Ordering::Relaxed);
    renderer.recreate_surface(window.clone()).unwrap();
    render(renderer, &retained).unwrap();
    assert_eq!(probe.renders.load(Ordering::Relaxed), before_recreate);
    assert_eq!(renderer.gpu_canvas_stats().hits_this_frame, 1);

    let creation_failure = canvas_list(retry_id, 1, Size::new(32.0, 20.0), 0, false);
    render(renderer, &creation_failure).unwrap();
    assert_eq!(retry_probe.creates.load(Ordering::Relaxed), 1);
    assert!(
        renderer.take_gpu_canvas_diagnostics()[0]
            .message
            .contains("factory failure")
    );
    render(renderer, &creation_failure).unwrap();
    assert_eq!(retry_probe.creates.load(Ordering::Relaxed), 1);
    assert!(renderer.take_gpu_canvas_diagnostics().is_empty());

    retry_probe.create_fail.store(false, Ordering::Relaxed);
    let creation_recovery = canvas_list(retry_id, 2, Size::new(32.0, 20.0), 0, false);
    render(renderer, &creation_recovery).unwrap();
    assert_eq!(retry_probe.creates.load(Ordering::Relaxed), 2);
    assert_eq!(retry_probe.renders.load(Ordering::Relaxed), 1);
    assert_eq!(
        renderer.take_gpu_canvas_diagnostics()[0].kind,
        GpuCanvasDiagnosticKind::Recovered
    );
}

/// Renders one invalid `primitive` and returns its single diagnostic message.
fn failure_message(
    renderer: &mut SurfaceRenderer,
    primitive: GpuCanvasPrimitive,
    render: &mut impl FnMut(&mut SurfaceRenderer, &DisplayList) -> Result<(), RendererError>,
) -> String {
    let mut list = DisplayList::new();
    list.push_gpu_canvas(primitive);
    render(renderer, &list).unwrap();
    let diagnostics = renderer.take_gpu_canvas_diagnostics();
    assert_eq!(diagnostics.len(), 1);
    diagnostics[0].message.clone()
}

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
pub(super) struct CanvasProbe {
    pub(super) creates: AtomicUsize,
    pub(super) renders: AtomicUsize,
    pub(super) extent: AtomicU64,
    pub(super) canvas: AtomicU64,
    pub(super) generation: AtomicU64,
    pub(super) last_frame: AtomicU64,
    pub(super) last_slot: AtomicUsize,
    pub(super) create_fail: AtomicBool,
    pub(super) fail: AtomicBool,
}

pub(super) struct ComputeFactory(pub(super) Arc<CanvasProbe>);

pub(super) struct RequiredFeatureFactory;

pub(super) struct RequiredLimitFactory;

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

impl GpuCanvasFactory for RequiredLimitFactory {
    /// Requests a bind-group capacity beyond every usable WGPU adapter.
    fn requirements(&self) -> GpuCanvasRequirements {
        GpuCanvasRequirements::default()
            .required_limits(wgpu::Limits {
                max_bind_groups: u32::MAX,
                ..wgpu::Limits::default()
            })
            .reason("validates adapter and shared-device limit rejection")
    }

    /// Fails if negotiation mistakenly reaches factory creation.
    ///
    /// # Errors
    /// Always returns a test error when called.
    fn create(
        &self,
        _context: &GpuCanvasDeviceContext<'_>,
    ) -> Result<Box<dyn GpuCanvasRenderer>, GpuCanvasError> {
        Err(GpuCanvasError::new(
            "required device limit should fail before factory creation",
        ))
    }
}

impl GpuCanvasFactory for ComputeFactory {
    fn create(
        &self,
        context: &GpuCanvasDeviceContext<'_>,
    ) -> Result<Box<dyn GpuCanvasRenderer>, GpuCanvasError> {
        assert_eq!(context.features(), context.device().features());
        assert_eq!(context.limits(), &context.device().limits());
        assert!(context.target_format().is_srgb());
        assert_ne!(context.device_generation(), 0);
        self.0
            .generation
            .store(context.device_generation(), Ordering::Relaxed);
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
    features: wgpu::Features,
    format: wgpu::TextureFormat,
    queue_timestamp_period: f32,
    compute: wgpu::ComputePipeline,
    compute_group: wgpu::BindGroup,
    render: wgpu::RenderPipeline,
    render_group: wgpu::BindGroup,
    _color: wgpu::Buffer,
}

impl ComputeRenderer {
    /// Creates GPU pipelines from `context` and retains `probe` for callback assertions.
    ///
    /// Returns the renderer for subsequent canvas revisions.
    fn new(context: &GpuCanvasDeviceContext<'_>, probe: Arc<CanvasProbe>) -> Self {
        let device = context.device();
        let queue_timestamp_period = context.queue().get_timestamp_period();
        assert!(queue_timestamp_period.is_finite() && queue_timestamp_period > 0.0);
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
            features: context.features(),
            format: context.target_format(),
            queue_timestamp_period,
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
        assert_eq!(context.device().features(), self.features);
        assert_eq!(context.target_format(), self.format);
        assert_eq!(
            context.queue().get_timestamp_period(),
            self.queue_timestamp_period
        );
        assert_eq!(
            context.canvas_id().get(),
            self.probe.canvas.load(Ordering::Relaxed)
        );
        assert_eq!(
            context.object_id(),
            RenderObjectId::new(ProfileDomain::Ui, 777)
        );
        self.probe
            .last_slot
            .store(context.slot() as usize, Ordering::Relaxed);
        let frame = context.frame_number();
        let previous = self.probe.last_frame.swap(frame, Ordering::Relaxed);
        assert!(frame > previous, "canvas frame numbers must increase");
        let [width, height] = context.physical_extent();
        let bounds = context.logical_bounds();
        let scale = f64::from(context.scale_factor()) * f64::from(context.resolution_scale());
        assert_eq!(
            [width, height],
            [bounds.size.width, bounds.size.height]
                .map(|dimension| (f64::from(dimension) * scale).ceil() as u32)
        );
        self.probe.extent.store(
            (u64::from(width) << 32) | u64::from(height),
            Ordering::Relaxed,
        );
        if self.probe.fail.load(Ordering::Relaxed) {
            return Err(GpuCanvasError::new("intentional integration-test failure"));
        }
        let target_ptr = context.target_view() as *const wgpu::TextureView;
        let encoder_ptr = context.encoder() as *mut wgpu::CommandEncoder;
        let (encoder, target) = context.encoder_and_target();
        assert!(std::ptr::eq(target_ptr, target as *const _));
        assert!(std::ptr::eq(encoder_ptr, encoder as *mut _));
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
