use std::{
    collections::HashMap,
    fmt,
    sync::{
        Arc, Mutex, Weak,
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    },
};

use argui_paint::GpuCanvasId;

use super::GpuCanvasFactory;

/// Device capabilities declared by one GPU-canvas factory.
#[derive(Clone, Debug, PartialEq)]
pub struct GpuCanvasRequirements {
    /// Features that must be enabled or renderer initialization fails.
    pub required_features: wgpu::Features,
    /// Features enabled when supported without forcing renderer fallback.
    pub optional_features: wgpu::Features,
    /// Minimum effective limits needed by the factory.
    pub required_limits: wgpu::Limits,
    /// Human-readable reason for any non-baseline capability.
    pub reason: Option<String>,
}

impl Default for GpuCanvasRequirements {
    fn default() -> Self {
        Self {
            required_features: wgpu::Features::empty(),
            optional_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            reason: None,
        }
    }
}

impl GpuCanvasRequirements {
    /// Adds features that must be enabled on the selected device.
    #[must_use]
    pub fn required_features(mut self, features: wgpu::Features) -> Self {
        self.required_features |= features;
        self
    }

    /// Adds features that should be enabled only when the adapter supports them.
    #[must_use]
    pub fn optional_features(mut self, features: wgpu::Features) -> Self {
        self.optional_features |= features;
        self
    }

    /// Merges required limits using WGPU's direction-aware limit ordering.
    #[must_use]
    pub fn required_limits(mut self, limits: wgpu::Limits) -> Self {
        self.required_limits = self.required_limits.or_better_values_from(&limits);
        self
    }

    /// Explains why the factory needs capabilities beyond the WebGPU baseline.
    #[must_use]
    pub fn reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// Returns whether the declaration needs only WGPU baseline capabilities.
    fn is_baseline(&self) -> bool {
        self.required_features.is_empty()
            && self.optional_features.is_empty()
            && self.required_limits == wgpu::Limits::default()
    }
}

type CanvasWake = Arc<dyn Fn(GpuCanvasId) + Send + Sync>;

struct RegistrationInner {
    id: GpuCanvasId,
    label: String,
    factory: Arc<dyn GpuCanvasFactory>,
    requirements: GpuCanvasRequirements,
    native_revision: AtomicU64,
    active_surfaces: AtomicUsize,
    wake_pending: AtomicBool,
    wake: Mutex<Option<CanvasWake>>,
}

/// Cloneable GPU-canvas registration with stable process-local identity.
#[derive(Clone)]
pub struct GpuCanvasRegistration(Arc<RegistrationInner>);

impl fmt::Debug for GpuCanvasRegistration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("GpuCanvasRegistration")
            .field("id", &self.id())
            .field("label", &self.label())
            .field("requirements", self.requirements())
            .finish_non_exhaustive()
    }
}

impl GpuCanvasRegistration {
    /// Creates a registration for `factory` with a stable diagnostic `label`.
    ///
    /// Label and requirement validation happens when constructing a
    /// [`GpuCanvasRegistry`]. Cloning this value preserves its identity.
    #[must_use]
    pub fn new(label: impl Into<String>, factory: impl GpuCanvasFactory) -> Self {
        let requirements = factory.requirements();
        Self(Arc::new(RegistrationInner {
            id: GpuCanvasId::fresh(),
            label: label.into(),
            factory: Arc::new(factory),
            requirements,
            native_revision: AtomicU64::new(0),
            active_surfaces: AtomicUsize::new(0),
            wake_pending: AtomicBool::new(false),
            wake: Mutex::new(None),
        }))
    }

    /// Returns the opaque identity referenced by [`argui_paint::GpuCanvasPrimitive`].
    #[must_use]
    pub fn id(&self) -> GpuCanvasId {
        self.0.id
    }

