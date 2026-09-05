use argui_animation::{Duration, Frame, Time};
use argui_core::{ColorScheme, Key, KeyInput, KeyState, Modifiers, Point, ScrollDelta, Size};
use argui_layout::LayoutEngine;
use argui_paint::{Fill, Filter};
use argui_runtime::{Context, Entity, ViewUpdate};
use argui_showcase::{StateShowcase, text_engine};
use argui_text::TextStyle;
use argui_ui::{GestureEvent, GestureKind, GesturePhase, UiEvent, UiEventKind, UiTree};
fn render_view(app: &StateShowcase) -> argui_ui::Element {
    app.view(argui_runtime::WindowEnvironment::default())
}
fn node_index(root: &argui_ui::Element, key: &str) -> usize {
    fn visit(element: &argui_ui::Element, key: &str, index: &mut usize) -> Option<usize> {
        let current = *index;
        *index += 1;
        if element.key.as_deref() == Some(key) {
            return Some(current);
        }
        element
            .children
            .iter()
            .find_map(|child| visit(child, key, index))
    }
    visit(root, key, &mut 0).unwrap()
}

fn events_for(app: &StateShowcase, key: &str) -> Vec<UiEvent> {
    let root = render_view(app);
    let index = node_index(&root, key);
    let tree = UiTree::new(root);
    let node = tree.node_id_at(index).unwrap();
    let pointer = |phase| {
        UiEvent::new(
            node,
            Some(key.into()),
            UiEventKind::Pointer(argui_core::PointerEvent::mouse(
                phase,
                Point::new(10.0, 10.0),
            )),
        )
    };
    vec![
        pointer(argui_core::PointerPhase::Entered),
        pointer(argui_core::PointerPhase::Pressed),
        pointer(argui_core::PointerPhase::Released),
        UiEvent::new(
            node,
            Some(key.into()),
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        ),
    ]
}

fn keyed_event(app: &StateShowcase, key: &str, kind: UiEventKind) -> UiEvent {
    let root = render_view(app);
    let index = node_index(&root, key);
    let tree = UiTree::new(root);
    UiEvent::new(tree.node_id_at(index).unwrap(), Some(key.into()), kind)
}

fn key_input(app: &StateShowcase, key: Key, state: KeyState, repeat: bool) -> UiEvent {
    keyed_event(
        app,
        "effects-popover",
        UiEventKind::KeyInput(KeyInput {
            key,
            state,
            modifiers: Modifiers::default(),
            repeat,
            text: None,
        }),
    )
}

fn click(app: &mut StateShowcase, key: &str) -> ViewUpdate {
    let event = events_for(app, key)
        .into_iter()
        .find(|event| matches!(event.kind, UiEventKind::Click(_)))
        .unwrap();
    app.update(&event)
}

fn open_popover(app: &mut StateShowcase) {
    assert_eq!(click(app, "popover-toggle"), ViewUpdate::None);
    assert!(node_index_optional(&render_view(app), "effects-popover").is_none());
    assert_eq!(
        app.animation_frame(Frame {
            now: Time::ZERO,
            elapsed: Duration::ZERO,
        }),
        ViewUpdate::None
    );
    assert_eq!(
        app.animation_frame(Frame {
            now: Time::from_nanos(16_000_000),
            elapsed: Duration::from_millis(16),
        }),
        ViewUpdate::Rebuild
    );
    assert!(node_index_optional(&render_view(app), "effects-popover").is_some());
}
#[test]
fn shared_showcase_builds_one_tree_and_embeds_its_fonts() {
    let app = StateShowcase::default();
    let mut text = text_engine();

    assert_eq!(render_view(&app).children.len(), 1);
    assert_eq!(
        app.view(argui_runtime::WindowEnvironment {
            color_scheme: ColorScheme::Dark,
            ..argui_runtime::WindowEnvironment::default()
        })
        .children
        .len(),
        1
    );
    assert!(text.measure("Argui", &TextStyle::default(), None).width > 0.0);
}

