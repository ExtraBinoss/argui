//! Native GTK host used by WebView-enabled applications on Wayland.

use gtk::prelude::*;
use std::sync::Arc;
use tao::{
    event_loop::EventLoopWindowTarget,
    platform::unix::{WindowBuilderExtUnix, WindowExtUnix},
    window::{Window, WindowBuilder},
};

use crate::WindowConfig;

mod canvas;
pub use canvas::GtkCanvas;

/// GTK-backed native window with a transparent WGPU canvas child.
pub struct GtkWindow {
    window: Arc<Window>,
    container: gtk::Fixed,
    canvas: Arc<GtkCanvas>,
    css: gtk::CssProvider,
}

impl GtkWindow {
    /// Creates a GTK host for the supplied window configuration.
    ///
    /// # Errors
    /// Returns an error if native window or GTK canvas setup fails.
    /// `target` is the Winit event-loop target and `config` supplies window dimensions and appearance.
    pub fn new<T: 'static>(
        target: &EventLoopWindowTarget<T>,
        config: &WindowConfig,
    ) -> Result<Self, String> {
        let window = Arc::new(
            WindowBuilder::new()
                .with_title(&config.title)
                .with_inner_size(tao::dpi::LogicalSize::new(config.width, config.height))
                .with_decorations(config.decorations)
                .with_resizable(config.resizable)
                .with_app_paintable(true)
                .with_transparent(true)
                .build(target)
                .map_err(|error| error.to_string())?,
        );
        let css = gtk::CssProvider::new();
        css.load_from_data(b"window.argui-host, window.argui-host > box, fixed.argui-canvas { background-color: transparent; }")
            .map_err(|error| error.to_string())?;
        let native = window.gtk_window();
        native.style_context().add_class("argui-host");
        let screen = GtkWindowExt::screen(native).ok_or("GTK screen unavailable")?;
        gtk::StyleContext::add_provider_for_screen(
            &screen,
            &css,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        let container = gtk::Fixed::new();
        container.set_can_focus(true);
        container.style_context().add_class("argui-canvas");
        window
            .default_vbox()
            .ok_or("GTK client container unavailable")?
            .pack_start(&container, true, true, 0);
        container.show_all();
        let canvas = Arc::new(GtkCanvas::new(window.clone(), container.scale_factor())?);
        let weak_canvas = Arc::downgrade(&canvas);
        native.connect_unmap(move |_| {
            if let Some(canvas) = weak_canvas.upgrade() {
                canvas.detach();
            }
        });
        Ok(Self {
            window,
            container,
            canvas,
            css,
        })
    }

    /// Returns the Tao window that owns this GTK host.
    pub fn native(&self) -> &Window {
        &self.window
    }
    /// Returns the GTK container that hosts child surfaces.
    pub fn container(&self) -> &gtk::Fixed {
        &self.container
    }
    /// Returns a retained handle to the WGPU canvas surface.
    pub fn canvas(&self) -> Arc<GtkCanvas> {
        self.canvas.clone()
    }

    /// Query in the toplevel's GDK coordinates, never in the child window under the pointer.
    /// Returns the pointer position in the host's logical coordinates, if available.
    pub fn pointer_position(&self) -> Option<argui_core::Point> {
        let window = self.window.gtk_window().window()?;
        let pointer = window.display().default_seat()?.pointer()?;
        let (_, x, y, _) = window.device_position_double(&pointer);
        let allocation = self.container.allocation();
        Some(argui_core::Point::new(
            x as f32 - allocation.x() as f32,
            y as f32 - allocation.y() as f32,
        ))
    }

    /// Reports whether a native child surface currently owns keyboard focus.
    pub fn native_content_focused(&self) -> bool {
        self.window
            .gtk_window()
            .focused_widget()
            .is_some_and(|widget| {
                widget != *self.container.upcast_ref::<gtk::Widget>()
                    && widget.is_ancestor(&self.container)
            })
    }

    /// Requests keyboard focus for the WGPU canvas container.
    pub fn focus_canvas(&self) {
        self.container.grab_focus();
    }

    /// Returns the GTK theme's current window corner radius in logical pixels.
    pub fn corner_radius(&self) -> f32 {
        if self.window.is_decorated() && !self.window.is_maximized() {
            8.0
        } else {
            0.0
        }
    }

    /// The client allocation excludes title bar and CSD shadow extents.
    /// Returns client width, height, and scale factor.
    pub fn client_size(&self) -> (u32, u32, f32) {
        let allocation = self.container.allocation();
        let scale = self.container.scale_factor().max(1);
        (
            (allocation.width().max(0) * scale) as u32,
            (allocation.height().max(0) * scale) as u32,
            scale as f32,
        )
    }

    /// Updates the canvas placement and dispatches pending Wayland events.
    ///
    /// # Errors
    /// Returns an error if canvas placement or event dispatch fails.
    pub fn sync_canvas(&self) -> Result<(), String> {
        if !self.window.gtk_window().is_mapped() {
            return Ok(());
        }
        let allocation = self.container.allocation();
        if self.canvas.place(
            allocation.x(),
            allocation.y(),
            self.container.scale_factor(),
        )? {
            // GTK owns parent commits and must first acknowledge the compositor's configure.
            self.window.gtk_window().queue_draw();
        }
        self.canvas.dispatch_pending()
    }
}

impl Drop for GtkWindow {
    fn drop(&mut self) {
        if let Some(screen) = GtkWindowExt::screen(self.window.gtk_window()) {
            gtk::StyleContext::remove_provider_for_screen(&screen, &self.css);
        }
    }
}
