use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
use wayland_client::{
    Connection, Dispatch, Proxy, QueueHandle, delegate_noop,
    protocol::{wl_registry, wl_surface},
};
use wayland_protocols::xdg::activation::v1::client::xdg_activation_v1::XdgActivationV1;

#[derive(Default)]
struct Globals {
    activation: Option<XdgActivationV1>,
}

#[cfg_attr(coverage_nightly, coverage(off))]
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
            && interface == "xdg_activation_v1"
        {
            state.activation = Some(registry.bind(name, version.min(1), queue, ()));
        }
    }
}

delegate_noop!(Globals: ignore XdgActivationV1);
delegate_noop!(Globals: ignore wl_surface::WlSurface);

/// Activates an existing Wayland window with a compositor-issued token.
///
/// `window` supplies the borrowed display and surface handles. `token` must
/// come from the compositor, such as the XDG global-shortcuts portal.
///
/// Returns `Ok(true)` after submitting a Wayland activation request and
/// `Ok(false)` when `window` is not a Wayland surface or the compositor does
/// not expose `xdg_activation_v1`.
///
/// # Errors
/// Returns a platform error when native handles or Wayland communication fail.
#[cfg_attr(coverage_nightly, coverage(off))]
#[allow(unsafe_code)]
pub fn activate_wayland_window(
    window: &(impl HasDisplayHandle + HasWindowHandle),
    token: &str,
) -> Result<bool, String> {
    let display = window
        .display_handle()
        .map_err(|error| error.to_string())?
        .as_raw();
    let surface = window
        .window_handle()
        .map_err(|error| error.to_string())?
        .as_raw();
    let (RawDisplayHandle::Wayland(display), RawWindowHandle::Wayland(surface)) =
        (display, surface)
    else {
        return Ok(false);
    };
    // SAFETY: `window` owns both borrowed handles for this call. The guest
    // connection neither disconnects the display nor destroys the surface.
    let backend = unsafe {
        wayland_backend::client::Backend::from_foreign_display(display.display.as_ptr().cast())
    };
    let connection = Connection::from_backend(backend);
    let surface_id = unsafe {
        wayland_backend::client::ObjectId::from_ptr(
            wl_surface::WlSurface::interface(),
            surface.surface.as_ptr().cast(),
        )
    }
    .map_err(|error| error.to_string())?;
    let surface = wl_surface::WlSurface::from_id(&connection, surface_id)
        .map_err(|error| error.to_string())?;
    let mut queue = connection.new_event_queue();
    let handle = queue.handle();
    connection.display().get_registry(&handle, ());
    let mut globals = Globals::default();
    queue
        .roundtrip(&mut globals)
        .map_err(|error| error.to_string())?;
    let Some(activation) = globals.activation else {
        return Ok(false);
    };
    activation.activate(token.to_owned(), &surface);
    connection.flush().map_err(|error| error.to_string())?;
    Ok(true)
}
