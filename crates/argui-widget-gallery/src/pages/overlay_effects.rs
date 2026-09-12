use crate::app::text;
use argui::{
    core::Color,
    paint::{
        Border, CornerRadii, EffectId, EffectInstance, EffectValue, Filter, LayerStyle, PaintStyle,
        QuadStyle,
    },
    render::{EffectDefinition, EffectParameter, EffectParameterType, EffectPassDefinition},
    ui::{Element, Sides, length, percent},
    widgets::WidgetTheme,
};

pub(crate) const PRISM: EffectId = EffectId::new("gallery.overlay.prism");
const PARAMETERS: &[EffectParameter] =
    &[EffectParameter::new("strength", EffectParameterType::F32)];
const PASSES: &[EffectPassDefinition] = &[EffectPassDefinition::fragment(
    "prism",
    r"
fn argui_effect(uv: vec2<f32>, source: vec4<f32>, backdrop: vec4<f32>) -> vec4<f32> {
    let band = 0.5 + 0.5 * sin((uv.x + uv.y * 0.7) * 12.0);
    let tint = mix(argui_srgb_to_linear(vec3<f32>(0.30, 0.50, 1.0)),
                   argui_srgb_to_linear(vec3<f32>(0.90, 0.30, 0.65)), band);
    return vec4<f32>(mix(source.rgb, tint, argui_param_f32(0u)), source.a);
}
",
)];

pub(crate) fn definition() -> EffectDefinition {
    EffectDefinition::new(PRISM, PARAMETERS, PASSES)
}

#[derive(Clone, Copy)]
pub(crate) enum Surface {
    Solid,
    Frosted,
    Prism,
}

impl Surface {
    pub(crate) const ALL: [Self; 3] = [Self::Solid, Self::Frosted, Self::Prism];

    pub(crate) const fn title(self) -> &'static str {
        match self {
            Self::Solid => "Solid surface",
            Self::Frosted => "Backdrop blur",
            Self::Prism => "Custom effect",
        }
    }

    pub(crate) const fn description(self) -> &'static str {
        match self {
            Self::Solid => "An opaque panel with no blur.",
            Self::Frosted => "A translucent panel over a blurred background.",
            Self::Prism => "A registered WGSL effect colors the background.",
        }
    }

    pub(crate) fn layer(self, theme: &WidgetTheme) -> LayerStyle {
        let layer = theme.overlay_layer(8.0, 0.0);
        match self {
            Self::Solid => layer,
            Self::Frosted => layer.backdrop(argui_effects::Blur(14.0).filter()),
            Self::Prism => layer.backdrop(Filter::Effect(EffectInstance::new(
                PRISM,
                [("strength", EffectValue::F32(0.42))],
            ))),
        }
    }

    pub(crate) fn paint(self, theme: &WidgetTheme) -> PaintStyle {
        let alpha = match self {
            Self::Solid => 1.0,
            Self::Frosted => 0.76,
            Self::Prism => 0.68,
        };
        PaintStyle::new(
            QuadStyle::solid(theme.popover.with_alpha(alpha))
                .border(Border::all(1.0, theme.popover_border))
                .radius(CornerRadii::all(8.0)),
        )
    }

    pub(crate) fn card(self, trigger: Element, theme: &WidgetTheme) -> Element {
        let background = Element::column((0..6).map(|index| {
            Element::row([
                Element::container([])
                    .width(length(7.0))
                    .height(length(22.0))
                    .background(if index % 2 == 0 {
                        Color::from_srgb8(91, 115, 241)
                    } else {
                        Color::from_srgb8(216, 96, 165)
                    }),
                text(
                    ["Design review", "Project notes", "Release checklist"][index % 3],
                    13.0,
                    theme.muted_foreground,
                    400,
                ),
            ])
            .gap(10.0)
        }))
        .gap(8.0)
        .padding(Sides {
            top: length(64.0),
            ..Sides::length(16.0)
        });
        let stage = Element::container([
            background,
            trigger.absolute(Sides {
                top: length(18.0),
                left: length(16.0),
                right: argui::ui::auto(),
                bottom: argui::ui::auto(),
            }),
        ])
        .height(length(280.0))
        .width(percent(1.0))
        .background(theme.muted)
        .border(Border::all(1.0, theme.border))
        .radius(CornerRadii::all(10.0));
        Element::column([
            text(self.title(), 16.0, theme.foreground, 600),
            text(self.description(), 13.0, theme.muted_foreground, 400).min_height(length(36.0)),
            stage,
        ])
        .gap(12.0)
        .width(length(268.0))
        .max_width(percent(1.0))
        .grow(1.0)
    }
}
