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

/// Canvas validation ignores unrelated updates but rejects a malformed canvas value.
#[test]
fn canvas_validation_only_checks_present_canvas_properties() {
    let registry = GpuCanvasRegistry::new([]).unwrap();
    let id = HostId::new(1, 1);
    let unrelated = [
        HostOperation::SetRoot { id: None },
        HostOperation::SetProperty {
            id,
            property: PropertyId::from_raw(1),
            value: Some(SchemaValue::Int(-1)),
        },
        HostOperation::SetProperty {
            id,
            property: builtin::CANVAS_ID,
            value: None,
        },
    ];
    assert!(validate_native_host_canvases(&unrelated, &registry).is_ok());

    let malformed = HostOperation::SetProperty {
        id,
        property: builtin::CANVAS_ID,
        value: Some(SchemaValue::String("canvas".into())),
    };
    assert_eq!(
        validate_native_host_canvases(&[malformed], &registry),
        Err("invalid native canvas identity".into())
    );
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

mod pointer {
    use argui_core::{Point, PointerEvent, PointerPhase, Rect, Size};
    use argui_layout::{LayoutNode, LayoutOutput};
    use argui_runtime::NativePointerPosition;
    use argui_ui::{Element, UiEvent, UiEventKind, UiTree};

    /// Pointer coordinates delivered to a keyed callback are relative to its bounds.
    #[test]
    fn pointer_coordinates_are_relative_to_the_callback_node_bounds() {
        let tree = UiTree::new(Element::container(
            [Element::text("slider").keyed("slider")],
        ));
        let node = tree.node_ids()[1];
        let mut layout = LayoutOutput::default();
        layout.nodes.push(LayoutNode {
            index: 1,
            node,
            bounds: Rect::new(Point::new(45.0, 30.0), Size::new(180.0, 24.0)),
            layout_bounds: Rect::default(),
            clip: None,
            text_index: None,
        });
        let event = UiEvent::new(
            node,
            Some("slider".into()),
            UiEventKind::Pointer(PointerEvent::mouse(
                PointerPhase::Moved,
                Point::new(90.0, 42.0),
            )),
        );
        let position = NativePointerPosition::from_event(&event, &layout).unwrap();
        assert_eq!((position.local_x, position.local_y), (45.0, 12.0));
        assert_eq!((position.width, position.height), (180.0, 24.0));
        assert_eq!((position.x, position.y), (90.0, 42.0));
    }
}
