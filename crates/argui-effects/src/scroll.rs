//! Edge treatments are regular registered effects, optionally driven by scroll metrics.
use argui_paint::{Color, EffectId, EffectInstance, EffectValue, Filter, LayerStyle};
use argui_render::{EffectDefinition, EffectParameter, EffectParameterType, EffectPassDefinition};
use argui_ui::{ScrollEffect, ScrollMetric};

pub const EDGE_FADE_ID: EffectId = EffectId::new("argui.scroll.edge-fade");
pub const EDGE_SHADOW_ID: EffectId = EffectId::new("argui.scroll.edge-shadow");

const PARAMETERS: &[EffectParameter] = &[
    EffectParameter::new("width", EffectParameterType::LogicalPixels),
    EffectParameter::new("intensity", EffectParameterType::F32),
    EffectParameter::new("edges", EffectParameterType::Vec4),
    EffectParameter::new("local-x", EffectParameterType::Vec3),
    EffectParameter::new("local-y", EffectParameterType::Vec3),
    EffectParameter::new("viewport-width", EffectParameterType::LogicalPixels),
    EffectParameter::new("viewport-height", EffectParameterType::LogicalPixels),
    EffectParameter::new("color", EffectParameterType::Color),
    EffectParameter::new("shadow", EffectParameterType::Bool),
];
const PASSES: &[EffectPassDefinition] = &[EffectPassDefinition::fragment(
    "edges",
    include_str!("shaders/effects/edges.wgsl"),
)];

pub(crate) fn definitions() -> [EffectDefinition; 2] {
    [
        EffectDefinition::new(EDGE_FADE_ID, PARAMETERS, PASSES),
        EffectDefinition::new(EDGE_SHADOW_ID, PARAMETERS, PASSES),
    ]
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EdgeFade {
    pub width: f32,
    pub intensity: f32,
    /// Left, top, right, bottom. Also controls which edges respond to scrolling.
    pub strengths: [f32; 4],
}

impl Default for EdgeFade {
    fn default() -> Self {
        Self::new(20.0)
    }
}

impl EdgeFade {
    #[must_use]
    pub const fn new(width: f32) -> Self {
        Self {
            width,
            intensity: 1.0,
            strengths: [1.0; 4],
        }
    }

    #[must_use]
    pub const fn intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity;
        self
    }

    #[must_use]
    pub const fn strengths(mut self, strengths: [f32; 4]) -> Self {
        self.strengths = strengths;
        self
    }

    #[must_use]
    pub fn filter(self) -> Filter {
        self.make_filter(false, Color::TRANSPARENT)
    }

    #[must_use]
    pub fn scroll(self) -> ScrollEffect {
        self.scroll_with(0.0, 12.0)
    }

    #[must_use]
    pub fn scroll_with(self, threshold: f32, ramp: f32) -> ScrollEffect {
        self.bind(self.filter(), threshold, ramp)
    }

    fn make_filter(self, shadow: bool, color: Color) -> Filter {
        Filter::Effect(EffectInstance::new(
            if shadow { EDGE_SHADOW_ID } else { EDGE_FADE_ID },
            [
                ("width", EffectValue::LogicalPixels(positive(self.width))),
                (
                    "intensity",
                    EffectValue::F32(positive(self.intensity).min(1.0)),
                ),
                (
                    "edges",
                    EffectValue::Vec4(self.strengths.map(|v| positive(v).min(1.0))),
                ),
                ("local-x", EffectValue::Vec3([1.0, 0.0, 0.0])),
                ("local-y", EffectValue::Vec3([0.0, 1.0, 0.0])),
                // Zero selects the layer dimensions for static, non-scroll use.
                ("viewport-width", EffectValue::LogicalPixels(0.0)),
                ("viewport-height", EffectValue::LogicalPixels(0.0)),
                ("color", EffectValue::Color(color)),
                ("shadow", EffectValue::Bool(shadow)),
            ],
        ))
    }

    fn bind(self, filter: Filter, threshold: f32, ramp: f32) -> ScrollEffect {
        let strengths = self.strengths.map(|value| {
            if positive(self.width) > 0.0 && positive(self.intensity) > 0.0 {
                value
            } else {
                0.0
            }
        });
        ScrollEffect::new(LayerStyle::new(Default::default()).filter(filter))
            .when_edges(strengths, threshold, ramp)
            .bind(
                0,
                "edges",
                ScrollMetric::Edges {
                    strengths,
                    threshold,
                    ramp,
                },
            )
            .bind(0, "local-x", ScrollMetric::LocalX)
            .bind(0, "local-y", ScrollMetric::LocalY)
            .bind(0, "viewport-width", ScrollMetric::ViewportWidth)
            .bind(0, "viewport-height", ScrollMetric::ViewportHeight)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EdgeShadow {
    pub edges: EdgeFade,
    pub color: Color,
}

impl EdgeShadow {
    #[must_use]
    pub const fn new(width: f32, color: Color) -> Self {
        Self {
            edges: EdgeFade::new(width),
            color,
        }
    }

    #[must_use]
    pub const fn intensity(mut self, intensity: f32) -> Self {
        self.edges.intensity = intensity;
        self
    }

    #[must_use]
    pub const fn strengths(mut self, strengths: [f32; 4]) -> Self {
        self.edges.strengths = strengths;
        self
    }

    #[must_use]
    pub fn filter(self) -> Filter {
        self.edges.make_filter(true, self.color)
    }

    #[must_use]
    pub fn scroll(self) -> ScrollEffect {
        self.scroll_with(0.0, 12.0)
    }

    #[must_use]
    pub fn scroll_with(self, threshold: f32, ramp: f32) -> ScrollEffect {
        self.edges.bind(self.filter(), threshold, ramp)
    }
}

fn positive(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}
