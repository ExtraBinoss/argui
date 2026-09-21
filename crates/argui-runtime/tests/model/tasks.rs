use argui_runtime::{
    Context, Entity, ModelContext, Render,
    tasks::{TaskError, TaskRuntime, TaskSlot},
};
use argui_ui::Element;
use std::{cell::Cell, rc::Rc, sync::mpsc, time::Duration};

#[derive(Default)]
struct Model {
    value: usize,
    slot: TaskSlot,
}
impl Render for Model {
    fn render(&mut self, _: &mut Context<Self>) -> Element {
        Element::text(self.value.to_string())
    }
}
fn runtime() -> (TaskRuntime, mpsc::Receiver<()>) {
    let (sender, receiver) = mpsc::channel();
    (
        TaskRuntime::new(move || {
            let _ = sender.send(());
        }),
        receiver,
    )
}

#[test]
fn closing_a_mount_during_update_rejects_work_in_the_existing_context() {
    let (runtime, _) = runtime();
    let model = Entity::new(Model::default());
    model.set_task_runtime(runtime.clone());
    let mount = model.mount().unwrap();
    mount
        .update(|_, cx| {
            mount.close();
            assert!(matches!(
                cx.spawn(async {}, |_, _, _| {}),
                Err(TaskError::ScopeClosed)
            ));
        })
        .unwrap();
    assert_eq!(runtime.pending(), 0);
    let scope = argui_runtime::ResourceScope::default();
    scope.close();
    let task = model
        .update(|_, cx| cx.spawn(std::future::pending::<()>(), |_, _, _| {}))
        .unwrap();
    let cancelled = task.cancellation_token();
    assert!(task.in_scope(&scope).is_err());
    assert!(cancelled.is_cancelled());
    runtime.shutdown();
}

#[test]
fn view_tasks_obey_explicit_scope_lifetime_without_closing_the_mount() {
    let (runtime, wake) = runtime();
    let model = Entity::new(Model::default());
    model.set_task_runtime(runtime.clone());
    let mount = model.mount().unwrap();
    let scope = argui_runtime::ResourceScope::default();
    let completed = mount
        .update(|_, cx| {
            cx.spawn_in(&scope, async { 19 }, |model, result, cx| {
                model.value = result.unwrap();
                cx.notify();
            })
        })
        .unwrap()
        .unwrap();
    while !completed.is_finished() {
        wake.recv_timeout(Duration::from_secs(5)).unwrap();
        runtime.drain();
    }
    assert_eq!(model.read(|model| model.value), 19);
    let cancelled = mount
        .update(|_, cx| {
            cx.spawn_in(&scope, std::future::pending::<()>(), |_, _, _| {
                panic!("closed explicit scope delivered a view callback");
            })
        })
        .unwrap()
        .unwrap();
    scope.close();
    runtime.drain();
    assert!(cancelled.cancellation_token().is_cancelled());
    assert!(mount.is_visible());
    assert_eq!(scope.resource_count(), 0);
    assert!(matches!(
        mount
            .update(|_, cx| { cx.spawn_in(&scope, async {}, |_, _, _| {}) })
            .unwrap(),
        Err(TaskError::ScopeClosed)
    ));
    while runtime.pending() > 0 {
        wake.recv_timeout(Duration::from_secs(5)).unwrap();
        runtime.drain();
    }
    assert_eq!(runtime.pending(), 0);
}

#[test]
fn closed_mount_rejects_latest_task_and_cancels_previous_slot() {
    let (runtime, wake) = runtime();
    let model = Entity::new(Model::default());
    model.set_task_runtime(runtime.clone());
    let mount = model.mount().unwrap();
    let mut slot = TaskSlot::default();
    mount
        .spawn_latest(&mut slot, std::future::pending::<()>(), |_, _, _| {})
        .unwrap();
    assert!(slot.is_running());
    mount.close();
    assert!(matches!(
        mount.spawn_latest(&mut slot, async {}, |_, _, _| {}),
        Err(TaskError::ScopeClosed)
    ));
    assert!(!slot.is_running());
    while runtime.pending() > 0 {
        wake.recv_timeout(Duration::from_secs(5)).unwrap();
        runtime.drain();
    }
    assert_eq!(runtime.pending(), 0);
}