#[test]
fn every_showcase_control_rebuilds_the_single_shared_app() {
    let mut app = StateShowcase::default();
    let entered = events_for(&app, "increment").remove(0);
    assert_eq!(app.update(&entered), ViewUpdate::None);

    for key in ["increment", "theme", "reorder", "polarity"] {
        assert_eq!(click(&mut app, key), ViewUpdate::Rebuild);
    }
    assert_eq!(render_view(&app).children.len(), 1);
}

#[test]
fn theme_editors_and_resize_are_fully_controlled_by_showcase_state() {
    let mut app = StateShowcase::default();
    for _ in 0..3 {
        assert_eq!(click(&mut app, "theme"), ViewUpdate::Rebuild);
    }
    for _ in 0..5 {
        assert_eq!(click(&mut app, "primary"), ViewUpdate::Rebuild);
    }
    let theme_event = keyed_event(
        &app,
        "theme",
        UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
    );
    let primary_event = keyed_event(
        &app,
        "primary",
        UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
    );
    let unrelated = keyed_event(
        &app,
        "increment",
        UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
    );
    let entity = Entity::new(app);
    entity.update(|showcase, cx: &mut Context<StateShowcase>| {
        for event in [&theme_event, &primary_event, &unrelated] {
            if showcase.update(event) == ViewUpdate::Rebuild {
                cx.notify();
            }
        }
    });
    let mut app = StateShowcase::default();
    for (key, value) in [
        ("message", "short"),
        ("long-message", "a longer controlled line"),
        ("notes", "multiple\ncontrolled\nlines"),
    ] {
        let event = keyed_event(&app, key, UiEventKind::TextChanged(value.into()));
        assert_eq!(app.update(&event), ViewUpdate::Rebuild);
    }
    let unknown = UiEvent::new(
        UiTree::new(render_view(&app)).node_ids()[0],
        Some("unknown-editor".into()),
        UiEventKind::TextChanged("ignored".into()),
    );
    assert_eq!(app.update(&unknown), ViewUpdate::None);

    let entity = Entity::new(app);
    let mut tree = UiTree::new(entity.render());
    let target = tree
        .node_ids()
        .iter()
        .copied()
        .find(|node| tree.key(*node) == Some("notes-resize"))
        .unwrap();
    let resize = UiEventKind::Gesture(GestureEvent {
        target,
        pointer: argui_core::PointerId::MOUSE,
        phase: GesturePhase::Changed,
        delivery: Default::default(),
        kind: GestureKind::Pan {
            position: Point::new(48.0, 32.0),
            delta: Point::new(48.0, 32.0),
            total: Point::new(48.0, 32.0),
            velocity: Point::default(),
        },
    });
    for event in tree.event_deliveries(target, resize) {
        if event.should_dispatch() {
            entity.dispatch_event(&event);
        }
    }
    assert_eq!(tree.update(entity.render()), argui_ui::TreeUpdate::Layout);
}
#[test]
fn shared_animation_activates_samples_and_returns_to_idle() {
    let mut app = StateShowcase::default();
    assert!(!app.wants_animation_frame());
    assert_eq!(click(&mut app, "animation-play"), ViewUpdate::None);
    assert!(app.wants_animation_frame());

    assert_eq!(
        app.animation_frame(Frame {
            now: Time::ZERO,
            elapsed: Duration::ZERO,
        }),
        ViewUpdate::Rebuild
    );
    assert!(app.wants_animation_frame());
    assert_eq!(
        app.animation_frame(Frame {
            now: Time::from_nanos(16_000_000),
            elapsed: Duration::from_millis(16),
        }),
        ViewUpdate::Paint,
        "keyframe sampling must not rebuild the retained showcase"
    );
    let _ = app.animation_frame(Frame {
        now: Time::from_nanos(3_000_000_000),
        elapsed: Duration::from_millis(2_984),
    });
    assert!(!app.wants_animation_frame());

    assert_eq!(click(&mut app, "animation-reverse"), ViewUpdate::None);
    assert!(app.wants_animation_frame());
    let _ = app.animation_frame(Frame {
        now: Time::from_nanos(4_000_000_000),
        elapsed: Duration::from_secs(1),
    });
    assert!(app.wants_animation_frame());
    assert_eq!(click(&mut app, "animation-pause"), ViewUpdate::None);
    let _ = app.animation_frame(Frame {
        now: Time::from_nanos(4_100_000_000),
        elapsed: Duration::from_millis(100),
    });
    assert!(!app.wants_animation_frame());
}

