#[cfg(not(target_arch = "wasm32"))]
use argui_render::SurfaceRenderer;
use argui_render::{
    GpuCanvasDeviceContext, GpuCanvasError, GpuCanvasFactory, GpuCanvasRegistration,
    GpuCanvasRegistry, GpuCanvasRegistryError, GpuCanvasRenderContext, GpuCanvasRenderer,
    GpuCanvasRequirements, RendererConfig, wgpu,
};

#[derive(Clone, Copy)]
struct Factory {
    requirements: Option<wgpu::Features>,
}

impl GpuCanvasFactory for Factory {
    fn requirements(&self) -> GpuCanvasRequirements {
        self.requirements
            .map_or_else(GpuCanvasRequirements::default, |features| {
                GpuCanvasRequirements::default()
                    .optional_features(features)
                    .reason("uses an optional test capability")
            })
    }

    fn create(
        &self,
        _context: &GpuCanvasDeviceContext<'_>,
    ) -> Result<Box<dyn GpuCanvasRenderer>, GpuCanvasError> {
        Ok(Box::new(Renderer))
    }
}

struct Renderer;

impl GpuCanvasRenderer for Renderer {
    fn render(&mut self, context: &mut GpuCanvasRenderContext<'_>) -> Result<(), GpuCanvasError> {
        let _ = context.device();
        let _ = context.queue();
        let _ = context.encoder_and_target();
        Ok(())
    }
}

fn registration(label: &str) -> GpuCanvasRegistration {
    GpuCanvasRegistration::new(label, Factory { requirements: None })
}

#[test]
fn registration_clones_preserve_identity_and_registry_deduplicates_them() {
    let registration = registration("test.canvas");
    let clone = registration.clone();
    let registry = GpuCanvasRegistry::new([registration.clone(), clone]).unwrap();

    assert_eq!(registry.len(), 1);
    assert_eq!(registry.registrations()[0].id(), registration.id());
    assert_eq!(registry.registrations()[0].label(), "test.canvas");
    assert!(!registry.is_empty());
}

#[test]
fn registry_rejects_invalid_duplicate_and_unexplained_capabilities() {
    assert!(matches!(
        GpuCanvasRegistry::new([registration("")]),
        Err(GpuCanvasRegistryError::InvalidLabel(label)) if label.is_empty()
    ));
    assert!(matches!(
        GpuCanvasRegistry::new([registration(" padded")]),
        Err(GpuCanvasRegistryError::InvalidLabel(label)) if label == " padded"
    ));
    assert!(matches!(
        GpuCanvasRegistry::new([registration("line\nbreak")]),
        Err(GpuCanvasRegistryError::InvalidLabel(label)) if label == "line\nbreak"
    ));
    assert!(matches!(
        GpuCanvasRegistry::new([registration("same"), registration("same")]),
        Err(GpuCanvasRegistryError::DuplicateLabel(label)) if label == "same"
    ));

    struct Unexplained;
    impl GpuCanvasFactory for Unexplained {
        fn requirements(&self) -> GpuCanvasRequirements {
            GpuCanvasRequirements::default().optional_features(wgpu::Features::TIMESTAMP_QUERY)
        }

        fn create(
            &self,
            _context: &GpuCanvasDeviceContext<'_>,
        ) -> Result<Box<dyn GpuCanvasRenderer>, GpuCanvasError> {
            Ok(Box::new(Renderer))
        }
    }
    let unexplained = GpuCanvasRegistration::new("unexplained", Unexplained);
    assert!(matches!(
        GpuCanvasRegistry::new([unexplained]),
        Err(GpuCanvasRegistryError::MissingRequirementReason(label)) if label == "unexplained"
    ));

    struct WhitespaceReason;
    impl GpuCanvasFactory for WhitespaceReason {
        fn requirements(&self) -> GpuCanvasRequirements {
            GpuCanvasRequirements::default()
                .optional_features(wgpu::Features::TIMESTAMP_QUERY)
                .reason("  ")
        }

        fn create(
            &self,
            _context: &GpuCanvasDeviceContext<'_>,
        ) -> Result<Box<dyn GpuCanvasRenderer>, GpuCanvasError> {
            Ok(Box::new(Renderer))
        }
    }
    assert!(matches!(
        GpuCanvasRegistry::new([GpuCanvasRegistration::new(
            "whitespace-reason",
            WhitespaceReason,
        )]),
        Err(GpuCanvasRegistryError::MissingRequirementReason(label))
            if label == "whitespace-reason"
    ));
}

