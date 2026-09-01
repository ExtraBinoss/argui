use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use argui_core::{Affine2D, Color, Rect, Size};

use crate::{ClipChain, ImageFit};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct VectorId(pub u64);

static NEXT_VECTOR_ID: AtomicU64 = AtomicU64::new(1);

impl VectorId {
    #[must_use]
    pub fn fresh() -> Self {
        let id = NEXT_VECTOR_ID.fetch_add(1, Ordering::Relaxed);
        assert_ne!(id, u64::MAX, "vector handle space exhausted");
        Self(id)
    }
}

/// A validated, resolution-independent SVG resource.
#[derive(Clone, Debug, PartialEq)]
pub struct VectorAsset {
    pub id: VectorId,
    pub size: Size,
    pub svg: Arc<[u8]>,
    pub tintable: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VectorPrimitive {
    pub vector: VectorId,
    pub bounds: Rect,
    pub fit: ImageFit,
    pub color: Color,
    pub opacity: f32,
    pub transform: Affine2D,
    pub clips: ClipChain,
}
