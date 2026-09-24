//! UI-thread commits from an external JavaScript actor into the native tree.

use std::sync::mpsc::Sender;

use argui_host::Operation;
use argui_paint::{ImageAsset, VectorAsset};
use argui_platform::WindowKey;
use argui_render::{DamageTracking, EffectRegistry};
use argui_schema::{AssetHandle, SchemaValue};
use argui_ui::{TreeUpdate, UiEventKind};

use crate::CallbackDelivery;

mod wire;
pub use wire::{WireHostId, WireOperation, WireValue};

/// Decoded media owned by a native JavaScript presentation host.
#[derive(Clone, Debug, Default)]
pub struct NativeHostAssets {
    /// Decoded raster images addressed by their stable `ImageId`.
    pub images: Vec<ImageAsset>,
    /// Validated SVG vectors addressed by their stable `VectorId`.
    pub vectors: Vec<VectorAsset>,
}

/// Rejects asset handles that have not been registered with the native renderer.
///
/// * `operations` — decoded transaction operations to inspect before a host commit.
/// * `images` — raster assets currently registered with the renderer.
/// * `vectors` — SVG assets currently registered with the renderer.
///
/// # Errors
///
/// Returns an error naming the first unregistered image or vector handle.
pub fn validate_native_host_assets(
    operations: &[Operation],
    images: &[ImageAsset],
    vectors: &[VectorAsset],
) -> Result<(), String> {
    for operation in operations {
        if let Operation::SetProperty {
            value: Some(SchemaValue::Asset(handle)),
            ..
        } = operation
        {
            let registered = match handle {
                AssetHandle::Image(id) => images.iter().any(|asset| asset.id == *id),
                AssetHandle::Vector(id) => vectors.iter().any(|asset| asset.id == *id),
            };
            if !registered {
                return Err(format!("unregistered native asset {handle:?}"));
            }
        }
    }
    Ok(())
}

/// A batch posted from a JavaScript actor to the UI thread.
#[derive(Debug)]
pub struct NativeHostBatch {
    pub window: WindowKey,
    pub operations: Vec<WireOperation>,
    /// Renderer controls applied after the operations in this UI-thread batch.
    pub controls: Vec<NativeHostControl>,
    pub reply: Sender<Result<NativeHostCommit, String>>,
}

/// A renderer control sent by a native JavaScript presentation host.
#[derive(Clone, Debug)]
pub enum NativeHostControl {
    /// Changes the damage policy for subsequent frames of the target window.
    SetDamageTracking(DamageTracking),
    /// Enables or disables frame profile work for the target window.
    SetRendererProfiling(bool),
    /// Replaces all host-supplied custom effects after preparing GPU pipelines.
    ReplaceEffects(EffectRegistry),
    /// Registers one SVG vector before a host batch can reference it.
    RegisterVector(VectorAsset),
}

/// Work performed by a native host commit on the UI thread.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeHostCommit {
    pub update: TreeUpdate,
    pub changed_nodes: usize,
}

/// One native event delivered to a live JavaScript callback.
#[derive(Clone, Debug, PartialEq)]
pub struct NativeHostDelivery {
    pub callback: CallbackDelivery,
    pub kind: UiEventKind,
}
