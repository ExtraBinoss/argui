pub fn keys() -> Vec<(gtk::gdk::keys::Key, argui_core::Key)> {
    use argui_core::Key;
    use gtk::gdk::keys::constants as keys;
    let mut keys = vec![
        (keys::Page_Up, Key::PageUp),
        (keys::Page_Down, Key::PageDown),
        (keys::Menu, Key::ContextMenu),
        (keys::Left, Key::ArrowLeft),
        (keys::Right, Key::ArrowRight),
        (keys::Up, Key::ArrowUp),
        (keys::Down, Key::ArrowDown),
        (keys::Home, Key::Home),
        (keys::End, Key::End),
        (keys::BackSpace, Key::Backspace),
        (keys::Delete, Key::Delete),
        (keys::Return, Key::Enter),
        (keys::Tab, Key::Tab),
        (keys::Escape, Key::Escape),
        (keys::a, Key::Character("a".into())),
    ];
    keys.extend((1..=24).map(|number| {
        (
            gtk::gdk::keys::Key::from_name(&format!("F{number}")),
            Key::Function(number),
        )
    }));
    keys
}
pub fn send() {
    use gtk::{gdk, prelude::*};
    let window = gtk::Window::list_toplevels()
        .into_iter()
        .filter_map(|widget| widget.downcast::<gtk::Window>().ok())
        .find(|window| window.title().as_deref() == Some("Argui updated lifecycle check"))
        .expect("the lifecycle window is mounted");
    send_pointer(&window);
    for (key, _) in keys() {
        for (kind, signal) in [
            (gdk::EventType::KeyPress, "key-press-event"),
            (gdk::EventType::KeyRelease, "key-release-event"),
        ] {
            let mut event = gdk::Event::new(kind).downcast::<gdk::EventKey>().unwrap();
            event.as_mut().keyval = *key;
            event.as_mut().send_event = 1;
            window.emit_by_name::<bool>(signal, &[&*event]);
        }
    }
}

fn buttons() -> [(u32, argui_core::PointerButton, u16); 5] {
    use argui_core::PointerButton;
    [
        (1, PointerButton::Primary, 1),
        (2, PointerButton::Middle, 4),
        (3, PointerButton::Secondary, 2),
        (8, PointerButton::Other(8), 8192),
        (11, PointerButton::Other(11), 0),
    ]
}

fn send_pointer(window: &gtk::Window) {
    use gtk::{gdk, prelude::*};
    for (button, _, _) in buttons() {
        for (kind, signal) in [
            (gdk::EventType::ButtonPress, "button-press-event"),
            (gdk::EventType::ButtonRelease, "button-release-event"),
        ] {
            let mut event = gdk::Event::new(kind)
                .downcast::<gdk::EventButton>()
                .unwrap();
            event.as_mut().button = button;
            event.as_mut().time = button;
            event.as_mut().send_event = 1;
            window.emit_by_name::<bool>(signal, &[&*event]);
        }
    }
    let mut event = gdk::Event::new(gdk::EventType::Scroll)
        .downcast::<gdk::EventScroll>()
        .unwrap();
    event.as_mut().direction = gdk::ffi::GDK_SCROLL_SMOOTH;
    event.as_mut().delta_x = 2.0;
    event.as_mut().delta_y = -3.0;
    window.emit_by_name::<bool>("scroll-event", &[&*event]);
}

pub fn assert_pointer(pointer: &[argui_core::PointerEvent], wheel: &[argui_core::ScrollDelta]) {
    use argui_core::{Point, PointerPhase, ScrollDelta};
    let expected: Vec<_> = buttons()
        .into_iter()
        .flat_map(|(_, button, mask)| {
            [
                (Some(button), PointerPhase::Pressed, mask),
                (Some(button), PointerPhase::Released, 0),
            ]
        })
        .collect();
    assert_eq!(
        pointer
            .iter()
            .map(|event| (event.button, event.phase, event.buttons))
            .collect::<Vec<_>>(),
        expected
    );
    assert_eq!(wheel, [ScrollDelta::Lines(Point::new(-2.0, 3.0))]);
}
