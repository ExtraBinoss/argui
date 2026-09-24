//! Wire protocol checks for the native JavaScript host.

#![cfg(not(target_arch = "wasm32"))]

#[path = "native_host/wire.rs"]
mod wire;

use argui_core::Size;
use argui_paint::{ImageAsset, ImageId, VectorAsset, VectorId};
use argui_render::{
    GpuCanvasDeviceContext, GpuCanvasError, GpuCanvasFactory, GpuCanvasRegistration,
    GpuCanvasRegistry, GpuCanvasRenderer,
};
use argui_runtime::{
    HostId, HostOperation, validate_native_host_assets, validate_native_host_canvases,
};
use argui_schema::{AssetHandle, PropertyId, SchemaValue, builtin};

struct CanvasFactory;

impl GpuCanvasFactory for CanvasFactory {
    fn create(
        &self,
        _context: &GpuCanvasDeviceContext<'_>,
    ) -> Result<Box<dyn GpuCanvasRenderer>, GpuCanvasError> {
        Err(GpuCanvasError::new("not used by validation test"))
    }
}

#[test]
fn canvas_ids_must_be_registered_before_a_native_commit() {
    let registration = GpuCanvasRegistration::new("preview", CanvasFactory);
    let registry = GpuCanvasRegistry::new([registration.clone()]).unwrap();
    let operation = |raw| HostOperation::SetProperty {
        id: HostId::new(1, 1),
        property: builtin::CANVAS_ID,
        value: Some(SchemaValue::Int(raw)),
    };
    assert!(
        validate_native_host_canvases(&[operation(registration.id().get() as i64)], &registry)
            .is_ok()
    );
    for raw in [0, -1, registration.id().get() as i64 + 1] {
        assert!(validate_native_host_canvases(&[operation(raw)], &registry).is_err());
    }
}

#[test]
fn registered_image_and_vector_handles_are_required_before_commit() {
    let images = [ImageAsset::rgba8(ImageId(17), 1, 1, vec![1, 2, 3, 255]).unwrap()];
    let vectors = [VectorAsset {
        id: VectorId(23),
        size: Size::new(1.0, 1.0),
        svg: b"<svg xmlns='http://www.w3.org/2000/svg'/>"
            .as_slice()
            .into(),
        tintable: false,
    }];
    let property = |handle| HostOperation::SetProperty {
        id: HostId::new(1, 1),
        property: PropertyId::from_raw(1),
        value: Some(SchemaValue::Asset(handle)),
    };
    let image = property(AssetHandle::Image(ImageId(17)));
    let vector = property(AssetHandle::Vector(VectorId(23)));
    assert!(
        validate_native_host_assets(&[image.clone(), vector.clone()], &images, &vectors).is_ok()
    );
    assert!(
        validate_native_host_assets(
            &[property(AssetHandle::Image(ImageId(99)))],
            &images,
            &vectors
        )
        .is_err()
    );
    assert!(
        validate_native_host_assets(
            &[property(AssetHandle::Vector(VectorId(99)))],
            &images,
            &vectors
        )
        .is_err()
    );
    assert!(validate_native_host_assets(&[image, vector], &[], &[]).is_err());
    assert!(validate_native_host_assets(&[HostOperation::SetRoot { id: None }], &[], &[]).is_ok());
}
