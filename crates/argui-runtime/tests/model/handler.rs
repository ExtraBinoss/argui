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
