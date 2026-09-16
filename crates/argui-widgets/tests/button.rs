use argui_core::{Affine2D, Color, ColorScheme, Point, Rect, Size};
use argui_paint::{ClipChain, PaintStyle, QuadStyle};
use argui_text::{TextStyle, TextWrap};
use argui_ui::{
    CursorIcon, Element, ElementKind, EventHandler, EventHandlerId, EventOwnerId, EventPhase,
    EventType, HitRegion, KeyboardActivation, Role, UiEvent, UiEventKind, UiTree, UserSelect,
    VisualState,
};
use argui_widgets::{
    Button, ButtonAction, ButtonBehavior, ButtonStyle, TablerIcon, WidgetAssets, shadcn,
};

#[test]
fn changing_button_variants_interpolates_label_color_and_retargets_without_a_jump() {
    use argui_animation::Time;
    let themes = shadcn(Color::from_srgb8(25, 100, 230));
    let theme = themes.resolve(ColorScheme::Light);
    let view = |selected| {
        Button::new(
            "page",
            "Page",
            if selected {
                theme.button()
            } else {
                theme.ghost_button()
            },
        )
        .build()
    };
    let mut tree = UiTree::new(view(false));
    let color = |tree: &UiTree| {
        let node = tree.node_ids()[1];
        let ElementKind::Text { style, .. } = &tree.element_at(1).unwrap().kind else {
            panic!("button label");
        };
        tree.resolved_text_color(node, style.color)
    };
    let resting = color(&tree);
    tree.update(view(true));
    assert_eq!(
        color(&tree),
        resting,
        "selection must not snap the label to white"
    );
    tree.advance_animations(Time::from_nanos(1));
    tree.advance_animations(Time::from_nanos(60_000_001));
    let halfway = color(&tree);
    assert_ne!(halfway, resting);
    assert_ne!(halfway, theme.primary_foreground);
    tree.update(view(false));
    assert_eq!(
        color(&tree),
        halfway,
        "a rapid reversal starts at the current color"
    );
    tree.advance_animations(Time::from_nanos(60_000_002));
    tree.advance_animations(Time::from_nanos(180_000_002));
    assert_eq!(color(&tree), resting);
    tree.set_reduced_motion(true);
    tree.update(view(true));
    assert_eq!(color(&tree), theme.primary_foreground);
}

#[test]
fn button_exposes_variants_content_and_busy_state() {
    let theme = shadcn(Color::srgb(0.2, 0.5, 0.9));
    let theme = theme.resolve(ColorScheme::Dark);
    let assets = WidgetAssets::tabler(theme.foreground);
    let button = Button::new("save", "Save", theme.button())
        .leading(assets.icon(TablerIcon::Check, 16.0))
        .trailing(Element::text("⌘S"))
        .build();
    assert_eq!(button.key.as_deref(), Some("save"));
    assert_eq!(button.user_select, UserSelect::None);
    assert_eq!(button.children.len(), 3);
    assert_eq!(button.semantics.as_ref().unwrap().role, Role::Button);
    assert_eq!(
        button.interaction.as_ref().unwrap().keyboard_activation,
        KeyboardActivation::EnterOrSpace
    );

    let loading = Button::new("load", "Loading", theme.button())
        .loading(Element::text("spinner"))
        .build();
    let semantics = loading.semantics.as_ref().unwrap();
    assert!(semantics.state.busy);
    assert!(semantics.state.disabled);
    assert!(!loading.interaction.as_ref().unwrap().enabled);
    assert_eq!(
        loading.interaction.as_ref().unwrap().cursor,
        CursorIcon::Progress
    );

    let disabled = Button::new("disabled", "Disabled", theme.button())
        .enabled(false)
        .build();
    assert_eq!(
        disabled.interaction.as_ref().unwrap().cursor,
        CursorIcon::NotAllowed
    );
}

