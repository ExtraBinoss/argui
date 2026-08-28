use argui::{
    devtools::DevtoolsHost,
    platform::{
        AppIcon, ApplicationConfig, ApplicationId, ApplicationIdentity, IconSet, TrayAction,
        TrayConfig, TrayItemId, TrayMenuItem, WindowConfig,
    },
    render::RendererConfig,
    runtime::{RuntimeEvent, WindowRuntimeEvent, run_app_with_text_engine},
};
use argui_showcase::{StateShowcase, text_engine};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let profiling = std::env::var_os("ARGUI_PROFILE").is_some();
    run_app_with_text_engine(
        ApplicationConfig::new(
            ApplicationIdentity::new(
                ApplicationId::new("dev.argui.state")?,
                "Argui state showcase",
                packaged_icons()?,
            )
            .with_linux_application_id("argui-state"),
            WindowConfig {
                title: "Argui state showcase".into(),
                width: 1050.0,
                height: 680.0,
                ..WindowConfig::default()
            },
        )
        .with_tray(TrayConfig {
            tooltip: Some("Argui state showcase".into()),
            menu: vec![TrayMenuItem::Action {
                id: TrayItemId::new("quit"),
                label: "Quit Argui".into(),
                enabled: true,
                action: TrayAction::Quit,
            }],
            ..TrayConfig::default()
        }),
        RendererConfig::default().profiling(profiling),
        text_engine(),
        DevtoolsHost::new(StateShowcase::default()),
        move |event| {
            if profiling {
                log_profile(event);
            }
        },
    )?;
    Ok(())
}

fn packaged_icons() -> Result<IconSet, Box<dyn std::error::Error>> {
    Ok(IconSet::new()
        .with(AppIcon::from_png(include_bytes!("../assets/icon-32.png"))?)
        .with(AppIcon::from_png(include_bytes!("../assets/icon-128.png"))?)
        .with(AppIcon::from_png(include_bytes!("../assets/icon-256.png"))?))
}

fn log_profile(event: RuntimeEvent) {
    match event {
        RuntimeEvent::AnimationProfile(profile)
        | RuntimeEvent::Window {
            event: WindowRuntimeEvent::AnimationProfile(profile),
            ..
        } => eprintln!(
            "[argui][ui] frame={:.2}ms model={:.2}ms tree={:.2}ms paint={:.2}ms update={:?}",
            profile.frame_interval.as_secs_f64() * 1_000.0,
            profile.model_time.as_secs_f64() * 1_000.0,
            profile.tree_time.as_secs_f64() * 1_000.0,
            profile.paint_time.as_secs_f64() * 1_000.0,
            profile.tree_update,
        ),
        RuntimeEvent::RenderProfile(profile)
        | RuntimeEvent::Window {
            event: WindowRuntimeEvent::RenderProfile(profile),
            ..
        } => eprintln!(
            "[argui][gpu] cpu={:.2}ms layers={}/{} passes={} pixels={} textures={} reused={} memory={:.1}MiB",
            profile.cpu_time.as_secs_f64() * 1_000.0,
            profile.effects.offscreen_layers,
            profile.effects.layers,
            profile.effects.filter_passes,
            profile.effects.offscreen_pixels,
            profile.texture_pool.textures,
            profile.texture_pool.reused_this_frame,
            profile.texture_pool.allocated_bytes as f64 / (1024.0 * 1024.0),
        ),
        _ => {}
    }
}
