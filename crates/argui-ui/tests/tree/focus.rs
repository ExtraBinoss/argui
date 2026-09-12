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
