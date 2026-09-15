//! Paint-only bindings between scroll geometry and registered graphic effects.
use argui_core::{Affine2D, Point, Rect};
use argui_paint::{EffectValue, Filter, LayerStyle};

/// Offsets are positive distances from the beginning, including during elastic scroll.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollMetrics {
    pub viewport: Rect,
    pub transform: Affine2D,
    pub offset: Point,
    pub max_offset: Point,
}

/// Four-component values use left, top, right, bottom order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScrollMetric {
    Offset,
    Progress,
    Remaining,
    Edges {
        strengths: [f32; 4],
        threshold: f32,
        ramp: f32,
    },
    ViewportWidth,
    ViewportHeight,
    /// Maps normalized, transformed layer coordinates to local viewport coordinates.
    LocalX,
    LocalY,
}

impl ScrollMetrics {
    /// Returns distances from the viewport's left, top, right, and bottom edges.
    #[must_use]
    pub fn remaining(self) -> [f32; 4] {
        let x = positive(self.max_offset.x);
        let y = positive(self.max_offset.y);
        let left = positive(self.offset.x).min(x);
        let top = positive(self.offset.y).min(y);
        [left, top, x - left, y - top]
    }

    /// Resolves one scroll metric into the effect value expected by its binding.
    ///
    /// * `source` — metric to evaluate from this scroll state.
    #[must_use]
    pub fn value(self, source: ScrollMetric) -> EffectValue {
        let distances = self.remaining();
        match source {
            ScrollMetric::Offset => EffectValue::Vec2([distances[0], distances[1]]),
            ScrollMetric::Progress => EffectValue::Vec2([
                distances[0] / positive(self.max_offset.x).max(f32::EPSILON),
                distances[1] / positive(self.max_offset.y).max(f32::EPSILON),
            ]),
            ScrollMetric::Remaining => EffectValue::Vec4(distances),
            ScrollMetric::Edges {
                strengths,
                threshold,
                ramp,
            } => EffectValue::Vec4(std::array::from_fn(|index| {
                let value = ((distances[index] - positive(threshold))
                    / positive(ramp).max(f32::EPSILON))
                .clamp(0.0, 1.0);
                value * value * (3.0 - 2.0 * value) * positive(strengths[index]).min(1.0)
            })),
            ScrollMetric::ViewportWidth => {
                EffectValue::LogicalPixels(positive(self.viewport.size.width))
            }
            ScrollMetric::ViewportHeight => {
                EffectValue::LogicalPixels(positive(self.viewport.size.height))
            }
            ScrollMetric::LocalX | ScrollMetric::LocalY => {
                let bounds = self.transform.transform_rect(self.viewport);
                let inverse = self.transform.inverse().unwrap_or(Affine2D::IDENTITY);
                let origin = inverse.transform_point(bounds.origin);
                let (a, b, translation, extent) = if source == ScrollMetric::LocalX {
                    (
                        inverse.matrix[0],
                        inverse.matrix[2],
                        origin.x - self.viewport.origin.x,
                        self.viewport.size.width,
                    )
                } else {
                    (
                        inverse.matrix[1],
                        inverse.matrix[3],
                        origin.y - self.viewport.origin.y,
                        self.viewport.size.height,
                    )
                };
                let extent = positive(extent).max(f32::EPSILON);
                EffectValue::Vec3([
                    a * bounds.size.width / extent,
                    b * bounds.size.height / extent,
                    translation / extent,
                ])
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScrollEffect {
    pub layer: LayerStyle,
    bindings: Vec<ScrollEffectBinding>,
    active_edges: Option<ScrollMetric>,
}

#[derive(Clone, Debug, PartialEq)]
struct ScrollEffectBinding {
    filter: usize,
    parameter: &'static str,
    source: ScrollMetric,
}

impl ScrollEffect {
    /// Creates an effect layer whose parameters can be bound to scroll metrics.
    ///
    /// * `layer` — layer and custom-filter configuration to resolve.
    #[must_use]
    pub fn new(layer: LayerStyle) -> Self {
        Self {
            layer,
            bindings: Vec::new(),
            active_edges: None,
        }
    }

    /// Skips this entire layer when none of the selected edges is visible.
    /// Kept separate from bindings: a zero parameter need not disable a custom shader.
    ///
    /// * `strengths` — left, top, right, and bottom edge strengths.
    /// * `threshold` — edge distance before the effect begins.
    /// * `ramp` — distance over which strength ramps up after the threshold.
    #[must_use]
    pub fn when_edges(mut self, strengths: [f32; 4], threshold: f32, ramp: f32) -> Self {
        self.active_edges = Some(ScrollMetric::Edges {
            strengths,
            threshold,
            ramp,
        });
        self
    }

    /// Binds an existing custom-filter parameter; invalid targets fail at construction.
    ///
    /// * `filter` — index of the custom filter in the layer's filter list.
    /// * `parameter` — name of the parameter to bind.
    /// * `source` — scroll metric used as the parameter value.
    ///
    /// # Panics
    ///
    /// Panics if the filter is missing or is not a custom effect, if the parameter
    /// does not exist, or if its value type does not match the selected metric.
    #[must_use]
    pub fn bind(mut self, filter: usize, parameter: &'static str, source: ScrollMetric) -> Self {
        let Some(Filter::Effect(effect)) = self.layer.filters.get(filter) else {
            panic!("scroll binding requires a custom filter");
        };
        let argument = effect
            .parameters
            .iter()
            .find(|argument| argument.name == parameter)
            .expect("scroll binding requires an existing parameter");
        let value = ScrollMetrics {
            viewport: Rect::default(),
            transform: Affine2D::IDENTITY,
            offset: Point::default(),
            max_offset: Point::default(),
        }
        .value(source);
        assert_eq!(
            std::mem::discriminant(&argument.value),
            std::mem::discriminant(&value),
            "scroll binding parameter type mismatch"
        );
        self.bindings
            .retain(|binding| binding.filter != filter || binding.parameter != parameter);
        self.bindings.push(ScrollEffectBinding {
            filter,
            parameter,
            source,
        });
        self
    }

    /// Inactive effects produce no layer and therefore no offscreen passes.
    ///
    /// * `metrics` — current viewport, transform, offset, and maximum offset.
    ///
    /// Returns the resolved layer, or `None` when the effect is inactive or cannot
    /// be mapped through the current geometry.
    #[must_use]
    pub fn resolve(&self, metrics: ScrollMetrics) -> Option<LayerStyle> {
        if !metrics.remaining().iter().any(|distance| *distance > 0.0)
            || positive(metrics.viewport.size.width) == 0.0
            || positive(metrics.viewport.size.height) == 0.0
            || metrics.transform.inverse().is_none()
        {
            return None;
        }
        if self
            .active_edges
            .is_some_and(|source| metrics.value(source) == EffectValue::Vec4([0.0; 4]))
        {
            return None;
        }
        let mut layer = self.layer.clone();
        for binding in &self.bindings {
            let value = metrics.value(binding.source);
            if let Some(Filter::Effect(effect)) = layer.filters.get_mut(binding.filter)
                && let Some(argument) = effect
                    .parameters
                    .iter_mut()
                    .find(|argument| argument.name == binding.parameter)
            {
                argument.value = value;
            }
        }
        layer.bounds = metrics.transform.transform_rect(metrics.viewport);
        Some(layer)
    }
}

fn positive(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}
