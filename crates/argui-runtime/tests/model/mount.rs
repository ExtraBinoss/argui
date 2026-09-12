use argui_core::ColorScheme;
use argui_runtime::{Context, Entity, LayoutSnapshot, Render, ScopeClosed, WindowEnvironment};
use argui_ui::{ClickEvent, Element, EventType, UiEventKind, UiTree};

#[derive(Default)]
struct View {
    on_click: Option<std::rc::Rc<dyn Fn()>>,
    layouts: Vec<ColorScheme>,
    renders: usize,
    clicks: usize,
    dark_clicks: usize,
}
impl Render for View {
    fn animation_frame(&mut self, _: argui_animation::Frame, cx: &mut Context<Self>) {
        self.layouts.push(cx.environment().color_scheme);
    }
    fn layout_changed(&mut self, _: &argui_runtime::LayoutSnapshot, cx: &mut Context<Self>) {
        self.layouts.push(cx.environment().color_scheme);
    }
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        self.renders += 1;
        let scheme = cx.environment().color_scheme;
        Element::text(format!("{scheme:?}:{}", self.clicks)).on(cx.listener(
            EventType::Click,
            |view, _, cx| {
                if let Some(close) = &view.on_click {
                    close();
                }
                view.clicks += 1;
                view.dark_clicks += usize::from(cx.environment().color_scheme == ColorScheme::Dark);
                cx.notify();
            },
        ))
    }
}

#[test]
fn updates_use_the_target_mount_environment_and_reject_closed_presentations() {
    let model = Entity::new(View::default());
    let dark = model.mount().unwrap();
    let light = model.mount().unwrap();
    dark.render(WindowEnvironment {
        color_scheme: ColorScheme::Dark,
        ..Default::default()
    })
    .unwrap();
    light.render(WindowEnvironment::default()).unwrap();
    assert_eq!(
        dark.update(|_, cx| cx.environment().color_scheme),
        Ok(ColorScheme::Dark)
    );
    assert_eq!(
        light.update(|_, cx| cx.environment().color_scheme),
        Ok(ColorScheme::Light)
    );
    dark.close();
    let frame = argui_animation::Frame {
        now: argui_animation::Time::ZERO,
        elapsed: argui_animation::Duration::from_millis(16),
    };
    assert_eq!(dark.animation_frame(frame), Err(ScopeClosed));
    light.animation_frame(frame).unwrap();
    assert_eq!(
        light.read(|view| view.layouts.clone()),
        vec![ColorScheme::Light]
    );
    assert_eq!(
        dark.layout_changed(&LayoutSnapshot::default()),
        Err(ScopeClosed)
    );
    assert_eq!(
        model.read(|view| view.layouts.clone()),
        vec![ColorScheme::Light]
    );
    let result = dark.update(|_, _| panic!("closed mount must not invoke its callback"));
    assert_eq!(result, Err::<(), _>(ScopeClosed));
    assert_eq!(
        light.update(|view, cx| {
            view.clicks += 1;
            cx.notify();
            view.clicks
        }),
        Ok(1)
    );
    assert_eq!(model.read(|view| view.clicks), 1);
}

#[test]
fn mounts_share_data_but_keep_caches_environments_and_event_owners_independent() {
    let model = Entity::new(View::default());
    let left = model.mount().unwrap();
    let right = model.mount().unwrap();
    assert_ne!(left.id(), right.id());
    assert_ne!(left.id().get(), model.id().get());
    assert_eq!(left.model_id(), right.model_id());
    let dark = WindowEnvironment {
        color_scheme: ColorScheme::Dark,
        ..Default::default()
    };
    let light = WindowEnvironment::default();
    let first = left.render(dark).unwrap();
    let other = right.render(light).unwrap();
    assert_ne!(first, other);
    assert_eq!(left.render(dark).unwrap(), first);
    assert_eq!(model.read(|view| view.renders), 2);
    let mut tree = UiTree::new(first);
    let events = tree.event_deliveries(
        tree.node_ids()[0],
        UiEventKind::Click(ClickEvent::accessibility()),
    );
    for event in &events {
        right.dispatch_event(event).unwrap();
    }
    assert_eq!(model.read(|view| view.clicks), 0);
    for event in &events {
        left.dispatch_event(event).unwrap();
    }
    assert_eq!(model.read(|view| (view.clicks, view.dark_clicks)), (1, 1));
    let _ = left.render(dark).unwrap();
    let _ = right.render(light).unwrap();
    assert_eq!(model.read(|view| view.renders), 4);
}

