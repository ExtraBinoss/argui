use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
use std::sync::Arc;
use wayland_client::{
    Connection, Dispatch, Proxy, QueueHandle, delegate_noop,
    protocol::{
        wl_compositor, wl_region, wl_registry, wl_subcompositor, wl_subsurface, wl_surface,
    },
};

#[derive(Default)]
struct Globals {
    compositor: Option<wl_compositor::WlCompositor>,
    subcompositor: Option<wl_subcompositor::WlSubcompositor>,
}
impl Dispatch<wl_registry::WlRegistry, ()> for Globals {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        queue: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            match interface.as_str() {
                "wl_compositor" => {
                    state.compositor = Some(registry.bind(name, version.min(4), queue, ()))
                }
                "wl_subcompositor" => state.subcompositor = Some(registry.bind(name, 1, queue, ())),
                _ => (),
            }
        }
    }
}
delegate_noop!(Globals: ignore wl_compositor::WlCompositor);
delegate_noop!(Globals: ignore wl_subcompositor::WlSubcompositor);
delegate_noop!(Globals: ignore wl_subsurface::WlSubsurface);
delegate_noop!(Globals: ignore wl_surface::WlSurface);
delegate_noop!(Globals: ignore wl_region::WlRegion);

pub struct GtkCanvas {
    surface: wl_surface::WlSurface,
    subsurface: wl_subsurface::WlSubsurface,
    parent: wl_surface::WlSurface,
    queue: std::sync::Mutex<wayland_client::EventQueue<Globals>>,
    _connection: Connection,
    window: Arc<tao::window::Window>,
}
impl GtkCanvas {
    #[allow(unsafe_code)]
    pub(super) fn new(window: Arc<tao::window::Window>) -> Result<Self, String> {
        let RawDisplayHandle::Wayland(display) = window
            .display_handle()
            .map_err(|error| error.to_string())?
            .as_raw()
        else {
            return Err("the GTK WGPU host currently requires a native Wayland display".into());
        };
        let RawWindowHandle::Wayland(parent) = window
            .window_handle()
            .map_err(|error| error.to_string())?
            .as_raw()
        else {
            return Err("the GTK WGPU host currently requires a native Wayland display".into());
        };
        // The retained Tao window owns GTK's display and parent surface for this canvas's lifetime.
        // Only newly created proxies are managed by this guest backend; it never disconnects GTK.
        let backend = unsafe {
            wayland_backend::client::Backend::from_foreign_display(display.display.as_ptr().cast())
        };
        let connection = Connection::from_backend(backend);
        let parent_id = unsafe {
            wayland_backend::client::ObjectId::from_ptr(
                wl_surface::WlSurface::interface(),
                parent.surface.as_ptr().cast(),
            )
            .map_err(|error| error.to_string())?
        };
        let parent = wl_surface::WlSurface::from_id(&connection, parent_id)
            .map_err(|error| error.to_string())?;
        let mut queue = connection.new_event_queue();
        let handle = queue.handle();
        connection.display().get_registry(&handle, ());
        let mut globals = Globals::default();
        queue
            .roundtrip(&mut globals)
            .map_err(|error| error.to_string())?;
        let compositor = globals.compositor.ok_or("Wayland compositor unavailable")?;
        let surface = compositor.create_surface(&handle, ());
        let subsurface = globals
            .subcompositor
            .ok_or("Wayland subcompositor unavailable")?
            .get_subsurface(&surface, &parent, &handle, ());
        subsurface.set_desync();
        subsurface.set_position(0, 0);
        subsurface.place_below(&parent);
        let empty = compositor.create_region(&handle, ());
        surface.set_input_region(Some(&empty));
        empty.destroy();
        parent.commit();
        connection.flush().map_err(|error| error.to_string())?;
        Ok(Self {
            surface,
            subsurface,
            parent,
            queue: std::sync::Mutex::new(queue),
            _connection: connection,
            window,
        })
    }

    pub fn place(&self, x: i32, y: i32) -> Result<(), String> {
        self.subsurface.set_position(x, y);
        self.parent.commit();
        self._connection.flush().map_err(|error| error.to_string())
    }

    pub fn dispatch_pending(&self) -> Result<(), String> {
        self.queue
            .lock()
            .map_err(|error| error.to_string())?
            .dispatch_pending(&mut Globals::default())
            .map(|_| ())
            .map_err(|error| error.to_string())
    }
}
impl HasWindowHandle for GtkCanvas {
    #[allow(unsafe_code)]
    fn window_handle(
        &self,
    ) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        let surface = std::ptr::NonNull::new(self.surface.id().as_ptr().cast()).unwrap();
        // This owned proxy is destroyed only after the renderer releases its Arc<Canvas>.
        Ok(unsafe {
            raw_window_handle::WindowHandle::borrow_raw(RawWindowHandle::Wayland(
                raw_window_handle::WaylandWindowHandle::new(surface),
            ))
        })
    }
}
impl HasDisplayHandle for GtkCanvas {
    fn display_handle(
        &self,
    ) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        self.window.display_handle()
    }
}
impl Drop for GtkCanvas {
    fn drop(&mut self) {
        self.subsurface.destroy();
        self.surface.destroy();
    }
}