#[test]
fn property_motion_is_retained_outside_the_app_model() {
    let mut app = StateShowcase::default();
    let mut tree = UiTree::new(render_view(&app));
    assert_eq!(click(&mut app, "motion"), ViewUpdate::Rebuild);
    assert_eq!(tree.update(render_view(&app)), argui_ui::TreeUpdate::Paint);
    assert!(tree.wants_animation_frame());
    tree.advance_animations(Time::ZERO);
    tree.advance_animations(Time::from_nanos(500_000_000));
    assert!(!tree.wants_animation_frame());
}

#[test]
fn spring_retarget_and_bounded_inertia_return_the_scheduler_to_idle() {
    let mut app = StateShowcase::default();
    assert_eq!(click(&mut app, "physics-spring"), ViewUpdate::None);
    assert!(app.wants_animation_frame());
    let mut now = 0_u64;
    assert_eq!(
        app.animation_frame(Frame {
            now: Time::ZERO,
            elapsed: Duration::ZERO,
        }),
        ViewUpdate::Rebuild
    );
    now += 16_000_000;
    assert_eq!(
        app.animation_frame(Frame {
            now: Time::from_nanos(now),
            elapsed: Duration::from_millis(16),
        }),
        ViewUpdate::Paint,
        "physics sampling must only update retained paint properties"
    );
    for step in 0..1_000 {
        now += 16_000_000;
        if step == 4 {
            assert_eq!(click(&mut app, "physics-spring"), ViewUpdate::None);
        }
        let _ = app.animation_frame(Frame {
            now: Time::from_nanos(now),
            elapsed: Duration::from_millis(16),
        });
        if !app.wants_animation_frame() {
            break;
        }
    }
    assert!(!app.wants_animation_frame());

    assert_eq!(click(&mut app, "physics-inertia"), ViewUpdate::None);
    for _ in 0..1_000 {
        now += 16_000_000;
        let _ = app.animation_frame(Frame {
            now: Time::from_nanos(now),
            elapsed: Duration::from_millis(16),
        });
        if !app.wants_animation_frame() {
            break;
        }
    }
    assert!(!app.wants_animation_frame());
}

#[test]
fn held_inertia_is_powered_until_release_then_decays() {
    let mut app = StateShowcase::default();
    let events = events_for(&app, "physics-inertia");
    let pressed = events
        .iter()
        .find(|event| {
            matches!(
                event.kind,
                UiEventKind::Pointer(argui_core::PointerEvent {
                    phase: argui_core::PointerPhase::Pressed,
                    ..
                })
            )
        })
        .unwrap();
    let released = events
        .iter()
        .find(|event| {
            matches!(
                event.kind,
                UiEventKind::Pointer(argui_core::PointerEvent {
                    phase: argui_core::PointerPhase::Released,
                    ..
                })
            )
        })
        .unwrap();

    assert_eq!(app.update(pressed), ViewUpdate::Rebuild);
    assert!(app.wants_animation_frame());
    for step in 1..=90 {
        let _ = app.animation_frame(Frame {
            now: Time::from_nanos(step * 16_000_000),
            elapsed: Duration::from_millis(16),
        });
    }
    assert!(
        app.wants_animation_frame(),
        "holding keeps powering inertia"
    );

    assert_eq!(app.update(released), ViewUpdate::Rebuild);
    assert!(
        app.wants_animation_frame(),
        "release preserves the outgoing velocity"
    );
    for step in 91..=1_090 {
        let _ = app.animation_frame(Frame {
            now: Time::from_nanos(step * 16_000_000),
            elapsed: Duration::from_millis(16),
        });
        if !app.wants_animation_frame() {
            break;
        }
    }
    assert!(!app.wants_animation_frame());
}

