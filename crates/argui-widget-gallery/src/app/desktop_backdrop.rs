use super::{WidgetGallery, text};
use argui::{
    core::ColorInterpolation,
    runtime::{Context, LayoutSnapshot, WindowEnvironment},
    ui::{DesktopBackdrop, Element, FloatingPlacement, Placement, UiEvent, UiEventKind},
    widgets::{
        Button, Popover, PopoverAction, PopoverBehavior, RangeBehavior, RangeConfig, RangeState,
        Slider, Switch, WidgetTheme,
    },
};

pub(super) struct BackdropSettings {
    pub(super) enabled: bool,
    open: bool,
    translucent_fallback: bool,
    opacity: f32,
    tint: f32,
    inactive_opacity: f32,
    sliders: [RangeState; 3],
}

impl Default for BackdropSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            open: false,
            translucent_fallback: false,
            opacity: 78.0,
            tint: 0.0,
            inactive_opacity: 92.0,
            sliders: Default::default(),
        }
    }
}

impl BackdropSettings {
    pub(super) fn layout_changed(&mut self, layout: &LayoutSnapshot) {
        for (state, (key, label)) in self.sliders.iter_mut().zip(RANGES) {
            state.layout_changed(
                layout,
                &RangeBehavior::new(key, label, 0.0, RangeConfig::new(0.0, 100.0, 1.0)),
            );
        }
    }

    pub(super) fn paint(&self, theme: &WidgetTheme) -> DesktopBackdrop {
        let tint = theme
            .card
            .mix(theme.primary, self.tint / 100.0, ColorInterpolation::Oklab);
        DesktopBackdrop::new(
            tint.with_alpha(self.opacity / 100.0),
            tint.with_alpha(if self.translucent_fallback {
                self.opacity / 100.0
            } else {
                1.0
            }),
        )
        .inactive_tint(tint.with_alpha(self.inactive_opacity / 100.0))
        .inactive_fallback(tint.with_alpha(if self.translucent_fallback {
            self.inactive_opacity / 100.0
        } else {
            1.0
        }))
    }
}

const RANGES: [(&str, &str); 3] = [
    ("sidebar-opacity", "Surface opacity"),
    ("sidebar-tint", "Accent tint"),
    ("sidebar-inactive-opacity", "Inactive opacity"),
];

impl WidgetGallery {
    pub(super) fn backdrop_controls(
        &self,
        theme: &WidgetTheme,
        environment: WindowEnvironment,
    ) -> Element {
        let mut content = vec![
            text("Sidebar appearance", 16.0, theme.foreground, 650),
            text(
                "Let your desktop show through the navigation.",
                12.0,
                theme.muted_foreground,
                400,
            ),
            Switch::new("sidebar-glass", "Desktop glass", self.backdrop.enabled).build(theme),
            text(
                if environment.desktop_backdrop_available {
                    "Native desktop blur is available."
                } else {
                    "Desktop blur is unavailable here. Choose a fallback below."
                },
                12.0,
                theme.muted_foreground,
                400,
            ),
        ];
        for ((key, label), value) in RANGES.into_iter().zip([
            self.backdrop.opacity,
            self.backdrop.tint,
            self.backdrop.inactive_opacity,
        ]) {
            content.push(
                Element::column([
                    text(
                        format!("{label} · {value:.0}%"),
                        12.0,
                        theme.foreground,
                        500,
                    ),
                    Slider::new(key, label, value, RangeConfig::new(0.0, 100.0, 1.0)).build(theme),
                ])
                .gap(4.0),
            );
        }
        content.push(
            Switch::new(
                "sidebar-fallback",
                "Allow transparency without blur",
                self.backdrop.translucent_fallback,
            )
            .build(theme),
        );
        content.push(text(
            "Blur strength follows your desktop’s material settings.",
            12.0,
            theme.muted_foreground,
            400,
        ));
        Popover::new(
            "gallery-appearance",
            "Sidebar appearance",
            self.backdrop.open,
            Button::new("appearance-trigger", "Appearance", theme.outline_button()).build(),
            Element::column(content).gap(14.0),
        )
        .placement(FloatingPlacement::new(Placement::BottomEnd))
        .size(330.0, 540.0)
        .padding(18.0)
        .build(theme)
    }

    pub(super) fn backdrop_event(&mut self, event: &UiEvent, cx: &mut Context<Self>) -> bool {
        if let Some(action) = PopoverBehavior::new(
            "gallery-appearance",
            "Sidebar appearance",
            self.backdrop.open,
        )
        .action(event)
        {
            self.backdrop.open = action == PopoverAction::Toggle && !self.backdrop.open;
        } else if matches!(event.kind, UiEventKind::Click(_))
            && event.target_key() == Some("sidebar-glass")
        {
            self.backdrop.enabled = !self.backdrop.enabled;
        } else if matches!(event.kind, UiEventKind::Click(_))
            && event.target_key() == Some("sidebar-fallback")
        {
            self.backdrop.translucent_fallback = !self.backdrop.translucent_fallback;
        } else {
            let mut changed = false;
            for (index, ((key, label), value)) in RANGES
                .into_iter()
                .zip([
                    &mut self.backdrop.opacity,
                    &mut self.backdrop.tint,
                    &mut self.backdrop.inactive_opacity,
                ])
                .enumerate()
            {
                let behavior =
                    RangeBehavior::new(key, label, *value, RangeConfig::new(0.0, 100.0, 1.0));
                if let Some(action) = self.backdrop.sliders[index].update(event, &behavior) {
                    *value = action.value();
                    changed = true;
                    break;
                }
            }
            if !changed {
                return false;
            }
        }
        cx.notify();
        event.stop_propagation();
        true
    }
}
