use std::{cell::Cell, rc::Rc};

use argui_ui::{
    Element, ElementKind, FocusRequest, FocusTarget, TextSelection, TextSelectionRequest, UiEvent,
    UiEventKind, UiTree,
};

use argui_animation::{Duration, Frame, Time};
use argui_core::{Color, ColorScheme, Point, PointerId, Rect, Size};
use argui_ui::ClipboardRequest;

use crate::{
    AppCommand, Context, Entity, LayoutBounds, LayoutSnapshot, Render, ThemeRequest,
    WindowEnvironment,
};

struct Counted {
    renders: Rc<Cell<u32>>,
}

impl Render for Counted {
    fn render(&mut self, _cx: &mut Context<Self>) -> Element {
        self.renders.set(self.renders.get() + 1);
        Element::text("retained")
    }
}

#[test]
fn clean_entity_returns_the_exact_cached_subtree() {
    let renders = Rc::new(Cell::new(0));
    let entity = Entity::new(Counted {
        renders: renders.clone(),
    });
    let first = entity.render();
    let second = entity.render();
    assert!(first.ptr_eq(&second));
    assert_eq!(renders.get(), 1);
}

#[test]
fn notify_rebuilds_only_on_the_next_render() {
    let renders = Rc::new(Cell::new(0));
    let entity = Entity::new(Counted {
        renders: renders.clone(),
    });
    let first = entity.render();
    entity.update(|_, cx| cx.notify());
    assert_eq!(renders.get(), 1);
    let second = entity.render();
    assert!(!first.ptr_eq(&second));
    assert_eq!(renders.get(), 2);
}

struct Child {
    events: Rc<Cell<u32>>,
    stop: bool,
}

impl Render for Child {
    fn render(&mut self, _cx: &mut Context<Self>) -> Element {
        Element::text("child").keyed("deep-child")
    }

    fn event(&mut self, event: &UiEvent, cx: &mut Context<Self>) {
        self.events.set(self.events.get() + 1);
        cx.notify();
        if self.stop {
            event.stop_propagation();
        }
    }
}

struct Parent {
    child: Entity<Child>,
    events: Rc<Cell<u32>>,
}

impl Render for Parent {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        Element::column([cx.entity(&self.child)])
    }

    fn event(&mut self, _event: &UiEvent, cx: &mut Context<Self>) {
        self.events.set(self.events.get() + 1);
        cx.notify();
    }
}

fn event() -> UiEvent {
    UiEvent::new(
        UiTree::new(Element::container([])).node_id_at(0).unwrap(),
        Some("deep-child".into()),
        UiEventKind::Clicked,
    )
}

fn owned_event(owner: argui_ui::EventOwnerId) -> UiEvent {
    event().with_current_owner(owner)
}

fn unkeyed_event() -> UiEvent {
    UiEvent::new(
        UiTree::new(Element::container([])).node_id_at(0).unwrap(),
        None,
        UiEventKind::Clicked,
    )
}

#[test]
fn event_owner_routes_directly_to_the_deepest_entity() {
    let child_events = Rc::new(Cell::new(0));
    let parent_events = Rc::new(Cell::new(0));
    let root = Entity::new(Parent {
        child: Entity::new(Child {
            events: child_events.clone(),
            stop: false,
        }),
        events: parent_events.clone(),
    });
    let rendered = root.render();
    let owner = rendered.children[0].event_owner.unwrap();
    root.event(&owned_event(owner));
    assert_eq!(child_events.get(), 1);
    assert_eq!(parent_events.get(), 0);
}

#[test]
fn propagation_control_lives_on_the_dispatched_event() {
    let child_events = Rc::new(Cell::new(0));
    let parent_events = Rc::new(Cell::new(0));
    let root = Entity::new(Parent {
        child: Entity::new(Child {
            events: child_events.clone(),
            stop: true,
        }),
        events: parent_events.clone(),
    });
    let rendered = root.render();
    let owner = rendered.children[0].event_owner.unwrap();
    let event = owned_event(owner);
    root.event(&event);
    assert_eq!(child_events.get(), 1);
    assert_eq!(parent_events.get(), 0);
    assert!(event.propagation_stopped());
}

#[test]
fn child_notification_invalidates_its_composed_ancestors() {
    let root = Entity::new(Parent {
        child: Entity::new(Child {
            events: Rc::new(Cell::new(0)),
            stop: false,
        }),
        events: Rc::new(Cell::new(0)),
    });
    let first = root.render();
    let child = root.read(|parent| parent.child.clone());
    child.update(|_, cx| cx.notify());
    let second = root.render();
    assert!(!first.ptr_eq(&second));
}

