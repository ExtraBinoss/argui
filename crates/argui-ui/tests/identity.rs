use argui_animation::{Duration, Time, Transition, Tween};
use argui_core::{Affine2D, Point, Rect, Size};
use argui_paint::{ClipChain, Color};
use argui_text::TextStyle;
use argui_ui::{
    CaretStyle, CursorIcon, Element, FocusPolicy, FocusRequest, FocusScope, GestureSet, HitRegion,
    InitialFocus, Interaction, RetainedIdentity, ScrollConfig, StylePatch, StyleTransition,
    TextEditorSpec, TextInputFilter, TextSelection, TextSelectionRequest, UiCommand, UiTree,
    VisualState, property,
};

fn source(site: u64) -> RetainedIdentity {
    RetainedIdentity::new(7, site)
}

fn input(site: u64) -> Element {
    Element::text_editor(TextEditorSpec {
        value: String::from("retained text"),
        placeholder: String::new(),
        multiline: false,
        read_only: false,
        filter: TextInputFilter::default(),
        text: TextStyle::default(),
        placeholder_text: TextStyle::default(),
        selection: Color::WHITE,
        caret: CaretStyle::default(),
    })
    .interaction(Interaction::default().focus_policy(FocusPolicy::TabStop))
    .retained_identity(source(site))
}

fn focus_region(node: argui_ui::NodeId) -> HitRegion {
    HitRegion {
        node,
        bounds: Rect::new(Point::default(), Size::new(200.0, 40.0)),
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
        shape: argui_ui::HitShape::Bounds,
        slop: argui_ui::HitTestStyle::default().slop,
        enabled: true,
        focus_policy: FocusPolicy::TabStop,
        cursor: CursorIcon::Text,
        gestures: GestureSet::EMPTY,
        window_drag: None,
    }
}

fn stateful_tree(with_prefix: bool) -> Element {
    let mut children = Vec::new();
    if with_prefix {
        children.push(Element::text("new sibling").retained_identity(source(5)));
    }
    children.push(input(10));
    children.push(
        Element::container([])
            .scroll_config(ScrollConfig::default())
            .retained_identity(source(20)),
    );
    Element::column(children).retained_identity(source(1))
}

#[test]
fn source_insertion_preserves_focus_selection_editing_and_scroll_state() {
    let mut tree = UiTree::new(stateful_tree(false));
    let input_id = tree.node_ids()[1];
    let scroll_id = tree.node_ids()[2];
    tree.sync_focus(
        &[focus_region(input_id)],
        Some(FocusRequest::Focus(input_id.into())),
    );
    tree.select_text(TextSelectionRequest::new(input_id, TextSelection::All));
    assert!(tree.set_scroll_offset(scroll_id, Point::new(12.0, 80.0)));

    tree.update(stateful_tree(true));

    assert_eq!(tree.node_ids()[2], input_id);
    assert_eq!(tree.node_ids()[3], scroll_id);
    assert_eq!(tree.focused_node(), Some(input_id));
    assert_eq!(tree.text_input_selection(input_id), Some((0, 13)));
    assert_eq!(tree.text_input_value(input_id), Some("retained text"));
    assert_eq!(tree.scroll_offset(scroll_id), Point::new(12.0, 80.0));
}

#[test]
fn repeater_keys_preserve_items_across_reordering() {
    let item = |key| Element::container([]).retained_identity(source(30).with_unsigned_key(key));
    let mut tree =
        UiTree::new(Element::row([item(1), item(2), item(3)]).retained_identity(source(1)));
    let original = tree.node_ids()[1..].to_vec();

    tree.update(Element::row([item(3), item(1), item(2)]).retained_identity(source(1)));

    assert_eq!(
        tree.node_ids()[1..],
        [original[2], original[0], original[1]]
    );
}

#[test]
fn retained_identity_target_requires_a_unique_current_node() {
    let identity = source(50);
    let mut tree = UiTree::new(Element::container([
        Element::container([]).retained_identity(identity.clone()),
        Element::container([]).retained_identity(identity.clone()),
    ]));
    assert_eq!(tree.resolve_node(&identity.clone().into()), None);
    assert_eq!(tree.resolve_node(&source(51).into()), None);

    tree.update(Element::container([
        Element::container([]).retained_identity(identity.clone()),
        Element::container([]).retained_identity(source(52)),
    ]));
    assert_eq!(tree.resolve_node(&identity.into()), tree.node_id_at(1));
}

#[test]
fn retained_identity_focus_target_respects_active_trap() {
    let outside_identity = source(61);
    let inside_identity = source(62);
    let mut tree = UiTree::new(Element::container([
        input(61),
        Element::container([input(62)]).focus_scope(FocusScope::trapped(InitialFocus::First)),
    ]));
    let outside = tree.node_id_at(1).unwrap();
    let inside = tree.node_id_at(3).unwrap();
    let regions = [focus_region(outside), focus_region(inside)];
    tree.sync_focus(&regions, None);
    assert_eq!(tree.focused_node(), Some(inside));

    tree.sync_focus(&regions, Some(FocusRequest::Focus(outside_identity.into())));
    assert_eq!(tree.focused_node(), Some(inside));
    tree.sync_focus(&regions, Some(FocusRequest::Focus(inside_identity.into())));
    assert_eq!(tree.focused_node(), Some(inside));
}

#[test]
fn retained_identity_targets_text_replacement_and_selection() {
    let mut tree = UiTree::new(input(63));
    let node = tree.node_id_at(0).unwrap();
    tree.apply_command(UiCommand::ReplaceText {
        target: source(63).into(),
        value: "updated".into(),
    });
    assert_eq!(tree.text_input_value(node), Some("updated"));
    tree.select_text(TextSelectionRequest::new(source(63), TextSelection::All));
    assert_eq!(tree.text_input_selection(node), Some((0, 7)));
}

#[test]
fn a_different_branch_site_never_inherits_retained_editor_state() {
    let mut tree = UiTree::new(Element::container([input(40)]).retained_identity(source(1)));
    let old = tree.node_ids()[1];
    tree.select_text(TextSelectionRequest::new(old, TextSelection::All));

    tree.update(Element::container([input(41)]).retained_identity(source(1)));

    let replacement = tree.node_ids()[1];
    assert_ne!(replacement, old);
    assert_eq!(tree.text_input_selection(replacement), None);
}

fn animated(site: u64) -> Element {
    Element::container([])
        .background(Color::BLACK)
        .interaction(Interaction::default())
        .when(
            VisualState::Hovered,
            StylePatch::new().set(property::BackgroundColor, Color::WHITE),
        )
        .transition(StyleTransition::new(Transition::tween(Tween::new(
            Duration::from_millis(100),
        ))))
        .retained_identity(source(site))
}

#[test]
fn source_insertion_keeps_an_active_transition_on_the_same_node() {
    let root = Element::container([animated(50)]).retained_identity(source(1));
    let mut tree = UiTree::new(root);
    let node = tree.node_ids()[1];
    tree.pointer_moved(Point::new(10.0, 10.0), &[focus_region(node)]);
    tree.advance_animations(Time::from_nanos(1));
    tree.advance_animations(Time::from_nanos(25_000_001));
    assert!(tree.wants_animation_frame());

    tree.update(
        Element::container([
            Element::text("prefix").retained_identity(source(49)),
            animated(50),
        ])
        .retained_identity(source(1)),
    );

    assert_eq!(tree.node_ids()[2], node);
    assert!(tree.wants_animation_frame());
    assert_ne!(tree.visual_revision(node), 0);
}
