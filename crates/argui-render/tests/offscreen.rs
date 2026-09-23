#[path = "../src/offscreen.rs"]
mod offscreen;

use offscreen::TexturePool;
use wgpu::{Instance, TextureFormat};

#[test]
fn phone_sized_effect_root_fits_pool_and_survives_next_frame() {
    let instance = Instance::default();
    let Ok(adapter) = pollster::block_on(instance.request_adapter(&Default::default())) else {
        return;
    };
    let (device, _) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();
    let mut pool = TexturePool::new(TextureFormat::Rgba8Unorm, 32 * 1024 * 1024);

    assert!(!pool.begin_frame());
    let root = pool.acquire(&device, 1080, 2400);
    let layer = pool.acquire(&device, 256, 256);
    assert_eq!(pool.extent(root), [1088, 2432]);
    assert_eq!(pool.bytes(root), 1088 * 2432 * 4);
    assert!(pool.stats().allocated_bytes < 32 * 1024 * 1024);
    assert_eq!(pool.stats_excluding(root).textures, 1);
    assert_eq!(pool.texture(root).width(), 1088);
    let _ = pool.view(root);

    assert!(!pool.begin_frame());
    assert!(pool.retain(root));
    assert!(pool.retain(layer));
    pool.clear();
    assert_eq!(pool.stats().textures, 0);
}

#[test]
fn idle_spares_within_budget_do_not_discard_retained_root() {
    let instance = Instance::default();
    let Ok(adapter) = pollster::block_on(instance.request_adapter(&Default::default())) else {
        return;
    };
    let (device, _) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();
    let mut pool = TexturePool::new(TextureFormat::Rgba8Unorm, 1024 * 1024);

    assert!(!pool.begin_frame());
    let root = pool.acquire(&device, 64, 64);
    let _spare = pool.acquire(&device, 64, 128);
    for _ in 0..61 {
        assert!(!pool.begin_frame());
        assert!(pool.retain(root));
    }
    assert_eq!(pool.stats().textures, 2);

    let mut constrained = TexturePool::new(TextureFormat::Rgba8Unorm, 64 * 64 * 4);
    assert!(!constrained.begin_frame());
    constrained.acquire(&device, 64, 64);
    constrained.acquire(&device, 64, 128);
    assert!(constrained.begin_frame());
    assert_eq!(constrained.stats().textures, 1);
}
