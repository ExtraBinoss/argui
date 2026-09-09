use argui_ui::{Element, Role, SemanticReferenceError, Semantics, UiTree};

fn field() -> Element {
    Element::column([
        Element::text("Name").keyed("label"),
        Element::text("Required").keyed("help"),
        Element::container([])
            .keyed("input")
            .semantics(Semantics::new(Role::TextInput))
            .labelled_by(["label", "label"])
            .described_by(["help"]),
    ])
    .semantic_scope()
}

#[test]
fn repeated_instances_resolve_locally_and_rebuilds_remove_dangling_relations() {
    let mut tree = UiTree::new(Element::column([field().keyed("a"), field().keyed("b")]));
    let semantic = tree.semantic_tree(&[], 1.0);
    let inputs: Vec<_> = semantic
        .nodes
        .iter()
        .filter(|node| node.semantics.role == Role::TextInput)
        .collect();
    assert_eq!(inputs.len(), 2);
    assert_ne!(
        inputs[0].semantics.relations.labelled_by,
        inputs[1].semantics.relations.labelled_by
    );
    for input in inputs {
        assert_eq!(input.semantics.relations.labelled_by.len(), 1);
        let label = semantic
            .node(input.semantics.relations.labelled_by[0])
            .unwrap();
        assert_eq!(label.semantics.label.as_deref(), Some("Name"));
        assert_eq!(input.semantics.relations.described_by.len(), 1);
    }
    assert!(tree.semantic_diagnostics().is_empty());
    let mut next = field().keyed("a");
    next.children.remove(0);
    tree.update(Element::column([next]));
    let updated = tree.semantic_tree(&[], 1.0);
    let input = updated
        .nodes
        .iter()
        .find(|node| node.semantics.role == Role::TextInput)
        .unwrap();
    assert!(input.semantics.relations.labelled_by.is_empty());
    assert!(!semantic.diff(&updated).removed.is_empty());
    assert!(
        tree.semantic_diagnostics()
            .iter()
            .all(|error| error.error == SemanticReferenceError::Missing)
    );
}

#[test]
fn missing_ambiguous_hidden_and_self_references_never_choose_a_target() {
    let tree = UiTree::new(Element::column([
        Element::text("First").keyed("duplicate"),
        Element::text("Second").keyed("duplicate"),
        Element::text("Secret")
            .keyed("hidden")
            .semantic_hidden(true),
        Element::text("Source").keyed("source").labelled_by([
            "duplicate",
            "missing",
            "hidden",
            "source",
        ]),
    ]));
    let errors = tree.semantic_diagnostics();
    assert_eq!(
        errors.iter().map(|error| error.error).collect::<Vec<_>>(),
        [
            SemanticReferenceError::Ambiguous,
            SemanticReferenceError::Missing,
            SemanticReferenceError::Missing,
            SemanticReferenceError::SelfReference,
        ]
    );
    let semantic = tree.semantic_tree(&[], 1.0);
    assert!(
        semantic
            .nodes
            .iter()
            .all(|node| node.semantics.relations.labelled_by.is_empty())
    );
}

#[test]
fn controlled_popup_and_active_option_resolve_by_stable_key() {
    let mut tree = UiTree::new(
        Element::column([
            Element::container([])
                .keyed("entry")
                .semantics(Semantics::new(Role::ComboBox))
                .controls(["options"])
                .active_descendant("option"),
            Element::column([Element::text("Choice")
                .keyed("option")
                .semantics(Semantics::new(Role::Option).label("Choice"))])
            .keyed("options")
            .semantics(Semantics::new(Role::ListBox)),
        ])
        .semantic_scope(),
    );
    let first = tree.semantic_tree(&[], 1.0);
    let entry = first
        .nodes
        .iter()
        .find(|node| node.semantics.role == Role::ComboBox)
        .unwrap();
    assert_eq!(entry.semantics.relations.controls.len(), 1);
    assert_eq!(
        first
            .node(entry.semantics.relations.active_descendant.unwrap())
            .unwrap()
            .semantics
            .label
            .as_deref(),
        Some("Choice")
    );
    let mut next = tree.root().clone();
    next.children[1].children.clear();
    tree.update(next);
    let next = tree.semantic_tree(&[], 1.0);
    assert!(
        next.node(entry.id)
            .unwrap()
            .semantics
            .relations
            .active_descendant
            .is_none()
    );
}
