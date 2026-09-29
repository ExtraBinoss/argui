//! Public metadata, transport requirements and thread contracts of native GPU frames.
use argui_render::{
    ExternalFrame, ExternalFrameDescriptor, ExternalFrameFormat, ExternalFrameMemory,
    ImportedExternalFrame, wgpu,
};

/// Reject empty and device-limited extents before any native graphics call.
#[test]
fn external_frame_extent_validation_covers_empty_limits_and_boundaries() {
    for size in [
        [0, 0],
        [0, 1],
        [1, 0],
        [17, 16],
        [16, 17],
        [u32::MAX, u32::MAX],
    ] {
        assert!(
            ExternalFrameDescriptor {
                size,
                format: ExternalFrameFormat::Rgba8
            }
            .validate(16)
            .is_err()
        );
    }
    for size in [[1, 1], [16, 1], [1, 16], [16, 16]] {
        assert!(
            ExternalFrameDescriptor {
                size,
                format: ExternalFrameFormat::Rgba8Srgb
            }
            .validate(16)
            .is_ok()
        );
    }
    assert!(
        ExternalFrameDescriptor {
            size: [1, 1],
            format: ExternalFrameFormat::Rgba8
        }
        .validate(0)
        .is_err()
    );
}

/// Preserve the producer's declared color interpretation, without implicit conversion.
#[test]
fn packed_formats_map_to_exact_sampling_formats() {
    for (format, expected) in [
        (ExternalFrameFormat::Rgba8, wgpu::TextureFormat::Rgba8Unorm),
        (
            ExternalFrameFormat::Rgba8Srgb,
            wgpu::TextureFormat::Rgba8UnormSrgb,
        ),
        (ExternalFrameFormat::Bgra8, wgpu::TextureFormat::Bgra8Unorm),
        (
            ExternalFrameFormat::Bgra8Srgb,
            wgpu::TextureFormat::Bgra8UnormSrgb,
        ),
    ] {
        assert_eq!(format.texture_format(), expected);
    }
}

/// Native handles and submission leases remain usable across producer/render threads.
#[test]
#[cfg(not(target_arch = "wasm32"))]
fn native_frame_ownership_and_optional_requirements_are_explicit() {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<ExternalFrame>();
    send_sync::<ExternalFrameMemory>();
    send_sync::<ImportedExternalFrame>();
    let requirements = ExternalFrame::requirements();
    assert!(requirements.required_features.is_empty());
    #[cfg(target_os = "linux")]
    assert_eq!(
        requirements.optional_features,
        wgpu::Features::VULKAN_EXTERNAL_MEMORY_DMA_BUF
    );
    #[cfg(not(target_os = "linux"))]
    assert!(requirements.optional_features.is_empty());
}

/// Padding is permitted; invalid row widths and arithmetic overflow are rejected.
#[test]
fn native_row_layout_handles_padding_and_overflow() {
    let descriptor = ExternalFrameDescriptor {
        size: [16, 16],
        format: ExternalFrameFormat::Bgra8,
    };
    for (stride, offset) in [(64, 0), (128, 32), (64, u64::MAX - 1024)] {
        assert!(descriptor.validate_row_layout(stride, offset).is_ok());
    }
    for (stride, offset) in [(0, 0), (63, 0), (64, u64::MAX - 1023), (u64::MAX, 0)] {
        assert!(descriptor.validate_row_layout(stride, offset).is_err());
    }
    assert!(
        ExternalFrameDescriptor {
            size: [0, 0],
            format: ExternalFrameFormat::Rgba8
        }
        .validate_row_layout(0, 0)
        .is_err()
    );
}
