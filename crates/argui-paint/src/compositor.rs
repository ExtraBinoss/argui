use argui_core::{Affine2D, Rect};

use crate::{LayerStyle, ProfileDomain, RenderObjectId};

/// Stable identity for one retained compositor layer.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CompositorId {
    owner: u64,
    subpart: u32,
}

impl CompositorId {
    /// Creates an identity from a retained engine node value.
    ///
    /// * `value` — stable value that identifies the layer between frames.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self {
            owner: value,
            subpart: 0,
        }
    }

    /// Creates an identity for a retained subpart owned by an engine node.
    ///
    /// * `owner` — stable owner value shared with the surrounding element.
    /// * `subpart` — non-zero namespace local to that owner.
    #[must_use]
    pub const fn subpart(owner: u64, subpart: u32) -> Self {
        Self { owner, subpart }
    }

    /// Returns the opaque numeric layer identity.
    #[must_use]
    pub const fn get(self) -> u64 {
        if self.subpart == 0 {
            self.owner
        } else {
            self.owner
                .wrapping_mul(0x9e37_79b9_7f4a_7c15)
                .wrapping_add(self.subpart as u64)
        }
    }
}

/// Retained group whose content can be transformed and faded without repainting it.
#[derive(Clone, Debug, PartialEq)]
pub struct CompositorLayer {
    /// Stable identity used by scene updates and the renderer cache.
    pub id: CompositorId,
    /// Bounds of the already-painted content in surface coordinates.
    pub bounds: Rect,
    /// Parent transform captured when the content was painted.
    pub base_parent: Affine2D,
    /// Full transform captured when the content was painted.
    pub base_transform: Affine2D,
    /// Delta applied to the retained content at composition time.
    pub transform: Affine2D,
    /// Group opacity applied at composition time.
    pub opacity: f32,
}

impl CompositorLayer {
    /// Creates an identity compositor layer around already-painted content.
    ///
    /// * `id` — stable retained identity.
    /// * `bounds` — initial painted bounds in surface coordinates.
    /// * `base_parent` — parent surface transform used by the paint snapshot.
    /// * `base_transform` — full surface transform used by the paint snapshot.
    /// * `opacity` — initial group opacity.
    #[must_use]
    pub fn new(
        id: CompositorId,
        bounds: Rect,
        base_parent: Affine2D,
        base_transform: Affine2D,
        opacity: f32,
    ) -> Self {
        Self {
            id,
            bounds,
            base_parent,
            base_transform,
            transform: Affine2D::IDENTITY,
            opacity: normalized_opacity(opacity),
        }
    }

    /// Replaces the composition-only properties of this retained layer.
    ///
    /// * `transform` — delta from the painted snapshot to the current presentation.
    /// * `opacity` — current group opacity.
    ///
    /// Returns whether either property changed.
    pub fn update(&mut self, transform: Affine2D, opacity: f32) -> bool {
        let opacity = normalized_opacity(opacity);
        let changed = self.transform != transform || self.opacity != opacity;
        self.transform = transform;
        self.opacity = opacity;
        changed
    }

    /// Converts this compositor entry into the renderer's generic layer representation.
    #[must_use]
    pub fn style(&self) -> LayerStyle {
        LayerStyle::new(self.bounds)
            .retained(true)
            .opacity(self.opacity)
            .transform(self.transform)
            .profile(RenderObjectId::new(ProfileDomain::Engine, self.id.get()))
    }
}

/// One lightweight update to an existing retained compositor layer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CompositorPatch {
    /// Layer receiving the update.
    pub id: CompositorId,
    /// Delta from the layer's painted snapshot.
    pub transform: Affine2D,
    /// Current group opacity.
    pub opacity: f32,
}

impl CompositorPatch {
    /// Creates a patch for an existing compositor layer.
    ///
    /// * `id` — target retained layer.
    /// * `transform` — composition-time transform delta.
    /// * `opacity` — composition-time group opacity.
    #[must_use]
    pub const fn new(id: CompositorId, transform: Affine2D, opacity: f32) -> Self {
        Self {
            id,
            transform,
            opacity,
        }
    }
}

/// Clamps finite opacity and replaces invalid floating-point values with identity.
fn normalized_opacity(opacity: f32) -> f32 {
    if opacity.is_finite() {
        opacity.clamp(0.0, 1.0)
    } else {
        1.0
    }
}
