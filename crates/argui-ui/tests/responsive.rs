use argui_core::Size;
use argui_ui::{
    ContainerQuery, ContainerScopeId, Element, ScrollConfig, ScrollbarPartStyle, ScrollbarStyle,
    StateName, StyleCondition, StylePatch, TreeUpdate, UiTree, property,
};

#[test]
fn every_container_predicate_has_explicit_boundaries_and_scope() {
    let scope = ContainerScopeId::new("panel");
    assert_eq!(scope.as_str(), "panel");

    let cases = [
        (ContainerQuery::min_width(scope, 100.0), true),
        (ContainerQuery::max_width(scope, 100.0), false),
        (ContainerQuery::min_height(scope, 60.0), true),
        (ContainerQuery::max_height(scope, 60.0), false),
        (ContainerQuery::landscape(scope), true),
        (ContainerQuery::portrait(scope), false),
    ];
    for (query, expected) in cases {
        assert_eq!(query.scope(), scope);
        assert_eq!(query.matches(100.0, 60.0), expected);
    }
}

#[test]
fn paint_queries_and_compound_conditions_resolve_without_relayout() {
    let scope = ContainerScopeId::new("paint-panel");
    let selected = StateName::new("selected");
    let condition = StyleCondition::any([
        StyleCondition::state(selected),
        StyleCondition::container(ContainerQuery::min_width(scope, 100.0)),
    ]);
    let element = Element::container([])
        .container_scope(scope)
        .when(condition, StylePatch::new().set(property::Opacity, 0.4));
    let mut tree = UiTree::new(element.clone());
    let node = tree.node_ids()[0];

    tree.set_reduced_motion(true);
    assert_eq!(
        tree.resolve_container_queries(&[Size::new(120.0, 40.0)]),
        TreeUpdate::Paint
    );
    assert!((tree.resolved_quad(node, &element).opacity - 0.4).abs() < 0.001);
    assert_eq!(
        tree.resolve_container_queries(&[Size::new(120.0, 40.0)]),
        TreeUpdate::None
    );
    assert_eq!(
        tree.resolve_container_queries(&[Size::new(80.0, 40.0)]),
        TreeUpdate::Paint
    );
    assert!((tree.resolved_quad(node, &element).opacity - 1.0).abs() < 0.001);
}

#[test]
fn scrollbar_queries_are_discovered_on_track_and_thumb() {
    let scope = ContainerScopeId::new("scroll-host");
    let queried = |track: bool| {
        let conditional = ScrollbarPartStyle::new(Default::default()).when(
            ContainerQuery::portrait(scope),
            StylePatch::new().set(property::Opacity, 0.5),
        );
        let plain = ScrollbarPartStyle::new(Default::default());
        let scrollbar = if track {
            ScrollbarStyle::new(conditional, plain)
        } else {
            ScrollbarStyle::new(plain, conditional)
        };
        Element::container([])
            .container_scope(scope)
            .scroll_config(ScrollConfig::default().scrollbar(scrollbar))
    };

    assert!(UiTree::new(queried(true)).has_container_queries());
    assert!(UiTree::new(queried(false)).has_container_queries());
}