#[test]
fn weak_model_survives_a_mount_while_weak_mount_does_not_resurrect_it() {
    let model = Entity::new(View::default());
    let weak_model = model.downgrade();
    let first = model.mount().unwrap();
    let second = model.mount().unwrap();
    let weak_mount = first.downgrade();
    let scope = first.resources().clone();
    let id = first.id();
    assert_eq!(first.clone().id(), id);
    assert_eq!(weak_mount.clone().upgrade().unwrap().id(), id);
    drop(model);
    drop(first);
    assert!(scope.is_closed());
    assert!(weak_mount.upgrade().is_none());
    let retained = weak_model.upgrade().unwrap();
    assert_eq!(retained.id(), second.model_id());
    let third = retained.mount().unwrap();
    assert_ne!(third.id(), id);
    drop(retained);
    drop(second);
    assert!(weak_model.upgrade().is_some());
    drop(third);
    assert!(weak_model.upgrade().is_none());
}

#[test]
fn closing_the_model_closes_mount_resources_and_rejects_more_rendering() {
    let model = Entity::new(View::default());
    let mount = model.mount().unwrap();
    assert_eq!(model.resources().resource_count(), 1);
    model.resources().close();
    assert!(mount.resources().is_closed());
    assert!(matches!(
        mount.render(WindowEnvironment::default()),
        Err(ScopeClosed)
    ));
    assert!(matches!(model.mount(), Err(ScopeClosed)));
    assert_eq!(model.resources().resource_count(), 0);
}

struct Parent {
    child: Entity<View>,
    visible: bool,
}
impl Render for Parent {
    fn layout_changed(&mut self, layout: &argui_runtime::LayoutSnapshot, cx: &mut Context<Self>) {
        cx.layout_entity(&self.child, layout);
    }
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        if self.visible {
            cx.entity(&self.child)
        } else {
            Element::container([])
        }
    }
}

#[test]
fn nested_mounts_are_distinct_reused_and_released_without_destroying_child_data() {
    let child = Entity::new(View::default());
    let parent = Entity::new(Parent {
        child: child.clone(),
        visible: true,
    });
    let first = parent.mount().unwrap();
    let second = parent.mount().unwrap();
    let dark = WindowEnvironment {
        color_scheme: ColorScheme::Dark,
        ..Default::default()
    };
    let light = WindowEnvironment::default();
    let left = first.render(dark).unwrap();
    let _ = second.render(light).unwrap();
    assert_eq!(child.read(|view| view.renders), 2);
    assert_eq!(child.resources().resource_count(), 2);
    first.layout_changed(&Default::default()).unwrap();
    second.layout_changed(&Default::default()).unwrap();
    assert_eq!(
        child.read(|view| view.layouts.clone()),
        [ColorScheme::Dark, ColorScheme::Light]
    );
    parent.update(|_, cx| cx.notify());
    assert!(first.render(dark).unwrap().ptr_eq(&left));
    assert_eq!(child.read(|view| view.renders), 2);
    let mut tree = UiTree::new(left);
    let events = tree.event_deliveries(
        tree.node_ids()[0],
        UiEventKind::Click(ClickEvent::accessibility()),
    );
    for event in &events {
        second.dispatch_event(event).unwrap();
    }
    assert_eq!(child.read(|view| view.clicks), 0);
    for event in &events {
        first.dispatch_event(event).unwrap();
    }
    assert_eq!(child.read(|view| (view.clicks, view.dark_clicks)), (1, 1));
    parent.update(|parent, cx| {
        parent.visible = false;
        cx.notify();
    });
    let _ = first.render(dark).unwrap();
    assert_eq!(child.resources().resource_count(), 1);
    drop(second);
    assert_eq!(child.resources().resource_count(), 0);
    assert!(!child.resources().is_closed());
    assert_eq!(child.read(|view| view.clicks), 1);
}

