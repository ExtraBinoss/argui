use std::{cell::RefCell, rc::Rc, sync::Arc};

use argui_platform::{PlatformError, PlatformEvent, WindowConfig};
use argui_render::{RenderStatus, RendererConfig, SurfaceRenderer};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

use crate::{RuntimeError, RuntimeEvent};

enum RendererState {
    Loading,
    Ready(Box<SurfaceRenderer>),
    #[cfg(target_arch = "wasm32")]
    Failed(String),
}

struct Application {
    window_config: WindowConfig,
    renderer_config: RendererConfig,
    window: Option<Arc<Window>>,
    renderer: Rc<RefCell<RendererState>>,
    renderer_announced: bool,
    fatal_error: Option<RuntimeError>,
    on_event: Box<dyn FnMut(RuntimeEvent)>,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl Application {
    fn new(
        window_config: WindowConfig,
        renderer_config: RendererConfig,
        on_event: impl FnMut(RuntimeEvent) + 'static,
    ) -> Self {
        Self {
            window_config,
            renderer_config,
            window: None,
            renderer: Rc::new(RefCell::new(RendererState::Loading)),
            renderer_announced: false,
            fatal_error: None,
            on_event: Box::new(on_event),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn initialize_renderer(&mut self, window: &Arc<Window>, event_loop: &ActiveEventLoop) {
        let size = window.inner_size();
        match pollster::block_on(SurfaceRenderer::new(
            Arc::clone(window),
            size.width,
            size.height,
            self.renderer_config,
        )) {
            Ok(renderer) => {
                *self.renderer.borrow_mut() = RendererState::Ready(Box::new(renderer));
                window.request_redraw();
            }
            Err(error) => {
                (self.on_event)(RuntimeEvent::RendererFailed(error.to_string()));
                self.fatal_error = Some(error.into());
                event_loop.exit();
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn initialize_renderer(&mut self, window: &Arc<Window>, _event_loop: &ActiveEventLoop) {
        let size = window.inner_size();
        let window = Arc::clone(window);
        let renderer = Rc::clone(&self.renderer);
        let config = self.renderer_config;

        wasm_bindgen_futures::spawn_local(async move {
            let result =
                SurfaceRenderer::new(Arc::clone(&window), size.width, size.height, config).await;
            *renderer.borrow_mut() = match result {
                Ok(renderer) => RendererState::Ready(Box::new(renderer)),
                Err(error) => RendererState::Failed(error.to_string()),
            };
            window.request_redraw();
        });
    }

    fn render(&mut self, event_loop: &ActiveEventLoop) {
        let Some(window) = self.window.as_ref().map(Arc::clone) else {
            return;
        };
        let mut state = self.renderer.borrow_mut();

        #[cfg(target_arch = "wasm32")]
        {
            if let RendererState::Failed(message) = &*state {
                (self.on_event)(RuntimeEvent::RendererFailed(message.clone()));
                event_loop.exit();
                return;
            }
        }
        let RendererState::Ready(renderer) = &mut *state else {
            return;
        };
        if !self.renderer_announced {
            (self.on_event)(RuntimeEvent::RendererReady);
            self.renderer_announced = true;
        }

        let result = match renderer.render() {
            Ok(RenderStatus::Presented | RenderStatus::Skipped) => Ok(()),
            Ok(RenderStatus::Reconfigure) => {
                let size = window.inner_size();
                renderer.resize(size.width, size.height);
                window.request_redraw();
                Ok(())
            }
            Ok(RenderStatus::RecreateSurface) => renderer
                .recreate_surface(Arc::clone(&window))
                .inspect(|()| window.request_redraw()),
            Err(error) => Err(error),
        };
        if let Err(error) = result {
            (self.on_event)(RuntimeEvent::RendererFailed(error.to_string()));
            self.fatal_error = Some(error.into());
            event_loop.exit();
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        match event_loop.create_window(self.window_config.clone().into_attributes()) {
            Ok(window) => {
                let window = Arc::new(window);
                let size = window.inner_size();
                (self.on_event)(RuntimeEvent::Platform(PlatformEvent::Opened {
                    width: size.width,
                    height: size.height,
                    scale_factor: window.scale_factor(),
                }));
                self.initialize_renderer(&window, event_loop);
                self.window = Some(window);
            }
            Err(error) => {
                (self.on_event)(RuntimeEvent::Platform(PlatformEvent::WindowCreationFailed(
                    error.to_string(),
                )));
                self.fatal_error = Some(PlatformError::from(error).into());
                event_loop.exit();
            }
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        (self.on_event)(RuntimeEvent::Platform(PlatformEvent::Suspended));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.window.as_ref().map(Arc::clone) else {
            return;
        };
        if window.id() != window_id {
            return;
        }

        let platform_event = match event {
            WindowEvent::CloseRequested => PlatformEvent::CloseRequested,
            WindowEvent::Resized(size) => {
                if let RendererState::Ready(renderer) = &mut *self.renderer.borrow_mut() {
                    renderer.resize(size.width, size.height);
                }
                PlatformEvent::Resized {
                    width: size.width,
                    height: size.height,
                }
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                PlatformEvent::ScaleFactorChanged(scale_factor)
            }
            WindowEvent::RedrawRequested => {
                self.render(event_loop);
                PlatformEvent::RedrawRequested
            }
            _ => return,
        };

        if platform_event.requires_redraw() {
            window.request_redraw();
        }
        if platform_event.closes_window() {
            event_loop.exit();
        }
        (self.on_event)(RuntimeEvent::Platform(platform_event));
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn run(
    window_config: WindowConfig,
    renderer_config: RendererConfig,
    on_event: impl FnMut(RuntimeEvent) + 'static,
) -> Result<(), RuntimeError> {
    let event_loop = EventLoop::new().map_err(PlatformError::from)?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut application = Application::new(window_config, renderer_config, on_event);
    event_loop
        .run_app(&mut application)
        .map_err(PlatformError::from)?;
    application.fatal_error.map_or(Ok(()), Err)
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn run(
    window_config: WindowConfig,
    renderer_config: RendererConfig,
    on_event: impl FnMut(RuntimeEvent) + 'static,
) -> Result<(), RuntimeError> {
    use winit::platform::web::EventLoopExtWebSys;

    let event_loop = EventLoop::new().map_err(PlatformError::from)?;
    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop.spawn_app(Application::new(window_config, renderer_config, on_event));
    Ok(())
}
