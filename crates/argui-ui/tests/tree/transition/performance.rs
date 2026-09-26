use argui_animation::{Duration, Time, Transition, Tween};
use argui_core::{Affine2D, Point, Rect, Size, Transform2D};
use argui_paint::{ClipChain, Fill};
use argui_ui::{
    Color, CursorIcon, Element, FocusPolicy, FocusRequest, GestureSet, HitRegion, HitShape,
    Interaction, StateName, StateScopeId, StateSelector, StylePatch, StyleTransition,
    TextEditorSpec, TextInputFilter, TreeUpdate, UiTree, VisualState, WindowLayer, property,
};

/// Creates the retained transition used by interaction performance regressions.
fn transition() -> StyleTransition {
    StyleTransition::new(Transition::tween(Tween::new(Duration::from_millis(100))))
}

/// Creates an enabled hit region for `node` at the origin.
fn region(node: argui_ui::NodeId) -> HitRegion {
    HitRegion {
        node,
        bounds: Rect::new(Point::default(), Size::new(100.0, 40.0)),
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
        shape: HitShape::Bounds,
        slop: argui_ui::Sides::default(),
        enabled: true,
        focus_policy: FocusPolicy::None,
        cursor: CursorIcon::Auto,
        gestures: GestureSet::EMPTY,
        window_drag: None,
    }
}

#[test]
fn pressed_transform_after_hover_only_composites_on_rapid_clicks() {
    let element = Element::row([Element::text("Play")])
        .interaction(Interaction::default())
        .when(
            VisualState::Hovered,
            StylePatch::new().set(property::BackgroundColor, Color::WHITE),
        )
        .when(
            VisualState::Pressed,
            StylePatch::new().set(
                property::Transform,
                Transform2D::IDENTITY.translate(0.0, 1.0),
            ),
        )
        .transition(transition());
    let mut tree = UiTree::new(element);
    let node = tree.node_ids()[0];
    let regions = [region(node)];
    let hover = tree.pointer_moved(Point::new(10.0, 10.0), &regions);
    assert!(hover.paint_changed);

    for _ in 0..3 {
        let press = tree.primary_pressed(&regions);
        assert!(press.composite_changed);
        assert!(!press.paint_changed);
        assert!(!press.layout_changed);
        let release = tree.primary_released();
        assert!(release.composite_changed);
        assert!(!release.paint_changed);
        assert!(!release.layout_changed);
    }
}

#[test]
fn press_bounce_restarts_on_every_rapid_press_and_returns_while_held() {
    let element = Element::row([Element::text("Play")])
        .interaction(Interaction::default().press_bounce_scale(0.97));
    assert!(element.needs_compositor_layer());
    let mut tree = UiTree::new(element);
    let node = tree.node_ids()[0];
    let regions = [region(node)];
    tree.pointer_moved(Point::new(10.0, 10.0), &regions);

    for start in [0, 80, 160] {
        let press = tree.primary_pressed(&regions);
        assert!(press.composite_changed);
        assert!(!press.layout_changed);
        assert_eq!(
            tree.resolved_transform(node, tree.element_for(node).unwrap())
                .scale
                .x,
            1.0
        );
        tree.advance_animations(Time::from_nanos(start * 1_000_000));
        assert_eq!(
            tree.advance_animations(Time::from_nanos((start + 40) * 1_000_000)),
            TreeUpdate::Composite
        );
        assert!(
            tree.resolved_transform(node, tree.element_for(node).unwrap())
                .scale
                .x
                < 0.98
        );
        if start < 160 {
            tree.primary_released();
        }
    }

    assert_eq!(
        tree.advance_animations(Time::from_nanos(340_000_000)),
        TreeUpdate::Paint
    );
    assert_eq!(
        tree.resolved_transform(node, tree.element_for(node).unwrap())
            .scale
            .x,
        1.0
    );
    assert!(!tree.wants_animation_frame());
}

