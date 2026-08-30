use argui_core::Color;
use argui_ui::{ElementKind, Role};
use argui_widgets::{TablerIcon, WidgetAssets};

#[test]
fn tabler_assets_are_parsed_once_and_reused_by_vector_elements() {
    let assets = WidgetAssets::tabler(Color::rgb(0.1, 0.7, 0.4));
    assert_eq!(assets.assets().len(), 10);
    let icon = assets.icon(TablerIcon::Search, 19.0);
    assert!(icon.semantic_hidden);
    assert!(matches!(icon.kind, ElementKind::Vector { .. }));
    let labeled = assets.labeled_icon(TablerIcon::Close, 16.0, "Close");
    assert!(!labeled.semantic_hidden);
    assert_eq!(labeled.semantics.as_ref().unwrap().role, Role::Image);
    assert_ne!(
        assets.vector_id(TablerIcon::Search),
        assets.vector_id(TablerIcon::Close)
    );
}
