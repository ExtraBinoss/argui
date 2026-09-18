use argui_platform::{
    GlobalShortcut, GlobalShortcutConfigError, GlobalShortcutEvent, GlobalShortcutId,
    GlobalShortcutState,
};

#[test]
fn shortcut_values_keep_portable_accelerators_and_event_identity() {
    let shortcut = GlobalShortcut::new("spotlight", "CmdOrCtrl+Space");
    assert_eq!(shortcut.id.as_str(), "spotlight");
    assert_eq!(shortcut.accelerator, "CmdOrCtrl+Space");
    assert_eq!(
        GlobalShortcutEvent {
            id: shortcut.id,
            state: GlobalShortcutState::Pressed,
            activation_token: Some("portal-token".into()),
        },
        GlobalShortcutEvent {
            id: GlobalShortcutId::new("spotlight"),
            state: GlobalShortcutState::Pressed,
            activation_token: Some("portal-token".into()),
        }
    );
    assert_ne!(GlobalShortcutState::Pressed, GlobalShortcutState::Released);
}

#[test]
fn shortcut_configuration_errors_are_actionable() {
    assert_eq!(
        GlobalShortcutConfigError::EmptyId.to_string(),
        "global shortcut ids cannot be empty"
    );
    assert_eq!(
        GlobalShortcutConfigError::DuplicateId(GlobalShortcutId::new("open")).to_string(),
        "duplicate global shortcut id: open"
    );
    assert_eq!(
        GlobalShortcutConfigError::EmptyAccelerator(GlobalShortcutId::new("open")).to_string(),
        "global shortcut 'open' has an empty accelerator"
    );
    assert_eq!(
        GlobalShortcutConfigError::DuplicateAccelerator("Ctrl+Space".into()).to_string(),
        "duplicate global shortcut: Ctrl+Space"
    );
}