#[test]
fn button_focus_style_is_explicitly_opt_in() {
    let paint = PaintStyle::new(QuadStyle::solid(Color::srgb(0.0, 0.0, 0.0)));
    let default = ButtonStyle::new(paint.clone(), TextStyle::default());
    assert!(default.focused.is_none());

    let focused =
        ButtonStyle::new(paint, TextStyle::default()).focused(QuadStyle::solid(Color::WHITE));
    assert!(focused.focused.is_some());
}

#[test]
fn button_hover_uses_the_shared_retained_visual_state_path() {
    let resting = QuadStyle::solid(Color::srgb(0.0, 0.0, 0.0));
    let hovered = QuadStyle::solid(Color::WHITE);
    let element = Button::new(
        "save",
        "Save",
        ButtonStyle::new(PaintStyle::new(resting.clone()), TextStyle::default()).hovered(hovered),
    )
    .build();
    let mut tree = UiTree::new(element.clone());
    let node = tree.node_ids()[0];
    tree.pointer_moved(
        Point::new(2.0, 2.0),
        &[HitRegion {
            node,
            bounds: Rect::new(Point::default(), Size::new(20.0, 20.0)),
            transform: Affine2D::IDENTITY,
            clips: ClipChain::default(),
            shape: argui_ui::HitShape::Bounds,
            slop: argui_ui::HitTestStyle::default().slop,
            enabled: true,
            focus_policy: argui_ui::FocusPolicy::TabStop,
            cursor: CursorIcon::Pointer,
            gestures: argui_ui::GestureSet::EMPTY,
            window_drag: None,
        }],
    );
    assert!(tree.visual_states(node).contains(VisualState::Hovered));
    assert_eq!(
        tree.resolved_quad(node, &element).background,
        Some(argui_paint::Fill::Solid(Color::WHITE))
    );
    assert!(matches!(
        &element.children[0].kind,
        ElementKind::Text { style, .. } if style.wrap == TextWrap::None
    ));
}

#[test]
fn rapid_hover_leaves_no_trail_and_ignores_disabled_buttons() {
    let style = ButtonStyle::new(
        PaintStyle::new(QuadStyle::solid(Color::BLACK)),
        TextStyle::default(),
    )
    .hovered(QuadStyle::solid(Color::WHITE));
    let mut tree = UiTree::new(Element::column([
        Button::new("a", "A", style.clone()).build(),
        Button::new("b", "B", style.clone()).build(),
        Button::new("disabled", "Disabled", style)
            .enabled(false)
            .build(),
    ]));
    let nodes: Vec<_> = ["a", "b", "disabled"]
        .map(|key| {
            tree.node_ids()
                .iter()
                .copied()
                .find(|node| tree.key(*node) == Some(key))
                .unwrap()
        })
        .into();
    let regions: Vec<_> = nodes
        .iter()
        .enumerate()
        .map(|(index, &node)| hover_region(node, index as f32 * 40.0, index < 2))
        .collect();
    let background = |tree: &UiTree, node| {
        let index = tree
            .node_ids()
            .iter()
            .position(|candidate| *candidate == node)
            .unwrap();
        tree.resolved_quad(node, tree.element_at(index).unwrap())
            .background
    };
    // No animation frame elapses between these rapid crossings.
    for index in [0, 1, 0, 1] {
        tree.pointer_moved(Point::new(5.0, index as f32 * 40.0 + 5.0), &regions);
        assert_eq!(
            background(&tree, nodes[index]),
            Some(argui_paint::Fill::Solid(Color::WHITE))
        );
        assert!(
            tree.visual_states(nodes[index])
                .contains(VisualState::Hovered)
        );
        assert_eq!(
            background(&tree, nodes[1 - index]),
            Some(argui_paint::Fill::Solid(Color::BLACK))
        );
        assert!(
            !tree
                .visual_states(nodes[1 - index])
                .contains(VisualState::Hovered)
        );
    }
    tree.pointer_moved(Point::new(5.0, 85.0), &regions);
    assert!(!tree.visual_states(nodes[2]).contains(VisualState::Hovered));
    assert_eq!(
        background(&tree, nodes[2]),
        Some(argui_paint::Fill::Solid(Color::BLACK))
    );
    for node in nodes {
        assert_eq!(
            background(&tree, node),
            Some(argui_paint::Fill::Solid(Color::BLACK))
        );
    }
}

