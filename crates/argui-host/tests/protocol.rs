use argui_core::{Point, PointerEvent, PointerPhase};
use argui_host::{CallbackDelivery, CallbackId, Host, HostId, Operation};
use argui_schema::builtin;
use argui_ui::{ClickEvent, UiEventKind, UiTree};

/// Builds a first-generation presentation ID for one test slot.
fn id(slot: u32) -> HostId {
    HostId::new(slot, 1)
}

/// Creates a native node for one test slot.
fn create(slot: u32, native_type: argui_schema::NativeTypeId) -> Operation {
    Operation::Create {
        id: id(slot),
        native_type,
    }
}

#[test]
fn callback_resolution_rejects_replaced_and_removed_listeners() {
    let mut host = Host::with_builtins().unwrap();
    host.commit(&[
        create(1, builtin::FOCUS_SCOPE),
        Operation::SetListener {
            id: id(1),
            event: builtin::CLICK,
            callback: Some(CallbackId(7)),
        },
        Operation::SetRoot { id: Some(id(1)) },
    ])
    .unwrap();
    let mut tree = UiTree::new(host.root_element().unwrap());
    let click = || UiEventKind::Click(ClickEvent::accessibility());
    let previous = tree.event_deliveries(tree.node_ids()[0], click());
    let previous = previous
        .iter()
        .find(|event| event.current_handler().is_some())
        .unwrap();
    assert_eq!(
        host.callback_for(previous),
        Some(CallbackDelivery {
            node: id(1),
            callback: CallbackId(7)
        })
    );

    host.commit(&[Operation::SetListener {
        id: id(1),
        event: builtin::CLICK,
        callback: Some(CallbackId(8)),
    }])
    .unwrap();
    assert_eq!(host.callback_for(previous), None);
    tree.update(host.root_element().unwrap());
    let current = tree.event_deliveries(tree.node_ids()[0], click());
    let current = current
        .iter()
        .find(|event| event.current_handler().is_some())
        .unwrap();
    assert_eq!(
        host.callback_for(current),
        Some(CallbackDelivery {
            node: id(1),
            callback: CallbackId(8)
        })
    );

    host.commit(&[Operation::Remove { id: id(1) }]).unwrap();
    assert_eq!(host.callback_for(current), None);
}

#[test]
fn popup_dismiss_callback_accepts_outside_pointer_and_rejects_stale_handlers() {
    let mut host = Host::with_builtins().unwrap();
    host.commit(&[
        create(1, builtin::POPUP_WINDOW),
        Operation::SetListener {
            id: id(1),
            event: builtin::DISMISS,
            callback: Some(CallbackId(11)),
        },
        Operation::SetRoot { id: Some(id(1)) },
    ])
    .unwrap();
    let mut tree = UiTree::new(host.root_element().unwrap());
    let node = tree.node_ids()[0];
    for kind in [
        UiEventKind::DismissRequested,
        UiEventKind::PointerOutside(PointerEvent::mouse(
            PointerPhase::Pressed,
            Point::new(300.0, 300.0),
        )),
    ] {
        let delivery = tree
            .event_deliveries(node, kind)
            .into_iter()
            .find(|event| event.current_handler().is_some())
            .expect("popup dismissal listener");
        assert_eq!(
            host.callback_for(&delivery),
            Some(CallbackDelivery {
                node: id(1),
                callback: CallbackId(11),
            })
        );
        host.commit(&[Operation::SetListener {
            id: id(1),
            event: builtin::DISMISS,
            callback: Some(CallbackId(12)),
        }])
        .unwrap();
        assert_eq!(host.callback_for(&delivery), None);
        tree.update(host.root_element().unwrap());
        host.commit(&[Operation::SetListener {
            id: id(1),
            event: builtin::DISMISS,
            callback: Some(CallbackId(11)),
        }])
        .unwrap();
        tree.update(host.root_element().unwrap());
    }
}
