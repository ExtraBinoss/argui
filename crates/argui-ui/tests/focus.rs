use argui_core::{Affine2D, Key, KeyInput, KeyState, Modifiers, Point, Rect, Size};
use argui_paint::ClipChain;
use argui_ui::{
    CursorIcon, Element, FocusRequest, FocusScope, GestureSet, HitRegion, InitialFocus,
    Interaction, KeyboardActivation, Role, Semantics, UiEventKind, UiTree, VisualState,
};

fn control(key: &str) -> Element {
    Element::container([]).keyed(key).interaction(
        Interaction::default()
            .focusable(true)
            .keyboard_activation(KeyboardActivation::EnterOrSpace),
    )
}

fn region(node: argui_ui::NodeId, x: f32) -> HitRegion {
    HitRegion {
        node,
        bounds: Rect::new(Point::new(x, 0.0), Size::new(40.0, 30.0)),
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
        shape: argui_ui::HitShape::Bounds,
        slop: argui_ui::HitTestStyle::default().slop,
        enabled: true,
        focusable: true,
        cursor: CursorIcon::Auto,
        gestures: GestureSet::NONE,
        window_drag: None,
    }
}

fn key(key: Key, state: KeyState, repeat: bool) -> KeyInput {
    KeyInput {
        key,
        state,
        text: None,
        repeat,
        modifiers: Modifiers::default(),
    }
}

#[test]
fn focused_controls_receive_raw_keys_and_synthesized_activation() {
    let mut tree = UiTree::new(control("button"));
    let node = tree.node_id_at(0).unwrap();
    let regions = [region(node, 0.0)];
    tree.sync_focus(&regions, Some(FocusRequest::Focus(node.into())));

    let enter = key(Key::Enter, KeyState::Pressed, false);
    let update = tree.key_input(&enter, &regions);
    assert_eq!(
        update
            .events
            .iter()
            .map(|event| &event.kind)
            .collect::<Vec<_>>(),
        vec![
            &UiEventKind::KeyInput(enter),
            &UiEventKind::Pressed,
            &UiEventKind::Clicked,
            &UiEventKind::Released,
        ]
    );

    let repeat = key(Key::Enter, KeyState::Pressed, true);
    let update = tree.key_input(&repeat, &regions);
    assert_eq!(update.events.len(), 1);
    assert_eq!(update.events[0].kind, UiEventKind::KeyInput(repeat));
}

#[test]
fn space_holds_pressed_state_and_focus_change_cancels_the_click() {
    let mut tree = UiTree::new(Element::row([control("first"), control("second")]));
    let first = tree.node_id_at(1).unwrap();
    let second = tree.node_id_at(2).unwrap();
    let regions = [region(first, 0.0), region(second, 50.0)];
    tree.sync_focus(&regions, Some(FocusRequest::Focus(first.into())));

    let pressed = tree.key_input(
        &key(Key::Character(" ".into()), KeyState::Pressed, false),
        &regions,
    );
    assert!(
        pressed
            .events
            .iter()
            .any(|event| event.kind == UiEventKind::Pressed)
    );
    assert!(tree.visual_states(first).contains(VisualState::Pressed));

    let changed = tree.sync_focus(&regions, Some(FocusRequest::Focus(second.into())));
    assert!(
        changed
            .events
            .iter()
            .any(|event| { event.target == first && event.kind == UiEventKind::Released })
    );
    let released = tree.key_input(
        &key(Key::Character(" ".into()), KeyState::Released, false),
        &regions,
    );
    assert!(
        !released
            .events
            .iter()
            .any(|event| event.kind == UiEventKind::Clicked)
    );
}

#[test]
fn trapped_scope_autofocuses_cycles_and_restores_the_trigger() {
    fn view(open: bool) -> Element {
        let mut children = vec![control("trigger")];
        if open {
            children.push(
                Element::column([control("first"), control("last")])
                    .keyed("dialog")
                    .focus_scope(FocusScope::trapped(InitialFocus::First)),
            );
        }
        Element::column(children)
    }

    let mut tree = UiTree::new(view(false));
    let trigger = tree.node_id_at(1).unwrap();
    tree.sync_focus(
        &[region(trigger, 0.0)],
        Some(FocusRequest::Focus(trigger.into())),
    );

    tree.update(view(true));
    let first = tree.node_id_at(3).unwrap();
    let last = tree.node_id_at(4).unwrap();
    let outside = region(trigger, 0.0);
    let regions = [outside, region(first, 50.0), region(last, 100.0)];
    tree.sync_focus(&regions, None);
    assert_eq!(tree.focused_node(), Some(first));
    tree.sync_focus(&regions, Some(FocusRequest::Focus(trigger.into())));
    assert_eq!(tree.focused_node(), Some(first));
    tree.pointer_moved(Point::new(10.0, 10.0), &regions);
    tree.primary_pressed(&regions);
    assert_eq!(tree.focused_node(), Some(first));
    tree.primary_released();

    let tab = key(Key::Tab, KeyState::Pressed, false);
    tree.key_input(&tab, &regions);
    assert_eq!(tree.focused_node(), Some(last));
    tree.key_input(&tab, &regions);
    assert_eq!(tree.focused_node(), Some(first));

    tree.update(view(false));
    let restored = tree.sync_focus(&[region(trigger, 0.0)], None);
    assert_eq!(tree.focused_node(), Some(trigger));
    assert!(
        restored
            .events
            .iter()
            .any(|event| { event.target == trigger && event.kind == UiEventKind::Focused })
    );
}

