use argui_schema::{NativeElementInput, SchemaValue, builtin};
use argui_ui::{ElementKind, Role, length};

#[test]
fn gpu_canvas_maps_native_controls_and_accessible_description() {
    let registry = builtin::registry().unwrap();
    let element = registry
        .construct(
            builtin::GPU_CANVAS,
            &NativeElementInput::new()
                .property(builtin::CANVAS_ID, SchemaValue::Int(7))
                .property(builtin::CANVAS_REVISION, SchemaValue::Int(9))
                .property(builtin::RESOLUTION_SCALE, SchemaValue::Float(2.0))
                .property(builtin::SAMPLING, SchemaValue::String("nearest".into()))
                .property(builtin::WIDTH, SchemaValue::Dimension(length(240.0)))
                .property(builtin::ALT, SchemaValue::String("Video preview".into())),
        )
        .unwrap();
    let ElementKind::GpuCanvas(spec) = element.kind else {
        panic!("expected GPU canvas")
    };
    assert_eq!(spec.canvas().get(), 7);
    assert_eq!(spec.revision(), 9);
    assert_eq!(spec.scale(), 2.0);
    assert_eq!(element.style.size.width, length(240.0));
    assert_eq!(element.semantics.as_ref().unwrap().role, Role::Image);
    assert_eq!(
        element.semantics.as_ref().unwrap().label.as_deref(),
        Some("Video preview")
    );
}

#[test]
fn gpu_canvas_rejects_invalid_ids_revisions_and_scales() {
    let registry = builtin::registry().unwrap();
    let base = || NativeElementInput::new().property(builtin::CANVAS_ID, SchemaValue::Int(1));
    for id in [0, -1] {
        assert!(
            registry
                .construct(
                    builtin::GPU_CANVAS,
                    &NativeElementInput::new().property(builtin::CANVAS_ID, SchemaValue::Int(id))
                )
                .is_err()
        );
    }
    assert!(
        registry
            .construct(
                builtin::GPU_CANVAS,
                &base().property(builtin::CANVAS_REVISION, SchemaValue::Int(-1))
            )
            .is_err()
    );
    for scale in [0.0, 0.1, 4.1, f32::NAN] {
        assert!(
            registry
                .construct(
                    builtin::GPU_CANVAS,
                    &base().property(builtin::RESOLUTION_SCALE, SchemaValue::Float(scale))
                )
                .is_err()
        );
    }
    assert!(
        registry
            .construct(
                builtin::GPU_CANVAS,
                &base().property(builtin::SAMPLING, SchemaValue::String("cubic".into()))
            )
            .is_err()
    );
}