#[test]
fn context_tasks_cancel_with_the_view_while_model_tasks_keep_running() {
    let (runtime, wake) = runtime();
    let model = Entity::new(Model::default());
    model.set_task_runtime(runtime.clone());
    let mount = model.mount().unwrap();
    let task = mount
        .update(|_, cx| cx.spawn(async { 1 }, |_, _, _| panic!("closed view callback")))
        .unwrap()
        .unwrap();
    wake.recv_timeout(Duration::from_secs(5)).unwrap();
    let model_task = model
        .update(|_, cx| {
            cx.spawn(async { 7 }, |model, result, cx| {
                model.value = result.unwrap();
                cx.notify();
            })
        })
        .unwrap();
    mount.close();
    assert!(task.cancellation_token().is_cancelled());
    assert!(!model_task.cancellation_token().is_cancelled());
    while runtime.pending() > 0 {
        runtime.drain();
        if runtime.pending() > 0 {
            wake.recv_timeout(Duration::from_secs(5)).unwrap();
        }
    }
    assert_eq!(model.read(|model| model.value), 7);
    assert!(model_task.is_finished());
    assert_eq!(mount.resources().resource_count(), 0);
    assert_eq!(model.resources().resource_count(), 0);
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn context_blocking_task_cancellation_reaches_the_cooperative_worker() {
    let (runtime, wake) = runtime();
    let model = Entity::new(Model::default());
    model.set_task_runtime(runtime.clone());
    let mount = model.mount().unwrap();
    let (started, ready) = mpsc::channel();
    let (release, released) = mpsc::channel();
    let (finished, done) = mpsc::channel();
    let task = mount
        .update(|_, cx| {
            cx.spawn_blocking(
                move |token| {
                    started.send(()).unwrap();
                    released.recv_timeout(Duration::from_secs(5)).unwrap();
                    finished.send(token.is_cancelled()).unwrap();
                },
                |_, _, _| panic!("closed view blocking callback"),
            )
        })
        .unwrap()
        .unwrap();
    ready.recv_timeout(Duration::from_secs(5)).unwrap();
    mount.close();
    release.send(()).unwrap();
    assert!(done.recv_timeout(Duration::from_secs(5)).unwrap());
    assert!(task.cancellation_token().is_cancelled());
    pump(&runtime, &wake);
    assert_eq!(runtime.pending(), 0);
}

#[test]
fn mount_completion_uses_its_environment_and_releases_scoped_registrations() {
    use argui_core::ColorScheme;
    let (runtime, wake) = runtime();
    let model = Entity::new(Model::default());
    model.set_task_runtime(runtime.clone());
    let mount = model.mount().unwrap();
    let _ = mount
        .render(argui_runtime::WindowEnvironment {
            color_scheme: ColorScheme::Dark,
            ..Default::default()
        })
        .unwrap();
    let resources = mount.resources().clone();
    let baseline = resources.resource_count();
    let handle = mount
        .spawn(async { 12 }, move |model, result, cx| {
            assert_eq!(resources.resource_count(), baseline);
            assert_eq!(cx.environment().color_scheme, ColorScheme::Dark);
            model.value = result.unwrap();
            cx.notify();
        })
        .unwrap();
    pump(&runtime, &wake);
    assert!(handle.is_finished());
    assert_eq!(model.read(|model| model.value), 12);
    assert_eq!(mount.resources().resource_count(), baseline);
}

#[test]
fn dropping_a_mount_discards_its_already_queued_result_without_resurrection() {
    let (runtime, wake) = runtime();
    let model = Entity::new(Model::default());
    model.set_task_runtime(runtime.clone());
    let mount = model.mount().unwrap();
    let weak = mount.downgrade();
    let handle = mount
        .spawn(async { 1 }, |_, _, _| panic!("stale mount delivery"))
        .unwrap();
    wake.recv_timeout(Duration::from_secs(5)).unwrap();
    drop(mount);
    runtime.drain();
    assert!(weak.upgrade().is_none());
    assert!(handle.cancellation_token().is_cancelled());
    assert_eq!(model.read(|model| model.value), 0);
    assert_eq!(runtime.pending(), 0);
}

#[test]
fn mount_latest_replacement_and_closed_mount_reject_stale_work() {
    let (runtime, wake) = runtime();
    let model = Entity::new(Model::default());
    let mount = model.mount().unwrap();
    assert!(matches!(
        mount.spawn(async {}, |_, _, _| {}),
        Err(TaskError::Unavailable)
    ));
    model.set_task_runtime(runtime.clone());
    let mut slot = TaskSlot::default();
    mount
        .spawn_latest(&mut slot, async { 1 }, |_, _, _| panic!("replaced result"))
        .unwrap();
    wake.recv_timeout(Duration::from_secs(5)).unwrap();
    mount
        .spawn_latest(&mut slot, async { 2 }, |model, result, _| {
            model.value = result.unwrap()
        })
        .unwrap();
    while runtime.pending() > 0 {
        runtime.drain();
        if runtime.pending() > 0 {
            wake.recv_timeout(Duration::from_secs(5)).unwrap();
        }
    }
    assert_eq!(model.read(|model| model.value), 2);
    mount.resources().close();
    assert!(matches!(
        mount.spawn_latest(&mut slot, async {}, |_, _, _| {}),
        Err(TaskError::ScopeClosed)
    ));
    assert!(!slot.is_running());
}
fn pump(runtime: &TaskRuntime, wake: &mpsc::Receiver<()>) {
    wake.recv_timeout(Duration::from_secs(5))
        .expect("task wake");
    runtime.drain();
}
#[test]
fn results_arrive_only_on_ui_drain_and_invalidate_retained_view() {
    let (runtime, wake) = runtime();
    let app = Entity::new(Model::default());
    app.set_task_runtime(runtime.clone());
    let before = app.render();
    let ui_thread = std::thread::current().id();
    app.update(|model, cx| {
        cx.spawn_latest(&mut model.slot, async { 42 }, move |model, result, cx| {
            assert_eq!(std::thread::current().id(), ui_thread);
            model.value = result.unwrap();
            cx.notify();
        })
        .unwrap();
    });
    assert_eq!(app.read(|model| model.value), 0);
    pump(&runtime, &wake);
    assert_eq!(app.read(|model| model.value), 42);
    assert!(!app.read(|model| model.slot.is_running()));
    assert!(!app.render().ptr_eq(&before));
    assert_eq!(runtime.pending(), 0);
}
#[test]
fn latest_discards_already_queued_result() {
    let (runtime, wake) = runtime();
    let app = Entity::new(Model::default());
    app.set_task_runtime(runtime.clone());
    for number in [1, 2] {
        app.update(|model, cx| {
            cx.spawn_latest(
                &mut model.slot,
                async move { number },
                |model, result, cx| {
                    model.value = result.unwrap();
                    cx.notify();
                },
            )
            .unwrap();
        });
        if number == 1 {
            wake.recv_timeout(Duration::from_secs(5)).unwrap();
        }
    }
    runtime.drain();
    while runtime.pending() > 0 {
        pump(&runtime, &wake);
    }
    assert_eq!(app.read(|model| model.value), 2);
    assert_eq!(runtime.pending(), 0);
}
#[test]
fn dropped_owner_cancels_task_even_with_external_handle() {
    let (runtime, wake) = runtime();
    let app = Entity::new(Model::default());
    app.set_task_runtime(runtime.clone());
    let called = Rc::new(Cell::new(false));
    let capture = called.clone();
    let handle = app.update_task(move |cx| {
        cx.spawn(std::future::pending::<()>(), move |_, _, _| {
            capture.set(true)
        })
    });
    let token = handle.cancellation_token();
    drop(app);
    assert!(token.is_cancelled());
    pump(&runtime, &wake);
    assert!(!called.get());
    assert_eq!(runtime.pending(), 0);
}
trait Start {
    fn update_task(
        &self,
        start: impl FnOnce(
            &mut ModelContext<'_, Model>,
        ) -> Result<argui_runtime::tasks::TaskHandle, TaskError>,
    ) -> argui_runtime::tasks::TaskHandle;
}
impl Start for Entity<Model> {
    fn update_task(
        &self,
        start: impl FnOnce(
            &mut ModelContext<'_, Model>,
        ) -> Result<argui_runtime::tasks::TaskHandle, TaskError>,
    ) -> argui_runtime::tasks::TaskHandle {
        let mut handle = None;
        self.update(|_, cx| handle = Some(start(cx).unwrap()));
        handle.unwrap()
    }
}
#[test]
fn missing_executor_returns_error_and_panics_are_delivered_as_errors() {
    let app = Entity::new(Model::default());
    app.update(|_, cx| {
        assert!(matches!(
            cx.spawn(async {}, |_, _, _| {}),
            Err(TaskError::Unavailable)
        ))
    });
    let (runtime, wake) = runtime();
    app.set_task_runtime(runtime.clone());
    let called = Rc::new(Cell::new(false));
    let capture = called.clone();
    let _handle = app.update_task(move |cx| {
        cx.spawn(
            async { panic!("test failure") },
            move |_, result: Result<(), _>, _| {
                assert_eq!(result, Err(TaskError::Panicked));
                capture.set(true);
            },
        )
    });
    pump(&runtime, &wake);
    assert!(called.get());
}

#[test]
fn nested_tasks_reach_application_invalidation_and_commands_without_animation() {
    use argui_platform::WindowKey;
    use argui_runtime::{AppCommand, AppModel, SingleWindowModel, ViewUpdate, WindowEnvironment};
    struct Parent(Entity<Model>);
    impl Render for Parent {
        fn render(&mut self, cx: &mut Context<Self>) -> Element {
            Element::container([cx.entity(&self.0)])
        }
    }
    let (runtime, wake) = runtime();
    let child = Entity::new(Model::default());
    let mut app = SingleWindowModel::new(Parent(child.clone()));
    let window = WindowKey::main();
    app.event_router(&window)
        .unwrap()
        .set_task_runtime(runtime.clone());
    let before = app.view(&window, WindowEnvironment::default()).unwrap();
    let _handle = child.update_task(|cx| {
        cx.spawn(async { 19 }, |model, result, cx| {
            model.value = result.unwrap();
            cx.notify();
            cx.command(AppCommand::SetWindowTitle {
                window: WindowKey::main(),
                title: "Result".into(),
            });
        })
    });
    pump(&runtime, &wake);
    let update = app.tasks_ready(&window);
    assert_eq!(update.windows[0].update, ViewUpdate::Rebuild);
    assert_eq!(update.commands.len(), 1);
    assert!(
        !app.view(&window, WindowEnvironment::default())
            .unwrap()
            .ptr_eq(&before)
    );
    assert!(!app.wants_animation_frame(&window));
    assert!(app.tasks_ready(&window).windows.is_empty());
}

#[test]
fn capacity_is_bounded_and_cancellation_releases_slots() {
    let (runtime, wake) = runtime();
    let app = Entity::new(Model::default());
    app.set_task_runtime(runtime.clone());
    let mut handles = Vec::new();
    for _ in 0..256 {
        handles.push(app.update_task(|cx| {
            cx.spawn(std::future::pending::<()>(), |_, _, _| {
                panic!("cancelled callback")
            })
        }));
    }
    assert_eq!(runtime.pending(), 256);
    app.update(|_, cx| {
        assert!(matches!(
            cx.spawn(async {}, |_, _, _| {}),
            Err(TaskError::CapacityExceeded)
        ));
    });
    drop(handles);
    while runtime.pending() > 0 {
        pump(&runtime, &wake);
    }
    let _handle = app.update_task(|cx| {
        cx.spawn(async { 7 }, |model, result, _| {
            model.value = result.unwrap()
        })
    });
    while runtime.pending() > 0 {
        pump(&runtime, &wake);
    }
    assert_eq!(app.read(|model| model.value), 7);
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn blocking_task_gets_cooperative_cancellation_and_never_delivers_cancelled_result() {
    let (runtime, wake) = runtime();
    let app = Entity::new(Model::default());
    app.set_task_runtime(runtime.clone());
    let (started, started_rx) = mpsc::channel();
    let (release, release_rx) = mpsc::channel();
    let (observed, observed_rx) = mpsc::channel();
    let handle = app.update_task(|cx| {
        cx.spawn_blocking(
            move |token| {
                started.send(()).unwrap();
                release_rx.recv_timeout(Duration::from_secs(5)).unwrap();
                observed.send(token.is_cancelled()).unwrap();
            },
            |_, _, _| panic!("cancelled blocking callback"),
        )
    });
    started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    handle.cancel();
    release.send(()).unwrap();
    assert!(observed_rx.recv_timeout(Duration::from_secs(5)).unwrap());
    while runtime.pending() > 0 {
        pump(&runtime, &wake);
    }
}

#[test]
fn explicit_shutdown_cancels_surviving_owners_and_refuses_new_work() {
    let (runtime, _) = runtime();
    let app = Entity::new(Model::default());
    app.set_task_runtime(runtime.clone());
    let handle = app.update_task(|cx| {
        cx.spawn(std::future::pending::<()>(), |_, _, _| {
            panic!("closed runtime")
        })
    });
    runtime.shutdown();
    assert!(handle.cancellation_token().is_cancelled());
    assert_eq!(runtime.pending(), 0);
    app.update(|_, cx| {
        assert!(matches!(
            cx.spawn(async {}, |_, _, _| {}),
            Err(TaskError::Unavailable)
        ))
    });
    runtime.shutdown();
}

#[test]
fn closing_the_model_scope_closes_its_mount_and_rejects_new_tasks() {
    let (runtime, _) = runtime();
    let model = Entity::new(Model::default());
    model.set_task_runtime(runtime);
    let mount = model.mount().unwrap();
    assert!(!mount.resources().is_closed());

    model.resources().close();
    assert!(mount.resources().is_closed());
    assert!(model.mount().is_err());
    model.update(|_, cx| {
        assert!(matches!(
            cx.spawn(async {}, |_, _, _| {}),
            Err(TaskError::ScopeClosed)
        ));
    });
    assert!(matches!(
        mount.spawn(async {}, |_, _, _| {}),
        Err(TaskError::ScopeClosed)
    ));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn blocking_worker_panic_is_delivered_as_a_task_error() {
    let (runtime, wake) = runtime();
    let model = Entity::new(Model::default());
    model.set_task_runtime(runtime.clone());
    let received = Rc::new(Cell::new(false));
    let observed = received.clone();
    let handle = model.update_task(move |cx| {
        cx.spawn_blocking(
            |_| -> usize { panic!("worker failure") },
            move |_, result, _| {
                assert_eq!(result, Err(TaskError::Panicked));
                observed.set(true);
            },
        )
    });
    pump(&runtime, &wake);
    assert!(handle.is_finished());
    assert!(received.get());
    assert_eq!(runtime.pending(), 0);
}