#[test]
fn press_bounce_respects_reduced_motion() {
    let element = Element::row([Element::text("Play")])
        .interaction(Interaction::default().press_bounce_scale(0.97));
    let mut tree = UiTree::new(element);
    let node = tree.node_ids()[0];
    let regions = [region(node)];
    tree.pointer_moved(Point::new(10.0, 10.0), &regions);
    tree.set_reduced_motion(true);
    tree.primary_pressed(&regions);
    assert_eq!(
        tree.resolved_transform(node, tree.element_for(node).unwrap())
            .scale
            .x,
        1.0
    );
    assert!(!tree.wants_animation_frame());
}

#[test]
fn removing_press_bounce_stops_pending_frames() {
    let element = Element::row([Element::text("Play")])
        .interaction(Interaction::default().press_bounce_scale(0.97));
    let mut tree = UiTree::new(element);
    let node = tree.node_ids()[0];
    let regions = [region(node)];
    tree.pointer_moved(Point::new(10.0, 10.0), &regions);
    tree.primary_pressed(&regions);
    tree.advance_animations(Time::from_nanos(0));
    tree.advance_animations(Time::from_nanos(40_000_000));
    assert!(tree.wants_animation_frame());

    tree.update(Element::row([Element::text("Play")]).interaction(Interaction::default()));
    assert!(!tree.wants_animation_frame());
    assert_eq!(
        tree.resolved_transform(node, tree.element_for(node).unwrap())
            .scale
            .x,
        1.0
    );
}

#[test]
fn focusing_a_text_editor_keeps_its_refresh_during_a_composited_press() {
    let editor = Element::text_editor(TextEditorSpec {
        value: "Input".into(),
        placeholder: String::new(),
        multiline: false,
        read_only: false,
        filter: TextInputFilter::Any,
        text: Default::default(),
        placeholder_text: Default::default(),
        selection: Color::WHITE,
        caret: Default::default(),
    })
    .interaction(Interaction::default().focus_policy(FocusPolicy::TabStop))
    .when(
        VisualState::Pressed,
        StylePatch::new().set(
            property::Transform,
            Transform2D::IDENTITY.translate(0.0, 1.0),
        ),
    )
    .transition(transition());
    let mut tree = UiTree::new(editor);
    let node = tree.node_ids()[0];
    let mut hit = region(node);
    hit.focus_policy = FocusPolicy::TabStop;
    tree.pointer_moved(Point::new(10.0, 10.0), &[hit.clone()]);

    let press = tree.primary_pressed(&[hit]);
    assert!(press.composite_changed);
    assert!(press.text_input_changed);
    assert_eq!(tree.focused_node(), Some(node));
}

#[test]
fn active_transition_set_survives_retarget_settlement_and_removal() {
    let active = StateName::new("active");
    let root = |first: bool, second: bool| {
        Element::row(
            [first, second]
                .into_iter()
                .enumerate()
                .map(|(index, enabled)| {
                    Element::text("Retained")
                        .keyed(format!("item-{index}"))
                        .active_state(active.clone(), enabled)
                        .when(
                            active.clone(),
                            StylePatch::new().set(
                                property::Transform,
                                Transform2D::IDENTITY.translate(0.0, 1.0),
                            ),
                        )
                        .transition(StyleTransition::new(Transition::tween(Tween::new(
                            Duration::from_millis(if index == 0 { 50 } else { 200 }),
                        ))))
                }),
        )
    };
    let mut tree = UiTree::new(root(false, false));
    let first = tree.node_ids()[1];
    let second = tree.node_ids()[2];
    tree.update(root(true, true));
    assert!(tree.wants_animation_frame());
    assert_eq!(
        tree.advance_animations(Time::from_nanos(1)),
        TreeUpdate::None
    );
    assert_eq!(
        tree.advance_animations(Time::from_nanos(50_000_001)),
        TreeUpdate::Paint
    );
    let first_revision = tree.visual_revision(first);
    let second_revision = tree.visual_revision(second);
    assert!(tree.wants_animation_frame());
    assert_eq!(
        tree.advance_animations(Time::from_nanos(75_000_001)),
        TreeUpdate::Composite
    );
    assert_eq!(tree.visual_revision(first), first_revision);
    assert!(tree.visual_revision(second) > second_revision);

    tree.update(root(true, false));
    assert!(tree.wants_animation_frame());
    tree.update(root(false, false));
    assert!(tree.wants_animation_frame());
    tree.update(Element::row([]));
    assert!(!tree.wants_animation_frame());
    assert_eq!(
        tree.advance_animations(Time::from_nanos(100_000_001)),
        TreeUpdate::None
    );
}

