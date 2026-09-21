use argui_animation::{Compose, Composition, MotionBinding, MotionTrack};
use argui_core::{Color, Name, Point, Transform2D};
use argui_paint::{
    BorderWidths, CornerRadii, EffectId, Fill, GradientStop, GradientStops, LayerMask, LayerStyle,
    QuadStyle,
};

use crate::LayoutStyle;

mod effect;
pub(crate) mod layout;
use layout::{layout_value, set_layout_value};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BindingImpact {
    Composite,
    Paint,
    Layout,
    Scroll,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LayoutTarget {
    WidthPx,
    WidthPercent,
    HeightPx,
    HeightPercent,
    MinWidthPx,
    MinWidthPercent,
    MinHeightPx,
    MinHeightPercent,
    MaxWidthPx,
    MaxWidthPercent,
    MaxHeightPx,
    MaxHeightPercent,
    PaddingLeft,
    PaddingRight,
    PaddingTop,
    PaddingBottom,
    Gap,
    Grow,
    Shrink,
    InsetLeftPx,
    InsetRightPx,
    InsetTopPx,
    InsetBottomPx,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum GradientPointTarget {
    LinearStart,
    LinearEnd,
    RadialCenter,
    RadialRadius,
    ConicCenter,
}

#[derive(Clone, Debug, PartialEq)]
#[doc(hidden)]
pub enum PropertyBinding {
    Transform(MotionBinding<Transform2D>),
    BackgroundColor(MotionBinding<Color>),
    BorderColor(MotionBinding<Color>),
    BorderWidths(MotionBinding<[f32; 4]>),
    CornerRadii(MotionBinding<[f32; 4]>),
    Opacity(MotionBinding<f32>),
    GradientPoint(GradientPointTarget, MotionBinding<Point>),
    GradientStopOffset(usize, MotionBinding<f32>),
    GradientStopColor(usize, MotionBinding<Color>),
    LayerOpacity(MotionBinding<f32>),
    LayerMaskRadii(MotionBinding<[f32; 4]>),
    ShadowOffset(usize, MotionBinding<[f32; 2]>),
    ShadowBlur(usize, MotionBinding<f32>),
    ShadowSpread(usize, MotionBinding<f32>),
    ShadowColor(usize, MotionBinding<Color>),
    Layout(LayoutTarget, MotionBinding<f32>),
    Scroll(MotionBinding<Point>),
    Effect(EffectMotion),
}

#[derive(Clone, Debug, PartialEq)]
#[doc(hidden)]
pub enum EffectMotion {
    F32(EffectTarget, MotionBinding<f32>),
    LogicalPixels(EffectTarget, MotionBinding<f32>),
    Vec2(EffectTarget, MotionBinding<[f32; 2]>),
    Vec3(EffectTarget, MotionBinding<[f32; 3]>),
    Vec4(EffectTarget, MotionBinding<[f32; 4]>),
    Mat3(EffectTarget, MotionBinding<[f32; 9]>),
    Mat4(EffectTarget, MotionBinding<[f32; 16]>),
    Color(EffectTarget, MotionBinding<Color>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectTarget {
    pub effect: EffectId,
    pub parameter: Name,
}

impl PropertyBinding {
    /// Returns which part of the UI must be invalidated when this binding changes.
    #[must_use]
    pub const fn impact(&self) -> BindingImpact {
        match self {
            Self::Transform(_) | Self::LayerOpacity(_) => BindingImpact::Composite,
            Self::Layout(..) | Self::BorderWidths(_) => BindingImpact::Layout,
            Self::Scroll(_) => BindingImpact::Scroll,
            _ => BindingImpact::Paint,
        }
    }

    /// Returns the animation track used by this property binding.
    #[must_use]
    pub fn track(&self) -> &dyn MotionTrack {
        match self {
            Self::Transform(binding) => &binding.motion,
            Self::BackgroundColor(binding)
            | Self::BorderColor(binding)
            | Self::GradientStopColor(_, binding)
            | Self::ShadowColor(_, binding) => &binding.motion,
            Self::BorderWidths(binding)
            | Self::CornerRadii(binding)
            | Self::LayerMaskRadii(binding) => &binding.motion,
            Self::Opacity(binding)
            | Self::GradientStopOffset(_, binding)
            | Self::LayerOpacity(binding)
            | Self::ShadowBlur(_, binding)
            | Self::ShadowSpread(_, binding)
            | Self::Layout(_, binding) => &binding.motion,
            Self::GradientPoint(_, binding) => &binding.motion,
            Self::ShadowOffset(_, binding) => &binding.motion,
            Self::Scroll(binding) => &binding.motion,
            Self::Effect(effect) => effect.track(),
        }
    }

    fn priority(&self) -> i32 {
        match self {
            Self::Transform(binding) => binding.priority,
            Self::BackgroundColor(binding)
            | Self::BorderColor(binding)
            | Self::GradientStopColor(_, binding)
            | Self::ShadowColor(_, binding) => binding.priority,
            Self::BorderWidths(binding)
            | Self::CornerRadii(binding)
            | Self::LayerMaskRadii(binding) => binding.priority,
            Self::Opacity(binding)
            | Self::GradientStopOffset(_, binding)
            | Self::LayerOpacity(binding)
            | Self::ShadowBlur(_, binding)
            | Self::ShadowSpread(_, binding)
            | Self::Layout(_, binding) => binding.priority,
            Self::GradientPoint(_, binding) => binding.priority,
            Self::ShadowOffset(_, binding) => binding.priority,
            Self::Scroll(binding) => binding.priority,
            Self::Effect(effect) => effect.priority(),
        }
    }
}

impl EffectMotion {
    fn track(&self) -> &dyn MotionTrack {
        match self {
            Self::F32(_, binding) | Self::LogicalPixels(_, binding) => &binding.motion,
            Self::Vec2(_, binding) => &binding.motion,
            Self::Vec3(_, binding) => &binding.motion,
            Self::Vec4(_, binding) => &binding.motion,
            Self::Mat3(_, binding) => &binding.motion,
            Self::Mat4(_, binding) => &binding.motion,
            Self::Color(_, binding) => &binding.motion,
        }
    }

    fn priority(&self) -> i32 {
        match self {
            Self::F32(_, binding) | Self::LogicalPixels(_, binding) => binding.priority,
            Self::Vec2(_, binding) => binding.priority,
            Self::Vec3(_, binding) => binding.priority,
            Self::Vec4(_, binding) => binding.priority,
            Self::Mat3(_, binding) => binding.priority,
            Self::Mat4(_, binding) => binding.priority,
            Self::Color(_, binding) => binding.priority,
        }
    }
}

/// Sealed mapping from an animatable property marker to its value and binding type.
pub trait MotionProperty: private::Sealed {
    /// Value type animated or set by this property.
    type Value;

    #[doc(hidden)]
    /// Converts this property and motion binding into the internal binding representation.
    ///
    /// * `binding` — motion applied to the property value.
    fn into_binding(self, binding: MotionBinding<Self::Value>) -> PropertyBinding;
}

pub mod property;

pub(crate) fn sort_bindings(bindings: &mut [PropertyBinding]) {
    bindings.sort_by_key(PropertyBinding::priority);
}

pub(crate) fn resolved_transform(bindings: &[PropertyBinding], base: Transform2D) -> Transform2D {
    bindings
        .iter()
        .fold(base, |value, property| match property {
            PropertyBinding::Transform(binding) => apply(value, binding),
            _ => value,
        })
}

pub(crate) fn resolved_opacity(bindings: &[PropertyBinding], base: f32) -> f32 {
    bindings
        .iter()
        .fold(base, |value, property| match property {
            PropertyBinding::Opacity(binding) => apply(value, binding),
            _ => value,
        })
}

pub(crate) fn resolved_quad(bindings: &[PropertyBinding], mut base: QuadStyle) -> QuadStyle {
    base.opacity = resolved_opacity(bindings, base.opacity);
    for property in bindings {
        match property {
            PropertyBinding::BackgroundColor(binding) => {
                if let Some(Fill::Solid(color)) = &mut base.background {
                    *color = apply(*color, binding);
                }
            }
            PropertyBinding::BorderColor(binding) => {
                if let Some(border) = &mut base.border {
                    border.color = apply(border.color, binding);
                }
            }
            PropertyBinding::BorderWidths(binding) => {
                if let Some(border) = &mut base.border {
                    border.widths = border_widths(apply(border.widths.as_array(), binding));
                }
            }
            PropertyBinding::CornerRadii(binding) => {
                base.radii = corner_radii(apply(base.radii.as_array(), binding));
            }
            PropertyBinding::GradientPoint(target, binding) => {
                resolve_gradient_point(&mut base.background, *target, binding);
            }
            PropertyBinding::GradientStopOffset(index, binding) => {
                resolve_gradient_stop(&mut base.background, *index, |stop| {
                    stop.offset = apply(stop.offset, binding);
                });
            }
            PropertyBinding::GradientStopColor(index, binding) => {
                resolve_gradient_stop(&mut base.background, *index, |stop| {
                    stop.color = apply(stop.color, binding);
                });
            }
            _ => {}
        }
    }
    base
}

pub(crate) fn resolved_layout(bindings: &[PropertyBinding], mut style: LayoutStyle) -> LayoutStyle {
    for binding in bindings {
        let PropertyBinding::Layout(target, binding) = binding else {
            continue;
        };
        let base = layout_value(&style, *target);
        set_layout_value(&mut style, *target, apply(base, binding));
    }
    style
}

pub(crate) fn resolved_scroll(bindings: &[PropertyBinding], base: Point) -> Point {
    bindings
        .iter()
        .fold(base, |value, property| match property {
            PropertyBinding::Scroll(binding) => apply(value, binding),
            _ => value,
        })
}

pub(crate) fn resolved_layer(bindings: &[PropertyBinding], base: &LayerStyle) -> LayerStyle {
    let mut layer = base.clone();
    for property in bindings {
        match property {
            PropertyBinding::LayerOpacity(binding) => {
                layer.opacity = apply(layer.opacity, binding);
            }
            PropertyBinding::LayerMaskRadii(binding) => {
                if let LayerMask::Rounded(radii) = layer.mask {
                    layer.mask = LayerMask::Rounded(corner_radii(apply(radii.as_array(), binding)));
                }
            }
            PropertyBinding::ShadowOffset(index, binding) => {
                if let Some(shadow) = layer.shadows.get_mut(*index) {
                    shadow.offset = apply(shadow.offset, binding);
                }
            }
            PropertyBinding::ShadowBlur(index, binding) => {
                if let Some(shadow) = layer.shadows.get_mut(*index) {
                    shadow.blur = apply(shadow.blur, binding);
                }
            }
            PropertyBinding::ShadowSpread(index, binding) => {
                if let Some(shadow) = layer.shadows.get_mut(*index) {
                    shadow.spread = apply(shadow.spread, binding);
                }
            }
            PropertyBinding::ShadowColor(index, binding) => {
                if let Some(shadow) = layer.shadows.get_mut(*index) {
                    shadow.color = apply(shadow.color, binding);
                }
            }
            PropertyBinding::Effect(motion) => effect::resolve(&mut layer, motion),
            _ => {}
        }
    }
    layer
}

fn resolve_gradient_point(
    fill: &mut Option<Fill>,
    target: GradientPointTarget,
    binding: &MotionBinding<Point>,
) {
    match (fill, target) {
        (Some(Fill::Linear(gradient)), GradientPointTarget::LinearStart) => {
            gradient.start = apply(gradient.start, binding);
        }
        (Some(Fill::Linear(gradient)), GradientPointTarget::LinearEnd) => {
            gradient.end = apply(gradient.end, binding);
        }
        (Some(Fill::Radial(gradient)), GradientPointTarget::RadialCenter) => {
            gradient.center = apply(gradient.center, binding);
        }
        (Some(Fill::Radial(gradient)), GradientPointTarget::RadialRadius) => {
            gradient.radius = apply(gradient.radius, binding);
        }
        (Some(Fill::Conic(gradient)), GradientPointTarget::ConicCenter) => {
            gradient.center = apply(gradient.center, binding);
        }
        _ => {}
    }
}

fn resolve_gradient_stop(
    fill: &mut Option<Fill>,
    index: usize,
    resolve: impl FnOnce(&mut GradientStop),
) {
    let stops = match fill {
        Some(Fill::Linear(gradient)) => &mut gradient.stops,
        Some(Fill::Radial(gradient)) => &mut gradient.stops,
        _ => return,
    };
    let mut values = stops.as_slice().to_vec();
    let Some(stop) = values.get_mut(index) else {
        return;
    };
    resolve(stop);
    if let Ok(resolved) = GradientStops::from_vec(values) {
        *stops = resolved;
    }
}

const fn border_widths(values: [f32; 4]) -> BorderWidths {
    BorderWidths {
        left: values[0],
        right: values[1],
        top: values[2],
        bottom: values[3],
    }
}

const fn corner_radii(values: [f32; 4]) -> CornerRadii {
    CornerRadii {
        top_left: values[0],
        top_right: values[1],
        bottom_right: values[2],
        bottom_left: values[3],
    }
}

pub(super) fn apply<T: Compose + Copy>(base: T, binding: &MotionBinding<T>) -> T {
    let value = binding.motion.value();
    match binding.composition {
        Composition::Replace => value,
        Composition::Add => base.add(value),
        Composition::Accumulate => {
            base.add(value.scale(binding.motion.completed_iterations() as f32 + 1.0))
        }
    }
}

pub(crate) mod private {
    pub trait Sealed {}
}
