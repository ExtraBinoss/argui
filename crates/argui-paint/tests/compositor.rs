use argui_core::{Affine2D, Point, Rect, Size};
use argui_paint::{CompositorId, CompositorLayer, CompositorPatch, DisplayCommand, DisplayList};

fn bounds(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect::new(Point::new(x, y), Size::new(width, height))
}

#[test]
fn compositor_patches_change_only_presentation_properties() {
    let id = CompositorId::new(42);
    let layer = CompositorLayer::new(
        id,
        bounds(10.0, 20.0, 80.0, 40.0),
        Affine2D::IDENTITY,
        Affine2D::IDENTITY,
        f32::NAN,
    );
    let mut list = DisplayList::new();
    list.begin_compositor(layer.clone());
    list.end_compositor();

    assert_eq!(layer.opacity, 1.0);
    assert_eq!(
        list.apply_compositor_patches(&[CompositorPatch::new(
            id,
            Affine2D::translation(12.0, -4.0),
            1.5,
        )]),
        1
    );
    let DisplayCommand::BeginCompositor(patched) = &list.commands()[0] else {
        panic!("missing compositor layer");
    };
    assert_eq!(patched.bounds, layer.bounds);
    assert_eq!(patched.base_transform, layer.base_transform);
    assert_eq!(patched.transform, Affine2D::translation(12.0, -4.0));
    assert_eq!(patched.opacity, 1.0);
    assert!(patched.style().retained);
}

#[test]
fn compositor_bounds_include_retained_descendant_layers() {
    let outer = CompositorId::new(1);
    let inner = CompositorId::new(2);
    let mut list = DisplayList::new();
    list.begin_compositor(CompositorLayer::new(
        outer,
        bounds(10.0, 10.0, 20.0, 20.0),
        Affine2D::IDENTITY,
        Affine2D::IDENTITY,
        1.0,
    ));
    list.begin_compositor(CompositorLayer::new(
        inner,
        bounds(50.0, 60.0, 30.0, 40.0),
        Affine2D::IDENTITY,
        Affine2D::IDENTITY,
        1.0,
    ));
    list.end_compositor();
    list.end_compositor();

    list.resolve_compositor_bounds();

    let layers = list.compositor_layers().collect::<Vec<_>>();
    assert_eq!(layers[0].bounds, bounds(10.0, 10.0, 70.0, 90.0));
    assert_eq!(layers[1].bounds, bounds(50.0, 60.0, 30.0, 40.0));
}
