use argui_core::{Affine2D, Point, Rect, Size};
use argui_paint::{ClipChain, PaintStyle};
use argui_text::TextStyle;
use argui_ui::{
    CursorIcon, Display, Element, FocusRequest, GestureSet, HitRegion, Interaction, Role,
    SemanticAction, SemanticValue, Semantics, TreeUpdate, UiTree,
};
use argui_widgets::{Button, ButtonStyle, Input, InputStyle};

#[test]
fn semantic_changes_do_not_dirty_layout_or_paint() {
    let base = Element::container([]).semantics(Semantics::new(Role::Group).label("Before"));
    let mut tree = UiTree::new(base.clone());
    tree.mark_layout_clean();

    assert_eq!(
        tree.update(base.semantics(Semantics::new(Role::Group).label("After"))),
        TreeUpdate::Semantics
    );
    assert!(!tree.layout_dirty());
}

#[test]
fn buttons_publish_one_actionable_semantic_leaf() {
    let button = Button::new(
        "save",
        "Save changes",
        ButtonStyle::new(PaintStyle::default(), TextStyle::default()),
    )
    .build();
    let tree = UiTree::new(button);
    let root = tree.node_id_at(0).unwrap();
    let semantics = tree.semantic_tree(
        &[(
            root,
            Rect::new(Point::new(4.0, 8.0), Size::new(120.0, 36.0)),
        )],
        2.0,
    );

    assert_eq!(semantics.nodes.len(), 1);
    assert_eq!(semantics.nodes[0].semantics.role, Role::Button);
    assert_eq!(
        semantics.nodes[0].semantics.label.as_deref(),
        Some("Save changes")
    );
    assert!(
        semantics.nodes[0]
            .semantics
            .actions
            .contains(&SemanticAction::Click)
    );
    assert_eq!(semantics.nodes[0].bounds.origin, Point::new(8.0, 16.0));
}

#[test]
fn focus_falls_back_to_the_root_when_a_focused_node_is_semantically_hidden() {
    let child = Element::container([])
        .interaction(Interaction::default().focusable(true))
        .semantics(Semantics::new(Role::Button).label("Hidden"));
    let mut tree = UiTree::new(Element::column([child.clone()]));
    let child_id = tree.node_id_at(1).unwrap();
    let region = HitRegion {
        node: child_id,
        bounds: Rect::new(Point::default(), Size::new(100.0, 30.0)),
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
        shape: argui_ui::HitShape::Bounds,
        slop: argui_ui::HitTestStyle::default().slop,
        enabled: true,
        focusable: true,
        cursor: CursorIcon::Auto,
        gestures: GestureSet::NONE,
        window_drag: None,
    };
    tree.sync_focus(&[region], Some(FocusRequest::Focus(child_id.into())));
    tree.update(Element::column([child.semantic_hidden(true)]));

    let semantic_tree = tree.semantic_tree(&[], 1.0);
    assert_eq!(semantic_tree.focus, semantic_tree.root);
    assert_eq!(semantic_tree.nodes.len(), 1);
}

#[test]
fn structural_containers_promote_semantic_descendants() {
    let tree = UiTree::new(Element::column([Element::container([Element::text(
        "Promoted label",
    )])]));
    let root = tree.node_id_at(0).unwrap();
    let label = tree.node_id_at(2).unwrap();
    let semantics = tree.semantic_tree(&[], 1.0);

    assert_eq!(semantics.nodes.len(), 2);
    assert_eq!(semantics.nodes[0].id.get(), root.get());
    assert_eq!(semantics.nodes[0].children[0].get(), label.get());
    assert_eq!(semantics.nodes[1].semantics.role, Role::Text);
}

#[test]
fn display_none_removes_the_entire_semantic_subtree() {
    let tree = UiTree::new(Element::column([
        Element::container([Element::text("Invisible")])
            .display(Display::None)
            .semantics(Semantics::new(Role::Group).label("Hidden group")),
        Element::text("Visible"),
    ]));
    let semantics = tree.semantic_tree(&[], 1.0);

    assert_eq!(semantics.nodes.len(), 2);
    assert_eq!(
        semantics.nodes[1].semantics.label.as_deref(),
        Some("Visible")
    );
}

#[test]
fn text_inputs_publish_retained_values_and_disabled_state() {
    let style = InputStyle::new(PaintStyle::default(), TextStyle::default());
    let input = Input::new("query", "Argui", "Search", style)
        .build()
        .interaction(Interaction::default().focusable(true).enabled(false));
    let tree = UiTree::new(input);
    let semantics = tree.semantic_tree(&[], 1.0);
    let node = &semantics.nodes[0];

    assert_eq!(node.semantics.role, Role::TextInput);
    assert_eq!(
        node.semantics.value,
        Some(SemanticValue::Text("Argui".into()))
    );
    assert!(node.semantics.state.disabled);
}

#[test]
fn hidden_branches_skip_all_descendants_without_disturbing_siblings() {
    let tree = UiTree::new(Element::column([
        Element::column([Element::text("Hidden child")]).semantic_hidden(true),
        Element::text("Visible child"),
    ]));
    let semantics = tree.semantic_tree(&[], 1.0);

    assert_eq!(semantics.nodes.len(), 2);
    assert_eq!(
        semantics.nodes[1].semantics.label.as_deref(),
        Some("Visible child")
    );
}
