use std::collections::{HashMap, HashSet};

use argui_paint::{DisplayCommand, DisplayList, GpuCanvasId, GpuCanvasPrimitive, ImageSampling};
use web_time::Instant;

use crate::image::pipeline::{ImageInstance, ImagePipeline};

use super::{
    GpuCanvasDeviceContext, GpuCanvasDiagnostic, GpuCanvasDiagnosticKind, GpuCanvasFailureStage,
    GpuCanvasRegistry, GpuCanvasRenderContext, GpuCanvasRenderer, GpuCanvasStats,
    cache::{CanvasCache, CanvasExtent, CanvasKey},
    target::{clear_target, placeholder_texture},
};

enum RendererSlot {
    Ready(Box<dyn GpuCanvasRenderer>),
    Failed { revision: u64, message: String },
}

#[derive(Clone, Debug)]
struct FailureRecord {
    stage: GpuCanvasFailureStage,
    label: String,
    message: String,
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct DrawSource {
    key: Option<CanvasKey>,
    sampling: ImageSampling,
}

enum CommandPlan {
    Selected {
        key: CanvasKey,
        extent: CanvasExtent,
        label: String,
    },
    Placeholder {
        key: CanvasKey,
        label: String,
        stage: GpuCanvasFailureStage,
        message: String,
    },
    Empty,
}

pub(crate) struct CanvasPreparation {
    pub changed: bool,
    pub command_buffers: Vec<wgpu::CommandBuffer>,
}

pub(crate) struct CanvasGpu {
    registry: GpuCanvasRegistry,
    renderers: HashMap<GpuCanvasId, RendererSlot>,
    cache: CanvasCache,
    pipeline: ImagePipeline,
    _placeholder_texture: wgpu::Texture,
    placeholder_linear: wgpu::BindGroup,
    placeholder_nearest: wgpu::BindGroup,
    draws: Vec<DrawSource>,
    failures: HashMap<CanvasKey, FailureRecord>,
    diagnostics: Vec<GpuCanvasDiagnostic>,
    stats: GpuCanvasStats,
    format: wgpu::TextureFormat,
    generation: u64,
    max_dimension: u32,
    frame: u64,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl CanvasGpu {
    /// Creates a surface-local canvas manager and deterministic fallback texture.
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        generation: u64,
        registry: GpuCanvasRegistry,
        budget: usize,
    ) -> Self {
        let pipeline = ImagePipeline::new(device, format);
        let placeholder_texture = placeholder_texture(device, queue, format);
        let placeholder_view = placeholder_texture.create_view(&Default::default());
        let placeholder_linear =
            pipeline.texture_group(device, &placeholder_view, ImageSampling::Linear);
        let placeholder_nearest =
            pipeline.texture_group(device, &placeholder_view, ImageSampling::Nearest);
        Self {
            registry,
            renderers: HashMap::new(),
            cache: CanvasCache::new(format, budget),
            pipeline,
            _placeholder_texture: placeholder_texture,
            placeholder_linear,
            placeholder_nearest,
            draws: Vec::new(),
            failures: HashMap::new(),
            diagnostics: Vec::new(),
            stats: GpuCanvasStats::default(),
            format,
            generation,
            max_dimension: device.limits().max_texture_dimension_2d,
            frame: 0,
        }
    }

