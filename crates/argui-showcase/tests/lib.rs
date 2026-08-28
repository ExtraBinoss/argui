use argui_animation::{Duration, Frame, Time};
use argui_core::{Affine2D, Point, Rect, ScrollDelta, Size};
use argui_effects::{ANIMATED_GRADIENT_ID, LIQUID_GLASS_ID, WORLEY_BORDER_FIRE_ID};
use argui_layout::LayoutEngine;
use argui_paint::{ClipChain, ClipRegion};
use argui_runtime::{LayoutSnapshot, ViewUpdate};
use argui_showcase::{StateShowcase, text_engine};
use argui_text::TextStyle;
use argui_ui::{CursorIcon, HitRegion, ScrollConfig, ScrollRegion, UiEvent, UiEventKind, UiTree};

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
    let root = app.view();
    let index = node_index(&root, key);
    let mut tree = UiTree::new(root);
    let node = tree.node_id_at(index).unwrap();
    let bounds = Rect::new(Point::default(), Size::new(100.0, 40.0));
    let regions = [HitRegion {
        node,
        bounds,
        transform: Affine2D::IDENTITY,
        clips: ClipChain::from_regions([ClipRegion::new(bounds, Affine2D::IDENTITY)]),
        focusable: true,
        cursor: CursorIcon::Auto,
    }];
    let mut events = tree.pointer_moved(Point::new(10.0, 10.0), &regions).events;
    events.extend(tree.primary_pressed(&regions).events);
    events.extend(tree.primary_released().events);
    events
}

fn click(app: &mut StateShowcase, key: &str) -> ViewUpdate {
    let event = events_for(app, key)
        .into_iter()
        .find(|event| event.kind == UiEventKind::Clicked)
        .unwrap();
    app.update(&event)
}

#[test]
fn shared_showcase_builds_one_tree_and_embeds_its_fonts() {
    let app = StateShowcase::default();
    let mut text = text_engine();

    assert_eq!(app.view().children.len(), 1);
    assert!(text.measure("Argui", &TextStyle::default(), None).width > 0.0);
    let shader_ids: Vec<_> = app
        .effect_shaders()
        .iter()
        .map(|shader| shader.id)
        .collect();
    assert_eq!(
        shader_ids,
        [WORLEY_BORDER_FIRE_ID, LIQUID_GLASS_ID, ANIMATED_GRADIENT_ID]
    );
}

#[test]
fn every_showcase_control_rebuilds_the_single_shared_app() {
    let mut app = StateShowcase::default();
    let entered = events_for(&app, "increment").remove(0);
    assert_eq!(app.update(&entered), ViewUpdate::None);

    for key in ["increment", "theme", "reorder", "polarity"] {
        assert_eq!(click(&mut app, key), ViewUpdate::Rebuild);
    }
    assert_eq!(app.view().children.len(), 1);
}