#[test]
fn a_stalled_frame_cannot_teleport_the_physics_visual() {
    let mut app = StateShowcase::default();
    assert_eq!(click(&mut app, "physics-spring"), ViewUpdate::None);
    assert_eq!(
        app.animation_frame(Frame {
            now: Time::ZERO,
            elapsed: Duration::ZERO,
        }),
        ViewUpdate::Rebuild
    );
    assert_eq!(
        app.animation_frame(Frame {
            now: Time::from_nanos(500_000_000),
            elapsed: Duration::from_millis(500),
        }),
        ViewUpdate::Paint
    );

    let view = render_view(&app);
    let index = node_index(&view, "physics-visual");
    let tree = UiTree::new(view.clone());
    let element = element_by_key(&view, "physics-visual").unwrap();
    let quad = tree.resolved_quad(tree.node_id_at(index).unwrap(), element);
    let Some(Fill::Solid(color)) = quad.background else {
        panic!("the physics visual uses a solid animated fill");
    };
    assert!(
        color.to_linear_rgba()[0] < 0.4,
        "a long presentation stall must advance physics by one bounded visual step"
    );
}

#[test]
fn effects_popover_is_composed_and_only_animates_while_open() {
    let mut app = StateShowcase::default();
    assert!(node_index_optional(&render_view(&app), "effects-popover").is_none());
    let mut retained = UiTree::new(render_view(&app));
    open_popover(&mut app);
    assert_eq!(
        retained.update(render_view(&app)),
        argui_ui::TreeUpdate::Layout
    );
    assert!(app.wants_animation_frame());
    assert!(node_index(&render_view(&app), "effects-popover") > 0);
    let view = render_view(&app);
    let popover = element_by_key(&view, "effects-popover").unwrap();
    let layer = popover.layer.as_ref().unwrap();
    let [Filter::Blur(entry_blur)] = layer.filters.as_slice() else {
        panic!("the popover uses exactly one animated entry blur");
    };
    assert!((0.0..5.0).contains(entry_blur));
    assert_eq!(layer.backdrop_filters, [Filter::Blur(14.0)]);
    assert_eq!(layer.opacity, 1.0);
    assert_eq!(layer.shadows.len(), 1);
    assert_eq!(layer.shadows[0].color.to_linear_rgba()[3], 0.28);
    assert_eq!(popover.bindings.len(), 1);
    for input in [
        key_input(&app, Key::Enter, KeyState::Pressed, false),
        key_input(&app, Key::Escape, KeyState::Released, false),
        key_input(&app, Key::Escape, KeyState::Pressed, true),
    ] {
        assert_eq!(app.update(&input), ViewUpdate::None);
    }

    assert_eq!(
        app.animation_frame(Frame {
            now: Time::from_nanos(700_000_000),
            elapsed: Duration::from_millis(684),
        }),
        ViewUpdate::Rebuild
    );
    assert_eq!(click(&mut app, "popover-close"), ViewUpdate::None);
    assert!(app.wants_animation_frame());
    let mut now = 700_000_000;
    for _ in 0..120 {
        now += 16_000_000;
        let _ = app.animation_frame(Frame {
            now: Time::from_nanos(now),
            elapsed: Duration::from_millis(16),
        });
        if !app.wants_animation_frame() {
            break;
        }
    }
    assert!(!app.wants_animation_frame());
    assert!(
        node_index_optional(&render_view(&app), "effects-popover").is_none(),
        "a fully closed popover must leave the painted and interactive scene"
    );
    open_popover(&mut app);
    let escape = key_input(&app, Key::Escape, KeyState::Pressed, false);
    assert_eq!(app.update(&escape), ViewUpdate::None);
    assert!(app.wants_animation_frame());
}

