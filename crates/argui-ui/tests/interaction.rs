use argui_core::{
    Affine2D, Key, KeyInput, KeyState, Modifiers, Point, PointerButton, PointerEvent, PointerId,
    PointerKind, PointerPhase, Rect, Size,
};
use argui_paint::{ClipChain, ClipRegion, Color, QuadStyle};
use argui_ui::{
    ClipboardRequest, CursorIcon, Element, HitRegion, HitShape, HitTestStyle, Interaction,
    InteractionUpdate, PointerEvents, Sides, StylePatch, TreeUpdate, UiEventKind, UiTree,
    VisualState, VisualStates,
};

fn interactive(key: &str) -> Element {
    Element::container([])
        .keyed(key)
        .background(Color::srgb(0.0, 0.0, 0.0))
        .interaction(Interaction::default().focusable(true))
        .when(
            VisualState::Hovered,
            StylePatch::from_quad(QuadStyle::solid(Color::WHITE)),
        )
        .when(
            VisualState::Pressed,
            StylePatch::from_quad(QuadStyle::solid(Color::srgb(1.0, 0.0, 0.0))),
        )
        .when(
            VisualState::Focused,
            StylePatch::from_quad(QuadStyle::solid(Color::srgb(0.0, 1.0, 0.0))),
        )
}

fn region(node: argui_ui::NodeId) -> HitRegion {
    region_at(node, 10.0, true)
}

fn region_at(node: argui_ui::NodeId, x: f32, focusable: bool) -> HitRegion {
    HitRegion {
        node,
        bounds: Rect::new(Point::new(x, 10.0), Size::new(80.0, 40.0)),
        transform: Affine2D::IDENTITY,
        clips: ClipChain::from_regions([ClipRegion::new(
            Rect::new(Point::new(x + 10.0, 10.0), Size::new(70.0, 40.0)),
            Affine2D::IDENTITY,
        )]),
        shape: argui_ui::HitShape::Bounds,
        slop: argui_ui::HitTestStyle::default().slop,
        enabled: true,
        focusable,
        cursor: CursorIcon::Auto,
        gestures: argui_ui::GestureSet::NONE,
        window_drag: None,
    }
}

#[test]
fn interactions_expose_explicit_platform_cursors() {
    assert_eq!(Interaction::default().cursor, CursorIcon::Auto);
    assert_eq!(
        Interaction::default().cursor(CursorIcon::Grab).cursor,
        CursorIcon::Grab
    );
    assert_eq!(Interaction::blocker().cursor, CursorIcon::Auto);
}

#[test]
fn changing_hit_geometry_requests_a_new_paint_snapshot() {
    let root = interactive("surface");
    let mut tree = UiTree::new(root.clone());
    assert_eq!(
        tree.update(
            root.hit_test(HitTestStyle::default().pointer_events(PointerEvents::ContentsOnly))
        ),
        TreeUpdate::Paint
    );
}

#[test]
fn hit_shapes_and_slop_are_transform_and_clip_aware() {
    let bounds = Rect::new(Point::new(10.0, 10.0), Size::new(40.0, 20.0));
    let mut target = region_at(
        UiTree::new(Element::container([])).node_ids()[0],
        10.0,
        false,
    );
    target.bounds = bounds;
    target.clips = ClipChain::from_regions([ClipRegion::new(
        Rect::new(Point::default(), Size::new(100.0, 100.0)),
        Affine2D::IDENTITY,
    )]);
    target.shape = HitShape::Ellipse;
    target.slop = Sides {
        left: 5.0,
        right: 5.0,
        top: 5.0,
        bottom: 5.0,
    };
    assert!(target.contains(Point::new(5.0, 20.0)));
    assert!(!target.contains(Point::new(5.0, 5.0)));

    target.shape = HitShape::RoundedRect(argui_ui::CornerRadii::all(8.0));
    target.transform = Affine2D::translation(20.0, 0.0);
    assert!(target.contains(Point::new(35.0, 20.0)));
    assert!(!target.contains(Point::new(25.0, 10.0)));

    target.transform = Affine2D::IDENTITY;
    target.slop = Sides::default();
    assert!(!target.contains(Point::new(49.0, 11.0)));
    assert!(!target.contains(Point::new(49.0, 29.0)));

    target.shape = HitShape::Ellipse;
    target.bounds = Rect::new(Point::new(10.0, 10.0), Size::new(0.0, 20.0));
    assert!(!target.contains(Point::new(10.0, 20.0)));
    target.bounds = Rect::new(Point::new(10.0, 10.0), Size::new(20.0, 0.0));
    assert!(!target.contains(Point::new(20.0, 10.0)));
}