#[test]
fn requirement_builder_merges_capacity_and_alignment_limits_directionally() {
    let first = wgpu::Limits {
        max_bind_groups: 9,
        min_uniform_buffer_offset_alignment: 128,
        ..wgpu::Limits::default()
    };
    let second = wgpu::Limits {
        max_bind_groups: 7,
        min_uniform_buffer_offset_alignment: 64,
        ..wgpu::Limits::default()
    };
    let requirements = GpuCanvasRequirements::default()
        .required_features(wgpu::Features::TEXTURE_COMPRESSION_BC)
        .optional_features(wgpu::Features::TIMESTAMP_QUERY)
        .required_limits(first)
        .required_limits(second)
        .reason("tests direction-aware WGPU limit resolution");

    assert!(
        requirements
            .required_features
            .contains(wgpu::Features::TEXTURE_COMPRESSION_BC)
    );
    assert!(
        requirements
            .optional_features
            .contains(wgpu::Features::TIMESTAMP_QUERY)
    );
    assert_eq!(requirements.required_limits.max_bind_groups, 9);
    assert_eq!(
        requirements
            .required_limits
            .min_uniform_buffer_offset_alignment,
        64
    );
    assert!(requirements.reason.is_some());
}

#[test]
fn config_owns_an_immutable_registry_and_a_bounded_default_budget() {
    let registry = GpuCanvasRegistry::new([registration("configured")]).unwrap();
    let config = RendererConfig::default()
        .gpu_canvas_cache_bytes(4096)
        .gpu_canvases(registry.clone());

    assert_eq!(config.gpu_canvas_cache_bytes, 4096);
    assert_eq!(config.gpu_canvases.len(), 1);
    assert_eq!(
        RendererConfig::default().gpu_canvas_cache_bytes,
        128 * 1024 * 1024
    );
    assert_eq!(registry.len(), 1);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn public_canvas_types_preserve_renderer_thread_traits() {
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<GpuCanvasRegistration>();
    assert_send_sync::<GpuCanvasRegistry>();
    assert_send_sync::<RendererConfig>();
    assert_send_sync::<SurfaceRenderer>();
}

#[test]
fn application_errors_preserve_their_actionable_message() {
    let error = GpuCanvasError::new("pipeline creation failed");
    assert_eq!(error.message(), "pipeline creation failed");
    assert_eq!(error.to_string(), "pipeline creation failed");
}

#[derive(Clone)]
struct DeclaredRequirements(GpuCanvasRequirements);

impl GpuCanvasFactory for DeclaredRequirements {
    /// Returns the device contract declared for this test registration.
    fn requirements(&self) -> GpuCanvasRequirements {
        self.0.clone()
    }

    /// Creates the stateless test renderer without requiring an actual GPU.
    fn create(
        &self,
        _context: &GpuCanvasDeviceContext<'_>,
    ) -> Result<Box<dyn GpuCanvasRenderer>, GpuCanvasError> {
        Ok(Box::new(Renderer))
    }
}

/// Required features and non-baseline limits both demand a useful explanation.
#[test]
fn every_nonbaseline_requirement_needs_a_nonblank_reason() {
    let required_features =
        GpuCanvasRequirements::default().required_features(wgpu::Features::TIMESTAMP_QUERY);
    let required_limits = GpuCanvasRequirements::default().required_limits(wgpu::Limits {
        max_bind_groups: wgpu::Limits::default().max_bind_groups + 1,
        ..wgpu::Limits::default()
    });
    for (index, requirements) in [required_features, required_limits].into_iter().enumerate() {
        let label = format!("test.requirement-{index}");
        let registration =
            GpuCanvasRegistration::new(label.clone(), DeclaredRequirements(requirements.clone()));
        assert!(matches!(
            GpuCanvasRegistry::new([registration]),
            Err(GpuCanvasRegistryError::MissingRequirementReason(found)) if found == label
        ));

        let explained = GpuCanvasRegistration::new(
            label.clone(),
            DeclaredRequirements(requirements.reason("capability required for test")),
        );
        assert_eq!(
            GpuCanvasRegistry::new([explained]).unwrap().registrations()[0].label(),
            label
        );
    }
    assert!(GpuCanvasRegistry::default().is_empty());
}
