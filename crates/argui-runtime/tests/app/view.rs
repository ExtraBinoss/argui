#![cfg(test)]
use crate::{HostId, HostOperation, NativeHost, app::Application};
use argui_schema::{SchemaValue, builtin};
use argui_ui::ElementKind;

#[test]
fn reopening_a_native_scene_retains_the_live_root_without_a_rust_model() {
    let mut app = Application::new(
        Default::default(),
        Default::default(),
        Default::default(),
        None,
        None,
        None,
        |_| {},
    );
    assert!(app.inspected_view().is_none());
    let root = HostId::new(1, 400_000);
    let text = HostId::new(2, 400_000);
    let mut host = NativeHost::with_builtins().unwrap();
    host.commit(&[
        HostOperation::Create {
            id: root,
            native_type: builtin::COLUMN,
        },
        HostOperation::Create {
            id: text,
            native_type: builtin::TEXT,
        },
        HostOperation::SetProperty {
            id: text,
            property: builtin::TEXT_VALUE,
            value: Some(SchemaValue::String("Settings".into())),
        },
        HostOperation::Insert {
            parent: root,
            child: text,
            before: None,
        },
        HostOperation::SetRoot { id: Some(root) },
    ])
    .unwrap();
    app.native_host = Some(host);
    for label in ["Settings", "Capture", "Shortcuts"] {
        app.set_window_visible(false);
        app.native_host
            .as_mut()
            .unwrap()
            .commit(&[HostOperation::SetProperty {
                id: text,
                property: builtin::TEXT_VALUE,
                value: Some(SchemaValue::String(label.into())),
            }])
            .unwrap();
        app.set_window_visible(true);
        assert!(app.pending_ui_frame.needs_frame());
        let root = app
            .inspected_view()
            .expect("native root survives reopening");
        let ElementKind::Text { content, .. } = &root.children[0].kind else {
            panic!("native child must remain a text node");
        };
        assert_eq!(content.as_str(), label);
    }
}
