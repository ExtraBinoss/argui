use argui_text::TextStyle;
use argui_ui::{
    Axes, Border, Color, ContainerQuery, ContainerScopeId, CornerRadii, Element, ElementKind,
    FlexDirection, Interaction, LayerStyle, Overflow, Role, ScrollConfig, ScrollbarPartStyle,
    ScrollbarStyle, Semantics, StylePatch, TreeUpdate, UiTree, VectorId, VisualState, length,
    percent, property,
};

#[test]
fn rust_builders_form_the_future_dsl_lowering_target() {
    let root = Element::row([Element::text("hello")
        .keyed("greeting")
        .width(percent(0.5))
        .text_style(TextStyle {
            weight: 700,
            ..TextStyle::default()
        })])
    .background(Color::rgb(0.1, 0.2, 0.3))
    .border(Border::all(2.0, Color::WHITE))
    .radius(CornerRadii::all(8.0))
    .paint_opacity(0.9)
    .overflow(Axes {
        x: Overflow::Hidden,
        y: Overflow::Hidden,
    });
    let mut tree = UiTree::new(root.clone());

    assert_eq!(tree.root().kind, ElementKind::Container);
    assert_eq!(tree.root().style.flex_direction, FlexDirection::Row);
    assert_eq!(tree.root().children.len(), 1);
    assert_eq!(tree.root().children[0].key.as_deref(), Some("greeting"));
    assert!(tree.root().paint.is_visible());
    assert!(tree.layout_dirty());
    tree.mark_layout_clean();
    assert!(!tree.layout_dirty());
    assert!(!tree.replace(root));
    assert!(tree.replace(Element::text("changed")));
    assert_eq!(tree.revision(), 1);
    assert!(tree.layout_dirty());
}

#[test]
fn vector_builder_keeps_fit_and_tint() {
    let element = Element::vector(VectorId(9))
        .vector_fit(argui_ui::ImageFit::Contain)
        .vector_color(argui_ui::Color::rgb(0.2, 0.4, 0.6));
    assert_eq!(
        element.kind,
        ElementKind::Vector {
            vector: VectorId(9),
            fit: argui_ui::ImageFit::Contain,
            color: argui_ui::Color::rgb(0.2, 0.4, 0.6),
        }
    );
}

#[test]
fn vector_tint_is_a_paint_only_update() {
    let mut tree = UiTree::new(Element::vector(VectorId(9)).vector_color(argui_ui::Color::WHITE));
    let node = tree.node_id_at(0);
    tree.mark_layout_clean();
    assert_eq!(
        tree.update(Element::vector(VectorId(9)).vector_color(argui_ui::Color::rgb(0.5, 0.5, 0.5))),
        TreeUpdate::Paint
    );
    assert_eq!(tree.node_id_at(0), node);
    assert!(!tree.layout_dirty());
}

#[test]
fn tree_updates_distinguish_paint_from_layout() {
    let base = Element::container([])
        .keyed("panel")
        .background(Color::rgb(0.1, 0.2, 0.3));
    let mut tree = UiTree::new(base.clone());
    tree.mark_layout_clean();
    let revision = tree.revision();
    let node = tree.node_id_at(0);

    assert_eq!(
        tree.update(base.clone().background(Color::rgb(0.3, 0.2, 0.1))),
        TreeUpdate::Paint
    );
    assert_eq!(tree.revision(), revision);
    assert_eq!(tree.node_id_at(0), node);
    assert!(!tree.layout_dirty());

    assert_eq!(
        tree.update(
            base.clone()
                .text_effect(LayerStyle::new(Default::default()).opacity(0.7))
        ),
        TreeUpdate::Paint
    );
    assert_eq!(tree.revision(), revision);

    assert_eq!(tree.update(base.width(length(200.0))), TreeUpdate::Layout);
    assert!(tree.revision() > revision);
    assert!(tree.layout_dirty());
}

#[test]
fn enabling_retained_interaction_only_requires_repaint() {
    let mut tree =
        UiTree::new(Element::container([]).interaction(Interaction::blocker().enabled(false)));
    assert_eq!(
        tree.update(Element::container([]).interaction(Interaction::blocker())),
        TreeUpdate::Paint
    );
}

#[test]
fn cloned_subtree_is_classified_in_constant_work() {
    let subtree = Element::column((0..10_000).map(|index| Element::text(index.to_string())));
    let root = Element::column([subtree]);
    let mut tree = UiTree::new(root.clone());

    assert_eq!(tree.update(root), TreeUpdate::None);
    assert_eq!(tree.update_stats().visited, 1);
    assert_eq!(tree.update_stats().shared_subtrees, 1);
}

