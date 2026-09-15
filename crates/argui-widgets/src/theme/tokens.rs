use super::WidgetTheme;
use argui_theme::{ThemeOverrides, ThemeValue};

impl WidgetTheme {
    /// Editable tokens for the current resolved palette. Derived widget styles
    /// read these values when built, so previews update every mounted consumer.
    /// Returns the token names and values supported by this palette.
    #[must_use]
    pub fn tokens(&self) -> Vec<(&'static str, ThemeValue)> {
        let colors = [
            ("background", self.background),
            ("card", self.card),
            ("popover", self.popover),
            ("popover-border", self.popover_border),
            ("foreground", self.foreground),
            ("muted", self.muted),
            ("muted-foreground", self.muted_foreground),
            ("primary", self.primary),
            ("primary-foreground", self.primary_foreground),
            ("secondary", self.secondary),
            ("destructive", self.destructive),
            ("destructive-foreground", self.destructive_foreground),
            ("border", self.border),
            ("input-border", self.input_border),
            ("switch-unchecked", self.switch_unchecked),
            ("switch-thumb", self.switch_thumb),
            ("ring", self.ring),
            ("dialog-backdrop", self.dialog_backdrop),
        ];
        colors
            .into_iter()
            .map(|(name, color)| (name, ThemeValue::Color(color)))
            .chain([
                ("overlay-blur", ThemeValue::Number(self.overlay_blur)),
                (
                    "dialog-backdrop-blur",
                    ThemeValue::Number(self.dialog_backdrop_blur),
                ),
            ])
            .collect()
    }

    /// Applies recognized color and blur overrides from `overrides`; unknown tokens are ignored.
    pub fn apply_overrides(&mut self, overrides: &ThemeOverrides) {
        for (name, value) in overrides.iter() {
            match (name, value) {
                ("overlay-blur", ThemeValue::Number(value)) => self.overlay_blur = value.max(0.0),
                ("dialog-backdrop-blur", ThemeValue::Number(value)) => {
                    self.dialog_backdrop_blur = value.max(0.0)
                }
                (name, ThemeValue::Color(color)) => {
                    let target = match name {
                        "background" => &mut self.background,
                        "card" => &mut self.card,
                        "popover" => &mut self.popover,
                        "popover-border" => &mut self.popover_border,
                        "foreground" => &mut self.foreground,
                        "muted" => &mut self.muted,
                        "muted-foreground" => &mut self.muted_foreground,
                        "primary" => &mut self.primary,
                        "primary-foreground" => &mut self.primary_foreground,
                        "secondary" => &mut self.secondary,
                        "destructive" => &mut self.destructive,
                        "destructive-foreground" => &mut self.destructive_foreground,
                        "border" => &mut self.border,
                        "input-border" => &mut self.input_border,
                        "switch-unchecked" => &mut self.switch_unchecked,
                        "switch-thumb" => &mut self.switch_thumb,
                        "ring" => &mut self.ring,
                        "dialog-backdrop" => &mut self.dialog_backdrop,
                        _ => continue,
                    };
                    *target = color;
                }
                _ => {}
            }
        }
    }
}
