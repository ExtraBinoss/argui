use argui_runtime::{Context, Entity, Render};
use argui_ui::{ActionId, ActionScope, ActionState, Element, Shortcut, UiEventKind, UiTree};

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
