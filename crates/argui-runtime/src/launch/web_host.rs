//! Browser event-loop launch and synchronous host transaction validation.

use argui_platform::{PlatformError, WindowConfig};
use argui_render::{GpuCanvasRegistry, RendererConfig};
use argui_text::TextEngine;
use winit::{event_loop::EventLoop, platform::web::EventLoopExtWebSys};

use crate::{
    NativeHost, NativeHostAssets, NativeHostDelivery, RuntimeError, RuntimeEvent, WireOperation,
    app::Application, event::UserEvent, validate_native_host_assets, validate_native_host_canvases,
};

/// Browser presentation handle that validates each transaction before posting it to Winit.
/// The browser supports one active Winit event loop on a page.
pub struct WebHostHandle {
    host: NativeHost,
    proxy: winit::event_loop::EventLoopProxy<UserEvent>,
    canvases: GpuCanvasRegistry,
    assets: NativeHostAssets,
}

impl WebHostHandle {
    /// Starts the real Rust renderer and mounts its canvas into `window.web_parent_id`.
    /// `renderer` supplies GPU configuration, `on_delivery` receives UI callbacks,
    /// and `on_event` receives platform or renderer diagnostics.
    ///
    /// # Errors
    /// Returns a schema or browser event-loop initialization error.
    pub fn start(
        window: WindowConfig,
        renderer: RendererConfig,
        on_delivery: impl Fn(NativeHostDelivery) + 'static,
        on_event: impl FnMut(RuntimeEvent) + 'static,
    ) -> Result<Self, RuntimeError> {
        Self::start_with_assets(
            window,
            renderer,
            NativeHostAssets::default(),
            on_delivery,
            on_event,
        )
    }

    /// Starts the browser renderer with app-owned image and SVG assets.
    ///
    /// `window` chooses the canvas parent, `renderer` supplies GPU configuration,
    /// `assets` are registered before the first frame, and the callbacks receive
    /// UI deliveries and runtime diagnostics.
    ///
    /// # Errors
    /// Returns a schema or browser event-loop initialization error.
    pub fn start_with_assets(
        window: WindowConfig,
        renderer: RendererConfig,
        assets: NativeHostAssets,
        on_delivery: impl Fn(NativeHostDelivery) + 'static,
        on_event: impl FnMut(RuntimeEvent) + 'static,
    ) -> Result<Self, RuntimeError> {
        let host = NativeHost::with_builtins()
            .map_err(|error| RuntimeError::Configuration(error.to_string()))?;
        let shadow = NativeHost::with_builtins()
            .map_err(|error| RuntimeError::Configuration(error.to_string()))?;
        let canvases = renderer.gpu_canvases.clone();
        let event_loop = EventLoop::<UserEvent>::with_user_event()
            .build()
            .map_err(PlatformError::from)?;
        let proxy = event_loop.create_proxy();
        let mut application = Application::new(
            window,
            renderer,
            TextEngine::new(),
            None,
            None,
            None,
            on_event,
        );
        application.install_native_host_assets(assets.clone());
        application.native_host = Some(host);
        application.web_host_events = Some(Box::new(on_delivery));
        application.set_event_proxy(proxy.clone());
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
        event_loop.spawn_app(application);
        Ok(Self {
            host: shadow,
            proxy,
            canvases,
            assets,
        })
    }

    /// Returns the Rust schema fingerprint expected by the TypeScript adapter.
    #[must_use]
    pub fn abi_hash(&self) -> u64 {
        self.host.abi_hash()
    }

    /// Validates and commits `operations`, then queues them for the browser UI loop.
    /// Invalid batches leave both the presentation host and renderer unchanged.
    ///
    /// # Errors
    /// Returns a wire, schema, asset, canvas, or stopped event-loop error.
    pub fn commit(&mut self, operations: Vec<WireOperation>) -> Result<(), String> {
        let decoded = operations
            .iter()
            .cloned()
            .map(WireOperation::into_native)
            .collect::<Result<Vec<_>, _>>()?;
        validate_native_host_assets(&decoded, &self.assets.images, &self.assets.vectors)?;
        validate_native_host_canvases(&decoded, &self.canvases)?;
        self.host
            .validate(&decoded)
            .map_err(|error| error.to_string())?;
        self.host
            .commit(&decoded)
            .map_err(|error| error.to_string())?;
        self.proxy
            .send_event(UserEvent::WebHostCommit(operations))
            .map_err(|_| "browser event loop has stopped".to_owned())
    }
}
