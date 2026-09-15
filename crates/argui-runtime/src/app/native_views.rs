use super::Application;
#[cfg(not(target_arch = "wasm32"))]
use argui_webview::WryBackend;
#[cfg(not(target_arch = "wasm32"))]
use argui_webview::resolve_mounts;
use argui_webview::{PoolConfig, WebViewPool};
#[cfg(not(target_arch = "wasm32"))]
pub(super) type Backend = WryBackend;
#[cfg(target_arch = "wasm32")]
pub(super) type Backend = argui_webview::BrowserBackend;

impl Application {
    #[cfg(target_arch = "wasm32")]
    pub(super) fn initialize_browser_webviews(
        &mut self,
        window: &std::sync::Arc<winit::window::Window>,
    ) {
        use winit::platform::web::WindowExtWebSys;
        let Some(canvas) = window.canvas() else {
            return;
        };
        let window = window.clone();
        let mut backend = Backend::new(move || window.request_redraw());
        backend.register_host(1, canvas);
        self.native_views = Some(
            WebViewPool::new(backend, PoolConfig::default()).expect("nonzero default capacity"),
        );
    }
    #[cfg(target_os = "linux")]
    pub(super) fn initialize_gtk_webviews(&mut self, container: gtk::Fixed) {
        let proxy = self.event_proxy.clone();
        let window = self.window_key.clone();
        let mut backend = WryBackend::new().input_waker(move || {
            if let Some(proxy) = &proxy {
                let _ = proxy.send_event(crate::event::UserEvent::NativeInput {
                    window: window.clone(),
                });
            }
        });
        backend.register_gtk_host(1, container);
        self.native_views = Some(
            WebViewPool::new(backend, PoolConfig::default()).expect("nonzero default capacity"),
        );
    }

    #[cfg(any(target_os = "windows", target_os = "macos"))]
    pub(super) fn initialize_winit_webviews(
        &mut self,
        window: std::sync::Arc<winit::window::Window>,
    ) {
        let mut backend = WryBackend::new();
        backend.register_host(1, window);
        self.native_views = Some(
            WebViewPool::new(backend, PoolConfig::default()).expect("nonzero default capacity"),
        );
    }

    pub(crate) fn sync_native_views(&mut self) {
        let Some(pool) = &mut self.native_views else {
            return;
        };
        #[cfg(not(target_arch = "wasm32"))]
        let mounts = match (&self.ui_tree, &self.ui_layout) {
            (Some(ui), Some(layout)) => resolve_mounts(ui, layout, 1),
            _ => Vec::new(),
        };
        #[cfg(target_arch = "wasm32")]
        let clipped = match (&self.ui_tree, &self.ui_layout) {
            (Some(ui), Some(layout)) => argui_webview::resolve_clipped_mounts(ui, layout, 1),
            _ => Vec::new(),
        };
        #[cfg(target_arch = "wasm32")]
        let mounts: Vec<_> = clipped.iter().map(|(mount, _)| mount.clone()).collect();
        if let Err(error) = pool.reconcile(&mounts, self.input_epoch.elapsed()) {
            (self.on_event)(crate::RuntimeEvent::CommandFailed(error.to_string()));
        }
        #[cfg(target_arch = "wasm32")]
        {
            pool.backend().sync_clips(&clipped);
            pool.backend().schedule_maintenance(
                pool.next_expiry()
                    .map(|deadline| deadline.saturating_sub(self.input_epoch.elapsed())),
            );
        }
    }

    #[cfg(target_os = "linux")]
    pub(crate) fn native_deadline(&self) -> Option<web_time::Instant> {
        self.native_views
            .as_ref()
            .and_then(WebViewPool::next_expiry)
            .map(|deadline| self.input_epoch + deadline)
    }
}