    /// Returns the stable non-empty diagnostic label.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.0.label
    }

    /// Returns the capabilities captured when the registration was created.
    #[must_use]
    pub fn requirements(&self) -> &GpuCanvasRequirements {
        &self.0.requirements
    }

    /// Marks newer application-owned pixels ready for the mounted canvas.
    ///
    /// The producer can keep its newest frame in a one-slot mailbox; this method
    /// passes no pixels through JavaScript and coalesces redraw wakeups. An
    /// unmounted canvas retains the revision without waking an idle window.
    pub fn invalidate(&self) {
        self.0.native_revision.fetch_add(1, Ordering::AcqRel);
        if self.0.active_surfaces.load(Ordering::Acquire) == 0
            || self.0.wake_pending.swap(true, Ordering::AcqRel)
        {
            return;
        }
        if let Some(wake) = self
            .0
            .wake
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
        {
            wake(self.id());
        }
    }

    /// Returns whether at least one renderer surface currently displays this canvas.
    #[must_use]
    pub fn is_mounted(&self) -> bool {
        self.0.active_surfaces.load(Ordering::Acquire) > 0
    }

    /// Returns the current native frame revision for retained texture caching.
    #[must_use]
    pub(crate) fn native_revision(&self) -> u64 {
        self.0.native_revision.load(Ordering::Acquire)
    }

    /// Records that one renderer surface now displays this registration.
    pub(crate) fn activate(&self) {
        self.0.active_surfaces.fetch_add(1, Ordering::AcqRel);
    }

    /// Records that one renderer surface no longer displays this registration.
    pub(crate) fn deactivate(&self) {
        self.0.active_surfaces.fetch_sub(1, Ordering::AcqRel);
    }

    /// Clears a coalesced wake before the renderer snapshots the revision.
    pub(crate) fn begin_frame(&self) {
        self.0.wake_pending.store(false, Ordering::Release);
    }

    /// Installs the native event-loop callback that wakes a mounted canvas.
    ///
    /// The callback receives the registered canvas ID and must schedule a
    /// retained paint frame. Replacing it when a window closes is allowed.
    pub fn set_wake(&self, wake: impl Fn(GpuCanvasId) + Send + Sync + 'static) {
        *self
            .0
            .wake
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Arc::new(wake));
    }

    /// Returns the immutable application factory owned by this registration.
    pub(crate) fn factory(&self) -> &dyn GpuCanvasFactory {
        self.0.factory.as_ref()
    }

    /// Returns whether `other` is a clone of this exact registration.
    fn same_registration(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

struct MailboxInner<T> {
    latest: Mutex<Option<T>>,
    registration: Mutex<Weak<RegistrationInner>>,
}

/// One-slot native frame mailbox for an application-owned canvas producer.
///
/// The producer and GPU callback each keep a clone. Publishing replaces an
/// unconsumed frame and wakes the mounted canvas at most once before its next
/// render. The payload never enters a JavaScript transaction.
pub struct GpuCanvasMailbox<T>(Arc<MailboxInner<T>>);

impl<T> Clone for GpuCanvasMailbox<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<T> Default for GpuCanvasMailbox<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> GpuCanvasMailbox<T> {
    /// Creates an empty, unbound mailbox with capacity for one frame.
    #[must_use]
    pub fn new() -> Self {
        Self(Arc::new(MailboxInner {
            latest: Mutex::new(None),
            registration: Mutex::new(Weak::new()),
        }))
    }

    /// Binds this mailbox to `registration` after the factory has been created.
    ///
    /// The mailbox retains a weak registration reference, so a factory may own
    /// a clone of the mailbox without creating a reference cycle.
    pub fn bind(&self, registration: &GpuCanvasRegistration) {
        *self
            .0
            .registration
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Arc::downgrade(&registration.0);
    }

    /// Replaces the pending frame and signals the registered native viewport.
    ///
    /// Returns the displaced unconsumed frame, if any. The caller may recycle
    /// its backing storage instead of allocating for every decoded frame.
    pub fn publish(&self, frame: T) -> Option<T> {
        let displaced = self
            .0
            .latest
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .replace(frame);
        if let Some(registration) = self
            .0
            .registration
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .upgrade()
        {
            GpuCanvasRegistration(registration).invalidate();
        }
        displaced
    }

    /// Takes the newest pending frame for the GPU callback, if present.
    pub fn take(&self) -> Option<T> {
        self.0
            .latest
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
    }

    /// Drops the pending frame when playback stops or the viewport unmounts.
    pub fn clear(&self) {
        self.0
            .latest
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
    }
}

/// Validation error returned while building an immutable canvas registry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GpuCanvasRegistryError {
    /// A label was empty, untrimmed or contained control characters.
    InvalidLabel(String),
    /// Two distinct registrations used the same diagnostic label.
    DuplicateLabel(String),
    /// A repeated identity did not refer to the same cloned registration.
    InconsistentDuplicate(GpuCanvasId),
    /// Non-baseline capabilities did not include an explanatory reason.
    MissingRequirementReason(String),
}

impl fmt::Display for GpuCanvasRegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLabel(label) => write!(
                formatter,
                "GPU-canvas registration label must be non-empty, trimmed and free of control characters: {label:?}"
            ),
            Self::DuplicateLabel(label) => {
                write!(
                    formatter,
                    "GPU-canvas registration label '{label}' is duplicated"
                )
            }
            Self::InconsistentDuplicate(id) => write!(
                formatter,
                "GPU-canvas registration {} has inconsistent duplicate definitions",
                id.get()
            ),
            Self::MissingRequirementReason(label) => write!(
                formatter,
                "GPU-canvas registration '{label}' must explain its non-baseline device requirements"
            ),
        }
    }
}

impl std::error::Error for GpuCanvasRegistryError {}

/// Immutable validated collection of application GPU-canvas registrations.
#[derive(Clone, Debug, Default)]
pub struct GpuCanvasRegistry {
    registrations: Arc<[GpuCanvasRegistration]>,
}

