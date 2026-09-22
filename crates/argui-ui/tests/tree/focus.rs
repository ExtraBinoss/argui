use argui_core::{Affine2D, Point, Rect, Size};
use argui_paint::ClipChain;
use argui_ui::{
    Color, CursorIcon, Element, EventHandlerId, EventListener, EventOwnerId, EventType,
    FocusPolicy, FocusRequest, FocusScope, GestureSet, HitRegion, InitialFocus, Interaction,
    UiEventKind, UiTree,
};
use argui_widgets::{Input, shadcn};

fn listen(element: Element) -> Element {
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

fn region(node: argui_ui::NodeId, x: f32, focus_policy: FocusPolicy) -> HitRegion {
    HitRegion {
        node,
        bounds: Rect::new(Point::new(x, 0.0), Size::new(40.0, 30.0)),
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
        shape: argui_ui::HitShape::Bounds,
        slop: Default::default(),
        enabled: true,
        focus_policy,
        cursor: CursorIcon::Auto,
        gestures: GestureSet::EMPTY,
        window_drag: None,
    }
}

#[test]
fn outside_press_blurs_input_without_consuming_the_clicked_action() {
    let themes = shadcn(Color::BLACK);
    let theme = themes.resolve(argui_core::ColorScheme::Dark);
    for (x, policy) in [
        (55.0, FocusPolicy::None),
        (55.0, FocusPolicy::TabStop),
        (150.0, FocusPolicy::None),
    ] {
        let mut tree = UiTree::new(Element::row([
            listen(Input::new("value", "112", "", theme.input()).build()),
            listen(
                Element::container([])
                    .keyed("action")
                    .interaction(Interaction::default()),
            ),
        ]));
        let input = tree.node_id_at(1).unwrap();
        let action = *tree.node_ids().last().unwrap();
        let regions = [
            region(input, 0.0, FocusPolicy::TabStop),
            region(action, 50.0, policy),
        ];
        tree.sync_focus(&regions, Some(FocusRequest::Focus(input.into())));
        tree.replace_text_input(input, "234");
        tree.pointer_moved(Point::new(x, 10.0), &regions);
        let pressed = tree.primary_pressed(&regions);
        assert!(pressed.paint_changed);
        assert!(
            pressed
                .events
                .iter()
                .any(|e| e.target == input && e.kind == UiEventKind::Blurred)
        );
        assert_eq!(tree.text_input_value(input), Some("234"));
        assert_eq!(
            tree.focused_node(),
            (policy == FocusPolicy::TabStop).then_some(action)
        );
        let release = tree.primary_released();
        assert_eq!(
            release
                .events
                .iter()
                .any(|e| e.target == action && matches!(e.kind, UiEventKind::Click(_))),
            x < 100.0
        );
        tree.primary_pressed(&regions);
        assert!(
            !tree
                .primary_released()
                .events
                .iter()
                .any(|e| e.kind == UiEventKind::Blurred)
        );
    }
}

#[test]
fn click_inside_input_preserves_focus_and_blank_area_respects_modal_traps() {
    let themes = shadcn(Color::BLACK);
    let theme = themes.resolve(argui_core::ColorScheme::Dark);
    for trapped in [false, true] {
        let root = Element::column([listen(
            Input::new("value", "112", "", theme.input()).build(),
        )]);
        let root = if trapped {
            root.focus_scope(FocusScope::trapped(InitialFocus::First))
        } else {
            root
        };
        let mut tree = UiTree::new(root);
        let input = tree.node_id_at(1).unwrap();
        let regions = [region(input, 0.0, FocusPolicy::TabStop)];
        tree.sync_focus(&regions, Some(FocusRequest::Focus(input.into())));
        tree.pointer_moved(Point::new(10.0, 10.0), &regions);
        assert!(
            !tree
                .primary_pressed(&regions)
                .events
                .iter()
                .any(|e| e.kind == UiEventKind::Blurred)
        );
        tree.primary_released();
        tree.pointer_moved(Point::new(200.0, 10.0), &regions);
        tree.primary_pressed(&regions);
        assert_eq!(tree.focused_node(), trapped.then_some(input));
    }
}

#[test]
fn opted_focus_scope_focuses_when_nonfocusable_descendant_is_pressed() {
    for focus_on_descendant in [false, true] {
        let root = Element::container([Element::container([]).interaction(Interaction::default())])
            .focus_scope(FocusScope::restoring())
            .interaction(
                Interaction::default()
                    .focus_policy(FocusPolicy::TabStop)
                    .focus_on_descendant_press(focus_on_descendant),
            );
        let mut tree = UiTree::new(listen(root));
        let scope = tree.node_id_at(0).unwrap();
        let child = tree.node_id_at(1).unwrap();
        let regions = [
            region(scope, 0.0, FocusPolicy::TabStop),
            region(child, 0.0, FocusPolicy::None),
        ];

        tree.pointer_moved(Point::new(10.0, 10.0), &regions);
        let pressed = tree.primary_pressed(&regions);

        assert_eq!(tree.focused_node(), focus_on_descendant.then_some(scope));
        assert_eq!(
            pressed
                .events
                .iter()
                .any(|event| event.target == scope && event.kind == UiEventKind::Focused),
            focus_on_descendant
        );
    }
}

#[test]
fn focusable_descendant_takes_precedence_over_opted_focus_scope() {
    let root = Element::container([Element::container([])
        .interaction(Interaction::default().focus_policy(FocusPolicy::TabStop))])
    .focus_scope(FocusScope::restoring())
    .interaction(
        Interaction::default()
            .focus_policy(FocusPolicy::TabStop)
            .focus_on_descendant_press(true),
    );
    let mut tree = UiTree::new(root);
    let scope = tree.node_id_at(0).unwrap();
    let child = tree.node_id_at(1).unwrap();
    let regions = [
        region(scope, 0.0, FocusPolicy::TabStop),
        region(child, 0.0, FocusPolicy::TabStop),
    ];

    tree.pointer_moved(Point::new(10.0, 10.0), &regions);
    tree.primary_pressed(&regions);

    assert_eq!(tree.focused_node(), Some(child));
}

#[test]
fn programmatic_traversal_stays_in_trap_and_skips_disabled_entries() {
    let popup = Element::container([
        Element::container([]).keyed("first"),
        Element::container([]).keyed("disabled"),
        Element::container([]).keyed("last"),
    ])
    .focus_scope(FocusScope::trapped(InitialFocus::First));
    let mut tree = UiTree::new(Element::container([
        Element::container([]).keyed("outside"),
        popup,
    ]));
    let outside = tree.node_id_at(1).unwrap();
    let first = tree.node_id_at(3).unwrap();
    let disabled = tree.node_id_at(4).unwrap();
    let last = tree.node_id_at(5).unwrap();
    let mut disabled_region = region(disabled, 80.0, FocusPolicy::TabStop);
    disabled_region.enabled = false;
    let regions = [
        region(outside, 0.0, FocusPolicy::TabStop),
        region(first, 40.0, FocusPolicy::TabStop),
        disabled_region,
        region(last, 120.0, FocusPolicy::TabStop),
    ];

    tree.sync_focus(&regions, None);
    assert_eq!(tree.focused_node(), Some(first));
    tree.sync_focus(&regions, Some(FocusRequest::Next));
    assert_eq!(tree.focused_node(), Some(last));
    tree.sync_focus(&regions, Some(FocusRequest::Next));
    assert_eq!(tree.focused_node(), Some(first));
    tree.sync_focus(&regions, Some(FocusRequest::Previous));
    assert_eq!(tree.focused_node(), Some(last));
    tree.sync_focus(&regions, Some(FocusRequest::Focus(outside.into())));
    assert_eq!(tree.focused_node(), Some(last));
}
