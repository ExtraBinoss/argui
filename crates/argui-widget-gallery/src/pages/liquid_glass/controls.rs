use super::GlassDemo;
use crate::app::text;
use argui::{
    core::Point,
    runtime::{Context, LayoutSnapshot},
    ui::{Element, EventType, FlexWrap, Sides, UiEventKind, length, percent},
    widgets::{Button, RangeBehavior, RangeConfig, Slider, WidgetTheme},
};
use argui_effects::LiquidGlass;

pub(super) const COUNT: usize = 11;
struct Parameter {
    key: &'static str,
    label: &'static str,
    unit: &'static str,
    config: RangeConfig,
    field: fn(&mut LiquidGlass) -> &mut f32,
}
const PARAMETERS: [Parameter; COUNT] = [
    Parameter {
        key: "glass-strength",
        label: "Refraction",
        unit: "px",
        config: RangeConfig::new(0.0, 64.0, 1.0),
        field: |v| &mut v.refraction,
    },
    Parameter {
        key: "glass-blur",
        label: "Blur",
        unit: "px",
        config: RangeConfig::new(0.0, 16.0, 0.25),
        field: |v| &mut v.blur,
    },
    Parameter {
        key: "glass-tint-amount",
        label: "Tint amount",
        unit: "",
        config: RangeConfig::new(0.0, 1.0, 0.01),
        field: |v| &mut v.tint[3],
    },
    Parameter {
        key: "glass-ior",
        label: "Refractive index",
        unit: "",
        config: RangeConfig::new(1.0, 2.5, 0.01),
        field: |v| &mut v.ior,
    },
    Parameter {
        key: "glass-edge",
        label: "Rim width",
        unit: "px",
        config: RangeConfig::new(0.0, 64.0, 1.0),
        field: |v| &mut v.edge_width,
    },
    Parameter {
        key: "glass-fresnel",
        label: "Rim reflection",
        unit: "",
        config: RangeConfig::new(0.0, 1.0, 0.01),
        field: |v| &mut v.fresnel,
    },
    Parameter {
        key: "glass-highlight",
        label: "Highlight",
        unit: "",
        config: RangeConfig::new(0.0, 1.0, 0.01),
        field: |v| &mut v.highlight,
    },
    Parameter {
        key: "glass-chroma",
        label: "Dispersion",
        unit: "px",
        config: RangeConfig::new(0.0, 8.0, 0.05),
        field: |v| &mut v.chromatic_aberration,
    },
    Parameter {
        key: "glass-saturation",
        label: "Saturation",
        unit: "",
        config: RangeConfig::new(0.0, 4.0, 0.05),
        field: |v| &mut v.saturation,
    },
    Parameter {
        key: "glass-frequency",
        label: "Noise frequency",
        unit: "",
        config: RangeConfig::new(0.001, 1.0, 0.001),
        field: |v| &mut v.frequency,
    },
    Parameter {
        key: "glass-turbulence",
        label: "Noise strength",
        unit: "",
        config: RangeConfig::new(0.0, 1.0, 0.01),
        field: |v| &mut v.turbulence,
    },
];

impl Parameter {
    fn enabled(&self, noise: bool) -> bool {
        !matches!(self.key, "glass-frequency" | "glass-turbulence") || noise
    }
    fn behavior(&self, effect: &mut LiquidGlass, noise: bool) -> RangeBehavior {
        RangeBehavior::new(self.key, self.label, *(self.field)(effect), self.config)
            .enabled(self.enabled(noise))
    }
}

impl GlassDemo {
    pub(super) fn layout_controls(&mut self, layout: &LayoutSnapshot) {
        for (parameter, state) in PARAMETERS.iter().zip(&mut self.ranges) {
            state.layout_changed(layout, &parameter.behavior(&mut self.effect, self.noise));
        }
    }

