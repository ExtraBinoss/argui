use super::{BackdropBackend, BackdropError, Request};
use argui_core::Rect;
use wayland_client::{
    Connection, Dispatch, Proxy, QueueHandle, WEnum, delegate_noop,
    protocol::{wl_compositor, wl_region, wl_registry, wl_surface},
};
use wayland_protocols::ext::background_effect::v1::client::{
    ext_background_effect_manager_v1 as ext_manager,
    ext_background_effect_surface_v1 as ext_surface,
};
use wayland_protocols_plasma::blur::client::{
    org_kde_kwin_blur as kde_blur, org_kde_kwin_blur_manager as kde_manager,
};
use winit::raw_window_handle::{RawDisplayHandle, RawWindowHandle};

#[derive(Default)]
struct Globals {
    compositor: Option<wl_compositor::WlCompositor>,
    ext: Option<ext_manager::ExtBackgroundEffectManagerV1>,
    kde: Option<kde_manager::OrgKdeKwinBlurManager>,
    blur: bool,
    ext_name: u32,
    kde_name: u32,
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
        match event {
            wl_registry::Event::Global {
                name,
                interface,
                version,
            } => match interface.as_str() {
                "wl_compositor" => {
                    state.compositor = Some(registry.bind(name, version.min(4), queue, ()))
                }
                "ext_background_effect_manager_v1" => {
                    state.ext_name = name;
                    state.ext = Some(registry.bind(name, 1, queue, ()));
                }
                "org_kde_kwin_blur_manager" => {
                    state.kde_name = name;
                    state.kde = Some(registry.bind(name, 1, queue, ()));
                }
                _ => (),
            },
            wl_registry::Event::GlobalRemove { name } => {
                if state.ext_name == name {
                    state.blur = false;
                }
                if state.kde_name == name {
                    state.kde = None;
                }
            }
            _ => (),
        }
    }
}
#[cfg_attr(coverage_nightly, coverage(off))]
impl Dispatch<ext_manager::ExtBackgroundEffectManagerV1, ()> for Globals {
    fn event(
        state: &mut Self,
        _: &ext_manager::ExtBackgroundEffectManagerV1,
        event: ext_manager::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let ext_manager::Event::Capabilities { flags } = event {
            state.blur = matches!(flags, WEnum::Value(flags) if flags.contains(ext_manager::Capability::Blur));
        }
    }
}
delegate_noop!(Globals: ignore wl_compositor::WlCompositor);
delegate_noop!(Globals: ignore wl_region::WlRegion);
delegate_noop!(Globals: ignore ext_surface::ExtBackgroundEffectSurfaceV1);
delegate_noop!(Globals: ignore kde_manager::OrgKdeKwinBlurManager);
delegate_noop!(Globals: ignore kde_blur::OrgKdeKwinBlur);

pub(in crate::desktop_backdrop) struct Backend {
    connection: Connection,
    queue: wayland_client::EventQueue<Globals>,
    globals: Globals,
    surface: wl_surface::WlSurface,
    ext: Option<ext_surface::ExtBackgroundEffectSurfaceV1>,
    kde: Option<kde_blur::OrgKdeKwinBlur>,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl BackdropBackend for Backend {
    #[allow(unsafe_code)]
    fn new(display: RawDisplayHandle, window: RawWindowHandle) -> Result<Self, BackdropError> {
        let (RawDisplayHandle::Wayland(display), RawWindowHandle::Wayland(window)) =
            (display, window)
        else {
            return Err(BackdropError::Unsupported);
        };
        // SAFETY: NativeBackdrop retains the window/canvas owner until after this guest backend
        // is dropped. It never destroys the borrowed wl_surface or disconnects the host display.
        let backend = unsafe {
            wayland_backend::client::Backend::from_foreign_display(display.display.as_ptr().cast())
        };
        let connection = Connection::from_backend(backend);
        let id = unsafe {
            wayland_backend::client::ObjectId::from_ptr(
                wl_surface::WlSurface::interface(),
                window.surface.as_ptr().cast(),
            )
        }
        .map_err(error)?;
        let surface = wl_surface::WlSurface::from_id(&connection, id).map_err(error)?;
        let mut queue = connection.new_event_queue();
        let handle = queue.handle();
        connection.display().get_registry(&handle, ());
        let mut globals = Globals::default();
        queue.roundtrip(&mut globals).map_err(error)?;
        queue.roundtrip(&mut globals).map_err(error)?;
        if globals.compositor.is_none() || (globals.ext.is_none() && globals.kde.is_none()) {
            return Err(BackdropError::Unsupported);
        }
        let ext = globals
            .ext
            .as_ref()
            .map(|manager| manager.get_background_effect(&surface, &handle, ()));
        Ok(Self {
            connection,
            queue,
            globals,
            surface,
            ext,
            kde: None,
        })
    }

    fn available(&mut self) -> Result<bool, BackdropError> {
        self.queue
            .dispatch_pending(&mut self.globals)
            .map_err(error)?;
        Ok(if self.ext.is_some() {
            self.globals.blur
        } else {
            self.globals.kde.is_some()
        })
    }

    fn apply(&mut self, regions: &[Rect], _: &Request) -> Result<(), BackdropError> {
        let handle = self.queue.handle();
        let region = self
            .globals
            .compositor
            .as_ref()
            .ok_or(BackdropError::Unsupported)?
            .create_region(&handle, ());
        for rect in regions {
            region.add(
                rect.origin.x as i32,
                rect.origin.y as i32,
                rect.size.width as i32,
                rect.size.height as i32,
            );
        }
        if let Some(effect) = &self.ext {
            effect.set_blur_region(Some(&region));
        } else if let Some(manager) = &self.globals.kde {
            if regions.is_empty() {
                manager.unset(&self.surface);
                if let Some(effect) = self.kde.take() {
                    effect.release();
                }
            } else {
                let effect = self
                    .kde
                    .get_or_insert_with(|| manager.create(&self.surface, &handle, ()));
                effect.set_region(Some(&region));
                effect.commit();
            }
        }
        region.destroy();
        // The renderer commits the surface together with the frame showing the corresponding tint.
        self.connection.flush().map_err(error)
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl Drop for Backend {
    fn drop(&mut self) {
        if let Some(effect) = self.ext.take() {
            effect.destroy();
        }
        if let Some(effect) = self.kde.take() {
            if let Some(manager) = &self.globals.kde {
                manager.unset(&self.surface);
            }
            effect.release();
        }
        let _ = self.connection.flush();
    }
}

fn error(error: impl std::fmt::Display) -> BackdropError {
    BackdropError::Platform(error.to_string())
}
