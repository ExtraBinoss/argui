use argui_animation::{Compose, Composition, MotionBinding, MotionTrack};
use argui_core::{Color, Point, Transform2D};
use argui_paint::{
    BorderWidths, CornerRadii, EffectId, Fill, GradientStop, GradientStops, LayerMask, LayerStyle,
    QuadStyle,
};

use crate::LayoutStyle;

mod effect;
mod layout;
use layout::{layout_value, set_layout_value};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BindingImpact {
    Paint,
    Layout,
    Scroll,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GradientPointTarget {
    LinearStart,
    LinearEnd,
    RadialCenter,
    RadialRadius,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EffectTarget {
    pub effect: EffectId,
    pub parameter: &'static str,
}

impl PropertyBinding {
    #[must_use]
    pub const fn impact(&self) -> BindingImpact {
        match self {
            Self::Layout(..) => BindingImpact::Layout,
            Self::Scroll(_) => BindingImpact::Scroll,
            _ => BindingImpact::Paint,
        }
    }

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

pub trait MotionProperty: private::Sealed {
    type Value;

    #[doc(hidden)]
    fn into_binding(self, binding: MotionBinding<Self::Value>) -> PropertyBinding;
}

pub mod property {
    use super::{
        EffectId, EffectMotion, EffectTarget, GradientPointTarget, LayoutTarget, MotionBinding,
        MotionProperty, PropertyBinding, private,
    };
    use argui_core::{Color, Point, Transform2D};

    macro_rules! simple_property {
        ($name:ident, $value:ty, $variant:ident) => {
            #[derive(Clone, Copy, Debug, Default)]
            pub struct $name;

            impl private::Sealed for $name {}

            impl MotionProperty for $name {
                type Value = $value;

                fn into_binding(self, binding: MotionBinding<Self::Value>) -> PropertyBinding {
                    PropertyBinding::$variant(binding)
                }
            }
        };
    }

    macro_rules! layout_property {
        ($name:ident, $target:ident) => {
            #[derive(Clone, Copy, Debug, Default)]
            pub struct $name;

            impl private::Sealed for $name {}

            impl MotionProperty for $name {
                type Value = f32;

                fn into_binding(self, binding: MotionBinding<f32>) -> PropertyBinding {
                    PropertyBinding::Layout(LayoutTarget::$target, binding)
                }
            }
        };
    }

    simple_property!(Transform, Transform2D, Transform);
    simple_property!(BackgroundColor, Color, BackgroundColor);
    simple_property!(BorderColor, Color, BorderColor);
    simple_property!(BorderWidths, [f32; 4], BorderWidths);
    simple_property!(CornerRadii, [f32; 4], CornerRadii);
    simple_property!(Opacity, f32, Opacity);
    simple_property!(LayerOpacity, f32, LayerOpacity);
    simple_property!(LayerMaskRadii, [f32; 4], LayerMaskRadii);
    simple_property!(Scroll, Point, Scroll);

    macro_rules! indexed_property {
        ($name:ident, $constructor:ident, $value:ty, $variant:ident) => {
            #[derive(Clone, Copy, Debug)]
            pub struct $name(usize);

            #[must_use]
            pub const fn $constructor(index: usize) -> $name {
                $name(index)
            }

            impl private::Sealed for $name {}

            impl MotionProperty for $name {
                type Value = $value;

                fn into_binding(self, binding: MotionBinding<Self::Value>) -> PropertyBinding {
                    PropertyBinding::$variant(self.0, binding)
                }
            }
        };
    }

    indexed_property!(
        GradientStopOffset,
        gradient_stop_offset,
        f32,
        GradientStopOffset
    );
    indexed_property!(
        GradientStopColor,
        gradient_stop_color,
        Color,
        GradientStopColor
    );
    indexed_property!(ShadowOffset, shadow_offset, [f32; 2], ShadowOffset);
    indexed_property!(ShadowBlur, shadow_blur, f32, ShadowBlur);
    indexed_property!(ShadowSpread, shadow_spread, f32, ShadowSpread);
    indexed_property!(ShadowColor, shadow_color, Color, ShadowColor);

    macro_rules! gradient_point_property {
        ($name:ident, $target:ident) => {
            #[derive(Clone, Copy, Debug, Default)]
            pub struct $name;

            impl private::Sealed for $name {}

            impl MotionProperty for $name {
                type Value = Point;

                fn into_binding(self, binding: MotionBinding<Point>) -> PropertyBinding {
                    PropertyBinding::GradientPoint(GradientPointTarget::$target, binding)
                }
            }
        };
    }

    gradient_point_property!(LinearGradientStart, LinearStart);
    gradient_point_property!(LinearGradientEnd, LinearEnd);
    gradient_point_property!(RadialGradientCenter, RadialCenter);
    gradient_point_property!(RadialGradientRadius, RadialRadius);

    layout_property!(WidthPx, WidthPx);
    layout_property!(WidthPercent, WidthPercent);
    layout_property!(HeightPx, HeightPx);
    layout_property!(HeightPercent, HeightPercent);
    layout_property!(MinWidthPx, MinWidthPx);
    layout_property!(MinWidthPercent, MinWidthPercent);
    layout_property!(MinHeightPx, MinHeightPx);
    layout_property!(MinHeightPercent, MinHeightPercent);
    layout_property!(MaxWidthPx, MaxWidthPx);
    layout_property!(MaxWidthPercent, MaxWidthPercent);
    layout_property!(MaxHeightPx, MaxHeightPx);
    layout_property!(MaxHeightPercent, MaxHeightPercent);
    layout_property!(PaddingLeft, PaddingLeft);
    layout_property!(PaddingRight, PaddingRight);
    layout_property!(PaddingTop, PaddingTop);
    layout_property!(PaddingBottom, PaddingBottom);
    layout_property!(Gap, Gap);
    layout_property!(Grow, Grow);
    layout_property!(Shrink, Shrink);
    layout_property!(InsetLeftPx, InsetLeftPx);
    layout_property!(InsetRightPx, InsetRightPx);
    layout_property!(InsetTopPx, InsetTopPx);
    layout_property!(InsetBottomPx, InsetBottomPx);

    macro_rules! effect_property {
        ($name:ident, $constructor:ident, $value:ty, $variant:ident) => {
            #[derive(Clone, Copy, Debug)]
            pub struct $name(EffectTarget);

            #[must_use]
            pub const fn $constructor(effect: EffectId, parameter: &'static str) -> $name {
                $name(EffectTarget { effect, parameter })
            }

            impl private::Sealed for $name {}

            impl MotionProperty for $name {
                type Value = $value;

                fn into_binding(self, binding: MotionBinding<Self::Value>) -> PropertyBinding {
                    PropertyBinding::Effect(EffectMotion::$variant(self.0, binding))
                }
            }
        };
    }

    effect_property!(EffectF32, effect_f32, f32, F32);
    effect_property!(
        EffectLogicalPixels,
        effect_logical_pixels,
        f32,
        LogicalPixels
    );
    effect_property!(EffectVec2, effect_vec2, [f32; 2], Vec2);
    effect_property!(EffectVec3, effect_vec3, [f32; 3], Vec3);
    effect_property!(EffectVec4, effect_vec4, [f32; 4], Vec4);
    effect_property!(EffectMat3, effect_mat3, [f32; 9], Mat3);
    effect_property!(EffectMat4, effect_mat4, [f32; 16], Mat4);
    effect_property!(EffectColor, effect_color, Color, Color);
}

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

pub(crate) fn resolved_layout(bindings: &[PropertyBinding], base: &LayoutStyle) -> LayoutStyle {
    let mut style = base.clone();
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

mod private {
    pub trait Sealed {}
}
