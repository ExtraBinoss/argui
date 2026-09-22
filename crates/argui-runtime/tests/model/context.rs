use argui_runtime::{
    Context, Entity, Render,
    tasks::{TaskError, TaskRuntime},
};
use argui_ui::Element;
use std::sync::mpsc;
#[cfg(not(target_arch = "wasm32"))]
use std::{cell::Cell, rc::Rc, time::Duration};

struct TaskLeaf;

#[derive(Default)]
struct CaptureProbe {
    outside_handler: (bool, bool),
    inside_handler: (bool, bool),
}

impl Render for CaptureProbe {
    /// Records capture availability while building a view and handling a click.
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let pointer = argui_core::PointerId::MOUSE;
        self.outside_handler = (cx.capture_pointer(pointer), cx.release_pointer(pointer));
        Element::text("capture").on(cx.listener(argui_ui::EventType::Click, |model, _, cx| {
            let pointer = argui_core::PointerId::MOUSE;
            model.inside_handler = (cx.capture_pointer(pointer), cx.release_pointer(pointer));
        }))
    }
}

/// Pointer capture requests require a current event target.
#[test]
fn pointer_capture_is_rejected_during_render_and_accepted_in_a_handler() {
    let model = Entity::new(CaptureProbe::default());
    let mount = model.mount().unwrap();
    let mut tree = argui_ui::UiTree::new(mount.render(Default::default()).unwrap());
    assert_eq!(model.read(|probe| probe.outside_handler), (false, false));
    for delivery in tree.event_deliveries(
        tree.node_ids()[0],
        argui_ui::UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
    ) {
        mount.dispatch_event(&delivery).unwrap();
    }
    assert_eq!(model.read(|probe| probe.inside_handler), (true, true));
}

impl Render for TaskLeaf {
    /// Renders a routed task owner with no native window dependency.
    fn render(&mut self, _: &mut Context<Self>) -> Element {
        Element::text("task leaf")
    }
}

struct RoutedTaskOwner(Entity<TaskLeaf>);

impl Render for RoutedTaskOwner {
    /// Retains the child as a routed event and task endpoint.
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let element = self.0.render();
        cx.route_events_to(self.0.erase());
        element
    }
}

/// Routed descendants inherit an executor only after one is bound to their owner.
#[test]
fn routed_descendant_inherits_task_runtime() {
    let child = Entity::new(TaskLeaf);
    let parent = Entity::new(RoutedTaskOwner(child.clone()));
    assert!(matches!(
        child.update(|_, cx| cx.spawn(async {}, |_, _, _| {})),
        Err(TaskError::Unavailable)
    ));

    let runtime = TaskRuntime::new(|| {});
    parent.set_task_runtime(runtime.clone());
    let _ = parent.render();
    let handle = child
        .update(|_, cx| cx.spawn(std::future::pending::<()>(), |_, _, _| {}))
        .unwrap();
    assert_eq!(runtime.pending(), 1);
    handle.cancel();
    runtime.shutdown();
}

#[derive(Default)]
struct ContextTaskModel {
    value: usize,
}

impl Render for ContextTaskModel {
    /// Renders the latest result delivered to this model's context.
    fn render(&mut self, _: &mut Context<Self>) -> Element {
        Element::text(self.value.to_string())
    }
}

/// Creates a task runtime whose wakeups can be awaited by a test.
///
/// Returns the executor and the receiver for its wake notifications.
fn context_task_runtime() -> (TaskRuntime, mpsc::Receiver<()>) {
    let (sender, receiver) = mpsc::channel();
    (
        TaskRuntime::new(move || {
            let _ = sender.send(());
        }),
        receiver,
    )
}

/// Drains one pending task notification into the model thread.
///
/// `runtime` owns the pending completion and `wake` receives its signal.
#[cfg(not(target_arch = "wasm32"))]
fn drain_task(runtime: &TaskRuntime, wake: &mpsc::Receiver<()>) {
    wake.recv_timeout(Duration::from_secs(5)).unwrap();
    runtime.drain();
}

