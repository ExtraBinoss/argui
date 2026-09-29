use super::*;
use argui_core::{Affine2D, Color, Point, Rect, Size};
use argui_paint::{ClipChain, ImageFit, VectorPrimitive};

/// Creates the production popup resource wrapper around a windowless GPU surface.
fn renderer() -> PopupRenderer {
    PopupRenderer::new(
        pollster::block_on(SurfaceRenderer::new_offscreen(32, 32, Default::default()))
            .expect("offscreen GPU required for popup registration tests"),
    )
}

/// Creates a valid version of one vector resource.
fn vector(id: u64, color: &str) -> VectorAsset {
    VectorAsset {
        id: VectorId(id), size: Size::new(16.0, 16.0), tintable: false,
        svg: format!(r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"><rect width="16" height="16" fill="{color}"/></svg>"#).into_bytes().into(),
    }
}

/// Paints only the supplied vector ID in a popup scene.
fn scene(id: VectorId) -> DisplayList {
    let mut scene = DisplayList::new();
    scene.push_vector(VectorPrimitive {
        vector: id,
        bounds: Rect::new(Point::default(), Size::new(16.0, 16.0)),
        fit: ImageFit::Contain,
        color: Color::WHITE,
        opacity: 1.0,
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
    });
    scene
}

#[test]
fn popup_scene_ignores_unreferenced_invalid_assets() {
    let mut renderer = renderer();
    let wanted = vector(1, "red");
    let mut unused = vector(2, "green");
    unused.svg = b"invalid svg".as_slice().into();
    renderer
        .register_scene(&scene(wanted.id), &[], &[wanted.clone(), unused])
        .unwrap();
    assert_eq!(renderer.vectors.len(), 1);
    assert_eq!(renderer.vectors.get(&wanted.id), Some(&wanted));
}

#[test]
fn popup_scene_refreshes_changed_assets_and_preserves_version_on_failure() {
    let mut renderer = renderer();
    let first = vector(1, "red");
    let next = vector(1, "blue");
    let scene = scene(first.id);
    renderer.register_scene(&scene, &[], &[first]).unwrap();
    renderer
        .register_scene(&scene, &[], std::slice::from_ref(&next))
        .unwrap();
    assert_eq!(renderer.vectors.get(&next.id), Some(&next));
    let mut broken = next.clone();
    broken.svg = b"invalid svg".as_slice().into();
    assert!(renderer.register_scene(&scene, &[], &[broken]).is_err());
    assert_eq!(renderer.vectors.get(&next.id), Some(&next));
    renderer
        .register_scene(&scene, &[], std::slice::from_ref(&next))
        .unwrap();
    assert_eq!(renderer.vectors.len(), 1);
}