#[test]
fn weak_and_erased_entities_keep_explicit_identity() {
    let entity = Entity::new(Counted {
        renders: Rc::new(Cell::new(0)),
    });
    let weak = entity.downgrade();
    let erased = entity.erase();
    assert!(weak.upgrade().is_some());
    assert!(erased.ptr_eq(&entity.erase()));
    drop(entity);
    assert!(weak.upgrade().is_some());
    drop(erased);
    assert!(weak.upgrade().is_none());
}

struct Effects;

impl Render for Effects {
    fn render(&mut self, _cx: &mut Context<Self>) -> Element {
        Element::container([])
    }

    fn event(&mut self, _event: &UiEvent, cx: &mut Context<Self>) {
        cx.command(AppCommand::Quit);
        cx.write_clipboard(ClipboardRequest::Write("copied".into()));
        cx.scroll_to("target", Point::new(2.0, 4.0));
        cx.request_focus("target");
        cx.select_text("target", TextSelection::All);
        assert!(cx.capture_pointer(PointerId::MOUSE));
        assert!(cx.release_pointer(PointerId::MOUSE));
        cx.selection_command(argui_ui::SelectionCommand::Copy);
        cx.request_animation_frame();
    }
}

#[test]
fn context_bubbles_commands_clipboard_scroll_and_frame_requests() {
    let entity = Entity::new(Effects);
    entity.event(&event());
    let effects = entity.take_effects();
    assert_eq!(effects.commands, [AppCommand::Quit]);
    assert_eq!(
        effects.clipboard,
        Some(ClipboardRequest::Write("copied".into()))
    );
    assert_eq!(effects.scroll.unwrap().key, "target");
    assert_eq!(
        effects.focus,
        Some(FocusRequest::Focus(FocusTarget::Key("target".into())))
    );
    assert_eq!(
        effects.text_selection,
        Some(TextSelectionRequest::new("target", TextSelection::All))
    );
    assert!(effects.animation_frame);
    assert_eq!(effects.pointer_capture.len(), 2);
    assert_eq!(
        effects.selection_command.unwrap().command,
        argui_ui::SelectionCommand::Copy
    );
}

#[test]
fn contexts_create_entities_and_propagate_nested_effects() {
    let mut parent = Context::<Effects>::default();
    let direct_child: Context<Counted> = parent.child_context();
    assert_eq!(direct_child.environment(), parent.environment());
    let child = parent.new_entity(Counted {
        renders: Rc::new(Cell::new(0)),
    });
    assert!(matches!(
        parent.entity(&child).kind,
        ElementKind::Text { .. }
    ));

    let mut nested = Context::<Effects>::default();
    nested.notify();
    nested.request_animation_frame();
    nested.write_clipboard(ClipboardRequest::Write("nested".into()));
    nested.scroll_to("nested-target", Point::new(8.0, 13.0));
    nested.request_focus("nested-target");
    nested.select_text("nested-target", TextSelection::All);
    nested.clear_focus();
    nested.set_theme(ThemeRequest {
        color_scheme: Some(ColorScheme::Dark),
        primary: Some(Color::srgb(0.8, 0.2, 0.4)),
    });
    nested.command(AppCommand::Quit);
    let selection_target = UiTree::new(Element::container([])).node_ids()[0];
    nested.selection_command_for(selection_target, argui_ui::SelectionCommand::SelectAll);
    assert!(!nested.capture_pointer(PointerId::MOUSE));
    assert!(!nested.release_pointer(PointerId::MOUSE));
    parent.propagate(nested);
    parent.propagate(Context::<Effects>::default());

    assert_eq!(parent.effects.update, crate::ViewUpdate::Rebuild);
    assert!(parent.effects.animation_frame);
    assert_eq!(parent.effects.commands, [AppCommand::Quit]);
    assert_eq!(
        parent.effects.clipboard,
        Some(ClipboardRequest::Write("nested".into()))
    );
    assert_eq!(parent.effects.scroll.unwrap().key, "nested-target");
    assert_eq!(parent.effects.focus, Some(FocusRequest::Clear));
    assert_eq!(
        parent.effects.selection_command.unwrap().target,
        Some(selection_target)
    );
    assert_eq!(
        parent.effects.text_selection,
        Some(TextSelectionRequest::new(
            "nested-target",
            TextSelection::All,
        ))
    );
    assert_eq!(
        parent.effects.theme,
        Some(ThemeRequest {
            color_scheme: Some(ColorScheme::Dark),
            primary: Some(Color::srgb(0.8, 0.2, 0.4)),
        })
    );
}

struct ErasedSurface {
    rebuild_on_layout: bool,
    layouts: Rc<Cell<u32>>,
}

impl Render for ErasedSurface {
    fn render(&mut self, _cx: &mut Context<Self>) -> Element {
        Element::text("erased").keyed("erased")
    }

