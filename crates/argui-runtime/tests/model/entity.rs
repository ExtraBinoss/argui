use argui_runtime::{Context, Entity};

#[test]
fn data_entities_need_no_render_and_identities_are_never_reused() {
    struct Mailbox {
        unread: usize,
    }
    let model = Entity::new(Mailbox { unread: 4 });
    let clone = model.clone();
    let weak = model.downgrade();
    let id = model.id();
    clone.update(|model, cx| {
        model.unread -= 1;
        cx.notify();
    });
    assert_eq!(model.read(|model| model.unread), 3);
    assert_eq!(clone.id(), id);
    drop(model);
    assert_eq!(weak.upgrade().unwrap().id(), id);
    drop(clone);
    assert!(weak.upgrade().is_none());
    assert_ne!(Entity::new(Mailbox { unread: 0 }).id(), id);
    assert_ne!(id.get(), 0);
}

#[test]
fn detached_context_can_construct_a_data_entity() {
    let mut cx = Context::<()>::default();
    let model = cx.new_entity(String::from("draft"));
    assert_eq!(model.read(Clone::clone), "draft");
}

#[test]
#[cfg(all(feature = "tasks", not(target_arch = "wasm32")))]
fn data_models_receive_owned_async_results_without_implementing_render() {
    use argui_runtime::tasks::TaskRuntime;
    let (sender, wake) = std::sync::mpsc::channel();
    let runtime = TaskRuntime::new(move || {
        let _ = sender.send(());
    });
    let model = Entity::new(0_usize);
    model.set_task_runtime(runtime.clone());
    let mut handle = None;
    model.update(|_, cx| {
        handle = Some(
            cx.spawn(async { 42 }, |value, result, cx| {
                *value = result.unwrap();
                cx.notify();
            })
            .unwrap(),
        );
    });
    while runtime.pending() > 0 {
        wake.recv_timeout(std::time::Duration::from_secs(5))
            .unwrap();
        runtime.drain();
    }
    assert_eq!(model.read(|value| *value), 42);
    assert!(handle.unwrap().is_finished());
}
