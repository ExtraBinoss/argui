#[path = "../../src/app/frame_route.rs"]
mod implementation;

use argui_animation::Time;
use argui_host::{Host, HostId, Operation};
use argui_schema::{SchemaValue, builtin};
use argui_ui::{ElementKind, TreeUpdate, UiTree};

fn id(slot: u32) -> HostId {
    HostId::new(slot, 1)
}

#[test]
fn host_text_change_during_native_motion_uses_fresh_layout_scene() {
    let mut host = Host::with_builtins().unwrap();
    host.commit(&[
        Operation::Create {
            id: id(1),
            native_type: builtin::COLUMN,
        },
        Operation::Create {
            id: id(2),
            native_type: builtin::TEXT,
        },
        Operation::Create {
            id: id(3),
            native_type: builtin::RECTANGLE,
        },
        Operation::SetProperty {
            id: id(2),
            property: builtin::TEXT_VALUE,
            value: Some(SchemaValue::String("Pause loops".into())),
        },
        Operation::SetProperty {
            id: id(3),
            property: builtin::ROTATION_LOOP_MS,
            value: Some(SchemaValue::Float(800.0)),
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
    let mut tree = UiTree::new(host.root_element().unwrap());
    tree.advance_animations(Time::from_nanos(1));
    assert_eq!(
        tree.advance_animations(Time::from_nanos(150_000_001)),
        TreeUpdate::Composite
    );

    host.commit(&[
        Operation::SetProperty {
            id: id(2),
            property: builtin::TEXT_VALUE,
            value: Some(SchemaValue::String("Resume loops".into())),
        },
        Operation::SetProperty {
            id: id(3),
            property: builtin::LOOP_PLAYING,
            value: Some(SchemaValue::Bool(false)),
        },
    ])
    .unwrap();
    let update = tree.update(host.root_element().unwrap());
    assert_eq!(update, TreeUpdate::Layout);
    assert!(!implementation::retain_compositor_frame(
        true, update, false, false
    ));
    let ElementKind::Text { content, .. } = &tree.root().children[0].kind else {
        panic!("host text child must remain text");
    };
    assert_eq!(content.as_str(), "Resume loops");
}

#[test]
fn requested_layout_or_asset_refresh_cannot_reuse_compositor_scene() {
    assert!(!implementation::retain_compositor_frame(
        true,
        TreeUpdate::Composite,
        true,
        false
    ));
    assert!(!implementation::retain_compositor_frame(
        true,
        TreeUpdate::Composite,
        false,
        true
    ));
    assert!(implementation::retain_compositor_frame(
        true,
        TreeUpdate::Composite,
        false,
        false
    ));
}

/// Layout and asset changes require a fresh scene even when the frame is composite-only.
#[test]
fn compositor_frame_retention_requires_all_reuse_conditions() {
    for update in [
        TreeUpdate::None,
        TreeUpdate::Semantics,
        TreeUpdate::Composite,
        TreeUpdate::Paint,
        TreeUpdate::Scroll,
    ] {
        assert!(implementation::retain_compositor_frame(
            true, update, false, false
        ));
        assert!(!implementation::retain_compositor_frame(
            false, update, false, false
        ));
        assert!(!implementation::retain_compositor_frame(
            true, update, true, false
        ));
        assert!(!implementation::retain_compositor_frame(
            true, update, false, true
        ));
    }
    for pending_layout in [false, true] {
        for assets_changed in [false, true] {
            assert!(!implementation::retain_compositor_frame(
                true,
                TreeUpdate::Layout,
                pending_layout,
                assets_changed,
            ));
        }
    }
}