#[test]
fn modal_scope_hides_background_semantics_and_marks_the_dialog() {
    let tree = UiTree::new(Element::column([
        control("background").semantics(Semantics::new(Role::Button).label("Background")),
        Element::column(
            [control("inside").semantics(Semantics::new(Role::Button).label("Inside"))],
        )
        .keyed("modal")
        .focus_scope(FocusScope::modal(InitialFocus::First)),
    ]));

    let semantics = tree.semantic_tree(&[], 1.0);
    assert_eq!(semantics.nodes.len(), 3);
    assert!(
        !semantics
            .nodes
            .iter()
            .any(|node| { node.semantics.label.as_deref() == Some("Background") })
    );
    let dialog = semantics
        .nodes
        .iter()
        .find(|node| node.semantics.role == Role::Dialog)
        .unwrap();
    assert!(dialog.semantics.state.modal);
}

#[test]
fn invalid_explicit_targets_never_move_or_clear_valid_focus() {
    let mut tree = UiTree::new(Element::row([
        control("same"),
        control("same"),
        control("valid"),
    ]));
    let first = tree.node_id_at(1).unwrap();
    let second = tree.node_id_at(2).unwrap();
    let valid = tree.node_id_at(3).unwrap();
    let regions = [
        region(first, 0.0),
        region(second, 50.0),
        region(valid, 100.0),
    ];
    tree.sync_focus(&regions, Some(FocusRequest::Focus(valid.into())));

    tree.sync_focus(&regions, Some(FocusRequest::Focus("same".into())));
    assert_eq!(tree.focused_node(), Some(valid));
    tree.sync_focus(&regions, Some(FocusRequest::Focus("missing".into())));
    assert_eq!(tree.focused_node(), Some(valid));
}

#[test]
fn window_focus_restores_the_exact_node_and_releases_space() {
    let mut tree = UiTree::new(control("button"));
    let node = tree.node_id_at(0).unwrap();
    let regions = [region(node, 0.0)];
    tree.sync_focus(&regions, Some(FocusRequest::Focus(node.into())));
    tree.key_input(
        &key(Key::Character(" ".into()), KeyState::Pressed, false),
        &regions,
    );

    let blurred = tree.window_blurred();
    assert_eq!(tree.focused_node(), None);
    assert!(
        blurred
            .events
            .iter()
            .any(|event| event.kind == UiEventKind::Released)
    );
    assert!(
        blurred
            .events
            .iter()
            .any(|event| event.kind == UiEventKind::Blurred)
    );
    let focused = tree.window_focused(&regions);
    assert_eq!(tree.focused_node(), Some(node));
    assert!(
        focused
            .events
            .iter()
            .any(|event| event.kind == UiEventKind::Focused)
    );
}

#[test]
fn restoring_scope_without_initial_focus_respects_the_restore_policy() {
    fn view(open: bool, restore: bool) -> Element {
        let mut children = vec![control("trigger")];
        if open {
            children.push(
                Element::column([control("inside")])
                    .keyed("scope")
                    .focus_scope(FocusScope::restoring().restore(restore)),
            );
        }
        Element::column(children)
    }

    let mut tree = UiTree::new(view(false, true));
    let trigger = tree.node_id_at(1).unwrap();
    tree.sync_focus(
        &[region(trigger, 0.0)],
        Some(FocusRequest::Focus(trigger.into())),
    );
    tree.update(view(true, true));
    let inside = tree.node_id_at(3).unwrap();
    let regions = [region(trigger, 0.0), region(inside, 50.0)];
    tree.sync_focus(&regions, None);
    assert_eq!(tree.focused_node(), Some(trigger));
    tree.sync_focus(&regions, Some(FocusRequest::Focus(inside.into())));
    tree.update(view(false, true));
    tree.sync_focus(&[region(trigger, 0.0)], None);
    assert_eq!(tree.focused_node(), Some(trigger));

    tree.update(view(true, false));
    let inside = tree.node_id_at(3).unwrap();
    let regions = [region(trigger, 0.0), region(inside, 50.0)];
    tree.sync_focus(&regions, Some(FocusRequest::Focus(inside.into())));
    tree.update(view(false, false));
    tree.sync_focus(&[region(trigger, 0.0)], None);
    assert_eq!(tree.focused_node(), None);
}