#[test]
fn pointer_state_honors_clips_capture_clicks_and_focus() {
    let mut tree = UiTree::new(interactive("save"));
    let node = tree.node_id_at(0).unwrap();
    let regions = [region(node)];

    let clipped = tree.pointer_moved(Point::new(15.0, 20.0), &regions);
    assert!(clipped.events.is_empty());
    assert_eq!(tree.visual_states(node), VisualStates::NONE);

    let entered = tree.pointer_moved(Point::new(30.0, 20.0), &regions);
    assert!(entered.paint_changed);
    assert_eq!(entered.events[0].key.as_deref(), Some("save"));
    assert_eq!(entered.events[0].kind, UiEventKind::PointerEntered);
    assert!(tree.visual_states(node).contains(VisualState::Hovered));

    let pressed = tree.primary_pressed(&regions);
    assert_eq!(pressed.events[0].kind, UiEventKind::Pressed);
    assert!(
        pressed
            .events
            .iter()
            .any(|event| event.kind == UiEventKind::Focused)
    );
    assert!(tree.visual_states(node).contains(VisualState::Pressed));
    assert!(!tree.visual_states(node).contains(VisualState::FocusVisible));

    let dragged_out = tree.pointer_moved(Point::new(200.0, 200.0), &regions);
    assert!(
        dragged_out
            .events
            .iter()
            .any(|event| matches!(event.kind, UiEventKind::PointerMoved(_)))
    );
    assert!(tree.visual_states(node).contains(VisualState::Pressed));

    let released = tree.primary_released();
    assert_eq!(released.events.len(), 2);
    assert_eq!(released.events[0].kind, UiEventKind::Released);
    assert_eq!(
        released.events[1].kind,
        UiEventKind::LostPointerCapture(PointerId::MOUSE)
    );
    assert!(tree.visual_states(node).contains(VisualState::Focused));

    let blurred = tree.window_blurred();
    assert!(
        blurred
            .events
            .iter()
            .any(|event| event.kind == UiEventKind::Blurred)
    );
    assert_eq!(tree.visual_states(node), VisualStates::NONE);
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
fn tab_focus_wraps_in_both_directions() {
    let mut tree = UiTree::new(Element::row([interactive("first"), interactive("second")]));
    let first = tree.node_id_at(1).unwrap();
    let second = tree.node_id_at(2).unwrap();
    let regions = [region_at(first, 10.0, true), region_at(second, 100.0, true)];

    let tab = |shift| KeyInput {
        key: Key::Tab,
        state: KeyState::Pressed,
        text: None,
        repeat: false,
        modifiers: Modifiers {
            shift,
            ..Modifiers::default()
        },
    };
    tree.key_input(&tab(false), &regions);
    assert_eq!(tree.focused_node(), Some(first));
    assert!(
        tree.visual_states(first)
            .contains(VisualState::FocusVisible)
    );
    tree.key_input(&tab(false), &regions);
    assert_eq!(tree.focused_node(), Some(second));
    tree.key_input(&tab(false), &regions);
    assert_eq!(tree.focused_node(), Some(first));
    tree.key_input(&tab(true), &regions);
    assert_eq!(tree.focused_node(), Some(second));
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

    let pressed_again = tree.primary_pressed(&regions);
    assert!(
        pressed_again
            .events
            .iter()
            .all(|event| event.kind != UiEventKind::Focused)
    );
}

#[test]
fn blocker_regions_occlude_interactions_behind_overlays() {
    let mut tree = UiTree::new(Element::row([
        interactive("behind"),
        Element::container([])
            .keyed("overlay")
            .interaction(Interaction::blocker()),
    ]));
    let behind = tree.node_id_at(1).unwrap();
    let overlay = tree.node_id_at(2).unwrap();
    let regions = [
        region_at(behind, 10.0, true),
        region_at(overlay, 10.0, false),
    ];

    let update = tree.pointer_moved(Point::new(30.0, 20.0), &regions);
    assert_eq!(update.events[0].target, overlay);
    assert_eq!(update.events[0].key.as_deref(), Some("overlay"));
    assert_eq!(tree.visual_states(behind), VisualStates::NONE);
    assert!(tree.visual_states(overlay).contains(VisualState::Hovered));
    assert!(!Interaction::blocker().focusable);
}

#[test]
fn disabled_regions_occlude_controls_without_receiving_pointer_actions() {
    let mut tree = UiTree::new(Element::row([
        interactive("behind"),
        interactive("disabled"),
    ]));
    let behind = tree.node_id_at(1).unwrap();
    let disabled = tree.node_id_at(2).unwrap();
    let mut disabled_region = region_at(disabled, 10.0, false);
    disabled_region.enabled = false;
    let regions = [region_at(behind, 10.0, true), disabled_region];

    let moved = tree.pointer_moved(Point::new(30.0, 20.0), &regions);
    let pressed = tree.primary_pressed(&regions);

    assert!(moved.events.is_empty());
    assert!(pressed.events.is_empty());
    assert_eq!(tree.visual_states(behind), VisualStates::NONE);
    assert_eq!(tree.visual_states(disabled), VisualStates::NONE);
    assert_eq!(tree.focused_node(), None);
}

#[test]
fn interaction_updates_merge_every_dirty_signal_and_latest_clipboard_request() {
    let mut update = InteractionUpdate {
        paint_changed: true,
        clipboard: Some(ClipboardRequest::Read { target: None }),
        ..InteractionUpdate::default()
    };
    update.merge(InteractionUpdate {
        scroll_changed: true,
        layout_changed: true,
        text_input_changed: true,
        clipboard: Some(ClipboardRequest::Write("value".into())),
        ..InteractionUpdate::default()
    });
    update.merge(InteractionUpdate::default());

    assert!(update.paint_changed);
    assert!(update.scroll_changed);
    assert!(update.layout_changed);
    assert!(update.text_input_changed);
    assert_eq!(
        update.clipboard,
        Some(ClipboardRequest::Write("value".into()))
    );
}

#[test]
fn secondary_touches_do_not_replace_primary_interaction_capture() {
    let mut tree = UiTree::new(interactive("touch"));
    let node = tree.node_id_at(0).unwrap();
    let regions = [region(node)];
    let touch = |id, phase, primary| PointerEvent {
        id: PointerId::new(id),
        kind: PointerKind::Touch,
        phase,
        position: Point::new(30.0, 20.0),
        button: Some(PointerButton::Primary),
        buttons: 1,
        pressure: None,
        primary,
        timestamp: std::time::Duration::ZERO,
    };

    let pressed = tree.pointer_event(touch(1, PointerPhase::Pressed, true), &regions);
    assert!(
        pressed
            .events
            .iter()
            .any(|event| event.kind == UiEventKind::Pressed)
    );
    assert!(
        tree.pointer_event(touch(2, PointerPhase::Pressed, false), &regions)
            .events
            .is_empty()
    );
    let cancelled = tree.pointer_event(touch(1, PointerPhase::Cancelled, true), &regions);
    assert!(
        cancelled
            .events
            .iter()
            .any(|event| event.kind == UiEventKind::Released)
    );
    assert!(
        cancelled
            .events
            .iter()
            .all(|event| event.kind != UiEventKind::Clicked)
    );
    assert!(
        tree.pointer_event(touch(1, PointerPhase::Cancelled, true), &regions)
            .events
            .is_empty()
    );
}

#[test]
fn explicit_pointer_capture_retargets_motion_and_reports_every_release() {
    let mut tree = UiTree::new(Element::row([interactive("first"), interactive("second")]));
    let first = tree.node_id_at(1).unwrap();
    let second = tree.node_id_at(2).unwrap();
    let regions = [region_at(first, 10.0, true), region_at(second, 110.0, true)];
    let pointer = PointerId::new(7);

    let captured = tree.capture_pointer(pointer, first);
    assert_eq!(
        captured.events[0].kind,
        UiEventKind::GotPointerCapture(pointer)
    );
    let moved = tree.pointer_event(
        PointerEvent {
            id: pointer,
            kind: PointerKind::Pen,
            phase: PointerPhase::Moved,
            position: Point::new(130.0, 20.0),
            button: None,
            buttons: 0,
            pressure: Some(0.5),
            primary: true,
            timestamp: std::time::Duration::ZERO,
        },
        &regions,
    );
    assert!(moved.events.iter().any(|event| {
        event.target == first && matches!(event.kind, UiEventKind::PointerMoved(_))
    }));

    assert!(
        tree.release_pointer_capture(pointer, second)
            .events
            .is_empty()
    );
    assert_eq!(
        tree.release_pointer_capture(pointer, first).events[0].kind,
        UiEventKind::LostPointerCapture(pointer)
    );

    tree.capture_pointer(pointer, first);
    let blurred = tree.window_blurred();
    assert!(blurred.events.iter().any(|event| {
        event.target == first && event.kind == UiEventKind::LostPointerCapture(pointer)
    }));
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
    assert!(tree.visual_states(first).contains(VisualState::Pressed));
    assert_eq!(
        tree.resolved_quad(first, &tree.root().children[0])
            .background,
        QuadStyle::solid(Color::srgb(0.0, 1.0, 0.0)).background
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
    assert_eq!(tree.visual_states(old), VisualStates::NONE);
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
