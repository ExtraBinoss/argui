use argui_automation::{Action, Driver, Viewport};
use argui_host::{CallbackId, HostId, Operation};
use argui_render::{AdapterProfile, GpuFrameProfile, GpuPassProfile, RenderProfile};
use argui_schema::{SchemaValue, builtin};
use argui_ui::length;
use std::time::Duration;

/// Returns a first-generation native node ID for a test slot.
fn id(slot: u32) -> HostId {
    HostId::new(slot, 1)
}

#[test]
fn viewport_rejects_nonfinite_and_oversized_physical_dimensions() {
    assert!(
        Viewport {
            width: 0,
            ..Viewport::default()
        }
        .physical_size()
        .is_err()
    );
    for viewport in [
        Viewport {
            height: 0,
            ..Viewport::default()
        },
        Viewport {
            width: 4097,
            ..Viewport::default()
        },
        Viewport {
            height: 4097,
            ..Viewport::default()
        },
        Viewport {
            scale: 0.1,
            ..Viewport::default()
        },
        Viewport {
            width: 100,
            height: 4096,
            scale: 2.0,
        },
    ] {
        assert!(viewport.physical_size().is_err());
    }
    assert!(
        Viewport {
            scale: f32::NAN,
            ..Viewport::default()
        }
        .physical_size()
        .is_err()
    );
    assert!(
        Viewport {
            width: 4096,
            scale: 2.0,
            ..Viewport::default()
        }
        .physical_size()
        .is_err()
    );
    assert_eq!(
        Viewport {
            width: 200,
            height: 100,
            scale: 1.5
        }
        .physical_size()
        .unwrap(),
        (300, 150)
    );
}

#[test]
fn host_commit_inspection_and_pointer_delivery_share_the_real_tree() {
    let mut driver = Driver::new(Viewport::default()).unwrap();
    assert!(driver.snapshot().is_err());
    driver.set_viewport(Viewport::default()).unwrap();
    driver.record_render(Duration::from_millis(1));
    driver.commit(&[]).unwrap();
    driver
        .commit(&[
            Operation::Create {
                id: id(1),
                native_type: builtin::COLUMN,
            },
            Operation::Create {
                id: id(2),
                native_type: builtin::TEXT,
            },
            Operation::SetProperty {
                id: id(2),
                property: builtin::TEXT_VALUE,
                value: Some(SchemaValue::String("Count: 0".into())),
            },
            Operation::Create {
                id: id(3),
                native_type: builtin::TOUCH_AREA,
            },
            Operation::SetProperty {
                id: id(3),
                property: builtin::KEY,
                value: Some(SchemaValue::String("increment".into())),
            },
            Operation::SetProperty {
                id: id(3),
                property: builtin::WIDTH,
                value: Some(SchemaValue::Dimension(length(120.0))),
            },
            Operation::SetProperty {
                id: id(3),
                property: builtin::HEIGHT,
                value: Some(SchemaValue::Dimension(length(48.0))),
            },
            Operation::SetListener {
                id: id(3),
                event: builtin::CLICK,
                callback: Some(CallbackId(8)),
            },
            Operation::SetListener {
                id: id(3),
                event: builtin::CONTEXT_MENU,
                callback: Some(CallbackId(9)),
            },
            Operation::SetListener {
                id: id(3),
                event: builtin::DRAG_X,
                callback: Some(CallbackId(10)),
            },
            Operation::Insert {
                parent: id(1),
                child: id(2),
                before: None,
            },
            Operation::Insert {
                parent: id(1),
                child: id(3),
                before: None,
            },
            Operation::SetRoot { id: Some(id(1)) },
        ])
        .unwrap();
    driver.expect_text("Count: 0").unwrap();
    assert!(driver.expect_text("Count: 1").is_err());
    assert!(driver.point("increment").is_ok());
    assert_eq!(driver.point("10,20").unwrap().x, 10.0);
    assert!(driver.point("oops,20").is_err());
    assert!(driver.point("10,oops").is_err());
    assert!(driver.point("nan,20").is_err());
    assert!(driver.point("10,nan").is_err());
    assert!(driver.point("missing").is_err());
    assert!(driver.scene().is_ok());
    driver
        .set_viewport(Viewport {
            width: 640,
            height: 480,
            scale: 1.0,
        })
        .unwrap();
    driver
        .act(Action::Click {
            target: "increment".into(),
            right: false,
        })
        .unwrap();
    assert!(
        driver
            .take_deliveries()
            .iter()
            .any(|delivery| delivery.callback.callback == CallbackId(8))
    );
    driver
        .act(Action::Click {
            target: "increment".into(),
            right: true,
        })
        .unwrap();
    let context = driver.take_deliveries();
    assert!(
        context
            .iter()
            .any(|delivery| delivery.callback.callback == CallbackId(9))
    );
    assert!(
        !context
            .iter()
            .any(|delivery| delivery.callback.callback == CallbackId(8))
    );
    driver
        .act(Action::Drag {
            from: "increment".into(),
            to: "200,100".into(),
        })
        .unwrap();
    driver.advance().unwrap();
    assert!(
        driver
            .take_deliveries()
            .iter()
            .any(|delivery| delivery.callback.callback == CallbackId(10))
    );
    driver
        .act(Action::Scroll {
            target: "increment".into(),
            x: 0.0,
            y: 10.0,
            lines: false,
        })
        .unwrap();
    assert!(
        driver
            .act(Action::Scroll {
                target: "increment".into(),
                x: f32::NAN,
                y: 0.0,
                lines: true
            })
            .is_err()
    );
    assert!(
        driver
            .act(Action::Scroll {
                target: "increment".into(),
                x: 0.0,
                y: f32::NAN,
                lines: true
            })
            .is_err()
    );
    driver
        .act(Action::Scroll {
            target: "increment".into(),
            x: 0.0,
            y: 1.0,
            lines: true,
        })
        .unwrap();
    assert!(
        driver
            .act(Action::Fill {
                target: "increment".into(),
                value: "text".into()
            })
            .is_err()
    );
    assert!(
        driver
            .act(Action::Key {
                value: "NotAKey".into()
            })
            .is_err()
    );
    driver
        .act(Action::Key {
            value: "Enter".into(),
        })
        .unwrap();
    driver.act(Action::Key { value: "a".into() }).unwrap();
    assert!(
        driver
            .act(Action::Click {
                target: "799,599".into(),
                right: true
            })
            .is_err()
    );
    driver
        .commit(&[Operation::SetProperty {
            id: id(2),
            property: builtin::TEXT_VALUE,
            value: Some(SchemaValue::String("Count: 1".into())),
        }])
        .unwrap();
    driver.expect_text("Count: 1").unwrap();
    driver.record_render(Duration::from_millis(2));
    assert!(driver.frames().last().unwrap().render_cpu_ms.unwrap() >= 2.0);
    driver.record_render_profile(&RenderProfile {
        draw_batches: 4,
        adapter: AdapterProfile {
            name: "test adapter".into(),
            timestamp_queries: true,
            ..AdapterProfile::default()
        },
        gpu: Some(GpuFrameProfile {
            frame: 9,
            total: Duration::from_millis(3),
            passes: vec![GpuPassProfile {
                label: "surface.main".into(),
                duration: Duration::from_millis(2),
                ..GpuPassProfile::default()
            }],
        }),
        ..RenderProfile::default()
    });
    let record = driver.frame_records().last().unwrap();
    assert_eq!(record.passes, 4);
    assert_eq!(record.adapter.name, "test adapter");
    assert_eq!(record.gpu.as_ref().unwrap().passes[0].label, "surface.main");
    assert!(
        driver
            .metrics()
            .events()
            .iter()
            .any(|event| event.name == "render.gpu_time")
    );
    assert!(driver.frames().len() >= 2);
    assert_eq!(driver.frame_records().len(), driver.frames().len());
    driver.commit(&[Operation::SetRoot { id: None }]).unwrap();
    driver.advance().unwrap();
    assert!(driver.snapshot().is_err());
    assert!(driver.scene().is_err());
}