    pub(super) fn controls(&mut self, theme: &WidgetTheme, cx: &mut Context<Self>) -> Element {
        let buttons = [
            (
                "glass-enable",
                format!("Effect: {}", if self.enabled { "on" } else { "off" }),
            ),
            ("glass-reset", "Reset settings".into()),
            ("glass-recenter", "Recenter glass".into()),
            ("glass-tint-white", "Tint: white".into()),
            ("glass-tint-blue", "Tint: blue".into()),
            ("glass-tint-rose", "Tint: rose".into()),
            (
                "glass-noise",
                format!("Noise: {}", if self.noise { "on" } else { "off" }),
            ),
            ("glass-octaves", format!("Octaves: {}", self.effect.octaves)),
            ("glass-seed", format!("Seed: {}", self.effect.seed)),
        ];
        let toolbar = Element::row(buttons.into_iter().map(|(key, label)| {
            let selected = matches!(
                (key, self.tint),
                ("glass-tint-white", 0) | ("glass-tint-blue", 1) | ("glass-tint-rose", 2)
            );
            Button::new(
                key,
                label,
                if selected {
                    theme.button()
                } else {
                    theme.ghost_button()
                },
            )
            .build()
        }))
        .gap(6.0)
        .flex_wrap(FlexWrap::Wrap);
        let sliders = Element::row(PARAMETERS.iter().map(|parameter| {
            let value = *(parameter.field)(&mut self.effect);
            let label = if parameter.key == "glass-tint-amount" {
                format!("{} · {:.0}%", parameter.label, value * 100.0)
            } else {
                format!("{} · {:.3} {}", parameter.label, value, parameter.unit)
            };
            Element::column([
                text(label, 12.0, theme.foreground, 500),
                Slider::new(parameter.key, parameter.label, value, parameter.config)
                    .enabled(parameter.enabled(self.noise))
                    .build(theme),
            ])
            .width(length(210.0))
            .grow(1.0)
            .padding(Sides::length(8.0))
            .gap(4.0)
        }))
        .gap(8.0)
        .flex_wrap(FlexWrap::Wrap)
        .width(percent(1.0));
        let mut root = Element::column([toolbar, sliders]).gap(8.0);
        for kind in [
            EventType::Click,
            EventType::Gesture,
            EventType::Key,
            EventType::SemanticAction,
        ] {
            root = root.on(cx.listener(kind, |demo, event, cx| {
                for (parameter, state) in PARAMETERS.iter().zip(&mut demo.ranges) {
                    if let Some(action) =
                        state.update(event, &parameter.behavior(&mut demo.effect, demo.noise))
                    {
                        *(parameter.field)(&mut demo.effect) = action.value();
                        cx.notify();
                        return;
                    }
                }
                if !matches!(event.kind, UiEventKind::Click(_)) {
                    return;
                }
                match event.target_key() {
                    Some("glass-enable") => demo.enabled = !demo.enabled,
                    Some("glass-reset") => {
                        let defaults = Self::default();
                        demo.effect = defaults.effect;
                        demo.tint = defaults.tint;
                        demo.noise = defaults.noise;
                        demo.enabled = true;
                    }
                    Some("glass-recenter") => demo.position = Point::new(demo.max_x * 0.5, 105.0),
                    Some("glass-noise") => {
                        demo.noise = !demo.noise;
                        if demo.noise && demo.effect.turbulence == 0.0 {
                            demo.effect.turbulence = 0.15;
                        }
                    }
                    Some("glass-octaves") => demo.effect.octaves = demo.effect.octaves % 6 + 1,
                    Some("glass-seed") => demo.effect.seed = demo.effect.seed.wrapping_add(1),
                    Some(key @ ("glass-tint-white" | "glass-tint-blue" | "glass-tint-rose")) => {
                        demo.tint = match key {
                            "glass-tint-blue" => 1,
                            "glass-tint-rose" => 2,
                            _ => 0,
                        };
                        let rgb = [[1.0, 1.0, 1.0], [0.15, 0.4, 1.0], [1.0, 0.15, 0.3]][demo.tint];
                        demo.effect.tint[..3].copy_from_slice(&rgb);
                    }
                    _ => return,
                }
                cx.notify();
            }));
        }
        root
    }
}
