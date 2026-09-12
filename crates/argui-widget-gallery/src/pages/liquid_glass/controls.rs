use super::GlassDemo;
use crate::app::text;
use argui::{
    runtime::{Context, LayoutSnapshot},
    ui::{Element, EventType, FlexWrap, Sides, UiEventKind, length, percent},
    widgets::{Button, RangeBehavior, RangeConfig, Slider, WidgetTheme},
};
use argui_effects::LiquidGlass;

pub(super) const COUNT: usize = 9;
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
        key: "glass-edge",
        label: "Rim width",
        unit: "px",
        config: RangeConfig::new(0.0, 64.0, 1.0),
        field: |v| &mut v.edge_width,
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
        unit: "",
        config: RangeConfig::new(0.0, 1.0, 0.05),
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
        key: "glass-brightness",
        label: "Brightness",
        unit: "",
        config: RangeConfig::new(-1.0, 1.0, 0.01),
        field: |v| &mut v.brightness,
    },
    Parameter {
        key: "glass-contrast",
        label: "Contrast",
        unit: "",
        config: RangeConfig::new(0.0, 4.0, 0.05),
        field: |v| &mut v.contrast,
    },
];

impl Parameter {
    fn behavior(&self, effect: &mut LiquidGlass) -> RangeBehavior {
        RangeBehavior::new(self.key, self.label, *(self.field)(effect), self.config)
    }
}

impl GlassDemo {
    pub(super) fn layout_controls(&mut self, layout: &LayoutSnapshot) {
        for (parameter, state) in PARAMETERS.iter().zip(&mut self.ranges) {
            state.layout_changed(layout, &parameter.behavior(&mut self.effect));
        }
    }

    pub(super) fn controls(&mut self, theme: &WidgetTheme, cx: &mut Context<Self>) -> Element {
        let buttons = [
            (
                "glass-enable",
                format!("Effect: {}", if self.enabled { "on" } else { "off" }),
            ),
            ("glass-reset", "Reset settings".into()),
            ("glass-tint-theme", "Tint: theme".into()),
            ("glass-tint-white", "Tint: white".into()),
            ("glass-tint-blue", "Tint: blue".into()),
            ("glass-tint-rose", "Tint: rose".into()),
            (
                "glass-depth",
                format!(
                    "Depth: {}",
                    if self.effect.depth_effect {
                        "on"
                    } else {
                        "off"
                    }
                ),
            ),
        ];
        let toolbar = Element::row(buttons.into_iter().map(|(key, label)| {
            let selected = matches!(
                (key, self.tint),
                ("glass-tint-theme", 0)
                    | ("glass-tint-white", 1)
                    | ("glass-tint-blue", 2)
                    | ("glass-tint-rose", 3)
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
                Slider::new(parameter.key, parameter.label, value, parameter.config).build(theme),
            ])
            .width(length(140.0))
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
                    if let Some(action) = state.update(event, &parameter.behavior(&mut demo.effect))
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
                        demo.effect = Self::DEFAULT_EFFECT;
                        demo.tint = 0;
                        demo.enabled = true;
                    }
                    Some("glass-depth") => demo.effect.depth_effect = !demo.effect.depth_effect,
                    Some(
                        key @ ("glass-tint-theme" | "glass-tint-white" | "glass-tint-blue"
                        | "glass-tint-rose"),
                    ) => {
                        demo.tint = match key {
                            "glass-tint-white" => 1,
                            "glass-tint-blue" => 2,
                            "glass-tint-rose" => 3,
                            _ => 0,
                        };
                        let rgb = [
                            [0.0, 0.0, 0.0],
                            [1.0, 1.0, 1.0],
                            [0.15, 0.4, 1.0],
                            [1.0, 0.15, 0.3],
                        ][demo.tint];
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
