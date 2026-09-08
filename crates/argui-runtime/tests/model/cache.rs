use argui_runtime::{Context, Entity, ModelRuntime, Render, Subscription};
use argui_ui::Element;
use std::{cell::Cell, rc::Rc};

struct View {
    model: Entity<usize>,
    watching: bool,
    renders: Rc<Cell<usize>>,
}

#[test]
fn observing_or_tracking_another_runtime_is_rejected_without_registration() {
    let source = Entity::new(1_usize);
    let view = Entity::new(View {
        model: source.clone(),
        watching: true,
        renders: Rc::new(Cell::new(0)),
    });
    assert!(matches!(
        view.update(|_, cx| cx.observe(&source)),
        Err(argui_runtime::EventError::DifferentRuntime)
    ));
    let mount = view.mount().unwrap();
    assert!(matches!(
        mount.update(|_, cx| cx.observe(&source)).unwrap(),
        Err(argui_runtime::EventError::DifferentRuntime)
    ));
    let registrations = mount.resources().resource_count();
    let rejected = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        mount.render(Default::default()).unwrap();
    }));
    assert!(rejected.is_err());
    assert_eq!(source.resources().resource_count(), 0);
    assert_eq!(mount.resources().resource_count(), registrations);
}
impl Render for View {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        self.renders.set(self.renders.get() + 1);
        let value = if self.watching {
            cx.read(&self.model, |value| *value)
        } else {
            0
        };
        Element::text(value.to_string())
    }
}

#[test]
fn tracked_model_reads_invalidate_every_view_and_detach_when_no_longer_read() {
    let runtime = ModelRuntime::default();
    let model = runtime.entity(1);
    let renders = Rc::new(Cell::new(0));
    let first = runtime.entity(View {
        model: model.clone(),
        watching: true,
        renders: renders.clone(),
    });
    let second = runtime.entity(View {
        model: model.clone(),
        watching: true,
        renders: renders.clone(),
    });
    let before = first.render();
    let _ = second.render();
    assert!(first.render().ptr_eq(&before));
    assert_eq!(renders.get(), 2);
    model.update(|value, cx| {
        *value = 2;
        cx.notify();
    });
    assert!(!first.render().ptr_eq(&before));
    let _ = second.render();
    assert_eq!(renders.get(), 4);
    first.update(|view, cx| {
        view.watching = false;
        cx.notify();
    });
    let retained = first.render();
    model.update(|value, cx| {
        *value = 3;
        cx.notify();
    });
    assert!(first.render().ptr_eq(&retained));
    let _ = second.render();
    assert_eq!(renders.get(), 6);
}

struct Parent(Entity<View>);
impl Render for Parent {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        cx.entity(&self.0)
    }
}
struct Grandparent(Entity<Parent>);
impl Render for Grandparent {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        cx.entity(&self.0)
    }
}

#[test]
fn model_invalidation_propagates_through_multiple_cached_ancestors() {
    let runtime = ModelRuntime::default();
    let model = runtime.entity(1);
    let leaf = runtime.entity(View {
        model: model.clone(),
        watching: true,
        renders: Rc::new(Cell::new(0)),
    });
    let root = runtime.entity(Grandparent(runtime.entity(Parent(leaf))));
    let before = root.render();
    model.update(|value, cx| {
        *value = 2;
        cx.notify();
    });
    assert!(!root.render().ptr_eq(&before));
}

#[test]
fn explicit_observers_release_with_endpoints_and_cycles_do_not_recurse_forever() {
    let runtime = ModelRuntime::default();
    let first = runtime.entity(());
    let second = runtime.entity(());
    let mut forward: Option<Subscription> = None;
    let mut backward = None;
    first.update(|_, cx| forward = Some(cx.observe(&second).unwrap()));
    second.update(|_, cx| backward = Some(cx.observe(&first).unwrap()));
    first.update(|_, cx| cx.notify());
    second.update(|_, cx| cx.notify());
    drop(first);
    assert!(!forward.unwrap().is_active());
    assert!(!backward.unwrap().is_active());
    second.update(|_, cx| cx.notify());
}