#[test]
fn one_changed_branch_skips_large_shared_siblings() {
    let large = Element::column((0..20_000).map(|index| Element::text(index.to_string())));
    let root = Element::row([large.clone(), Element::text("old")]);
    let mut tree = UiTree::new(root);
    let changed = Element::row([large, Element::text("new")]);

    assert_eq!(tree.update(changed), TreeUpdate::Layout);
    assert!(
        tree.update_stats().visited <= 3,
        "{:?}",
        tree.update_stats()
    );
    assert_eq!(tree.update_stats().shared_subtrees, 1);
}

#[test]
fn update_classification_covers_empty_semantic_visual_and_structural_paths() {
    let base = Element::container([]);
    let mut tree = UiTree::new(base.clone());
    let mut equal_copy = base.clone();
    equal_copy.inspectable = true;
    assert_eq!(tree.update(equal_copy), TreeUpdate::None);
    assert_eq!(
        tree.update(base.clone().semantics(Semantics::new(Role::Group))),
        TreeUpdate::Semantics
    );
    assert_eq!(
        tree.update(base.clone().semantic_hidden(true)),
        TreeUpdate::Semantics
    );
    assert_eq!(
        tree.update(base.clone().background(Color::WHITE)),
        TreeUpdate::Paint
    );
    assert_eq!(
        tree.update(base.clone().overflow(Axes {
            x: Overflow::Hidden,
            y: Overflow::Hidden
        })),
        TreeUpdate::Layout
    );
    assert_eq!(
        tree.update(
            base.clone()
                .overflow(Axes {
                    x: Overflow::Hidden,
                    y: Overflow::Auto,
                })
                .scroll_config(ScrollConfig::default()),
        ),
        TreeUpdate::Layout
    );
    assert_eq!(
        tree.update(Element::image(argui_ui::ImageId(1))),
        TreeUpdate::Layout
    );
    assert_eq!(
        UiTree::new(Element::text("old")).update(Element::text("new")),
        TreeUpdate::Layout
    );
    assert_eq!(
        UiTree::new(Element::image(argui_ui::ImageId(1)))
            .update(Element::image(argui_ui::ImageId(2))),
        TreeUpdate::Paint
    );
    assert_eq!(
        UiTree::new(Element::container([Element::text("old")]))
            .update(Element::container([Element::text("new")])),
        TreeUpdate::Layout
    );
}

#[test]
fn responsive_and_scroll_configuration_changes_have_exact_invalidation() {
    let scope = ContainerScopeId::new("classification");
    let base = Element::container([]);

    assert_eq!(
        UiTree::new(base.clone()).update(base.clone().container_scope(scope)),
        TreeUpdate::Layout
    );
    assert_eq!(
        UiTree::new(base.clone()).update(base.clone().scroll_config(ScrollConfig::default())),
        TreeUpdate::Layout
    );
    assert_eq!(
        UiTree::new(base.clone().scroll_config(ScrollConfig::default())).update(
            base.clone()
                .scroll_config(ScrollConfig::default().multiplier(2.0))
        ),
        TreeUpdate::Paint
    );

    let plain_scrollbar = ScrollbarStyle::new(
        ScrollbarPartStyle::new(Default::default()),
        ScrollbarPartStyle::new(Default::default()),
    );
    let responsive_scrollbar = ScrollbarStyle::new(
        ScrollbarPartStyle::new(Default::default()).when(
            ContainerQuery::min_width(scope, 100.0),
            StylePatch::new().set(property::Opacity, 0.5),
        ),
        ScrollbarPartStyle::new(Default::default()),
    );
    let plain = base
        .clone()
        .container_scope(scope)
        .scroll_config(ScrollConfig::default().scrollbar(plain_scrollbar));
    let responsive = base
        .clone()
        .container_scope(scope)
        .scroll_config(ScrollConfig::default().scrollbar(responsive_scrollbar));
    assert_eq!(
        UiTree::new(plain.clone()).update(responsive.clone()),
        TreeUpdate::Layout
    );
    assert_eq!(UiTree::new(responsive).update(plain), TreeUpdate::Layout);

    assert_eq!(
        UiTree::new(base.clone()).update(base.when(
            VisualState::Hovered,
            StylePatch::new().set(property::WidthPx, 120.0),
        )),
        TreeUpdate::Layout
    );
}
