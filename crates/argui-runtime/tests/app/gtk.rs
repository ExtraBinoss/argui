use std::{cell::Cell, rc::Rc, time::Duration};
use web_time::Instant;

#[derive(Clone, Default)]
pub struct AnimationProbe(Rc<AnimationState>);

#[derive(Default)]
struct AnimationState {
    frames: Cell<usize>,
    first: Cell<Option<Instant>>,
    span: Cell<Option<Duration>>,
}

impl AnimationProbe {
    pub fn frame(&self) {
        let now = Instant::now();
        let frame = self.0.frames.get();
        if frame == 0 {
            self.0.first.set(Some(now));
        }
        if frame < 12 {
            self.0.frames.set(frame + 1);
            if frame == 11 {
                self.0.span.set(self.0.first.get().map(|first| now - first));
            }
        }
    }

    pub fn wants_frame(&self) -> bool {
        self.0.frames.get() < 12
    }

    pub fn assert_smooth(&self) {
        let span = self
            .0
            .span
            .get()
            .expect("GTK delivered all animation frames");
        eprintln!("GTK delivered 12 consecutive animation frames in {span:?}");
        assert!(
            span < Duration::from_millis(500),
            "GTK animation redraws stalled for {span:?}"
        );
    }
}

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
    use gtk::{gdk, glib::translate::ToGlibPtr, prelude::*};
    let window = gtk::Window::list_toplevels()
        .into_iter()
        .filter_map(|widget| widget.downcast::<gtk::Window>().ok())
        .find(|window| window.title().as_deref() == Some("Argui updated lifecycle check"))
        .expect("the lifecycle window is mounted");
    let native_window = window.window().expect("the lifecycle window is realized");
    send_pointer(&window);
    for (key, _) in keys() {
        for kind in [gdk::EventType::KeyPress, gdk::EventType::KeyRelease] {
            let mut event = gdk::Event::new(kind).downcast::<gdk::EventKey>().unwrap();
            event.as_mut().window = native_window.to_glib_full();
            event.as_mut().keyval = *key;
            event.as_mut().send_event = 1;
            event.put();
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
    use gtk::{gdk, glib::translate::ToGlibPtr, prelude::*};
    let native_window = window.window().expect("the lifecycle window is realized");
    let mut motion = gdk::Event::new(gdk::EventType::MotionNotify)
        .downcast::<gdk::EventMotion>()
        .unwrap();
    motion.as_mut().window = native_window.to_glib_full();
    motion.as_mut().send_event = 1;
    motion.as_mut().time = 100;
    motion.set_device(window.display().default_seat().unwrap().pointer().as_ref());
    motion.put();
    for (button, _, _) in buttons() {
        for kind in [gdk::EventType::ButtonPress, gdk::EventType::ButtonRelease] {
            let mut event = gdk::Event::new(kind)
                .downcast::<gdk::EventButton>()
                .unwrap();
            event.as_mut().window = native_window.to_glib_full();
            event.as_mut().button = button;
            event.as_mut().time = button;
            event.as_mut().send_event = 1;
            event.put();
        }
    }
    let mut event = gdk::Event::new(gdk::EventType::Scroll)
        .downcast::<gdk::EventScroll>()
        .unwrap();
    event.as_mut().window = native_window.to_glib_full();
    event.as_mut().direction = gdk::ffi::GDK_SCROLL_SMOOTH;
    event.as_mut().delta_x = 2.0;
    event.as_mut().delta_y = -3.0;
    event.put();
}

pub fn assert_pointer(pointer: &[argui_core::PointerEvent], wheel: &[argui_core::ScrollDelta]) {
    use argui_core::{Point, PointerPhase, ScrollDelta};
    assert!(
        pointer
            .iter()
            .any(|event| event.phase == PointerPhase::Moved)
    );
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
            .filter(|event| event.button.is_some())
            .map(|event| (event.button, event.phase, event.buttons))
            .collect::<Vec<_>>(),
        expected
    );
    assert_eq!(wheel, [ScrollDelta::Lines(Point::new(-2.0, 3.0))]);
}
