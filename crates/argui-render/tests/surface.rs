#![cfg(target_os = "linux")]

use std::sync::Arc;

use argui_core::{Affine2D, Color, Point, Rect, Size};
use argui_paint::{
    ClipChain, DisplayList, EffectId, EffectInstance, Filter, ImageFit, LayerStyle, VectorAsset,
    VectorId, VectorPrimitive,
};
use argui_render::{
    EffectDefinition, EffectPassDefinition, EffectRegistry, RenderStatus, RendererConfig,
    RendererError, SurfaceRenderer,
};
use argui_text::{PreparedText, TextEngine};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    platform::{pump_events::EventLoopExtPumpEvents, wayland::EventLoopBuilderExtWayland},
    window::{Window, WindowId},
};

const SHADER: &str = "fn argui_effect(uv: vec2<f32>, source: vec4<f32>, backdrop: vec4<f32>) -> vec4<f32> { return source * 0.5; }";
const PASSES: &[EffectPassDefinition] = &[
    EffectPassDefinition::fragment("first", SHADER),
    EffectPassDefinition::fragment("second", SHADER),
];

#[test]
#[ignore = "Native surface integration: requires a dedicated Wayland test display"]
fn native_surface_grows_its_atlas_recovers_from_capacity_and_uses_custom_effects() {
    struct TestApp(Option<Arc<Window>>, bool);
    impl ApplicationHandler for TestApp {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            let window = Arc::new(
                event_loop
                    .create_window(
                        Window::default_attributes()
                            .with_title("Argui renderer tests")
                            .with_inner_size(winit::dpi::PhysicalSize::new(256, 256)),
                    )
                    .unwrap(),
            );
            window.request_redraw();
            self.0 = Some(window);
        }

        fn window_event(&mut self, _: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
            self.1 |= matches!(event, WindowEvent::RedrawRequested);
        }
    }
    let mut event_loop = EventLoop::builder()
        .with_wayland()
        .with_any_thread(true)
        .build()
        .unwrap();
    let mut app = TestApp(None, false);
    let deadline = web_time::Instant::now() + std::time::Duration::from_secs(3);
    while !app.1 {
        event_loop.pump_app_events(Some(std::time::Duration::from_millis(16)), &mut app);
        assert!(
            web_time::Instant::now() < deadline,
            "window was not configured"
        );
    }
    let window = app.0.as_ref().unwrap().clone();
    exercise(window, || {
        event_loop.pump_app_events(Some(std::time::Duration::from_millis(16)), &mut app);
    });
}

fn exercise(window: Arc<Window>, mut pump: impl FnMut()) {
    let effect = EffectId::new("test.half");
    let unused = EffectId::new("test.unused");
    let registry = EffectRegistry::new([
        EffectDefinition::new(effect, &[], PASSES),
        EffectDefinition::new(unused, &[], PASSES),
    ])
    .unwrap();
    let size = window.inner_size();
    let mut renderer = pollster::block_on(SurfaceRenderer::new(
        window.clone(),
        size.width,
        size.height,
        RendererConfig::default().profiling(true).effects(registry),
    ))
    .unwrap();
    let mut render = |renderer: &mut SurfaceRenderer, list: &DisplayList| {
        render(renderer, list, &window, &mut pump)
    };
    render(&mut renderer, &DisplayList::new()).unwrap();
    assert_eq!(
        renderer.last_profile().vector_atlas.allocated_bytes,
        256 * 256 * 4
    );

    let assets: Vec<_> = (0..20).map(|_| asset()).collect();
    for asset in &assets {
        renderer.register_vector(asset).unwrap();
    }
    let small = vectors(&assets[..1], 32.0);
    render(&mut renderer, &small).unwrap();
    assert_eq!(
        renderer
            .last_profile()
            .vector_atlas
            .rasterizations_this_frame,
        1
    );
    let large = vectors(&assets[..1], 512.0);
    render(&mut renderer, &large).unwrap();
    assert_eq!(
        renderer.last_profile().vector_atlas.allocated_bytes,
        1024 * 1024 * 4
    );
    render(&mut renderer, &small).unwrap();
    render(&mut renderer, &small).unwrap();
    assert_eq!(
        renderer
            .last_profile()
            .vector_atlas
            .rasterizations_this_frame,
        0
    );
    assert_eq!(renderer.last_profile().vector_atlas.hits_this_frame, 1);

    assert!(matches!(
        render(&mut renderer, &vectors(&assets, 512.0)),
        Err(RendererError::VectorAtlasFull)
    ));
    render(&mut renderer, &small).unwrap();
    assert_eq!(
        renderer.last_profile().vector_atlas.allocated_bytes,
        2048 * 2048 * 4
    );

    for id in [effect, effect, unused] {
        let mut list = DisplayList::new();
        list.begin_layer(LayerStyle::new(bounds(32.0)).filter(Filter::Effect(
            EffectInstance::new(id, std::iter::empty::<argui_paint::EffectArgument>()),
        )));
        list.push_vector(vector(assets[0].id, 32.0));
        list.end_layer();
        render(&mut renderer, &list).unwrap();
        assert!(renderer.last_profile().effects.filter_passes >= 2);
    }
    let mut missing = DisplayList::new();
    missing.begin_layer(
        LayerStyle::new(bounds(32.0)).filter(Filter::Effect(EffectInstance::new(
            EffectId::new("test.missing"),
            std::iter::empty::<argui_paint::EffectArgument>(),
        ))),
    );
    missing.push_vector(vector(assets[0].id, 32.0));
    missing.end_layer();
    assert!(matches!(
        render(&mut renderer, &missing),
        Err(RendererError::MissingEffect("test.missing"))
    ));
}

fn render(
    renderer: &mut SurfaceRenderer,
    list: &DisplayList,
    window: &Window,
    pump: &mut impl FnMut(),
) -> Result<(), RendererError> {
    let mut text = TextEngine::from_embedded_fonts([], "sans-serif", "serif", "monospace");
    let deadline = web_time::Instant::now() + std::time::Duration::from_secs(3);
    loop {
        // A Wayland surface needs configure/frame callbacks between submissions.
        window.request_redraw();
        pump();
        let status =
            renderer.render_ui_notified(&mut text, &PreparedText::default(), list, 1.0, || {
                window.pre_present_notify()
            })?;
        if status == RenderStatus::Presented {
            return Ok(());
        }
        assert_eq!(status, RenderStatus::Skipped);
        assert!(
            web_time::Instant::now() < deadline,
            "surface did not present within three seconds"
        );
    }
}

fn asset() -> VectorAsset {
    VectorAsset {
        id: VectorId::fresh(),
        size: Size::new(16.0, 16.0),
        svg: Arc::from(br#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"><rect width="16" height="16" fill="red"/></svg>"#.as_slice()),
        tintable: false,
    }
}

fn bounds(size: f32) -> Rect {
    Rect::new(Point::default(), Size::new(size, size))
}

fn vector(id: VectorId, size: f32) -> VectorPrimitive {
    VectorPrimitive {
        vector: id,
        bounds: bounds(size),
        fit: ImageFit::Contain,
        color: Color::WHITE,
        opacity: 1.0,
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
    }
}

fn vectors(assets: &[VectorAsset], size: f32) -> DisplayList {
    let mut list = DisplayList::new();
    for asset in assets {
        list.push_vector(vector(asset.id, size));
    }
    list
}