#[test]
fn effects_popover_scroll_repaints_a_valid_clipped_scene() {
    let mut app = StateShowcase::default();
    open_popover(&mut app);
    let _ = app.animation_frame(Frame {
        now: Time::from_nanos(700_000_000),
        elapsed: Duration::from_millis(684),
    });

    let mut tree = UiTree::new(render_view(&app));
    let mut layout = LayoutEngine::new();
    let viewport = Size::new(1_100.0, 700.0);
    let mut output = layout
        .compute(&mut tree, &mut text_engine(), viewport)
        .unwrap();
    let popover = output
        .scroll_regions
        .iter()
        .find(|region| tree.key(region.node) == Some("effects-popover"))
        .cloned()
        .expect("the constrained popover is scrollable");
    let point = Point::new(
        popover.bounds.origin.x + 20.0,
        popover.bounds.origin.y + 40.0,
    );
    let update = tree.scroll(
        point,
        ScrollDelta::Pixels(Point::new(0.0, -80.0)),
        &output.scroll_regions,
    );
    assert!(update.scroll_changed);
    layout.apply_scroll(&tree, &mut output).unwrap();
    output.display_list.validate().unwrap();
    assert!(
        output
            .text
            .blocks()
            .iter()
            .any(|block| block.clip.size.width > 0.0 && block.clip.size.height > 0.0)
    );
}

#[test]
fn delayed_tooltip_appears_only_after_hover_delay() {
    let mut app = StateShowcase::default();
    let idle_leave = keyed_event(
        &app,
        "tooltip-anchor",
        UiEventKind::Pointer(argui_core::PointerEvent::mouse(
            argui_core::PointerPhase::Left,
            Point::default(),
        )),
    );
    assert_eq!(app.update(&idle_leave), ViewUpdate::None);
    let entered = events_for(&app, "tooltip-anchor")
        .into_iter()
        .find(|event| {
            matches!(
                event.kind,
                UiEventKind::Pointer(argui_core::PointerEvent {
                    phase: argui_core::PointerPhase::Entered,
                    ..
                })
            )
        })
        .unwrap();
    assert_eq!(app.update(&entered), ViewUpdate::None);
    assert!(app.wants_animation_frame());
    assert!(node_index_optional(&render_view(&app), "delayed-tooltip").is_none());

    assert_eq!(
        app.animation_frame(Frame {
            now: Time::from_nanos(500_000_000),
            elapsed: Duration::from_millis(500),
        }),
        ViewUpdate::Rebuild
    );
    assert!(node_index_optional(&render_view(&app), "delayed-tooltip").is_some());
    let left = UiEvent::new(
        entered.target,
        entered.target_key().map(str::to_owned),
        UiEventKind::Pointer(argui_core::PointerEvent::mouse(
            argui_core::PointerPhase::Left,
            Point::default(),
        )),
    );
    assert_eq!(app.update(&left), ViewUpdate::Rebuild);
    assert!(node_index_optional(&render_view(&app), "delayed-tooltip").is_none());
}

fn node_index_optional(root: &argui_ui::Element, key: &str) -> Option<usize> {
    fn visit(element: &argui_ui::Element, key: &str, index: &mut usize) -> Option<usize> {
        let current = *index;
        *index += 1;
        if element.key.as_deref() == Some(key) {
            return Some(current);
        }
        element
            .children
            .iter()
            .find_map(|child| visit(child, key, index))
    }
    visit(root, key, &mut 0)
}
fn element_by_key<'a>(element: &'a argui_ui::Element, key: &str) -> Option<&'a argui_ui::Element> {
    if element.key.as_deref() == Some(key) {
        return Some(element);
    }
    element
        .children
        .iter()
        .find_map(|child| element_by_key(child, key))
}
