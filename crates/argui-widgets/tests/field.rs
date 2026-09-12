use argui_core::{Color, ColorScheme};
use argui_ui::{ClickEvent, Element, UiEvent, UiEventKind, UiTree};
use argui_widgets::{Field, Input, shadcn};

#[test]
fn nested_controls_keep_their_label_help_error_and_disabled_state() {
    let palette = shadcn(Color::BLACK);
    let theme = palette.resolve(ColorScheme::Light);
    let node = UiTree::new(Element::container([])).node_ids()[0];
    for enabled in [false, true] {
        for error in [None, Some("Invalid email".into())] {
            let input = Input::new("email", "", "you@example.com", theme.input())
                .leading(Element::text("@"), 24.0)
                .build();
            let mut field = Field::new("contact", "Email", "email", input);
            field.description = Some("We will keep it private".into());
            field.error = error.clone();
            field.enabled = enabled;
            field.required = true;
            let tree = UiTree::new(field.build(theme));
            assert!(tree.semantic_diagnostics().is_empty());
            let semantics = tree.semantic_tree(&[], 1.0);
            let editor = semantics
                .nodes
                .iter()
                .find(|node| node.semantics.role == argui_ui::Role::TextInput)
                .unwrap();
            assert_eq!(editor.semantics.relations.labelled_by.len(), 1);
            assert_eq!(
                editor.semantics.relations.described_by.len(),
                1 + usize::from(error.is_some())
            );
            assert_eq!(editor.semantics.state.invalid, error.is_some());
            assert_eq!(editor.semantics.state.disabled, !enabled);
            assert!(editor.semantics.state.required);
            for key in ["contact::label", "other"] {
                let event = UiEvent::new(
                    node,
                    Some(key.into()),
                    UiEventKind::Click(ClickEvent::accessibility()),
                );
                assert_eq!(
                    field.focus_target(&event),
                    (enabled && key == "contact::label").then_some("email")
                );
            }
        }
    }
    assert!(
        UiTree::new(
            Field::new(
                "f",
                "Name",
                "name",
                Input::new("name", "", "", theme.input()).build()
            )
            .build(theme)
        )
        .semantic_diagnostics()
        .is_empty()
    );
}
