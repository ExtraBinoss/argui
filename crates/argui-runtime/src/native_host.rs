//! UI-thread commits from an external JavaScript actor into the native tree.

use std::sync::mpsc::Sender;

use argui_host::Operation;
use argui_paint::{GpuCanvasId, ImageAsset, VectorAsset};
use argui_platform::{CloseBehavior, GlobalShortcut, TrayConfig, WindowKey, WindowSpec};
use argui_render::{DamageTracking, EffectRegistry, GpuCanvasRegistry};
use argui_schema::{AssetHandle, SchemaValue, builtin};
use argui_ui::{TreeUpdate, UiEvent, UiEventKind};

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

/// Rejects native viewport IDs absent from the renderer's validated registry.
///
/// `operations` is an uncommitted native transaction; `canvases` contains the
/// registrations installed in the target renderer. Missing or malformed IDs
/// are rejected before the host tree can change.
///
/// # Errors
///
/// Returns the first invalid or unregistered canvas identity.
pub fn validate_native_host_canvases(
    operations: &[Operation],
    canvases: &GpuCanvasRegistry,
) -> Result<(), String> {
    for operation in operations {
        if let Operation::SetProperty {
            property,
            value: Some(value),
            ..
        } = operation
            && *property == builtin::CANVAS_ID
        {
            let SchemaValue::Int(raw) = value else {
                return Err("invalid native canvas identity".into());
            };
            let Some(id) = u64::try_from(*raw).ok().and_then(GpuCanvasId::from_raw) else {
                return Err("invalid native canvas identity".into());
            };
            if canvases.get(id).is_none() {
                return Err(format!("unregistered native canvas {}", id.get()));
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

/// An application-level change requested by a native JavaScript presentation.
#[derive(Debug)]
pub enum NativeHostApplicationRequest {
    /// Replaces the tray configuration; `None` removes the tray icon.
    SetTray(Option<TrayConfig>, Sender<Result<(), String>>),
    /// Replaces all system-wide keyboard shortcuts.
    SetGlobalShortcuts(Vec<GlobalShortcut>, Sender<Result<(), String>>),
    /// Changes what happens when the main window receives a close request.
    SetCloseBehavior(CloseBehavior, Sender<Result<(), String>>),
    /// Makes the main window visible and requests keyboard focus.
    FocusWindow(Sender<Result<(), String>>),
    /// Opens a new application window with its own model-provided view.
    OpenWindow(WindowSpec, Sender<Result<(), String>>),
    /// Sends a text message to the model associated with an existing window.
    SendWindowMessage(WindowKey, String, Sender<Result<(), String>>),
    /// Reads the current title, logical client size, and visibility of one window.
    GetWindowInfo(WindowKey, Sender<Result<NativeWindowInfo, String>>),
    /// Changes one window's native title.
    SetWindowTitle(WindowKey, String, Sender<Result<(), String>>),
    /// Requests a new logical client size for one window.
    SetWindowSize(WindowKey, f64, f64, Sender<Result<(), String>>),
    /// Requests outer top-left coordinates in logical screen pixels.
    SetWindowPosition(WindowKey, f64, f64, Sender<Result<(), String>>),
    /// Enables or removes a window's native decorations.
    SetWindowDecorations(WindowKey, bool, Sender<Result<(), String>>),
}

/// Current information about one native application window.
#[derive(Clone, Debug, PartialEq)]
pub struct NativeWindowInfo {
    /// Stable application window key.
    pub window: WindowKey,
    /// Last title requested by the application.
    pub title: String,
    /// Current drawable client width in logical pixels.
    pub width: f64,
    /// Current drawable client height in logical pixels.
    pub height: f64,
    /// Outer top-left X coordinate when the backend reports it.
    pub x: Option<f64>,
    /// Outer top-left Y coordinate when the backend reports it.
    pub y: Option<f64>,
    /// Native visibility when the backend reports it.
    pub visible: Option<bool>,
    /// Whether the native title bar and borders are requested.
    pub decorations: bool,
    /// Whether native window transparency was requested at creation.
    pub transparent: bool,
    /// Requested desktop backdrop material, if any.
    pub backdrop: Option<argui_core::BackdropMaterial>,
    /// Whether the compositor currently provides desktop backdrop blur.
    pub backdrop_available: bool,
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
    /// Pointer coordinates and measured callback bounds when the event has a position.
    pub pointer: Option<NativePointerPosition>,
}

/// Pointer position in the window and relative to the callback's measured node.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NativePointerPosition {
    /// Horizontal window coordinate in logical pixels.
    pub x: f32,
    /// Vertical window coordinate in logical pixels.
    pub y: f32,
    /// Horizontal coordinate relative to the callback node.
    pub local_x: f32,
    /// Vertical coordinate relative to the callback node.
    pub local_y: f32,
    /// Measured callback-node width.
    pub width: f32,
    /// Measured callback-node height.
    pub height: f32,
}

impl NativePointerPosition {
    /// Resolves a pointer event against the measured bounds of its callback node.
    /// `event` supplies the window position; `layout` supplies current logical bounds.
    /// Returns `None` for events without a pointer or without measured bounds.
    #[must_use]
    pub fn from_event(event: &UiEvent, layout: &argui_layout::LayoutOutput) -> Option<Self> {
        let position = match &event.kind {
            UiEventKind::Pointer(pointer) | UiEventKind::PointerOutside(pointer) => {
                Some(pointer.position)
            }
            UiEventKind::Click(click) => click.position(),
            UiEventKind::ContextMenu { position, .. } | UiEventKind::Wheel { position, .. } => {
                Some(*position)
            }
            _ => None,
        }?;
        let bounds = layout
            .nodes
            .iter()
            .find(|node| node.node == event.current_target())?
            .bounds;
        Some(Self {
            x: position.x,
            y: position.y,
            local_x: position.x - bounds.origin.x,
            local_y: position.y - bounds.origin.y,
            width: bounds.size.width,
            height: bounds.size.height,
        })
    }
}