#[test]
#[cfg(feature = "tasks")]
fn a_model_change_requests_a_window_rebuild_without_a_rendered_model_child() {
    use argui_runtime::{AppModel, SingleWindowModel, ViewUpdate, WindowEnvironment};
    let model = Entity::new(1);
    let mut app = SingleWindowModel::from_entity(model.runtime().entity(View {
        model: model.clone(),
        watching: true,
        renders: Rc::new(Cell::new(0)),
    }))
    .unwrap();
    let window = argui_platform::WindowKey::main();
    let _ = app.view(&window, WindowEnvironment::default());
    let _ = app.tasks_ready(&window);
    model.update(|value, cx| {
        *value = 2;
        cx.notify();
    });
    let update = app.tasks_ready(&window);
    assert!(
        update
            .windows
            .iter()
            .any(|entry| entry.window == window && entry.update == ViewUpdate::Rebuild)
    );
    let _ = app.view(&window, WindowEnvironment::default());
    assert!(app.tasks_ready(&window).windows.is_empty());
}

#[test]
fn many_notifications_coalesce_and_large_fanout_yields_between_batches() {
    let wakes = Rc::new(Cell::new(0));
    let notified = wakes.clone();
    let runtime = ModelRuntime::new(move || notified.set(notified.get() + 1));
    let model = runtime.entity(0);
    let renders = Rc::new(Cell::new(0));
    let views: Vec<_> = (0..128)
        .map(|_| {
            runtime.entity(View {
                model: model.clone(),
                watching: true,
                renders: renders.clone(),
            })
        })
        .collect();
    let before: Vec<_> = views.iter().map(Entity::render).collect();
    runtime.transaction(|| {
        for value in 1..=20 {
            model.update(|model, cx| {
                *model = value;
                cx.notify();
            });
        }
        assert_eq!(runtime.pending_invalidations(), 1);
        assert_eq!(renders.get(), 128);
    });
    assert_eq!(wakes.get(), 1);
    assert_eq!(runtime.pending_invalidations(), 1);
    let changed = runtime.transaction(|| {
        views
            .iter()
            .zip(&before)
            .filter(|(view, before)| !view.render().ptr_eq(before))
            .count()
    });
    assert_eq!(changed, 64);
    assert_eq!(runtime.pending_invalidations(), 0);
    for view in views {
        let _ = view.render();
    }
    assert_eq!(renders.get(), 256);
}

#[test]
fn long_cyclic_dependency_graph_is_iterative_and_dropped_queued_sources_are_released() {
    let runtime = ModelRuntime::default();
    let nodes: Vec<_> = (0..1024).map(|_| runtime.entity(())).collect();
    let mut subscriptions = Vec::new();
    for index in 0..nodes.len() {
        nodes[index].update(|_, cx| {
            subscriptions.push(cx.observe(&nodes[(index + 1) % nodes.len()]).unwrap())
        });
    }
    nodes[0].update(|_, cx| cx.notify());
    assert!(runtime.pending_invalidations() > 0);
    let mut batches = 0;
    while runtime.pending_invalidations() > 0 {
        runtime.dispatch_pending();
        batches += 1;
        assert!(batches < 32);
    }
    assert_eq!(batches, 15);
    nodes[0].update(|_, cx| cx.notify());
    let weak = nodes[0].downgrade();
    drop(nodes);
    assert!(weak.upgrade().is_none());
    assert!(
        subscriptions
            .iter()
            .all(|subscription| !subscription.is_active())
    );
    runtime.dispatch_pending();
    assert_eq!(runtime.pending_invalidations(), 0);
}

#[test]
fn a_new_transaction_reinvalidates_views_already_painted_during_an_older_wave() {
    let runtime = ModelRuntime::default();
    let model = runtime.entity(0);
    let views: Vec<_> = (0..128)
        .map(|_| {
            runtime.entity(View {
                model: model.clone(),
                watching: true,
                renders: Rc::new(Cell::new(0)),
            })
        })
        .collect();
    for view in &views {
        let _ = view.render();
    }
    model.update(|value, cx| {
        *value = 1;
        cx.notify();
    });
    assert_eq!(runtime.pending_invalidations(), 1);
    let old = runtime.transaction(|| {
        let painted = views[0].render();
        // This is a distinct outer transaction from the first update. The old
        // wave is still running, but its visited set must not suppress this change.
        model.update(|value, cx| {
            *value = 2;
            cx.notify();
        });
        assert_eq!(runtime.pending_invalidations(), 2);
        painted
    });
    while runtime.pending_invalidations() > 0 {
        runtime.dispatch_pending();
    }
    let current = views[0].render();
    assert!(!current.ptr_eq(&old));
    assert!(
        matches!(&current.kind, argui_ui::ElementKind::Text { content, .. } if content.as_str() == "2")
    );
}
