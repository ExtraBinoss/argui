use argui_core::{Color, ColorScheme, Size};
use argui_layout::LayoutEngine;
use argui_text::TextEngine;
use argui_ui::{ClickEvent, Element, Role, UiEvent, UiEventKind, UiTree};
use argui_widgets::{Input, Label, shadcn};

#[test]
fn label_names_the_control_without_adding_a_tab_stop_or_losing_other_semantics() {
    let themes = shadcn(Color::WHITE);
    let theme = themes.resolve(ColorScheme::Light);
    let label = Label::new("name-label", "Display name", "name");
    let input = label.associate(
        Input::new("old-key", "Ada", "", theme.input())
            .label("Old name")
            .description("Public profile")
            .build(),
    );
    let tree = UiTree::new(Element::column([label.build(theme), input]));
    assert!(tree.semantic_diagnostics().is_empty());
    let semantic = tree.semantic_tree(&[], 1.0);
    let control = semantic
        .nodes
        .iter()
        .find(|node| node.semantics.role == Role::TextInput)
        .unwrap();
    let name = semantic
        .nodes
        .iter()
        .find(|node| node.semantics.label.as_deref() == Some("Display name"))
        .unwrap();
    assert_eq!(control.semantics.relations.labelled_by, [name.id]);
    assert!(control.semantics.label.is_none());
    assert_eq!(
        control.semantics.description.as_deref(),
        Some("Public profile")
    );
    assert_eq!(
        semantic
            .nodes
            .iter()
            .filter(|node| node.semantics.focus_policy.is_tab_stop())
            .count(),
        1
    );
    let plain = label.associate(Element::container([]));
    assert_eq!(plain.key.as_deref(), Some("name"));
}

#[test]
fn only_enabled_label_clicks_request_focus_and_long_labels_wrap() {
    let themes = shadcn(Color::WHITE);
    let theme = themes.resolve(ColorScheme::Dark);
    let node = UiTree::new(Element::container([])).node_ids()[0];
    for enabled in [true, false] {
        let label = Label::new(
            "label",
            "A sufficiently long name for a narrow field",
            "field",
        )
        .enabled(enabled);
        for (key, kind, clicks) in [
            (
                Some("label"),
                UiEventKind::Click(ClickEvent::accessibility()),
                true,
            ),
            (
                Some("field"),
                UiEventKind::Click(ClickEvent::accessibility()),
                false,
            ),
            (None, UiEventKind::Click(ClickEvent::accessibility()), false),
            (Some("label"), UiEventKind::Focused, false),
        ] {
            let event = UiEvent::new(node, key.map(Into::into), kind);
            assert_eq!(
                label.focus_target(&event),
                (enabled && clicks).then_some("field")
            );
        }
        let root = label.build(theme);
        assert!(
            !root
                .interaction
                .as_ref()
                .unwrap()
                .focus_policy
                .is_tab_stop()
        );
        assert_eq!(root.interaction.as_ref().unwrap().enabled, enabled);
        let mut tree = UiTree::new(root);
        let layout = LayoutEngine::new()
            .compute(&mut tree, &mut TextEngine::new(), Size::new(150.0, 200.0))
            .unwrap();
        assert!(layout.nodes[0].bounds.size.height > 20.0);
        assert!(layout.nodes[0].bounds.size.width <= 150.0);
    }
}