#[test]
fn moving_hover_between_siblings_updates_both_local_styles() {
    let item = || {
        Element::container([])
            .background(Color::BLACK)
            .interaction(Interaction::default())
            .when(
                VisualState::Hovered,
                StylePatch::new().set(property::BackgroundColor, Color::WHITE),
            )
    };
    let mut tree = UiTree::new(Element::row([item(), item()]));
    let first = tree.node_ids()[1];
    let second = tree.node_ids()[2];
    let left = region(first);
    let mut right = region(second);
    right.bounds.origin.x = 120.0;
    let regions = [left, right];

    tree.pointer_moved(Point::new(10.0, 10.0), &regions);
    assert_eq!(
        tree.resolved_quad(first, tree.element_for(first).unwrap())
            .background,
        Some(Fill::Solid(Color::WHITE))
    );
    tree.pointer_moved(Point::new(130.0, 10.0), &regions);
    assert_eq!(
        tree.resolved_quad(first, tree.element_for(first).unwrap())
            .background,
        Some(Fill::Solid(Color::BLACK))
    );
    assert_eq!(
        tree.resolved_quad(second, tree.element_for(second).unwrap())
            .background,
        Some(Fill::Solid(Color::WHITE))
    );
}

#[test]
fn pressed_scope_updates_a_portal_descendant() {
    let scope = StateScopeId::new("button");
    let root = Element::container([Element::container([])
        .portal(WindowLayer::Popover)
        .background(Color::BLACK)
        .when(
            StateSelector::scope(scope.clone(), VisualState::Pressed),
            StylePatch::new().set(property::BackgroundColor, Color::WHITE),
        )])
    .state_scope(scope)
    .interaction(Interaction::default());
    let mut tree = UiTree::new(root);
    let owner = tree.node_ids()[0];
    let portal = tree.node_ids()[1];
    let regions = [region(owner)];
    tree.pointer_moved(Point::new(10.0, 10.0), &regions);
    tree.primary_pressed(&regions);
    assert_eq!(
        tree.resolved_quad(portal, tree.element_for(portal).unwrap())
            .background,
        Some(Fill::Solid(Color::WHITE))
    );
    tree.primary_released();
    assert_eq!(
        tree.resolved_quad(portal, tree.element_for(portal).unwrap())
            .background,
        Some(Fill::Solid(Color::BLACK))
    );
}

#[test]
fn window_blur_clears_hover_outside_the_focused_branch() {
    let hovered = Element::container([])
        .background(Color::BLACK)
        .interaction(Interaction::default())
        .when(
            VisualState::Hovered,
            StylePatch::new().set(property::BackgroundColor, Color::WHITE),
        );
    let focused = Element::container([])
        .interaction(Interaction::default().focus_policy(FocusPolicy::TabStop));
    let mut tree = UiTree::new(Element::row([hovered, focused]));
    let hover_node = tree.node_ids()[1];
    let focus_node = tree.node_ids()[2];
    let mut focus_hit = region(focus_node);
    focus_hit.bounds.origin.x = 120.0;
    focus_hit.focus_policy = FocusPolicy::TabStop;
    tree.sync_focus(&[focus_hit], Some(FocusRequest::Focus(focus_node.into())));
    tree.pointer_moved(Point::new(10.0, 10.0), &[region(hover_node)]);
    assert_eq!(
        tree.resolved_quad(hover_node, tree.element_for(hover_node).unwrap())
            .background,
        Some(Fill::Solid(Color::WHITE))
    );

    tree.window_blurred();
    assert_eq!(
        tree.resolved_quad(hover_node, tree.element_for(hover_node).unwrap())
            .background,
        Some(Fill::Solid(Color::BLACK))
    );
}