    /// Plans, renders and prepares compositor data for `display_list` canvases.
    ///
    /// The returned command buffers contain only successful canvas work and
    /// must be submitted before the main compositor buffer.
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        display_list: &DisplayList,
        scale_factor: f32,
    ) -> CanvasPreparation {
        let started = Instant::now();
        self.frame = self.frame.wrapping_add(1);
        self.stats = GpuCanvasStats::default();
        let previous_draws = std::mem::take(&mut self.draws);
        let canvases = display_list
            .commands()
            .iter()
            .filter_map(|command| match command {
                DisplayCommand::GpuCanvas(canvas) => Some(canvas),
                _ => None,
            })
            .collect::<Vec<_>>();
        let (plans, required) = self.plan(&canvases, scale_factor);
        self.cache.reserve(&required);
        for (key, extent) in &required {
            self.cache
                .ensure(device, &self.pipeline, *key, *extent, self.frame);
        }

        let mut changed = false;
        let mut command_buffers = Vec::new();
        for (canvas, plan) in canvases.iter().zip(plans) {
            match plan {
                CommandPlan::Selected { key, extent, label } => {
                    let source = self.prepare_selected(
                        device,
                        queue,
                        canvas,
                        key,
                        extent,
                        &label,
                        scale_factor,
                        &mut command_buffers,
                        &mut changed,
                    );
                    self.draws.push(source);
                }
                CommandPlan::Placeholder {
                    key,
                    label,
                    stage,
                    message,
                } => {
                    self.stats.failures_this_frame += 1;
                    self.record_failure(key, canvas, label, stage, message);
                    self.draws.push(DrawSource {
                        key: None,
                        sampling: canvas.sampling,
                    });
                }
                CommandPlan::Empty => self.draws.push(DrawSource {
                    key: None,
                    sampling: canvas.sampling,
                }),
            }
        }

        let mut instances = Vec::with_capacity(canvases.len());
        let mut clips = Vec::new();
        for canvas in canvases {
            instances.push(ImageInstance::from_gpu_canvas(
                canvas,
                scale_factor,
                &mut clips,
            ));
        }
        changed |= self.draws != previous_draws;
        changed |= self.pipeline.write(device, queue, &instances, &clips);
        self.stats.entries = self.cache.len();
        self.stats.allocated_bytes = self.cache.allocated_bytes();
        self.stats.encode_time = started.elapsed();
        CanvasPreparation {
            changed,
            command_buffers,
        }
    }

    /// Selects a deterministic within-budget subset of visible `canvases`.
    fn plan(
        &self,
        canvases: &[&GpuCanvasPrimitive],
        scale_factor: f32,
    ) -> (Vec<CommandPlan>, HashMap<CanvasKey, CanvasExtent>) {
        let mut plans = Vec::with_capacity(canvases.len());
        let mut required = HashMap::new();
        let mut seen = HashSet::new();
        let mut reserved = 0_u64;
        for canvas in canvases {
            let key = CanvasKey::from(*canvas);
            let label = self.registry.get(canvas.canvas).map_or_else(
                || format!("unregistered-{}", canvas.canvas.get()),
                |registration| registration.label().to_owned(),
            );
            if !seen.insert(key) {
                plans.push(CommandPlan::Placeholder {
                    key,
                    label,
                    stage: GpuCanvasFailureStage::Validation,
                    message: format!(
                        "retained object {} slot {} appears more than once in the frame",
                        key.object.value, key.slot
                    ),
                });
                continue;
            }
            if self.registry.get(canvas.canvas).is_none() {
                plans.push(CommandPlan::Placeholder {
                    key,
                    label,
                    stage: GpuCanvasFailureStage::Validation,
                    message: "the display list references a missing registration".into(),
                });
                continue;
            }
            let extent = match self.cache.extent(canvas, scale_factor, self.max_dimension) {
                Ok(Some(extent)) => extent,
                Ok(None) => {
                    plans.push(CommandPlan::Empty);
                    continue;
                }
                Err(message) => {
                    plans.push(CommandPlan::Placeholder {
                        key,
                        label,
                        stage: GpuCanvasFailureStage::Allocation,
                        message,
                    });
                    continue;
                }
            };
            let Some(next) = reserved.checked_add(extent.bytes) else {
                plans.push(CommandPlan::Placeholder {
                    key,
                    label,
                    stage: GpuCanvasFailureStage::Allocation,
                    message: "visible canvas byte demand overflowed".into(),
                });
                continue;
            };
            if next > self.cache.budget() {
                plans.push(CommandPlan::Placeholder {
                    key,
                    label,
                    stage: GpuCanvasFailureStage::Allocation,
                    message: format!(
                        "visible canvases need more than the {} byte cache budget",
                        self.cache.budget()
                    ),
                });
                continue;
            }
            reserved = next;
            required.insert(key, extent);
            plans.push(CommandPlan::Selected { key, extent, label });
        }
        (plans, required)
    }

