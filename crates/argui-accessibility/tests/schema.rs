use argui_accessibility::{
    LiveRegion, Orientation, Role, SemanticAction, SemanticState, Semantics,
};

#[test]
fn semantic_builders_preserve_explicit_metadata_and_deduplicate_actions() {
    let semantics = Semantics::new(Role::ListItem)
        .label("Result")
        .description("Search result")
        .action(SemanticAction::Click)
        .action(SemanticAction::Click)
        .state(SemanticState {
            selected: true,
            ..SemanticState::default()
        })
        .live(LiveRegion::Polite)
        .orientation(Orientation::Vertical)
        .level(2)
        .position_in_set(3, 10);

    assert_eq!(semantics.actions, vec![SemanticAction::Click]);
    assert!(semantics.state.selected);
    assert_eq!(semantics.orientation, Some(Orientation::Vertical));
    assert_eq!(semantics.level, Some(2));
    assert_eq!(semantics.position_in_set, Some(3));
    assert_eq!(semantics.set_size, Some(10));
}