    fn layout_changed(&mut self, _layout: &LayoutSnapshot, cx: &mut Context<Self>) {
        self.layouts.set(self.layouts.get() + 1);
        if self.rebuild_on_layout {
            cx.notify();
        }
    }
}

#[test]
fn erased_entities_forward_the_complete_retained_surface() {
    let layouts = Rc::new(Cell::new(0));
    let entity = Entity::new(ErasedSurface {
        rebuild_on_layout: false,
        layouts: layouts.clone(),
    });
    let erased = entity.erase();
    let first = erased.render(WindowEnvironment::default());

    let mut unmatched = event();
    unmatched.key = Some("absent".into());
    erased.event(&unmatched);
    erased.animation_frame(Frame {
        now: Time::ZERO,
        elapsed: Duration::ZERO,
    });
    erased.layout_changed(&LayoutSnapshot::default());
    assert!(first.ptr_eq(&erased.render(WindowEnvironment::default())));

    assert!(erased.image_assets().is_empty());
    assert!(erased.vector_assets().is_empty());
    assert!(erased.inspector().is_none());
    assert_eq!(erased.take_effects().update, crate::ViewUpdate::None);

    entity.update(|state, _| state.rebuild_on_layout = true);
    erased.layout_changed(&LayoutSnapshot::default());
    assert_eq!(layouts.get(), 2);
    assert!(!first.ptr_eq(&erased.render(WindowEnvironment::default())));
    assert_eq!(erased.take_effects().update, crate::ViewUpdate::Rebuild);
}

#[test]
fn unkeyed_events_and_idle_children_take_the_direct_path() {
    let parent_events = Rc::new(Cell::new(0));
    let root = Entity::new(Parent {
        child: Entity::new(Child {
            events: Rc::new(Cell::new(0)),
            stop: false,
        }),
        events: parent_events.clone(),
    });
    let _ = root.render();
    root.event(&unkeyed_event());
    assert_eq!(parent_events.get(), 1);
    assert!(!root.wants_frame());
    root.animation_frame(Frame {
        now: Time::ZERO,
        elapsed: Duration::ZERO,
    });
}

#[test]
fn context_observes_stable_layout_bounds() {
    let context = Context::<Effects>::default();
    let bounds = Rect::new(Point::new(3.0, 5.0), Size::new(20.0, 30.0));
    let snapshot = LayoutSnapshot {
        viewport: Rect::default(),
        nodes: vec![LayoutBounds {
            node: UiTree::new(Element::container([])).node_id_at(0).unwrap(),
            key: Some("observed".into()),
            bounds,
        }],
    };
    assert_eq!(context.observe_bounds(&snapshot, "observed"), Some(bounds));
    assert_eq!(context.observe_bounds(&snapshot, "missing"), None);
}

struct Animated {
    frames: Rc<Cell<u32>>,
}

impl Render for Animated {
    fn render(&mut self, _cx: &mut Context<Self>) -> Element {
        Element::text("animated").keyed("animated")
    }

    fn animation_frame(&mut self, _frame: Frame, cx: &mut Context<Self>) {
        self.frames.set(self.frames.get() + 1);
        cx.notify();
    }

    fn wants_animation_frame(&self) -> bool {
        true
    }
}

#[test]
fn animation_frames_reach_retained_children() {
    let frames = Rc::new(Cell::new(0));
    let animated = Entity::new(Animated {
        frames: frames.clone(),
    });
    struct AnimationRoot {
        child: Entity<Animated>,
    }
    impl Render for AnimationRoot {
        fn render(&mut self, cx: &mut Context<Self>) -> Element {
            cx.entity(&self.child)
        }
    }
    let root = Entity::new(AnimationRoot { child: animated });
    let _ = root.render();
    assert!(root.wants_frame());
    root.animation_frame(Frame {
        now: Time::ZERO,
        elapsed: Duration::ZERO,
    });
    assert_eq!(frames.get(), 1);
}

#[test]
fn environment_changes_rebuild_only_reading_subtrees() {
    struct EnvironmentReader {
        renders: Rc<Cell<u32>>,
        reads: bool,
    }
    impl Render for EnvironmentReader {
        fn render(&mut self, cx: &mut Context<Self>) -> Element {
            self.renders.set(self.renders.get() + 1);
            if self.reads {
                Element::text(format!("{:?}", cx.environment().color_scheme))
            } else {
                Element::text("static")
            }
        }
    }
    let dark = WindowEnvironment {
        color_scheme: ColorScheme::Dark,
        ..WindowEnvironment::default()
    };
    for (reads, expected) in [(false, 1), (true, 2)] {
        let renders = Rc::new(Cell::new(0));
        let entity = Entity::new(EnvironmentReader {
            renders: renders.clone(),
            reads,
        });
        let _ = entity.render_in(WindowEnvironment::default());
        let _ = entity.render_in(dark);
        assert_eq!(renders.get(), expected);
    }
}
