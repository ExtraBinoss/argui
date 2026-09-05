use argui::{
    core::Color,
    paint::{EffectId, EffectInstance, EffectValue, Filter, LayerStyle},
    render::{EffectDefinition, EffectParameter, EffectParameterType, EffectPassDefinition},
    runtime::{Context, Render},
    ui::{
        Axes, Element, EventType, Overflow, ScrollConfig, ScrollEffect, ScrollMetric, UiEventKind,
        length, percent,
    },
    widgets::{Button, VList, WidgetTheme, shadcn},
};
use argui_effects::{EdgeFade, EdgeShadow};

const TINT: EffectId = EffectId::new("gallery.scroll.progress-tint");
const PARAMETERS: &[EffectParameter] =
    &[EffectParameter::new("progress", EffectParameterType::Vec2)];
const PASSES: &[EffectPassDefinition] = &[EffectPassDefinition::fragment(
    "tint",
    r"
fn argui_effect(uv: vec2<f32>, source: vec4<f32>, backdrop: vec4<f32>) -> vec4<f32> {
    let progress = max(argui_param_f32(0u), argui_param_f32(1u));
    let tint = argui_srgb_to_linear(vec3<f32>(0.25, 0.55, 1.0));
    return vec4<f32>(mix(source.rgb, tint, progress * 0.65), source.a);
}
",
)];

pub(crate) fn definition() -> EffectDefinition {
    EffectDefinition::new(TINT, PARAMETERS, PASSES)
}

pub(crate) struct ScrollDemo {
    mode: usize,
    width: f32,
    intensity: f32,
    offset: f32,
}

impl Default for ScrollDemo {
    fn default() -> Self {
        Self {
            mode: 0,
            width: 20.0,
            intensity: 1.0,
            offset: 0.0,
        }
    }
}

impl ScrollDemo {
    fn effect(&self) -> ScrollEffect {
        match self.mode {
            0 => EdgeFade::new(self.width).intensity(self.intensity).scroll(),
            1 => EdgeShadow::new(self.width, Color::srgba(0.0, 0.0, 0.0, 0.65))
                .intensity(self.intensity)
                .scroll(),
            _ => ScrollEffect::new(LayerStyle::new(Default::default()).filter(Filter::Effect(
                EffectInstance::new(TINT, [("progress", EffectValue::Vec2([0.0; 2]))]),
            )))
            .bind(0, "progress", ScrollMetric::Progress),
        }
    }

    fn horizontal(&self, theme: &WidgetTheme) -> Element {
        Element::row((0..16).map(|index| {
            Button::new(
                format!("scroll-card-{index}"),
                format!("Card {}", index + 1),
                theme.ghost_button(),
            )
            .build()
            .width(length(110.0))
            .height(length(60.0))
            .shrink(0.0)
        }))
        .keyed("scroll-horizontal")
        .height(length(84.0))
        .width(percent(1.0))
        .overflow(Axes {
            x: Overflow::Auto,
            y: Overflow::Hidden,
        })
        .scroll_config(
            ScrollConfig::default()
                .scrollbar(theme.scrollbar.clone())
                .effect(self.effect()),
        )
        .scrollbar_gutter(argui::ui::ScrollbarGutter::Stable)
    }
}

impl Render for ScrollDemo {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = shadcn(cx.environment().primary);
        let theme = themes.resolve(cx.environment().color_scheme);
        let controls = Element::row([
            Button::new(
                "scroll-mode",
                ["Fade", "Shadow", "Custom WGSL"][self.mode],
                theme.ghost_button(),
            )
            .build(),
            Button::new(
                "scroll-width",
                format!("Width: {} px", self.width),
                theme.ghost_button(),
            )
            .build(),
            Button::new(
                "scroll-intensity",
                format!("Intensity: {:.0}%", self.intensity * 100.0),
                theme.ghost_button(),
            )
            .build(),
        ])
        .gap(8.0)
        .flex_wrap(argui::ui::FlexWrap::Wrap)
        .on(cx.listener(EventType::Click, |demo, event, cx| {
            match event.target_key() {
                Some("scroll-mode") => demo.mode = (demo.mode + 1) % 3,
                Some("scroll-width") => {
                    demo.width = if demo.width >= 60.0 {
                        0.0
                    } else {
                        demo.width + 10.0
                    }
                }
                Some("scroll-intensity") => {
                    demo.intensity = if demo.intensity >= 1.0 {
                        0.0
                    } else {
                        demo.intensity + 0.25
                    }
                }
                _ => return,
            }
            cx.notify();
        }));
        let list = VList::new("scroll-demo-list", 32.0, 220.0, self.offset)
            .effect(self.effect())
            .build(10_000, theme, |index| {
                Button::new(
                    format!("scroll-row-{index}"),
                    format!("Virtual row {:05}", index + 1),
                    theme.ghost_button(),
                )
                .build()
                .width(percent(1.0))
            })
            .on(cx.listener(EventType::Scroll, |demo, event, cx| {
                if event.target_key() == Some("scroll-demo-list")
                    && let UiEventKind::Scrolled { offset, .. } = event.kind
                {
                    let config = VList::new("scroll-demo-list", 32.0, 220.0, 0.0).config(10_000);
                    let changed = config.window(demo.offset).range != config.window(offset.y).range;
                    demo.offset = offset.y;
                    if changed {
                        cx.notify();
                    }
                }
            }));
        let nested = Element::column([
            crate::app::text(
                "Horizontal scroll inside a vertical viewport",
                13.0,
                theme.muted_foreground,
                400,
            ),
            self.horizontal(theme),
            Element::container([])
                .height(length(180.0))
                .shrink(0.0)
                .background(theme.muted),
        ])
        .gap(10.0)
        .height(length(160.0))
        .width(percent(1.0))
        .keyed("scroll-demo-nested")
        .overflow(Axes {
            x: Overflow::Hidden,
            y: Overflow::Auto,
        })
        .scroll_config(
            ScrollConfig::default()
                .scrollbar(theme.scrollbar.clone())
                .effect(self.effect()),
        )
        .scrollbar_gutter(argui::ui::ScrollbarGutter::Stable);
        super::preview(
            "Scroll-driven effects",
            "10,000 virtual rows. Effects follow scroll geometry without rebuilding rows between virtual windows. Click the controls to compare presets and a custom shader.",
            Element::column([controls, list, nested]).gap(14.0),
            theme,
        )
    }
}
