//! A standalone Argui window with a mutable application theme.

use argui_runtime::{
    ApplicationConfig, ApplicationIdentity, Color, Context, Element, EventType, Render,
    RendererConfig, SingleWindowModel, ThemeRuntime, ThemeSchema, ThemeTokenDefinition,
    ThemeTokenId, ThemeValue, WindowConfig, run_application,
};
use std::sync::Arc;

struct App {
    theme: ThemeRuntime,
    accent: ThemeTokenId,
}

impl Render for App {
    /// Renders the resolved accent and a click target that changes the theme.
    ///
    /// # Panics
    /// Panics only if the registered accent token unexpectedly rejects a color.
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let accent = cx.environment().theme.as_ref()
            .and_then(|snapshot| snapshot.value(self.accent))
            .and_then(|value| match value { ThemeValue::Color(color) => Some(*color), _ => None })
            .unwrap_or(Color::WHITE);
        let toggle = cx.callback(|app| {
            let next = if app.theme.value(app.accent)
                == Some(ThemeValue::Color(Color::srgb(0.78, 0.88, 1.0))) {
                    Color::srgb(1.0, 0.82, 0.85)
                } else {
                    Color::srgb(0.78, 0.88, 1.0)
                };
            app.theme.set_override(app.accent, ThemeValue::Color(next))
                .expect("accent token accepts colors");
        });
        Element::column([
            Element::text("Hello from Argui"),
            Element::text("Click to change the theme accent")
                .on(toggle.direct_listener(EventType::Click)),
        ]).background(accent)
    }
}

/// Creates a window with a theme shared by its model and environment.
///
/// # Errors
/// Returns an error if the theme schema, display, or renderer cannot start.
///
/// # Panics
/// Panics only if the validated schema loses its registered accent token.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let schema = Arc::new(ThemeSchema::new([ThemeTokenDefinition::new(
        "accent", ThemeValue::Color(Color::srgb(0.78, 0.88, 1.0)),
    )])?);
    let accent = schema.token("accent").expect("accent token was registered");
    let theme = ThemeRuntime::new(schema);
    let app = SingleWindowModel::new(App { theme: theme.clone(), accent }).with_theme(theme);
    let config = ApplicationConfig::new(
        ApplicationIdentity::development("Argui application"),
        WindowConfig::default(),
    );
    run_application(config, RendererConfig::default(), app, |_| {})?;
    Ok(())
}
