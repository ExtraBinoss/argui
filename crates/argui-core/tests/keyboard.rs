use argui_core::{ImeInput, Key, KeyInput, KeyState, Modifiers};

#[test]
fn keyboard_and_ime_data_remain_platform_independent() {
    let input = KeyInput {
        key: Key::Character("v".into()),
        state: KeyState::Pressed,
        modifiers: Modifiers {
            control: true,
            ..Modifiers::default()
        },
        repeat: false,
        text: Some("v".into()),
    };
    assert!(input.modifiers.command());
    assert_eq!(ImeInput::Commit("é".into()), ImeInput::Commit("é".into()));
}
