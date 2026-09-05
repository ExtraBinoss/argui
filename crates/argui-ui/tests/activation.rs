use argui_core::{Key, KeyInput, KeyState, Modifiers, Point, PointerEvent, PointerPhase};
use argui_ui::{ActivationSource, ClickEvent};

#[test]
fn click_metadata_is_available_for_pointer_keyboard_and_accessibility_activation() {
    let pointer = ClickEvent::pointer(
        PointerEvent::mouse(PointerPhase::Released, Point::new(24.0, 18.0)),
        2,
    );
    assert_eq!(pointer.pointer_id(), Some(argui_core::PointerId::MOUSE));
    assert_eq!(pointer.position(), Some(Point::new(24.0, 18.0)));
    assert_eq!(pointer.count, 2);

    let keyboard = ClickEvent::keyboard(KeyInput {
        key: Key::Enter,
        state: KeyState::Pressed,
        modifiers: Modifiers {
            shift: true,
            ..Modifiers::default()
        },
        repeat: false,
        text: None,
    });
    assert!(matches!(keyboard.source, ActivationSource::Keyboard(_)));
    assert_eq!(keyboard.pointer_id(), None);
    assert_eq!(keyboard.position(), None);
    assert!(keyboard.modifiers().shift);

    let accessible = ClickEvent::accessibility();
    assert!(matches!(accessible.source, ActivationSource::Accessibility));
    assert_eq!(accessible.modifiers(), Modifiers::default());
}
