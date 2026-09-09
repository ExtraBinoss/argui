use argui_core::{Affine2D, Key, KeyInput, KeyState, Modifiers, Point, Rect, Size};
use argui_paint::ClipChain;
use argui_ui::{
    ActivationSource, CursorIcon, Element, EventHandlerId, EventListener, EventOwnerId, EventType,
    FocusRequest, FocusScope, GestureSet, HitRegion, InitialFocus, Interaction, KeyboardActivation,
    Role, Semantics, UiEventKind, UiTree, VisualState,
};

fn control(key: &str) -> Element {
    listeners(
        Element::container([]).keyed(key).interaction(
            Interaction::default()
                .focus_policy(argui_ui::FocusPolicy::TabStop)
                .keyboard_activation(KeyboardActivation::EnterOrSpace),
        ),
    )
}

fn listeners(element: Element) -> Element {
    EventType::ALL
        .into_iter()
        .enumerate()
        .fold(element, |element, (slot, event)| {
            element.on(EventListener::new(
                event,
                EventHandlerId::new(EventOwnerId(1), slot as u32),
            ))
        })
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
        focus_policy: argui_ui::FocusPolicy::TabStop,
        cursor: CursorIcon::Auto,
        gestures: GestureSet::EMPTY,
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
            &UiEventKind::KeyInput(enter.clone()),
            &UiEventKind::Click(argui_ui::ClickEvent::keyboard(enter)),
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
    assert_eq!(pressed.events.len(), 1);
    assert!(tree.visual_states(first).contains(VisualState::Pressed));

    let changed = tree.sync_focus(&regions, Some(FocusRequest::Focus(second.into())));
    assert!(
        changed
            .events
            .iter()
            .any(|event| event.target == first && event.kind == UiEventKind::Blurred)
    );
    let released = tree.key_input(
        &key(Key::Character(" ".into()), KeyState::Released, false),
        &regions,
    );
    assert!(!released.events.iter().any(|event| matches!(
        event.kind,
        UiEventKind::Click(argui_ui::ClickEvent {
            source: ActivationSource::Keyboard(_),
            ..
        })
    )));
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
fn pointer_opened_scope_preserves_hidden_focus_rings() {
    fn view(open: bool) -> Element {
        Element::column(std::iter::once(control("trigger")).chain(open.then(|| {
            Element::column([control("inside")])
                .keyed("popover")
                .focus_scope(FocusScope::trapped(InitialFocus::First))
        })))
    }

    let mut tree = UiTree::new(view(false));
    let trigger = tree.node_id_at(1).unwrap();
    let closed_regions = [region(trigger, 0.0)];
    tree.pointer_moved(Point::new(10.0, 10.0), &closed_regions);
    tree.primary_pressed(&closed_regions);
    assert!(
        !tree
            .visual_states(trigger)
            .contains(VisualState::FocusVisible)
    );

    tree.update(view(true));
    let inside = tree.node_id_at(3).unwrap();
    let open_regions = [region(trigger, 0.0), region(inside, 50.0)];
    tree.sync_focus(&open_regions, None);
    assert_eq!(tree.focused_node(), Some(inside));
    assert!(
        !tree
            .visual_states(inside)
            .contains(VisualState::FocusVisible)
    );

    tree.update(view(false));
    tree.sync_focus(&closed_regions, None);
    assert_eq!(tree.focused_node(), Some(trigger));
    assert!(
        !tree
            .visual_states(trigger)
            .contains(VisualState::FocusVisible)
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
    tree.sync_focus(&regions, Some(FocusRequest::Clear));
    assert_eq!(tree.focused_node(), None);
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
    let enter_only = listeners(
        Element::container([]).keyed("inside").interaction(
            Interaction::default()
                .focus_policy(argui_ui::FocusPolicy::TabStop)
                .keyboard_activation(KeyboardActivation::Enter),
        ),
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
    assert!(enter.events.iter().any(|event| matches!(
        event.kind,
        UiEventKind::Click(argui_ui::ClickEvent {
            source: ActivationSource::Keyboard(_),
            ..
        })
    )));
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
    disabled_region.focus_policy = argui_ui::FocusPolicy::None;
    tree.sync_focus(&[disabled_region, region(enabled, 50.0)], None);
    assert_eq!(tree.focused_node(), Some(enabled));
}

#[test]
fn removing_a_focused_node_queues_one_blur_without_restoring_a_stale_id() {
    let mut tree = UiTree::new(Element::row([control("keep"), control("remove")]));
    let keep = tree.node_id_at(1).unwrap();
    let removed = tree.node_id_at(2).unwrap();
    let regions = [region(keep, 0.0), region(removed, 50.0)];
    tree.sync_focus(&regions, Some(FocusRequest::Focus(removed.into())));
    assert_eq!(tree.focused_node(), Some(removed));

    assert!(tree.update(Element::row([control("keep")])) == argui_ui::TreeUpdate::Layout);
    assert_eq!(tree.focused_node(), None);
    let update = tree.sync_focus(&[region(keep, 0.0)], None);
    assert_eq!(
        update
            .events
            .iter()
            .filter(|event| event.target == removed && event.kind == UiEventKind::Blurred)
            .count(),
        1
    );
    assert_eq!(tree.focused_node(), None);
    assert!(
        tree.sync_focus(&[region(keep, 0.0)], None)
            .events
            .is_empty()
    );
}

#[test]
fn keyboard_focus_handles_empty_cycles_and_repeated_space_activation() {
    let tab = key(Key::Tab, KeyState::Pressed, false);
    let mut empty = UiTree::new(Element::container([]));
    assert!(empty.key_input(&tab, &[]).events.is_empty());

    let mut tree = UiTree::new(control("button"));
    let node = tree.node_id_at(0).unwrap();
    let regions = [region(node, 0.0)];
    tree.sync_focus(&regions, Some(FocusRequest::Focus(node.into())));
    tree.key_input(&tab, &regions);
    assert_eq!(tree.focused_node(), Some(node));

    let space = key(Key::Character(" ".into()), KeyState::Pressed, false);
    tree.key_input(&space, &regions);
    tree.key_input(&space, &regions);
    let released = key(Key::Character(" ".into()), KeyState::Released, false);
    let update = tree.key_input(&released, &regions);
    assert!(update.events.iter().any(|event| matches!(
        event.kind,
        UiEventKind::Click(argui_ui::ClickEvent {
            source: ActivationSource::Keyboard(_),
            ..
        })
    )));
}

#[test]
fn programmatic_focus_is_excluded_from_tab_order_but_accepts_explicit_focus() {
    let mut middle = control("middle");
    middle.interaction.as_mut().unwrap().focus_policy = argui_ui::FocusPolicy::Programmatic;
    let mut tree = UiTree::new(Element::row([control("first"), middle, control("last")]));
    let nodes: Vec<_> = ["first", "middle", "last"]
        .map(|key| {
            tree.node_ids()
                .iter()
                .copied()
                .find(|node| tree.key(*node) == Some(key))
                .unwrap()
        })
        .into();
    let mut regions: Vec<_> = nodes
        .iter()
        .enumerate()
        .map(|(i, node)| region(*node, i as f32 * 40.0))
        .collect();
    regions[1].focus_policy = argui_ui::FocusPolicy::Programmatic;
    tree.key_input(&key(Key::Tab, KeyState::Pressed, false), &regions);
    assert_eq!(tree.focused_node(), Some(nodes[0]));
    tree.key_input(&key(Key::Tab, KeyState::Pressed, false), &regions);
    assert_eq!(tree.focused_node(), Some(nodes[2]));
    tree.sync_focus(&regions, Some(FocusRequest::Focus("middle".into())));
    assert_eq!(tree.focused_node(), Some(nodes[1]));
    tree.key_input(&key(Key::Tab, KeyState::Pressed, false), &regions);
    assert_eq!(tree.focused_node(), Some(nodes[2]));
}

#[test]
fn closing_a_nonmodal_scope_preserves_focus_that_already_left_it() {
    fn view(open: bool) -> Element {
        Element::column(
            std::iter::once(control("trigger"))
                .chain(open.then(|| {
                    Element::column([control("item")])
                        .keyed("popup")
                        .focus_scope(argui_ui::FocusScope {
                            containment: argui_ui::FocusContainment::None,
                            initial: Some(InitialFocus::First),
                            restore: true,
                        })
                }))
                .chain(std::iter::once(control("next"))),
        )
    }
    let mut tree = UiTree::new(view(false));
    let trigger = tree.node_ids()[1];
    let next = tree.node_ids()[2];
    tree.sync_focus(
        &[region(trigger, 0.0), region(next, 100.0)],
        Some(FocusRequest::Focus(trigger.into())),
    );
    tree.update(view(true));
    let item = tree.node_ids()[3];
    let next = *tree.node_ids().last().unwrap();
    let regions = [
        region(trigger, 0.0),
        region(item, 50.0),
        region(next, 100.0),
    ];
    tree.sync_focus(&regions, None);
    assert_eq!(tree.focused_node(), Some(item));
    tree.key_input(&key(Key::Tab, KeyState::Pressed, false), &regions);
    assert_eq!(tree.focused_node(), Some(next));
    tree.update(view(false));
    tree.sync_focus(&[region(trigger, 0.0), region(next, 100.0)], None);
    assert_eq!(tree.focused_node(), Some(next));
}
