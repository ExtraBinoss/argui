mod app;
mod host;
mod icons;
mod presentation;
mod style;
pub mod telemetry;
mod view;
pub use app::DevtoolsApp;
pub use presentation::DockMode;

pub use host::DevtoolsHost;

/// Adds the DevTools' graphic effects without replacing application shaders.
pub fn configure_renderer(
    mut config: argui_render::RendererConfig,
) -> Result<argui_render::RendererConfig, argui_render::RendererError> {
    if config.effects.get(argui_effects::EDGE_FADE_ID).is_some()
        && config.effects.get(argui_effects::EDGE_SHADOW_ID).is_some()
    {
        return Ok(config);
    }
    for definition in argui_effects::registry()?.definitions() {
        if matches!(
            definition.id,
            argui_effects::EDGE_FADE_ID | argui_effects::EDGE_SHADOW_ID
        ) && config.effects.get(definition.id).is_none()
        {
            config.effects = config.effects.with_definition(definition.clone())?;
        }
    }
    Ok(config)
}
