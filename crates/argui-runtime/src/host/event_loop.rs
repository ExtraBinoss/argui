use crate::event::UserEvent;

#[derive(Clone)]
pub(crate) enum EventProxy {
    Winit(winit::event_loop::EventLoopProxy<UserEvent>),
    #[cfg(all(feature = "webview", target_os = "linux"))]
    Gtk(tao::event_loop::EventLoopProxy<UserEvent>),
}

impl EventProxy {
    pub(crate) fn send_event(&self, event: UserEvent) -> Result<(), ()> {
        match self {
            Self::Winit(proxy) => proxy.send_event(event).map_err(|_| ()),
            #[cfg(all(feature = "webview", target_os = "linux"))]
            Self::Gtk(proxy) => proxy.send_event(event).map_err(|_| ()),
        }
    }
}

impl From<winit::event_loop::EventLoopProxy<UserEvent>> for EventProxy {
    fn from(proxy: winit::event_loop::EventLoopProxy<UserEvent>) -> Self {
        Self::Winit(proxy)
    }
}

#[cfg(all(feature = "webview", target_os = "linux"))]
impl From<tao::event_loop::EventLoopProxy<UserEvent>> for EventProxy {
    fn from(proxy: tao::event_loop::EventLoopProxy<UserEvent>) -> Self {
        Self::Gtk(proxy)
    }
}
