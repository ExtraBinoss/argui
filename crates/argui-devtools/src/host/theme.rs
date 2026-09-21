use argui_core::{ColorScheme, Key, KeyState};
use argui_runtime::{LayoutSnapshot, WindowEnvironment};
use argui_theme::{ThemeOverrides, ThemeValue};
use argui_ui::{ClipboardRequest, UiEvent, UiEventKind};
use argui_widgets::{ColorPickerState, WidgetTheme};
use std::sync::Arc;

#[derive(Default)]
pub(crate) struct ThemeEditing {
    base: WindowEnvironment,
    pub overrides: ThemeOverrides,
    effective: Option<Arc<ThemeOverrides>>,
    pub scheme: Option<ColorScheme>,
    pub color: Option<(String, ColorPickerState)>,
    pub draft: Option<(String, String, bool)>,
    pub clipboard: Option<ClipboardRequest>,
}

impl ThemeEditing {
    pub fn capture(&mut self, environment: &WindowEnvironment) {
        if self.base != *environment {
            self.base = environment.clone();
            self.refresh();
        }
    }

    pub fn environment(&self) -> WindowEnvironment {
        let mut environment = self.base.clone();
        environment.theme_overrides = self.effective.clone();
        if let Some(scheme) = self.scheme {
            environment.color_scheme = scheme;
        }
        if let Some(ThemeValue::Color(primary)) = self
            .effective
            .as_ref()
            .and_then(|tokens| tokens.get("primary"))
        {
            environment.primary = primary;
        }
        environment
    }

    pub fn palette(&self) -> WidgetTheme {
        let environment = self.environment();
        argui_widgets::shadcn(&environment)
            .resolve(environment.color_scheme)
            .clone()
    }

    pub fn layout_changed(&mut self, layout: &LayoutSnapshot) {
        if let Some((name, picker)) = &mut self.color {
            picker.layout_changed(&format!("__devtools-color-theme-{name}"), layout);
        }
    }

    pub fn update(&mut self, event: &UiEvent) -> bool {
        let Some(key) = event.target_key() else {
            return false;
        };
        if let Some((name, picker)) = &mut self.color {
            let before = picker.color();
            if picker.update(&format!("__devtools-color-theme-{name}"), event) {
                if picker.color() != before {
                    self.overrides
                        .set(name.clone(), ThemeValue::Color(picker.color()));
                    self.refresh();
                }
                return true;
            }
        }
        if let Some(name) = key.strip_prefix("__devtools-theme-number-") {
            if !self
                .palette()
                .tokens()
                .iter()
                .any(|(token, value)| *token == name && matches!(value, ThemeValue::Float(_)))
            {
                return false;
            }
            match &event.kind {
                UiEventKind::TextChanged(text) | UiEventKind::Submitted(text) => {
                    let value = text
                        .parse::<f32>()
                        .ok()
                        .filter(|value| value.is_finite() && (0.0..=100.0).contains(value));
                    self.draft = Some((name.into(), text.clone(), value.is_none()));
                    if let Some(value) = value {
                        self.overrides.set(name, ThemeValue::Float(value));
                        self.refresh();
                    }
                    return true;
                }
                UiEventKind::Blurred => {
                    self.draft = None;
                    return true;
                }
                UiEventKind::KeyInput(input)
                    if input.state == KeyState::Pressed && input.key == Key::Escape =>
                {
                    self.draft = None;
                    let _ = event.prevent_default();
                    return true;
                }
                _ => return false,
            }
        }
        if !matches!(event.kind, UiEventKind::Click(_)) {
            return false;
        }
        if let Some(name) = key.strip_prefix("__devtools-theme-color-") {
            let Some((_, ThemeValue::Color(color))) = self
                .palette()
                .tokens()
                .into_iter()
                .find(|(token, _)| *token == name)
            else {
                return false;
            };
            self.color = if self.color.as_ref().is_some_and(|(token, _)| token == name) {
                None
            } else {
                Some((name.into(), ColorPickerState::new(color)))
            };
        } else if let Some(name) = key.strip_prefix("__devtools-theme-reset-") {
            self.overrides.remove(name);
            self.color = None;
            self.draft = None;
            self.refresh();
        } else {
            match key {
                "__devtools-theme-light" | "__devtools-theme-dark" | "__devtools-theme-system" => {
                    self.scheme = match key {
                        "__devtools-theme-light" => Some(ColorScheme::Light),
                        "__devtools-theme-dark" => Some(ColorScheme::Dark),
                        _ => None,
                    };
                    self.color = None;
                    self.draft = None;
                }
                "__devtools-theme-reset" => {
                    self.overrides = ThemeOverrides::default();
                    self.scheme = None;
                    self.color = None;
                    self.draft = None;
                    self.refresh();
                }
                "__devtools-theme-copy" => {
                    self.clipboard = Some(ClipboardRequest::Write(self.export()))
                }
                _ => return false,
            }
        }
        true
    }

    fn refresh(&mut self) {
        if self.overrides.is_empty() {
            self.effective = self.base.theme_overrides.clone();
        } else {
            let mut merged = self
                .base
                .theme_overrides
                .as_deref()
                .cloned()
                .unwrap_or_default();
            for (name, value) in self.overrides.iter() {
                merged.set(name, value);
            }
            self.effective = Some(Arc::new(merged));
        }
    }

    fn export(&self) -> String {
        let tokens: serde_json::Map<_, _> = self
            .palette()
            .tokens()
            .into_iter()
            .map(|(name, value)| {
                let value = match value {
                    ThemeValue::Color(color) => {
                        let [r, g, b, a] = color.to_srgba8();
                        serde_json::Value::String(format!("#{r:02X}{g:02X}{b:02X}{a:02X}"))
                    }
                    ThemeValue::Float(number) => serde_json::json!(number),
                    value => serde_json::json!({
                        "type": format!("{:?}", value.value_type()),
                        "value": format!("{value:?}"),
                    }),
                };
                (name.into(), value)
            })
            .collect();
        serde_json::to_string_pretty(&serde_json::json!({
            "color-scheme": match self.environment().color_scheme { ColorScheme::Light => "light", ColorScheme::Dark => "dark" },
            "tokens": tokens,
        })).expect("finite theme values")
    }
}