#[test]
fn tracked_dependencies_only_rebuild_the_mount_that_reads_them() {
    struct Conditional {
        dark: Entity<usize>,
        light: Entity<usize>,
        renders: [usize; 2],
    }
    impl Render for Conditional {
        fn render(&mut self, cx: &mut Context<Self>) -> Element {
            let dark = cx.environment().color_scheme == ColorScheme::Dark;
            self.renders[usize::from(dark)] += 1;
            let source = if dark { &self.dark } else { &self.light };
            Element::text(cx.read(source, |value| value.to_string()))
        }
    }
    let runtime = argui_runtime::ModelRuntime::default();
    let dark_source = runtime.entity(0);
    let light_source = runtime.entity(0);
    let model = runtime.entity(Conditional {
        dark: dark_source.clone(),
        light: light_source.clone(),
        renders: [0, 0],
    });
    let dark = model.mount().unwrap();
    let light = model.mount().unwrap();
    let dark_env = WindowEnvironment {
        color_scheme: ColorScheme::Dark,
        ..Default::default()
    };
    let light_env = WindowEnvironment::default();
    let _ = dark.render(dark_env).unwrap();
    let original = light.render(light_env).unwrap();
    dark_source.update(|value, cx| {
        *value += 1;
        cx.notify();
    });
    let _ = dark.render(dark_env).unwrap();
    assert!(light.render(light_env).unwrap().ptr_eq(&original));
    assert_eq!(model.read(|model| model.renders), [1, 2]);
    assert_eq!(model.revision(), 0);
    drop(dark);
    assert_eq!(dark_source.resources().resource_count(), 0);
    light_source.update(|value, cx| {
        *value += 1;
        cx.notify();
    });
    let _ = light.render(light_env).unwrap();
    assert_eq!(model.read(|model| model.renders), [2, 2]);
}

#[test]
#[cfg(all(feature = "tasks", not(target_arch = "wasm32")))]
fn dropping_a_mount_cancels_its_task_but_not_shared_model_work() {
    let runtime = argui_runtime::tasks::TaskRuntime::new(|| {});
    let model = Entity::new(View::default());
    model.set_task_runtime(runtime.clone());
    let first = model.mount().unwrap();
    let second = model.mount().unwrap();
    let mut handles = None;
    model.update(|_, cx| {
        let local = cx
            .spawn_in(
                first.resources(),
                std::future::pending::<()>(),
                |_, _, _| {
                    panic!("closed mount completion");
                },
            )
            .unwrap();
        let shared = cx
            .spawn(std::future::pending::<()>(), |_, _, _| {
                panic!("destroyed model completion");
            })
            .unwrap();
        handles = Some((local, shared));
    });
    let (local, shared) = handles.unwrap();
    drop(first);
    assert!(local.cancellation_token().is_cancelled());
    assert!(!shared.cancellation_token().is_cancelled());
    drop(model);
    assert!(!shared.cancellation_token().is_cancelled());
    drop(second);
    assert!(shared.cancellation_token().is_cancelled());
    runtime.shutdown();
}

#[test]
fn closing_a_retained_parent_releases_child_registrations_and_rejects_events() {
    let child = Entity::new(View::default());
    let parent = Entity::new(Parent {
        child: child.clone(),
        visible: true,
    });
    let mount = parent.mount().unwrap();
    let mut tree = UiTree::new(mount.render(Default::default()).unwrap());
    let events = tree.event_deliveries(
        tree.node_ids()[0],
        UiEventKind::Click(ClickEvent::accessibility()),
    );
    assert_eq!(child.resources().resource_count(), 1);
    mount.close();
    mount.close();
    assert_eq!(child.resources().resource_count(), 0);
    assert_eq!(parent.resources().resource_count(), 0);
    assert_eq!(mount.resources().resource_count(), 0);
    for event in &events {
        assert_eq!(mount.dispatch_event(event), Err(ScopeClosed));
    }
    assert!(!child.resources().is_closed());
    assert!(mount.render(Default::default()).is_err());
}

