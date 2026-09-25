#![cfg(feature = "tasks")]
#[path = "model/tasks.rs"]
mod model_tasks;

use argui_runtime::tasks::{CancellationToken, TaskSlot};

#[test]
fn cancellation_is_shared_and_empty_slot_is_idle() {
    let token = CancellationToken::default();
    let clone = token.clone();
    assert!(!clone.is_cancelled());
    token.cancel();
    assert!(clone.is_cancelled());
    let mut slot = TaskSlot::default();
    assert!(!slot.is_running());
    slot.cancel();
}

/// The public scheduling helpers resume on a live executor without external wakeups.
#[test]
#[cfg(not(target_arch = "wasm32"))]
fn sleep_and_yield_resume_within_a_current_thread_runtime() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap();
    runtime.block_on(async {
        argui_runtime::tasks::yield_now().await;
        argui_runtime::tasks::sleep(std::time::Duration::from_millis(1)).await;
    });
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn dispatcher_clock_rejects_wall_time_control_and_shutdown() {
    use argui_runtime::tasks::{TaskError, TaskRuntime};

    let realtime = TaskRuntime::new(|| {});
    assert!(matches!(
        realtime.advance_time(std::time::Duration::from_millis(1)),
        Err(TaskError::Unavailable)
    ));
    realtime.advance_time(std::time::Duration::ZERO).unwrap();

    let paused = TaskRuntime::new_paused(|| {});
    paused
        .advance_time(std::time::Duration::from_millis(1))
        .unwrap();
    paused.shutdown();
    assert!(matches!(
        paused.advance_time(std::time::Duration::ZERO),
        Err(TaskError::Unavailable)
    ));
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn data_models_share_the_executor_without_rendering_or_parent_retention() {
    use argui_runtime::{ModelRuntime, tasks::TaskRuntime};
    let models = ModelRuntime::default();
    let parent = models.entity(());
    let before_attachment = models.entity(0);
    let (sender, wake) = std::sync::mpsc::channel();
    let tasks = TaskRuntime::new(move || {
        let _ = sender.send(());
    });
    models.set_task_runtime(tasks.clone());
    let mut child = None;
    parent.update(|_, cx| child = Some(cx.new_entity(0)));
    let child = child.unwrap();
    drop(parent);
    let mut handles = Vec::new();
    for model in [&before_attachment, &child] {
        model.update(|_, cx| {
            handles.push(
                cx.spawn(async { 7 }, |value, result, _| {
                    *value = result.unwrap();
                })
                .unwrap(),
            );
        });
    }
    while tasks.pending() > 0 {
        wake.recv_timeout(std::time::Duration::from_secs(5))
            .unwrap();
        tasks.drain();
    }
    assert_eq!(before_attachment.read(|value| *value), 7);
    assert_eq!(child.read(|value| *value), 7);
    assert!(handles.iter().all(|handle| handle.is_finished()));
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn explicit_scope_and_entity_lifetimes_cancel_only_their_owned_tasks() {
    use argui_runtime::{Entity, ResourceScope, tasks::TaskRuntime};
    let (sender, wake) = std::sync::mpsc::channel();
    let runtime = TaskRuntime::new(move || {
        let _ = sender.send(());
    });
    let model = Entity::new(());
    model.set_task_runtime(runtime.clone());
    let view_scope = ResourceScope::default();
    let model_scope = model.resources().clone();
    let mut tasks = None;
    model.update(|_, cx| {
        let view = cx
            .spawn_in(&view_scope, std::future::pending::<()>(), |_, _, _| {
                panic!("closed view result")
            })
            .unwrap();
        let model = cx
            .spawn(std::future::pending::<()>(), |_, _, _| {
                panic!("destroyed model result")
            })
            .unwrap();
        tasks = Some((view, model));
    });
    let (view, shared) = tasks.unwrap();
    view_scope.close();
    assert!(view.cancellation_token().is_cancelled());
    assert!(!shared.cancellation_token().is_cancelled());
    drop(model);
    assert!(model_scope.is_closed());
    assert!(shared.cancellation_token().is_cancelled());
    assert_eq!(model_scope.resource_count(), 0);
    while runtime.pending() > 0 {
        wake.recv_timeout(std::time::Duration::from_secs(5))
            .unwrap();
        runtime.drain();
    }
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn closed_scopes_reject_work_without_starting_a_future() {
    use argui_runtime::{
        Entity, ResourceScope,
        tasks::{TaskError, TaskRuntime},
    };
    let runtime = TaskRuntime::new(|| {});
    let model = Entity::new(());
    model.set_task_runtime(runtime.clone());
    let scope = ResourceScope::default();
    scope.close();
    model.update(|_, cx| {
        assert!(matches!(
            cx.spawn_in(&scope, async {}, |_, _, _| {}),
            Err(TaskError::ScopeClosed)
        ));
    });
    assert_eq!(runtime.pending(), 0);
    model.resources().close();
    model.update(|_, cx| {
        assert!(matches!(
            cx.spawn(async {}, |_, _, _| {}),
            Err(TaskError::ScopeClosed)
        ));
    });
    assert_eq!(runtime.pending(), 0);
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn completed_tasks_release_scopes_before_callbacks_with_handles_retained() {
    use argui_runtime::{Entity, ResourceScope, tasks::TaskRuntime};
    let (sender, wake) = std::sync::mpsc::channel();
    let runtime = TaskRuntime::new(move || {
        let _ = sender.send(());
    });
    let model = Entity::new(0);
    model.set_task_runtime(runtime.clone());
    let scope = ResourceScope::default();
    let owner_scope = model.resources().clone();
    let mut handle = None;
    model.update(|_, cx| {
        let observed = scope.clone();
        let owner = owner_scope.clone();
        handle = Some(
            cx.spawn_in(&scope, async { 42 }, move |value, result, _| {
                assert_eq!(observed.resource_count(), 0);
                assert_eq!(owner.resource_count(), 0);
                *value = result.unwrap();
            })
            .unwrap(),
        );
    });
    let handle = handle.unwrap();
    assert_eq!(scope.resource_count(), 1);
    while runtime.pending() > 0 {
        wake.recv_timeout(std::time::Duration::from_secs(5))
            .unwrap();
        runtime.drain();
    }
    assert!(handle.is_finished());
    assert!(!handle.cancellation_token().is_cancelled());
    assert_eq!(model.read(|value| *value), 42);
    let handle = handle.in_scope(&scope).unwrap();
    assert_eq!(scope.resource_count(), 0);
    drop(handle);
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn closing_one_attached_scope_cancels_work_and_releases_all_leases() {
    use argui_runtime::{Entity, ResourceScope, tasks::TaskRuntime};
    let (sender, wake) = std::sync::mpsc::channel();
    let runtime = TaskRuntime::new(move || {
        let _ = sender.send(());
    });
    let model = Entity::new(());
    model.set_task_runtime(runtime.clone());
    let first_scope = ResourceScope::default();
    let second_scope = ResourceScope::default();
    let mut handle = None;
    model.update(|_, cx| {
        handle = Some(
            cx.spawn(std::future::pending::<()>(), |_, _, _| {
                panic!("cancelled result must not be delivered");
            })
            .unwrap(),
        );
    });
    let handle = handle
        .unwrap()
        .in_scope(&first_scope)
        .unwrap()
        .in_scope(&second_scope)
        .unwrap();
    assert_eq!(model.resources().resource_count(), 1);
    assert_eq!(first_scope.resource_count(), 1);
    assert_eq!(second_scope.resource_count(), 1);

    first_scope.close();

    assert!(handle.cancellation_token().is_cancelled());
    assert!(handle.is_finished());
    assert_eq!(model.resources().resource_count(), 0);
    assert_eq!(first_scope.resource_count(), 0);
    assert_eq!(second_scope.resource_count(), 0);
    while runtime.pending() > 0 {
        wake.recv_timeout(std::time::Duration::from_secs(5))
            .unwrap();
        runtime.drain();
    }
    assert_eq!(runtime.pending(), 0);
}

/// A later cleanup may release the task owner before its saved cancellation hook runs.
#[test]
#[cfg(not(target_arch = "wasm32"))]
fn closing_scope_with_reentrant_shutdown_tolerates_expired_task_handles() {
    use argui_runtime::{Entity, ResourceScope, tasks::TaskRuntime};
    let runtime = TaskRuntime::new(|| {});
    let model = Entity::new(());
    model.set_task_runtime(runtime.clone());
    let scope = ResourceScope::default();
    let handle = model
        .update(|_, cx| cx.spawn(std::future::pending::<()>(), |_, _, _| {}))
        .unwrap()
        .in_scope(&scope)
        .unwrap();
    let shutdown = runtime.clone();
    let _cleanup = scope
        .defer(move || {
            drop(handle);
            shutdown.shutdown();
        })
        .unwrap();
    scope.close();
    assert_eq!(runtime.pending(), 0);
    assert_eq!(model.resources().resource_count(), 0);
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn cancellation_and_shutdown_release_all_scope_registrations() {
    use argui_runtime::{Entity, ResourceScope, tasks::TaskRuntime};
    for mode in 0..4 {
        let runtime = TaskRuntime::new(|| {});
        let model = Entity::new(());
        model.set_task_runtime(runtime.clone());
        let scope = ResourceScope::default();
        let mut task = None;
        model.update(|_, cx| {
            task = Some(
                cx.spawn_in(&scope, std::future::pending::<()>(), |_, _, _| {
                    panic!("cancelled completion");
                })
                .unwrap(),
            );
        });
        let task = task.unwrap();
        assert_eq!(model.resources().resource_count(), 1);
        assert_eq!(scope.resource_count(), 1);
        match mode {
            0 => task.cancel(),
            1 => scope.close(),
            2 => model.resources().close(),
            _ => runtime.shutdown(),
        }
        assert!(task.is_finished());
        assert_eq!(model.resources().resource_count(), 0);
        assert_eq!(scope.resource_count(), 0);
        task.cancel();
        runtime.shutdown();
    }
}
