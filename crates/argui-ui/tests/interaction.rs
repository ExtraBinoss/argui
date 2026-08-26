use argui_core::{Point, Rect, Size};
use argui_paint::{Color, QuadStyle};
use argui_ui::{Element, HitRegion, Interaction, UiEventKind, UiTree, VisualState};

fn interactive(key: &str) -> Element {
    Element::container([])
        .keyed(key)
        .background(Color::rgb(0.0, 0.0, 0.0))
        .interaction(
            Interaction::default()
                .focusable(true)
                .hovered(QuadStyle::solid(Color::WHITE))
                .pressed(QuadStyle::solid(Color::rgb(1.0, 0.0, 0.0)))
                .focused(QuadStyle::solid(Color::rgb(0.0, 1.0, 0.0))),
        )
}

fn region(node: argui_ui::NodeId) -> HitRegion {
    region_at(node, 10.0, true)
}

fn region_at(node: argui_ui::NodeId, x: f32, focusable: bool) -> HitRegion {
    HitRegion {
        node,
        bounds: Rect::new(Point::new(x, 10.0), Size::new(80.0, 40.0)),
        clip: Rect::new(Point::new(x + 10.0, 10.0), Size::new(70.0, 40.0)),
        focusable,
    }
}

#[test]
fn pointer_state_honors_clips_capture_clicks_and_focus() {
    let mut tree = UiTree::new(interactive("save"));
    let node = tree.node_id_at(0).unwrap();
    let regions = [region(node)];

    let clipped = tree.pointer_moved(Point::new(15.0, 20.0), &regions);
    assert!(clipped.events.is_empty());
    assert_eq!(tree.visual_state(node), VisualState::Rest);

    let entered = tree.pointer_moved(Point::new(30.0, 20.0), &regions);
    assert!(entered.paint_changed);
    assert_eq!(entered.events[0].key.as_deref(), Some("save"));
    assert_eq!(entered.events[0].kind, UiEventKind::PointerEntered);
    assert_eq!(tree.visual_state(node), VisualState::Hovered);

    let pressed = tree.primary_pressed(&regions);
    assert_eq!(pressed.events[0].kind, UiEventKind::Pressed);
    assert!(
        pressed
            .events
            .iter()
            .any(|event| event.kind == UiEventKind::Focused)
    );
    assert_eq!(tree.visual_state(node), VisualState::Pressed);

    let dragged_out = tree.pointer_moved(Point::new(200.0, 200.0), &regions);
    assert!(
        dragged_out
            .events
            .iter()
            .any(|event| matches!(event.kind, UiEventKind::PointerMoved(_)))
    );
    assert_eq!(tree.visual_state(node), VisualState::Pressed);

    let released = tree.primary_released();
    assert_eq!(released.events.len(), 1);
    assert_eq!(released.events[0].kind, UiEventKind::Released);
    assert_eq!(tree.visual_state(node), VisualState::Focused);

    let blurred = tree.window_blurred();
    assert!(
        blurred
            .events
            .iter()
            .any(|event| event.kind == UiEventKind::Blurred)
    );
    assert_eq!(tree.visual_state(node), VisualState::Rest);
}

#[test]
fn keyed_nodes_keep_their_identity_when_reordered() {
    let mut tree = UiTree::new(Element::row([interactive("first"), interactive("second")]));
    let first = tree.node_id_at(1).unwrap();
    let second = tree.node_id_at(2).unwrap();

    assert!(tree.replace(Element::row([interactive("second"), interactive("first"),])));
    assert_eq!(tree.node_id_at(1), Some(second));
    assert_eq!(tree.node_id_at(2), Some(first));
}

#[test]
fn duplicate_keys_never_duplicate_node_ids() {
    let mut tree = UiTree::new(Element::row([interactive("same")]));
    let original = tree.node_id_at(1).unwrap();

    tree.replace(Element::row([interactive("same"), interactive("same")]));
    assert_eq!(tree.node_id_at(1), Some(original));
    assert_ne!(tree.node_id_at(1), tree.node_id_at(2));
}

#[test]
fn release_over_the_captured_target_emits_a_click() {
    let mut tree = UiTree::new(interactive("open"));
    let node = tree.node_id_at(0).unwrap();
    let regions = [region(node)];
    tree.pointer_moved(Point::new(30.0, 20.0), &regions);
    tree.primary_pressed(&regions);

    let released = tree.primary_released();
    assert!(
        released
            .events
            .iter()
            .any(|event| event.kind == UiEventKind::Clicked)
    );
}

#[test]
fn empty_actions_focus_switches_and_blur_are_deterministic() {
    let mut tree = UiTree::new(Element::row([interactive("first"), interactive("second")]));
    let first = tree.node_id_at(1).unwrap();
    let second = tree.node_id_at(2).unwrap();
    assert!(first.get() > 0);
    let regions = [region_at(first, 10.0, true), region_at(second, 110.0, true)];

    assert!(tree.primary_pressed(&regions).events.is_empty());
    assert!(tree.primary_released().events.is_empty());
    assert!(tree.pointer_left().events.is_empty());

    tree.pointer_moved(Point::new(30.0, 20.0), &regions);
    tree.primary_pressed(&regions);
    assert_eq!(tree.visual_state(first), VisualState::Pressed);
    assert_eq!(
        tree.resolved_quad(first, &tree.root().children[0])
            .background,
        QuadStyle::solid(Color::rgb(1.0, 0.0, 0.0)).background
    );
    tree.primary_released();
    tree.pointer_moved(Point::new(130.0, 20.0), &regions);
    let switched = tree.primary_pressed(&regions);
    assert!(
        switched
            .events
            .iter()
            .any(|event| event.target == first && event.kind == UiEventKind::Blurred)
    );

    let left = tree.pointer_left();
    assert_eq!(left.events[0].kind, UiEventKind::PointerLeft);
    let blurred = tree.window_blurred();
    assert!(
        blurred
            .events
            .iter()
            .any(|event| event.target == second && event.kind == UiEventKind::Released)
    );
}

#[test]
fn incompatible_replacements_receive_new_ids_and_clear_state() {
    let mut tree = UiTree::new(interactive("target"));
    let old = tree.node_id_at(0).unwrap();
    let regions = [region(old)];
    tree.pointer_moved(Point::new(30.0, 20.0), &regions);
    tree.primary_pressed(&regions);

    tree.replace(Element::text("now text").keyed("target"));
    let new = tree.node_id_at(0).unwrap();
    assert_ne!(old, new);
    assert_eq!(tree.visual_state(old), VisualState::Rest);
    assert!(tree.window_blurred().events.is_empty());
}

#[test]
fn non_focusable_targets_do_not_steal_focus() {
    let mut tree = UiTree::new(interactive("plain"));
    let node = tree.node_id_at(0).unwrap();
    let regions = [region_at(node, 10.0, false)];
    tree.pointer_moved(Point::new(30.0, 20.0), &regions);
    let pressed = tree.primary_pressed(&regions);

    assert!(
        !pressed
            .events
            .iter()
            .any(|event| event.kind == UiEventKind::Focused)
    );
}