#[test]
fn a_child_handler_can_close_its_parent_without_borrow_conflicts() {
    let child = Entity::new(View::default());
    let parent = Entity::new(Parent {
        child: child.clone(),
        visible: true,
    });
    let mount = parent.mount().unwrap();
    let weak = mount.downgrade();
    child.update(|view, _| {
        view.on_click = Some(std::rc::Rc::new(move || {
            weak.upgrade().unwrap().close();
        }))
    });
    let mut tree = UiTree::new(mount.render(Default::default()).unwrap());
    let events = tree.event_deliveries(
        tree.node_ids()[0],
        UiEventKind::Click(ClickEvent::accessibility()),
    );
    let event = events
        .iter()
        .find(|event| event.current_handler().is_some())
        .unwrap();
    mount.dispatch_event(event).unwrap();
    assert!(mount.resources().is_closed());
    assert_eq!(child.read(|view| view.clicks), 1);
    assert_eq!(child.resources().resource_count(), 0);
}

#[test]
fn a_child_frame_can_close_its_parent_and_prevent_remaining_frame_callbacks() {
    use std::{cell::Cell, rc::Rc};
    struct Child(Rc<dyn Fn()>);
    impl Render for Child {
        fn render(&mut self, _: &mut Context<Self>) -> Element {
            Element::text("child")
        }
        fn wants_animation_frame(&self) -> bool {
            true
        }
        fn animation_frame(&mut self, _: argui_animation::Frame, _: &mut Context<Self>) {
            (self.0)();
        }
    }
    struct ParentFrame(Entity<Child>, Entity<Child>);
    impl Render for ParentFrame {
        fn render(&mut self, cx: &mut Context<Self>) -> Element {
            Element::row([cx.entity(&self.0), cx.entity(&self.1)])
        }
        fn animation_frame(&mut self, _: argui_animation::Frame, _: &mut Context<Self>) {
            panic!("closed parent received a frame");
        }
    }
    let close: Rc<Cell<Option<argui_runtime::WeakMount<ParentFrame>>>> = Rc::new(Cell::new(None));
    let target = close.clone();
    let child = Entity::new(Child(Rc::new(move || {
        target.take().unwrap().upgrade().unwrap().close();
    })));
    let sibling = Entity::new(Child(Rc::new(|| panic!("closed sibling received a frame"))));
    let parent = Entity::new(ParentFrame(child.clone(), sibling.clone()));
    let mount = parent.mount().unwrap();
    close.set(Some(mount.downgrade()));
    mount.render(Default::default()).unwrap();
    mount
        .animation_frame(argui_animation::Frame {
            now: argui_animation::Time::ZERO,
            elapsed: argui_animation::Duration::ZERO,
        })
        .unwrap();
    assert!(mount.resources().is_closed());
    assert_eq!(child.resources().resource_count(), 0);
    assert_eq!(sibling.resources().resource_count(), 0);
    assert_eq!(parent.resources().resource_count(), 0);
}

#[test]
fn closing_during_render_does_not_retain_new_handlers_or_publish_a_subtree() {
    use std::rc::Rc;
    struct Interrupted {
        close: Option<Box<dyn Fn()>>,
        marker: Rc<()>,
    }
    impl Render for Interrupted {
        fn render(&mut self, cx: &mut Context<Self>) -> Element {
            let marker = self.marker.clone();
            let element = Element::text("must not be published").on(cx.listener(
                EventType::Click,
                move |_, _, _| {
                    let _retained = &marker;
                },
            ));
            self.close.take().unwrap()();
            element
        }
    }
    let marker = Rc::new(());
    let model = Entity::new(Interrupted {
        close: None,
        marker: marker.clone(),
    });
    let mount = model.mount().unwrap();
    let weak = mount.downgrade();
    model.update(|model, _| model.close = Some(Box::new(move || weak.upgrade().unwrap().close())));
    let element = mount.render(Default::default()).unwrap();
    assert!(element.event_listeners.is_empty());
    assert!(element.children.is_empty());
    assert!(matches!(element.kind, argui_ui::ElementKind::Container));
    assert!(mount.resources().is_closed());
    assert_eq!(Rc::strong_count(&marker), 2);
    assert_eq!(model.resources().resource_count(), 0);
    assert!(mount.render(Default::default()).is_err());
}
