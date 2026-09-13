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
    subcompositor: wl_subcompositor::WlSubcompositor,
    attachment: std::sync::Mutex<Option<Attachment>>,
    queue: std::sync::Mutex<wayland_client::EventQueue<Globals>>,
    connection: Connection,
    window: Arc<tao::window::Window>,
}
struct Attachment {
    subsurface: wl_subsurface::WlSubsurface,
    position: (i32, i32),
}
impl Drop for Attachment {
    fn drop(&mut self) {
        self.subsurface.destroy();
    }
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
        // The retained Tao window owns GTK's display. Its parent surface lasts only until unmap.
        // Only newly created proxies are managed by this guest backend; it never disconnects GTK.
        let backend = unsafe {
            wayland_backend::client::Backend::from_foreign_display(display.display.as_ptr().cast())
        };
        let connection = Connection::from_backend(backend);
        let mut queue = connection.new_event_queue();
        let handle = queue.handle();
        connection.display().get_registry(&handle, ());
        let mut globals = Globals::default();
        queue
            .roundtrip(&mut globals)
            .map_err(|error| error.to_string())?;
        let compositor = globals.compositor.ok_or("Wayland compositor unavailable")?;
        let surface = compositor.create_surface(&handle, ());
        let subcompositor = globals
            .subcompositor
            .ok_or("Wayland subcompositor unavailable")?;
        let empty = compositor.create_region(&handle, ());
        surface.set_input_region(Some(&empty));
        empty.destroy();
        connection.flush().map_err(|error| error.to_string())?;
        let canvas = Self {
            surface,
            subcompositor,
            attachment: std::sync::Mutex::new(None),
            queue: std::sync::Mutex::new(queue),
            connection,
            window,
        };
        canvas.place(0, 0)?;
        Ok(canvas)
    }

    /// Return whether GTK must commit the changed child placement on its next frame.
    pub fn place(&self, x: i32, y: i32) -> Result<bool, String> {
        let mut attachment = self.attachment.lock().map_err(|error| error.to_string())?;
        if let Some(attachment) = attachment.as_mut() {
            if attachment.position == (x, y) {
                return Ok(false);
            }
            attachment.subsurface.set_position(x, y);
            attachment.position = (x, y);
        } else {
            let parent = self.parent()?;
            let handle = self
                .queue
                .lock()
                .map_err(|error| error.to_string())?
                .handle();
            let subsurface = self
                .subcompositor
                .get_subsurface(&self.surface, &parent, &handle, ());
            subsurface.set_desync();
            subsurface.set_position(x, y);
            subsurface.place_below(&parent);
            *attachment = Some(Attachment {
                subsurface,
                position: (x, y),
            });
        }
        self.connection
            .flush()
            .map(|_| true)
            .map_err(|error| error.to_string())
    }

    pub(super) fn detach(&self) {
        self.attachment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
    }

    #[allow(unsafe_code)]
    fn parent(&self) -> Result<wl_surface::WlSurface, String> {
        let RawWindowHandle::Wayland(parent) = self
            .window
            .window_handle()
            .map_err(|error| error.to_string())?
            .as_raw()
        else {
            return Err("the GTK WGPU host currently requires a native Wayland display".into());
        };
        // Called on GTK's thread while mapped; unmap clears the attachment before it is reused.
        let parent_id = unsafe {
            wayland_backend::client::ObjectId::from_ptr(
                wl_surface::WlSurface::interface(),
                parent.surface.as_ptr().cast(),
            )
            .map_err(|error| error.to_string())?
        };
        wl_surface::WlSurface::from_id(&self.connection, parent_id)
            .map_err(|error| error.to_string())
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
        self.detach();
        self.surface.destroy();
    }
}