#[test]
fn fill_uses_the_native_editor_edit_path() {
    let mut driver = Driver::new(Viewport::default()).unwrap();
    driver
        .commit(&[
            Operation::Create {
                id: id(1),
                native_type: builtin::TEXT_INPUT,
            },
            Operation::SetProperty {
                id: id(1),
                property: builtin::KEY,
                value: Some(SchemaValue::String("search".into())),
            },
            Operation::SetProperty {
                id: id(1),
                property: builtin::VALUE,
                value: Some(SchemaValue::String("old".into())),
            },
            Operation::SetListener {
                id: id(1),
                event: builtin::INPUT_CHANGED,
                callback: Some(CallbackId(12)),
            },
            Operation::SetRoot { id: Some(id(1)) },
        ])
        .unwrap();
    driver
        .act(Action::Fill {
            target: "search".into(),
            value: "Argui".into(),
        })
        .unwrap();
    assert!(
        driver
            .take_deliveries()
            .iter()
            .any(|delivery| delivery.callback.callback == CallbackId(12))
    );
    assert!(
        driver
            .act(Action::Fill {
                target: "missing".into(),
                value: "x".into()
            })
            .is_err()
    );
}

#[test]
fn scrolling_a_native_flickable_updates_its_position() {
    let mut driver = Driver::new(Viewport::default()).unwrap();
    driver
        .commit(&[
            Operation::Create {
                id: id(1),
                native_type: builtin::FLICKABLE,
            },
            Operation::SetProperty {
                id: id(1),
                property: builtin::KEY,
                value: Some(SchemaValue::String("list".into())),
            },
            Operation::SetProperty {
                id: id(1),
                property: builtin::WIDTH,
                value: Some(SchemaValue::Dimension(length(150.0))),
            },
            Operation::SetProperty {
                id: id(1),
                property: builtin::HEIGHT,
                value: Some(SchemaValue::Dimension(length(80.0))),
            },
            Operation::SetListener {
                id: id(1),
                event: builtin::SCROLL,
                callback: Some(CallbackId(14)),
            },
            Operation::Create {
                id: id(2),
                native_type: builtin::RECTANGLE,
            },
            Operation::SetProperty {
                id: id(2),
                property: builtin::HEIGHT,
                value: Some(SchemaValue::Dimension(length(400.0))),
            },
            Operation::SetProperty {
                id: id(2),
                property: builtin::MIN_HEIGHT,
                value: Some(SchemaValue::Float(400.0)),
            },
            Operation::Insert {
                parent: id(1),
                child: id(2),
                before: None,
            },
            Operation::SetRoot { id: Some(id(1)) },
        ])
        .unwrap();
    assert!(driver.scene().unwrap().0.scroll_regions[0].max_offset.y > 0.0);
    driver
        .act(Action::Scroll {
            target: "list".into(),
            x: 0.0,
            y: 80.0,
            lines: false,
        })
        .unwrap();
    assert!(
        driver
            .take_deliveries()
            .iter()
            .any(|delivery| delivery.callback.callback == CallbackId(14))
    );
    assert!(driver.frames().len() > 1);
}