#[test]
fn virtual_scroll_rebuilds_only_when_the_visible_window_changes() {
    let mut app = StateShowcase::default();
    let root = app.view();
    let index = node_index(&root, "million-list");
    let mut tree = UiTree::new(root);
    let node = tree.node_id_at(index).unwrap();
    let bounds = Rect::new(Point::default(), Size::new(300.0, 260.0));
    let regions = [ScrollRegion {
        node,
        bounds,
        clip: bounds,
        transform: Affine2D::IDENTITY,
        clips: ClipChain::from_regions([ClipRegion::new(bounds, Affine2D::IDENTITY)]),
        max_offset: Point::new(0.0, 1_000.0),
        config: ScrollConfig::default().line_size(36.0),
        scrollbar: None,
    }];

    for step in 1..=9 {
        let update = tree.scroll(
            Point::new(10.0, 10.0),
            ScrollDelta::Lines(Point::new(0.0, -1.0)),
            &regions,
        );
        let expected = if step == 9 {
            ViewUpdate::Rebuild
        } else {
            ViewUpdate::None
        };
        assert_eq!(app.update(&update.events[0]), expected);
    }
    let sub_row = tree.scroll(
        Point::new(10.0, 10.0),
        ScrollDelta::Pixels(Point::new(0.0, -1.0)),
        &regions,
    );
    assert_eq!(app.update(&sub_row.events[0]), ViewUpdate::None);
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
    let _ = app.animation_frame(Frame {
        now: Time::from_nanos(3_000_000_000),
        elapsed: Duration::from_secs(3),
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
fn implicit_paint_transition_is_retained_outside_the_app_model() {
    let mut app = StateShowcase::default();
    let mut tree = UiTree::new(app.view());
    assert_eq!(click(&mut app, "transition"), ViewUpdate::Rebuild);
    assert_eq!(tree.update(app.view()), argui_ui::TreeUpdate::Paint);
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
    let _ = app.animation_frame(Frame {
        now: Time::ZERO,
        elapsed: Duration::ZERO,
    });
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
fn effects_popover_is_composed_and_only_animates_while_open() {
    let mut app = StateShowcase::default();
    let mut retained = UiTree::new(app.view());
    assert_eq!(click(&mut app, "popover-toggle"), ViewUpdate::Rebuild);
    assert_eq!(retained.update(app.view()), argui_ui::TreeUpdate::None);
    assert!(app.wants_animation_frame());
    assert!(node_index(&app.view(), "effects-popover") > 0);

    assert_eq!(
        app.animation_frame(Frame {
            now: Time::ZERO,
            elapsed: Duration::ZERO,
        }),
        ViewUpdate::None
    );
    assert_eq!(
        app.animation_frame(Frame {
            now: Time::from_nanos(700_000_000),
            elapsed: Duration::from_millis(700),
        }),
        ViewUpdate::Rebuild
    );
    assert_eq!(click(&mut app, "popover-close"), ViewUpdate::Rebuild);
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
        node_index_optional(&app.view(), "effects-popover").is_some(),
        "the dormant popover stays retained to avoid layout work on its next opening"
    );
}

#[test]
fn effects_popover_scroll_repaints_a_valid_clipped_scene() {
    let mut app = StateShowcase::default();
    assert_eq!(click(&mut app, "popover-toggle"), ViewUpdate::Rebuild);
    let _ = app.animation_frame(Frame {
        now: Time::from_nanos(700_000_000),
        elapsed: Duration::from_millis(700),
    });

    let mut tree = UiTree::new(app.view());
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
    let entered = events_for(&app, "tooltip-anchor")
        .into_iter()
        .find(|event| event.kind == UiEventKind::PointerEntered)
        .unwrap();
    assert_eq!(app.update(&entered), ViewUpdate::None);
    assert!(app.wants_animation_frame());
    assert!(node_index_optional(&app.view(), "delayed-tooltip").is_none());

    assert_eq!(
        app.animation_frame(Frame {
            now: Time::from_nanos(500_000_000),
            elapsed: Duration::from_millis(500),
        }),
        ViewUpdate::Rebuild
    );
    assert!(node_index_optional(&app.view(), "delayed-tooltip").is_some());
    let left = UiEvent {
        kind: UiEventKind::PointerLeft,
        ..entered
    };
    assert_eq!(app.update(&left), ViewUpdate::Rebuild);
    assert!(node_index_optional(&app.view(), "delayed-tooltip").is_none());
}

#[test]
fn overlay_layout_is_resolved_by_the_layout_engine_without_a_model_rebuild() {
    let mut app = StateShowcase::default();
    assert_eq!(
        app.layout_changed(&LayoutSnapshot::default()),
        ViewUpdate::None
    );
    assert_eq!(click(&mut app, "popover-toggle"), ViewUpdate::Rebuild);
    let view = app.view();
    let popover = element_by_key(&view, "effects-popover").unwrap();
    assert!(popover.interaction.is_some());
    assert!(popover.scroll.is_some());
    assert!(matches!(popover.style.height, argui_ui::Length::Px(430.0)));

    let mut tree = UiTree::new(view);
    let mut layout = LayoutEngine::new();
    let output = layout
        .compute(&mut tree, &mut text_engine(), Size::new(500.0, 400.0))
        .unwrap();
    let bounds = |key: &str| {
        output
            .nodes
            .iter()
            .find(|node| tree.key(node.node) == Some(key))
            .map(|node| node.bounds)
            .unwrap()
    };
    let anchor = bounds("popover-toggle");
    let placed = bounds("effects-popover");
    assert!(placed.origin.y + placed.size.height <= output.viewport.size.height - 14.0);
    assert!(placed.origin.y + placed.size.height <= anchor.origin.y - 12.0);
    assert_eq!(
        app.layout_changed(&LayoutSnapshot::default()),
        ViewUpdate::None
    );
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