impl GpuCanvasRegistry {
    /// Validates, deduplicates cloned registrations and builds a registry.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid labels, duplicate labels, inconsistent
    /// identities or unexplained non-baseline requirements.
    pub fn new(
        registrations: impl IntoIterator<Item = GpuCanvasRegistration>,
    ) -> Result<Self, GpuCanvasRegistryError> {
        let mut by_id: HashMap<GpuCanvasId, GpuCanvasRegistration> = HashMap::new();
        let mut labels: HashMap<String, GpuCanvasId> = HashMap::new();
        for registration in registrations {
            let label = registration.label();
            if label.is_empty() || label.trim() != label || label.chars().any(char::is_control) {
                return Err(GpuCanvasRegistryError::InvalidLabel(label.into()));
            }
            if let Some(existing) = by_id.get(&registration.id()) {
                if !existing.same_registration(&registration) {
                    return Err(GpuCanvasRegistryError::InconsistentDuplicate(
                        registration.id(),
                    ));
                }
                continue;
            }
            if labels.insert(label.into(), registration.id()).is_some() {
                return Err(GpuCanvasRegistryError::DuplicateLabel(label.into()));
            }
            if !registration.requirements().is_baseline()
                && registration
                    .requirements()
                    .reason
                    .as_deref()
                    .is_none_or(|reason| reason.trim().is_empty())
            {
                return Err(GpuCanvasRegistryError::MissingRequirementReason(
                    label.into(),
                ));
            }
            by_id.insert(registration.id(), registration);
        }
        let mut registrations = by_id.into_values().collect::<Vec<_>>();
        registrations.sort_by_key(GpuCanvasRegistration::id);
        Ok(Self {
            registrations: registrations.into(),
        })
    }

    /// Returns whether no factories are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.registrations.is_empty()
    }

    /// Returns the number of distinct registrations.
    #[must_use]
    pub fn len(&self) -> usize {
        self.registrations.len()
    }

    /// Returns registrations in stable identity order.
    #[must_use]
    pub fn registrations(&self) -> &[GpuCanvasRegistration] {
        &self.registrations
    }

    /// Finds the registration for opaque `id`, if this validated registry contains it.
    ///
    /// `id` may originate from a native host transaction; an unknown ID
    /// returns `None` and must not be rendered.
    pub fn get(&self, id: GpuCanvasId) -> Option<&GpuCanvasRegistration> {
        self.registrations
            .binary_search_by_key(&id, GpuCanvasRegistration::id)
            .ok()
            .map(|index| &self.registrations[index])
    }

    /// Aggregates and validates requirements against one adapter attempt.
    ///
    /// Optional features are intersected with `adapter_features`; `profiling`
    /// requests timestamp queries when available.
    pub(crate) fn device_requirements(
        &self,
        adapter_features: wgpu::Features,
        adapter_limits: &wgpu::Limits,
        profiling: bool,
    ) -> Result<DeviceRequirements, crate::RendererError> {
        let mut required_features = wgpu::Features::empty();
        let mut optional_features = wgpu::Features::empty();
        let mut limits = wgpu::Limits::default();
        for registration in self.registrations() {
            let requirements = registration.requirements();
            let missing = requirements.required_features - adapter_features;
            if !missing.is_empty() {
                return Err(crate::RendererError::GpuCanvasCapability {
                    canvas: registration.label().into(),
                    message: format!("required features {missing:?} are unavailable"),
                });
            }
            let mut failure = None;
            requirements.required_limits.check_limits_with_fail_fn(
                adapter_limits,
                true,
                |name, requested, available| failure = Some((name, requested, available)),
            );
            if let Some((name, requested, available)) = failure {
                return Err(crate::RendererError::GpuCanvasCapability {
                    canvas: registration.label().into(),
                    message: format!(
                        "required limit {name}={requested} is unsupported; adapter provides {available}"
                    ),
                });
            }
            required_features |= requirements.required_features;
            optional_features |= requirements.optional_features;
            limits = limits.or_better_values_from(&requirements.required_limits);
        }
        if profiling && adapter_features.contains(wgpu::Features::TIMESTAMP_QUERY) {
            optional_features |= wgpu::Features::TIMESTAMP_QUERY;
        }
        Ok(DeviceRequirements {
            features: required_features | (optional_features & adapter_features),
            limits: limits
                .using_resolution(adapter_limits.clone())
                .using_alignment(adapter_limits.clone()),
        })
    }

    /// Validates that an already-created shared device satisfies the registry.
    pub(crate) fn validate_device(
        &self,
        features: wgpu::Features,
        limits: &wgpu::Limits,
    ) -> Result<(), crate::RendererError> {
        for registration in self.registrations() {
            let requirements = registration.requirements();
            let missing = requirements.required_features - features;
            if !missing.is_empty() {
                return Err(crate::RendererError::IncompatibleGpuCanvasDevice {
                    canvas: registration.label().into(),
                    message: format!("shared device did not enable required features {missing:?}"),
                });
            }
            let mut failure = None;
            requirements.required_limits.check_limits_with_fail_fn(
                limits,
                true,
                |name, requested, available| failure = Some((name, requested, available)),
            );
            if let Some((name, requested, available)) = failure {
                return Err(crate::RendererError::IncompatibleGpuCanvasDevice {
                    canvas: registration.label().into(),
                    message: format!(
                        "shared device limit {name}={available} does not satisfy {requested}"
                    ),
                });
            }
        }
        Ok(())
    }
}

pub(crate) struct DeviceRequirements {
    pub features: wgpu::Features,
    pub limits: wgpu::Limits,
}
