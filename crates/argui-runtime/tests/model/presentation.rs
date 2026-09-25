use argui_runtime::{Context, Entity, Render};
use argui_ui::{ClickEvent, Element, EventType, UiEventKind, UiTree};

struct View(usize);

#[test]
fn deliveries_from_removed_handlers_do_not_invoke_their_replacements() {
    struct Conditional {
        listening: bool,
        clicks: usize,
    }
    impl Render for Conditional {
        fn render(&mut self, cx: &mut Context<Self>) -> Element {
            let element = Element::text("click");
            if self.listening {
                element.on(cx.listener(EventType::Click, |model, _, _| model.clicks += 1))
            } else {
                element
            }
        }
    }
    let model = Entity::new(Conditional {
        listening: true,
        clicks: 0,
    });
    let mut tree = UiTree::new(model.render());
    let old = tree.event_deliveries(
        tree.node_ids()[0],
        UiEventKind::Click(ClickEvent::accessibility()),
    );
    let delivery = old
        .iter()
        .find(|event| event.current_handler().is_some())
        .unwrap();
    model.update(|model, cx| {
        model.listening = false;
        cx.notify();
    });
    let _ = model.render();
    model.dispatch_event(delivery);
    assert_eq!(model.read(|model| model.clicks), 0);
    model.update(|model, cx| {
        model.listening = true;
        cx.notify();
    });
    let mut tree = UiTree::new(model.render());
    model.dispatch_event(delivery);
    assert_eq!(model.read(|model| model.clicks), 0);
    let current = tree.event_deliveries(
        tree.node_ids()[0],
        UiEventKind::Click(ClickEvent::accessibility()),
    );
    model.dispatch_event(
        current
            .iter()
            .find(|event| event.current_handler().is_some())
            .unwrap(),
    );
    assert_eq!(model.read(|model| model.clicks), 1);
}
impl Render for View {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        Element::text("click").on(cx.listener(EventType::Click, |model, _, cx| {
            model.0 += 1;
            cx.notify();
        }))
    }
}

#[test]
fn handler_identity_is_presentation_owned_stable_and_not_reused() {
    let first = Entity::new(View(0));
    let mut tree = UiTree::new(first.render());
    let deliveries = tree.event_deliveries(
        tree.node_ids()[0],
        UiEventKind::Click(ClickEvent::accessibility()),
    );
    let delivery = deliveries
        .iter()
        .find(|event| event.current_handler().is_some())
        .unwrap();
    let owner = delivery.current_handler().unwrap().owner();
    assert_ne!(owner.0, first.id().get());
    first.dispatch_event(delivery);
    assert_eq!(first.read(|view| view.0), 1);
    tree.replace(first.render());
    let next = tree.event_deliveries(
        tree.node_ids()[0],
        UiEventKind::Click(ClickEvent::accessibility()),
    );
    assert!(
        next.iter()
            .filter_map(|event| event.current_handler())
            .all(|handler| handler.owner() == owner)
    );
    drop(first);
    let replacement = Entity::new(View(0));
    let _ = replacement.render();
    replacement.dispatch_event(delivery);
    assert_eq!(replacement.read(|view| view.0), 0);
}

mod theme {
    use std::{cell::Cell, rc::Rc, sync::Arc};

    use argui_core::Color;
    use argui_runtime::{
        Context, Entity, Render, ThemeRuntime, ThemeSchema, ThemeTokenDefinition, ThemeTokenId,
        ThemeValue, WindowEnvironment,
    };
    use argui_ui::Element;

    struct ThemeLeaf {
        token: ThemeTokenId,
        renders: Rc<Cell<usize>>,
    }

    impl Render for ThemeLeaf {
        /// Renders one token and increments `renders` for cache-invalidation assertions.
        fn render(&mut self, context: &mut Context<Self>) -> Element {
            self.renders.set(self.renders.get() + 1);
            Element::text(format!("{:?}", context.theme_value(self.token)))
        }
    }

    struct ThemeParent {
        child: Entity<ThemeLeaf>,
        renders: Rc<Cell<usize>>,
    }

    impl Render for ThemeParent {
        /// Renders `child` while recording whether the parent cache was rebuilt.
        fn render(&mut self, context: &mut Context<Self>) -> Element {
            self.renders.set(self.renders.get() + 1);
            context.entity(&self.child)
        }
    }

    /// Changes to unread tokens preserve both child and ancestor retained caches.
    #[test]
    fn token_reads_invalidate_only_the_presentations_using_changed_values() {
        let schema = Arc::new(
            ThemeSchema::new([
                ThemeTokenDefinition::new("foreground", ThemeValue::Color(Color::WHITE)),
                ThemeTokenDefinition::new("padding", ThemeValue::Length(8.0)),
            ])
            .unwrap(),
        );
        let foreground = schema.token("foreground").unwrap();
        let padding = schema.token("padding").unwrap();
        let theme = ThemeRuntime::new(schema);
        let child_renders = Rc::new(Cell::new(0));
        let parent_renders = Rc::new(Cell::new(0));
        let child = Entity::new(ThemeLeaf {
            token: foreground,
            renders: Rc::clone(&child_renders),
        });
        let parent = Entity::new(ThemeParent {
            child,
            renders: Rc::clone(&parent_renders),
        });
        let environment = || WindowEnvironment {
            theme: Some(theme.snapshot()),
            ..WindowEnvironment::default()
        };
        let _ = parent.render_in(environment());
        assert_eq!((parent_renders.get(), child_renders.get()), (1, 1));

        theme
            .set_override(padding, ThemeValue::Length(12.0))
            .unwrap();
        let _ = parent.render_in(environment());
        assert_eq!((parent_renders.get(), child_renders.get()), (1, 1));

        theme
            .set_override(foreground, ThemeValue::Color(Color::BLACK))
            .unwrap();
        let _ = parent.render_in(environment());
        assert_eq!((parent_renders.get(), child_renders.get()), (2, 2));
    }
}