    #[allow(clippy::too_many_arguments)]
    /// Prepares one validated canvas and returns the compositor texture source.
    fn prepare_selected(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        canvas: &GpuCanvasPrimitive,
        key: CanvasKey,
        extent: CanvasExtent,
        label: &str,
        scale_factor: f32,
        command_buffers: &mut Vec<wgpu::CommandBuffer>,
        changed: &mut bool,
    ) -> DrawSource {
        self.ensure_renderer(device, queue, canvas.canvas, canvas.content_revision);
        let renderer_error = match self.renderers.get(&canvas.canvas) {
            Some(RendererSlot::Failed { message, .. }) => Some(message.clone()),
            _ => None,
        };
        if let Some(message) = renderer_error {
            self.stats.failures_this_frame += 1;
            self.record_failure(
                key,
                canvas,
                label.into(),
                GpuCanvasFailureStage::Creation,
                message,
            );
            return DrawSource {
                key: None,
                sampling: canvas.sampling,
            };
        }

        let needs_render = self
            .cache
            .get(key)
            .is_some_and(|entry| entry.needs_render(canvas.content_revision));
        if !needs_render {
            let valid = self
                .cache
                .get(key)
                .is_some_and(super::cache::CacheEntry::valid);
            if valid {
                self.stats.hits_this_frame += 1;
                self.record_recovery(key, canvas, label);
                return DrawSource {
                    key: Some(key),
                    sampling: canvas.sampling,
                };
            }
            self.stats.failures_this_frame += 1;
            return DrawSource {
                key: None,
                sampling: canvas.sampling,
            };
        }

        self.stats.renders_this_frame += 1;
        *changed = true;
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("argui-gpu-canvas-encoder"),
        });
        let target = self
            .cache
            .get(key)
            .expect("reserved canvas target")
            .target();
        clear_target(&mut encoder, target);
        let result = {
            let renderer = match self.renderers.get_mut(&canvas.canvas) {
                Some(RendererSlot::Ready(renderer)) => renderer,
                _ => unreachable!("renderer creation was checked"),
            };
            renderer.render(&mut GpuCanvasRenderContext {
                device,
                queue,
                encoder: &mut encoder,
                target,
                format: self.format,
                extent: extent.size,
                logical_bounds: canvas.bounds,
                scale_factor,
                resolution_scale: canvas.resolution_scale,
                canvas: canvas.canvas,
                object: canvas.object,
                slot: canvas.slot,
                frame: self.frame,
            })
        };
        match result {
            Ok(()) => {
                command_buffers.push(encoder.finish());
                self.cache
                    .get_mut(key)
                    .expect("reserved canvas target")
                    .mark_success(canvas.content_revision);
                self.record_recovery(key, canvas, label);
                DrawSource {
                    key: Some(key),
                    sampling: canvas.sampling,
                }
            }
            Err(error) => {
                self.cache
                    .get_mut(key)
                    .expect("reserved canvas target")
                    .mark_failure(canvas.content_revision);
                self.stats.failures_this_frame += 1;
                self.record_failure(
                    key,
                    canvas,
                    label.into(),
                    GpuCanvasFailureStage::Render,
                    error.to_string(),
                );
                DrawSource {
                    key: None,
                    sampling: canvas.sampling,
                }
            }
        }
    }

    /// Lazily creates `id`, retrying a failed factory only for a new `revision`.
    fn ensure_renderer(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        id: GpuCanvasId,
        revision: u64,
    ) {
        let reusable = match self.renderers.get(&id) {
            Some(RendererSlot::Ready(_)) => true,
            Some(RendererSlot::Failed {
                revision: attempted,
                ..
            }) => *attempted == revision,
            None => false,
        };
        if reusable {
            return;
        }
        let registration = self.registry.get(id).expect("validated registration");
        let context = GpuCanvasDeviceContext {
            device,
            queue,
            features: device.features(),
            limits: device.limits(),
            format: self.format,
            generation: self.generation,
        };
        let renderer = match registration.factory().create(&context) {
            Ok(renderer) => RendererSlot::Ready(renderer),
            Err(error) => RendererSlot::Failed {
                revision,
                message: error.to_string(),
            },
        };
        self.renderers.insert(id, renderer);
    }

    /// Records and emits a changed failure for retained `key` exactly once.
    fn record_failure(
        &mut self,
        key: CanvasKey,
        canvas: &GpuCanvasPrimitive,
        label: String,
        stage: GpuCanvasFailureStage,
        message: String,
    ) {
        if self
            .failures
            .get(&key)
            .is_some_and(|current| current.stage == stage && current.message == message)
        {
            return;
        }
        let readable = format!("GPU canvas '{label}' failed during {stage:?}: {message}");
        eprintln!("{readable}");
        self.failures.insert(
            key,
            FailureRecord {
                stage,
                label: label.clone(),
                message: message.clone(),
            },
        );
        self.diagnostics.push(GpuCanvasDiagnostic {
            kind: GpuCanvasDiagnosticKind::Failed,
            stage,
            label,
            canvas: canvas.canvas,
            object: canvas.object,
            slot: canvas.slot,
            message: readable,
        });
    }

    /// Emits one recovery when `key` previously had a recorded failure.
    fn record_recovery(
        &mut self,
        key: CanvasKey,
        canvas: &GpuCanvasPrimitive,
        fallback_label: &str,
    ) {
        let Some(previous) = self.failures.remove(&key) else {
            return;
        };
        let label = if previous.label.is_empty() {
            fallback_label.to_owned()
        } else {
            previous.label
        };
        self.diagnostics.push(GpuCanvasDiagnostic {
            kind: GpuCanvasDiagnosticKind::Recovered,
            stage: previous.stage,
            label: label.clone(),
            canvas: canvas.canvas,
            object: canvas.object,
            slot: canvas.slot,
            message: format!("GPU canvas '{label}' recovered"),
        });
    }

    /// Starts the shared textured-quad compositor's frame state.
    pub fn begin_frame(&mut self) {
        self.pipeline.begin_frame();
    }

    /// Clears transient draw state while retaining cache totals for a skipped frame.
    pub fn clear_frame_stats(&mut self) {
        self.draws.clear();
        self.stats = GpuCanvasStats {
            entries: self.cache.len(),
            allocated_bytes: self.cache.allocated_bytes(),
            ..GpuCanvasStats::default()
        };
    }

    /// Stores `region` and returns its dynamic uniform offset for effect rendering.
    pub fn target_offset(&mut self, queue: &wgpu::Queue, region: [f32; 4]) -> u32 {
        self.pipeline.target_offset(queue, region)
    }

    /// Draws prepared `instances` for `canvas_index` into `pass`.
    pub fn draw<'pass>(
        &'pass self,
        pass: &mut wgpu::RenderPass<'pass>,
        canvas_index: u32,
        instances: std::ops::Range<u32>,
        viewport_offset: u32,
    ) {
        let Some(source) = self.draws.get(canvas_index as usize) else {
            return;
        };
        let group = source
            .key
            .and_then(|key| {
                self.cache
                    .get(key)
                    .map(|entry| entry.group(source.sampling))
            })
            .unwrap_or(match source.sampling {
                ImageSampling::Nearest => &self.placeholder_nearest,
                ImageSampling::Linear => &self.placeholder_linear,
            });
        self.pipeline.draw(pass, group, instances, viewport_offset);
    }

    /// Returns the current frame and retained-cache statistics.
    pub fn stats(&self) -> GpuCanvasStats {
        self.stats
    }

    /// Drains failure and recovery diagnostics accumulated since the last call.
    pub fn take_diagnostics(&mut self) -> Vec<GpuCanvasDiagnostic> {
        std::mem::take(&mut self.diagnostics)
    }
}
