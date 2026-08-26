use argui_animation::{Duration, Frame, Time};
use argui_core::{Point, Rect, ScrollDelta, Size};
use argui_runtime::{UiApp, ViewUpdate};
use argui_showcase::{StateShowcase, text_engine};
use argui_text::TextStyle;
use argui_ui::{HitRegion, ScrollConfig, ScrollRegion, UiEvent, UiEventKind, UiTree};

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
        clip: bounds,
        focusable: true,
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
