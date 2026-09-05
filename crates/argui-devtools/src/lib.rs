mod app;
mod host;
mod icons;
mod presentation;
mod style;
mod view;
pub use app::DevtoolsApp;
pub use presentation::DockMode;

pub use host::DevtoolsHost;

/// Adds the DevTools' graphic effects without replacing application shaders.
pub fn configure_renderer(
    mut config: argui_render::RendererConfig,
) -> Result<argui_render::RendererConfig, argui_render::RendererError> {
    let mut definitions = config.effects.definitions().to_vec();
    for definition in argui_effects::registry()?.definitions() {
        if matches!(
            definition.id,
            argui_effects::EDGE_FADE_ID | argui_effects::EDGE_SHADOW_ID
        ) && !definitions
            .iter()
            .any(|existing| existing.id == definition.id)
        {
            definitions.push(definition.clone());
        }
    }
    config.effects = argui_render::EffectRegistry::new(definitions)?;
    Ok(config)
}
