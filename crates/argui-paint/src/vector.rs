use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use argui_core::{Affine2D, Color, Rect, Size};

use crate::ClipChain;

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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VectorVertex {
    pub from: [f32; 2],
    pub to: [f32; 2],
    pub color_from: Color,
    pub color_to: Color,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VectorAsset {
    pub id: VectorId,
    pub size: Size,
    pub vertices: Arc<[VectorVertex]>,
    pub indices: Arc<[u32]>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VectorPrimitive {
    pub vector: VectorId,
    pub bounds: Rect,
    pub progress: f32,
    pub opacity: f32,
    pub transform: Affine2D,
    pub clips: ClipChain,
}