#[test]
fn trap_recovers_when_its_focused_child_is_removed_and_rejects_clear() {
    fn view(second: bool) -> Element {
        let mut children = vec![control("first")];
        if second {
            children.push(control("second"));
        }
        Element::column(children)
            .keyed("trap")
            .focus_scope(FocusScope::trapped(InitialFocus::First))
    }

    let mut tree = UiTree::new(view(true));
    let first = tree.node_id_at(1).unwrap();
    let second = tree.node_id_at(2).unwrap();
    let regions = [region(first, 0.0), region(second, 50.0)];
    tree.sync_focus(&regions, None);
    tree.sync_focus(&regions, Some(FocusRequest::Focus(second.into())));
    tree.sync_focus(&regions, Some(FocusRequest::Clear));
    assert_eq!(tree.focused_node(), Some(second));

    tree.update(view(false));
    let update = tree.sync_focus(&[region(first, 0.0)], None);
    assert_eq!(tree.focused_node(), Some(first));
    assert!(
        update
            .events
            .iter()
            .any(|event| event.kind == UiEventKind::Blurred)
    );
}

#[test]
fn initial_key_target_and_enter_only_activation_are_exact() {
    let enter_only = Element::container([]).keyed("inside").interaction(
        Interaction::default()
            .focusable(true)
            .keyboard_activation(KeyboardActivation::Enter),
    );
    let mut tree = UiTree::new(
        Element::column([enter_only])
            .focus_scope(FocusScope::trapped(InitialFocus::Target("inside".into()))),
    );
    let inside = tree.node_id_at(1).unwrap();
    let regions = [region(inside, 0.0)];
    tree.sync_focus(&regions, None);
    assert_eq!(tree.focused_node(), Some(inside));

    let space = tree.key_input(
        &key(Key::Character(" ".into()), KeyState::Pressed, false),
        &regions,
    );
    assert_eq!(space.events.len(), 1);
    let enter = tree.key_input(&key(Key::Enter, KeyState::Pressed, false), &regions);
    assert!(
        enter
            .events
            .iter()
            .any(|event| event.kind == UiEventKind::Clicked)
    );
}

#[test]
fn unfocused_non_navigation_keys_and_repeated_tab_do_nothing() {
    let mut tree = UiTree::new(control("button"));
    let node = tree.node_id_at(0).unwrap();
    let regions = [region(node, 0.0)];
    assert!(
        tree.key_input(&key(Key::Escape, KeyState::Pressed, false), &regions)
            .events
            .is_empty()
    );
    assert!(
        tree.key_input(&key(Key::Tab, KeyState::Pressed, true), &regions)
            .events
            .is_empty()
    );
    assert!(
        tree.key_input(&key(Key::Escape, KeyState::Released, false), &regions)
            .events
            .is_empty()
    );

    tree.sync_focus(&regions, Some(FocusRequest::Focus(node.into())));
    let repeated_tab = tree.key_input(&key(Key::Tab, KeyState::Pressed, true), &regions);
    assert_eq!(repeated_tab.events.len(), 1);
    let character = tree.key_input(
        &key(Key::Character("x".into()), KeyState::Released, false),
        &regions,
    );
    assert_eq!(character.events.len(), 1);
}

#[test]
fn empty_restoring_scope_and_disabled_initial_candidate_are_explicit() {
    let mut restoring = UiTree::new(Element::container([]).focus_scope(FocusScope::restoring()));
    restoring.sync_focus(&[], None);
    restoring.update(Element::container([]));
    assert!(restoring.sync_focus(&[], None).events.is_empty());

    let mut tree = UiTree::new(
        Element::column([control("disabled"), control("enabled")])
            .focus_scope(FocusScope::trapped(InitialFocus::First)),
    );
    let disabled = tree.node_id_at(1).unwrap();
    let enabled = tree.node_id_at(2).unwrap();
    let mut disabled_region = region(disabled, 0.0);
    disabled_region.focusable = false;
    tree.sync_focus(&[disabled_region, region(enabled, 50.0)], None);
    assert_eq!(tree.focused_node(), Some(enabled));
}