#[test]
fn explicit_button_transition_can_still_animate_hover_entry() {
    let element = Button::new(
        "custom",
        "Custom",
        ButtonStyle::new(
            PaintStyle::new(QuadStyle::solid(Color::BLACK)),
            TextStyle::default(),
        )
        .hovered(QuadStyle::solid(Color::WHITE))
        .transition(argui_ui::StyleTransition::default()),
    )
    .build();
    let mut tree = UiTree::new(element.clone());
    let node = tree.node_ids()[0];
    tree.pointer_moved(Point::new(5.0, 5.0), &[hover_region(node, 0.0, true)]);
    assert_eq!(
        tree.resolved_quad(node, &element).background,
        Some(argui_paint::Fill::Solid(Color::BLACK))
    );
    tree.advance_animations(argui_animation::Time::from_nanos(1));
    tree.advance_animations(argui_animation::Time::from_nanos(120_000_001));
    assert_eq!(
        tree.resolved_quad(node, &element).background,
        Some(argui_paint::Fill::Solid(Color::WHITE))
    );
}

fn hover_region(node: argui_ui::NodeId, y: f32, enabled: bool) -> HitRegion {
    HitRegion {
        node,
        bounds: Rect::new(Point::new(0.0, y), Size::new(100.0, 36.0)),
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
        shape: argui_ui::HitShape::Bounds,
        slop: argui_ui::HitTestStyle::default().slop,
        enabled,
        focus_policy: argui_ui::FocusPolicy::TabStop,
        cursor: CursorIcon::Pointer,
        gestures: argui_ui::GestureSet::EMPTY,
        window_drag: None,
    }
}

#[test]
fn dense_row_hover_leaves_no_trail_without_advancing_time() {
    let element = Button::new(
        "row",
        "Row",
        ButtonStyle::new(
            PaintStyle::new(QuadStyle::solid(Color::BLACK)),
            TextStyle::default(),
        )
        .hovered(QuadStyle::solid(Color::WHITE))
        .transition(argui_ui::StyleTransition::default())
        .instant_hover(),
    )
    .build();
    let mut tree = UiTree::new(element.clone());
    let node = tree.node_ids()[0];
    let regions = [hover_region(node, 0.0, true)];
    for _ in 0..10 {
        tree.pointer_moved(Point::new(5.0, 5.0), &regions);
        assert_eq!(
            tree.resolved_quad(node, &element).background,
            Some(argui_paint::Fill::Solid(Color::WHITE))
        );
        tree.pointer_moved(Point::new(500.0, 500.0), &regions);
        assert_eq!(
            tree.resolved_quad(node, &element).background,
            Some(argui_paint::Fill::Solid(Color::BLACK))
        );
    }
}

#[test]
fn themed_outline_button_has_a_visible_hover_surface() {
    let themes = shadcn(Color::srgb(0.2, 0.5, 0.9));
    let theme = themes.resolve(ColorScheme::Dark);
    let element = Button::new("close", "Close dialog", theme.outline_button()).build();
    let resting = element.paint.quad.background.clone();
    let mut tree = UiTree::new(element);
    let node = tree.node_ids()[0];
    tree.set_reduced_motion(true);
    tree.pointer_moved(
        Point::new(2.0, 2.0),
        &[HitRegion {
            node,
            bounds: Rect::new(Point::default(), Size::new(120.0, 36.0)),
            transform: Affine2D::IDENTITY,
            clips: ClipChain::default(),
            shape: argui_ui::HitShape::Bounds,
            slop: argui_ui::HitTestStyle::default().slop,
            enabled: true,
            focus_policy: argui_ui::FocusPolicy::TabStop,
            cursor: CursorIcon::Pointer,
            gestures: argui_ui::GestureSet::EMPTY,
            window_drag: None,
        }],
    );

    assert_ne!(
        tree.resolved_quad(node, tree.element_at(0).unwrap())
            .background,
        resting
    );
}