/// Detached contexts and closed models cannot create work on an attached executor.
#[test]
fn detached_context_and_closed_model_reject_new_tasks() {
    let mut detached = Context::<ContextTaskModel>::default();
    assert!(matches!(
        detached.spawn(async {}, |_, _, _| {}),
        Err(TaskError::Unavailable)
    ));

    let (runtime, _) = context_task_runtime();
    let model = Entity::new(ContextTaskModel::default());
    model.set_task_runtime(runtime.clone());
    let mount = model.mount().unwrap();
    model.resources().close();
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
    assert_eq!(runtime.pending(), 0);
    runtime.shutdown();
}

/// A successful native blocking task delivers its value on the model thread.
#[test]
#[cfg(not(target_arch = "wasm32"))]
fn successful_blocking_result_reaches_its_model_owner() {
    let (runtime, wake) = context_task_runtime();
    let model = Entity::new(ContextTaskModel::default());
    model.set_task_runtime(runtime.clone());
    let ui_thread = std::thread::current().id();
    let task = model.update(|_, cx| {
        cx.spawn_blocking(
            |_| 31_usize,
            move |model, result, _| {
                assert_eq!(std::thread::current().id(), ui_thread);
                model.value = result.unwrap();
            },
        )
        .unwrap()
    });
    while !task.is_finished() {
        drain_task(&runtime, &wake);
    }
    assert_eq!(model.read(|model| model.value), 31);
    runtime.shutdown();
}

/// A native worker panic is surfaced without poisoning the UI thread.
#[test]
#[cfg(not(target_arch = "wasm32"))]
fn panicked_blocking_worker_reports_an_error() {
    let (runtime, wake) = context_task_runtime();
    let model = Entity::new(ContextTaskModel::default());
    model.set_task_runtime(runtime.clone());
    let reported = Rc::new(Cell::new(false));
    let observed = reported.clone();
    let task = model.update(move |_, cx| {
        cx.spawn_blocking(
            |_| -> usize { panic!("intentional worker failure") },
            move |_, result, _| {
                assert_eq!(result, Err(TaskError::Panicked));
                observed.set(true);
            },
        )
        .unwrap()
    });
    while !task.is_finished() {
        drain_task(&runtime, &wake);
    }
    assert!(reported.get());
    runtime.shutdown();
}

struct TaskReadyLeaf(usize);

impl Render for TaskReadyLeaf {
    /// Exposes the number of delivered task-ready notifications.
    fn render(&mut self, _: &mut Context<Self>) -> Element {
        Element::text(self.0.to_string())
    }

    /// Invalidates this routed view when its task completion is delivered.
    fn tasks_ready(&mut self, cx: &mut Context<Self>) {
        self.0 += 1;
        cx.notify();
    }
}

struct RoutedTaskProbe(Entity<TaskReadyLeaf>);

impl Render for RoutedTaskProbe {
    /// Publishes an independently rendered child as a routed descendant.
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let element = self.0.render();
        cx.route_events_to(self.0.erase());
        element
    }
}

/// Task effects from a routed child propagate to the owning window adapter.
#[test]
fn routed_child_task_effects_rebuild_its_parent_window() {
    use argui_runtime::{AppModel, SingleWindowModel, ViewUpdate};

    let child = Entity::new(TaskReadyLeaf(0));
    let mut app = SingleWindowModel::new(RoutedTaskProbe(child.clone()));
    let window = argui_platform::WindowKey::main();
    let first = app.view(&window, Default::default()).unwrap();
    let update = app.tasks_ready(&window);
    assert_eq!(update.windows[0].update, ViewUpdate::Rebuild);
    assert_eq!(child.read(|child| child.0), 1);
    assert!(
        !app.view(&window, Default::default())
            .unwrap()
            .ptr_eq(&first)
    );
}
