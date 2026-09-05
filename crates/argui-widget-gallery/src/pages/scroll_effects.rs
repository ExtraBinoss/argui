use crate::app::text;
use argui::{
    core::Color,
    paint::{Border, CornerRadii, EffectId, EffectInstance, EffectValue, Filter, LayerStyle},
    render::{EffectDefinition, EffectParameter, EffectParameterType, EffectPassDefinition},
    runtime::{Context, Render},
    ui::{
        AlignItems, Axes, Element, EventType, JustifyContent, Overflow, ScrollConfig, ScrollEffect,
        ScrollMetric, Sides, UiEventKind, length, percent,
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
            Element::column([
                Element::container([])
                    .height(length(3.0))
                    .width(length(28.0))
                    .background(theme.primary),
                text(
                    format!("Collection {:02}", index + 1),
                    13.0,
                    theme.foreground,
                    600,
                ),
                text("12 components", 11.0, theme.muted_foreground, 400),
            ])
            .keyed(format!("scroll-card-{index}"))
            .padding(Sides::length(12.0))
            .gap(8.0)
            .background(theme.card)
            .border(Border::all(1.0, theme.border))
            .radius(CornerRadii::all(8.0))
            .width(length(150.0))
            .height(length(88.0))
            .shrink(0.0)
        }))
        .keyed("scroll-horizontal")
        .min_width(length(0.0))
        .height(length(110.0))
        .shrink(0.0)
        .gap(10.0)
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
        let modes = Element::row(
            ["Fade", "Shadow", "Custom WGSL"]
                .into_iter()
                .enumerate()
                .map(|(index, label)| {
                    Button::new(
                        format!("scroll-mode-{index}"),
                        label,
                        if self.mode == index {
                            theme.button()
                        } else {
                            theme.ghost_button()
                        },
                    )
                    .build()
                }),
        )
        .gap(4.0)
        .flex_wrap(argui::ui::FlexWrap::Wrap);
        let settings = if self.mode == 2 {
            text(
                "The custom shader changes tint with scroll progress.",
                12.0,
                theme.muted_foreground,
                400,
            )
        } else {
            Element::row([
                setting(
                    "Width",
                    &format!("{} px", self.width),
                    "scroll-width",
                    theme,
                ),
                setting(
                    "Intensity",
                    &format!("{:.0}%", self.intensity * 100.0),
                    "scroll-intensity",
                    theme,
                ),
            ])
            .gap(16.0)
            .flex_wrap(argui::ui::FlexWrap::Wrap)
        };
        let controls = Element::column([modes, settings])
            .keyed("scroll-effect-controls")
            .gap(8.0)
            .on(cx.listener(EventType::Click, |demo, event, cx| {
                match event.target_key() {
                    Some("scroll-mode-0") => demo.mode = 0,
                    Some("scroll-mode-1") => demo.mode = 1,
                    Some("scroll-mode-2") => demo.mode = 2,
                    Some("scroll-width-less") => demo.width = (demo.width - 10.0).max(0.0),
                    Some("scroll-width-more") => demo.width = (demo.width + 10.0).min(60.0),
                    Some("scroll-intensity-less") => {
                        demo.intensity = (demo.intensity - 0.25).max(0.0)
                    }
                    Some("scroll-intensity-more") => {
                        demo.intensity = (demo.intensity + 0.25).min(1.0)
                    }
                    _ => return,
                }
                cx.notify();
            }));
        let list = VList::new("scroll-demo-list", 32.0, 220.0, self.offset)
            .effect(self.effect())
            .build(10_000, theme, |index| {
                let mut style = theme.ghost_button();
                style.layout.justify_content = Some(JustifyContent::START);
                style.layout.padding = argui::ui::sides(12.0, 4.0);
                style.label.font_size = 13.0;
                Button::new(
                    format!("scroll-row-{index}"),
                    [
                        "Application shell",
                        "Navigation menu",
                        "Search field",
                        "Settings panel",
                        "Activity timeline",
                    ][index % 5],
                    style,
                )
                .leading(
                    text(
                        format!("{:05}", index + 1),
                        11.0,
                        theme.muted_foreground,
                        400,
                    )
                    .width(length(46.0)),
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
        let mut nested_content = vec![self.horizontal(theme)];
        for label in [
            "Scroll sideways to reveal more collections.",
            "Scroll down here to test the parent viewport.",
            "Each viewport controls its own edge effect.",
        ] {
            nested_content.push(
                text(label, 12.0, theme.muted_foreground, 400)
                    .padding(Sides::length(12.0))
                    .min_height(length(52.0))
                    .shrink(0.0)
                    .background(theme.card)
                    .radius(CornerRadii::all(6.0)),
            );
        }
        let nested = Element::column(nested_content)
            .padding(Sides::length(10.0))
            .min_width(length(0.0))
            .gap(10.0)
            .height(length(190.0))
            .shrink(0.0)
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
            "Scroll each panel to reveal the effect at its edges. Compare a fade, a shadow and a custom shader.",
            Element::column([
                controls,
                frame(
                    "Virtual list",
                    "10,000 items · only visible rows are mounted",
                    list,
                    theme,
                ),
                frame(
                    "Nested scroll",
                    "Horizontal collections inside a vertical viewport",
                    nested,
                    theme,
                ),
            ])
            .keyed("scroll-effects-demo")
            .width(percent(1.0))
            .min_width(length(0.0))
            .gap(18.0)
            .shrink(0.0),
            theme,
        )
    }
}

fn setting(label: &str, value: &str, key: &str, theme: &WidgetTheme) -> Element {
    Element::row([
        text(label, 12.0, theme.muted_foreground, 500),
        Button::new(format!("{key}-less"), "−", theme.ghost_button())
            .build()
            .width(length(30.0))
            .padding(Sides::length(4.0)),
        text(value, 12.0, theme.foreground, 500).width(length(52.0)),
        Button::new(format!("{key}-more"), "+", theme.ghost_button())
            .build()
            .width(length(30.0))
            .padding(Sides::length(4.0)),
    ])
    .align_items(AlignItems::CENTER)
    .gap(4.0)
}

fn frame(title: &str, hint: &str, content: Element, theme: &WidgetTheme) -> Element {
    Element::column([
        Element::column([
            text(title, 14.0, theme.foreground, 600),
            text(hint, 11.0, theme.muted_foreground, 400),
        ])
        .gap(4.0)
        .padding(Sides::length(14.0))
        .shrink(0.0),
        Element::container([])
            .height(length(1.0))
            .shrink(0.0)
            .background(theme.border),
        content
            .width(percent(1.0))
            .min_width(length(0.0))
            .shrink(0.0),
    ])
    .width(percent(1.0))
    .min_width(length(0.0))
    .shrink(0.0)
    .background(theme.background)
    .border(Border::all(1.0, theme.border))
    .radius(CornerRadii::all(12.0))
    .overflow(Axes {
        x: Overflow::Hidden,
        y: Overflow::Hidden,
    })
}