#[test]
fn headless_button_decodes_only_enabled_activation() {
    let tree = UiTree::new(Element::container([]));
    let clicked = UiEvent::new(
        tree.node_ids()[0],
        Some("save".into()),
        UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
    );
    assert_eq!(
        ButtonBehavior::new("save", "Save").action(&clicked),
        Some(ButtonAction::Activate)
    );
    assert_eq!(
        ButtonBehavior::new("save", "Save")
            .enabled(false)
            .action(&clicked),
        None
    );
    let other = UiEvent::new(
        tree.node_ids()[0],
        Some("other".into()),
        UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
    );
    assert_eq!(ButtonBehavior::new("save", "Save").action(&other), None);
}

#[test]
fn icon_and_custom_content_buttons_keep_label_but_hide_visual_content() {
    let themes = shadcn(Color::srgb(0.2, 0.5, 0.9));
    let theme = themes.resolve(ColorScheme::Dark);
    let custom = Element::text("icon");
    let button = Button::new("custom", "Accessible label", theme.button())
        .content(custom)
        .build();
    assert_eq!(button.children.len(), 1);
    assert!(button.children[0].semantic_hidden);
    assert_eq!(
        button.semantics.as_ref().unwrap().label.as_deref(),
        Some("Accessible label")
    );
    let icon = Button::icon(
        "icon-only",
        "Open settings",
        Element::text("⚙"),
        theme.ghost_button(),
    )
    .build();
    assert_eq!(icon.children.len(), 1);
    assert_eq!(
        icon.semantics.as_ref().unwrap().label.as_deref(),
        Some("Open settings")
    );
}

#[test]
fn busy_button_is_not_activatable_even_when_enabled() {
    let tree = UiTree::new(Element::container([]));
    let clicked = UiEvent::new(
        tree.node_ids()[0],
        Some("save".into()),
        UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
    );
    assert_eq!(
        ButtonBehavior::new("save", "Save")
            .busy(true)
            .action(&clicked),
        None
    );
    let decorated = ButtonBehavior::new("save", "Save")
        .busy(true)
        .decorate(argui_widgets::ButtonPart::Root, Element::container([]));
    assert!(!decorated.interaction.as_ref().unwrap().enabled);
    assert!(
        !decorated
            .interaction
            .as_ref()
            .unwrap()
            .focus_policy
            .is_focusable()
    );
    assert!(decorated.semantics.as_ref().unwrap().state.busy);
}

#[test]
fn direct_click_handlers_are_additive_target_only_and_use_normal_propagation() {
    let themes = shadcn(Color::srgb(0.2, 0.5, 0.9));
    let theme = themes.resolve(ColorScheme::Dark);
    let handler = |slot| EventHandler::from_identity(EventHandlerId::new(EventOwnerId(7), slot));
    let root = Button::new("save", "Save", theme.button())
        .on_click(handler(0))
        .on_click(handler(1))
        .build();
    assert_eq!(root.event_listeners.len(), 2);
    assert!(
        root.event_listeners
            .iter()
            .all(|listener| { listener.event == EventType::Click && listener.options.target_only })
    );

    let mut tree =
        UiTree::new(Element::container([root]).on(handler(2).listener(EventType::Click)));
    let button = tree
        .node_ids()
        .iter()
        .copied()
        .find(|node| tree.key(*node) == Some("save"))
        .unwrap();
    let deliveries = tree.event_deliveries(
        button,
        UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
    );
    assert_eq!(deliveries.len(), 3);
    assert_eq!(deliveries[0].phase(), EventPhase::Target);
    assert_eq!(deliveries[1].phase(), EventPhase::Target);
    assert_eq!(deliveries[2].phase(), EventPhase::Bubble);
}
