use argui_runtime::{Context, Entity, Render};
use argui_ui::{
    ActionId, ActionScope, ActionState, ClickEvent, Element, ElementKind, EventType, Shortcut,
    UiEventKind, UiTree,
};

#[derive(Default)]
struct Commands {
    hits: usize,
    enabled: bool,
}
impl Render for Commands {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let binding = cx.on_action(
            ActionId("run"),
            ActionState::new("Run")
                .enabled(self.enabled)
                .shortcut(Shortcut::primary("j")),
            |model, _, cx| {
                model.hits += 1;
                cx.notify();
            },
        );
        Element::container([])
            .action(ActionId("run"))
            .action_scope(ActionScope::new([binding]).unwrap())
    }
}

#[test]
fn action_default_routes_once_to_the_retained_owner_and_revalidates_disabled_state() {
    let entity = Entity::new(Commands {
        enabled: true,
        ..Default::default()
    });
    let mut tree = UiTree::new(entity.render());
    for expected in [1, 2] {
        let node = tree.node_ids()[0];
        let defaults = tree.event_deliveries(
            node,
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        );
        for event in defaults {
            assert!(event.should_dispatch());
            let UiEventKind::Action(invocation) = event.kind else {
                panic!("action default");
            };
            for delivery in tree.invoke_action(invocation).events {
                if delivery.should_dispatch() {
                    entity.dispatch_event(&delivery);
                }
            }
        }
        assert_eq!(entity.read(|model| model.hits), expected);
        tree.replace(entity.render());
    }
    entity.update(|model, cx| {
        model.enabled = false;
        cx.notify();
    });
    tree.replace(entity.render());
    assert!(
        tree.invoke_action(argui_ui::ActionInvocation::new(ActionId("run")))
            .events
            .is_empty()
    );
    assert_eq!(entity.read(|model| model.hits), 2);
}

#[derive(Default)]
struct CallbackModes {
    value: usize,
}

impl Render for CallbackModes {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let automatic = cx.callback(|model| model.value += 1);
        let explicit = cx.event_handler(|model, event, _| {
            assert_eq!(event.target_key(), Some("explicit"));
            model.value += 10;
        });
        let incompatible = cx.value_callback(|model, value: usize| model.value += value);
        Element::column([
            Element::text(self.value.to_string()).keyed("value"),
            Element::text("automatic")
                .keyed("automatic")
                .on(automatic.direct_listener(EventType::Click)),
            Element::text("explicit")
                .keyed("explicit")
                .on(explicit.direct_listener(EventType::Click)),
            Element::text("incompatible")
                .keyed("incompatible")
                .on(incompatible.direct_listener(EventType::Click)),
        ])
    }
}

fn click_key(entity: &Entity<CallbackModes>, tree: &mut UiTree, key: &str) {
    let target = tree
        .node_ids()
        .iter()
        .copied()
        .find(|node| tree.key(*node) == Some(key))
        .unwrap();
    for event in tree.event_deliveries(target, UiEventKind::Click(ClickEvent::accessibility())) {
        if event.should_dispatch() {
            entity.dispatch_event(&event);
        }
    }
}

#[test]
fn callback_invalidates_while_event_handler_requires_explicit_invalidation() {
    let entity = Entity::new(CallbackModes::default());
    let first = entity.render();
    let mut tree = UiTree::new(first.clone());

    click_key(&entity, &mut tree, "explicit");
    assert_eq!(entity.read(|model| model.value), 10);
    assert!(entity.render().ptr_eq(&first));

    click_key(&entity, &mut tree, "incompatible");
    assert_eq!(entity.read(|model| model.value), 10);
    assert!(entity.render().ptr_eq(&first));

    click_key(&entity, &mut tree, "automatic");
    assert_eq!(entity.read(|model| model.value), 11);
    let rebuilt = entity.render();
    assert!(!rebuilt.ptr_eq(&first));
    let ElementKind::Text { content, .. } = &rebuilt.children[0].kind else {
        panic!("value text");
    };
    assert_eq!(content.as_str(), "11");
}

struct RemovableHandler {
    enabled: bool,
    hits: usize,
}

impl Render for RemovableHandler {
    /// Registers a callback only while this presentation exposes its action.
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let element = Element::text("action").keyed("action");
        if self.enabled {
            element.on(cx
                .callback(|model| model.hits += 1)
                .direct_listener(EventType::Click))
        } else {
            element
        }
    }
}

/// A queued delivery cannot invoke a handler removed by a later render.
#[test]
fn removed_handler_identity_ignores_stale_event_delivery() {
    let entity = Entity::new(RemovableHandler {
        enabled: true,
        hits: 0,
    });
    let mut tree = UiTree::new(entity.render());
    let event = tree
        .event_deliveries(
            tree.node_ids()[0],
            UiEventKind::Click(ClickEvent::accessibility()),
        )
        .into_iter()
        .next()
        .unwrap();
    entity.update(|model, cx| {
        model.enabled = false;
        cx.notify();
    });
    tree.replace(entity.render());
    entity.dispatch_event(&event);
    assert_eq!(entity.read(|model| model.hits), 0);
}

/// A parent mount routes a live child-owned event to the retained child model.
#[test]
fn parent_dispatches_child_owned_handler_events_to_the_child_mount() {
    struct Child(usize);
    impl Render for Child {
        /// Renders the child's click target and registers its retained handler.
        fn render(&mut self, cx: &mut Context<Self>) -> Element {
            Element::text("child").on(cx
                .callback(|model| model.0 += 1)
                .direct_listener(EventType::Click))
        }
    }
    struct Parent(Entity<Child>);
    impl Render for Parent {
        /// Renders the retained child into the parent's event-routing tree.
        fn render(&mut self, cx: &mut Context<Self>) -> Element {
            cx.entity(&self.0)
        }
    }

    let child = Entity::new(Child(0));
    let parent = Entity::new(Parent(child.clone()));
    let mount = parent.mount().unwrap();
    let mut tree = UiTree::new(mount.render(Default::default()).unwrap());
    let event = tree
        .event_deliveries(
            tree.node_ids()[0],
            UiEventKind::Click(ClickEvent::accessibility()),
        )
        .into_iter()
        .find(|event| event.should_dispatch())
        .unwrap();

    mount.dispatch_event(&event).unwrap();

    assert_eq!(child.read(|model| model.0), 1);
}

#[derive(Default)]
struct InputProbe {
    text: String,
    calls: usize,
}

impl Render for InputProbe {
    /// Binds a typed input value to this retained element.
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let input = cx.input_callback(|model, value| {
            model.text = value;
            model.calls += 1;
        });
        Element::text("input").on(input.direct_listener(EventType::Input))
    }
}

/// A compatible text delivery reaches a typed callback; a click cannot reuse it.
#[test]
fn typed_input_callback_accepts_text_and_ignores_incompatible_click() {
    let entity = Entity::new(InputProbe::default());
    let mut tree = UiTree::new(entity.render());
    let target = tree.node_ids()[0];
    for kind in [
        UiEventKind::TextChanged("edited".into()),
        UiEventKind::Click(ClickEvent::accessibility()),
    ] {
        for delivery in tree.event_deliveries(target, kind) {
            if delivery.should_dispatch() {
                entity.dispatch_event(&delivery);
            }
        }
    }
    assert_eq!(
        entity.read(|model| (model.text.clone(), model.calls)),
        ("edited".into(), 1)
    );
}
