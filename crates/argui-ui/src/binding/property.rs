use argui_animation::MotionBinding;
use argui_core::{Color, Point, Transform2D};
use argui_paint::{EffectId, Fill};

use super::{
    EffectMotion, EffectTarget, GradientPointTarget, LayoutTarget, MotionProperty, PropertyBinding,
    private,
};
use crate::state::StateValue;
use crate::{EffectPropertyKey, PropertyKey, StyleProperty, StylePropertyValue};

macro_rules! simple_property {
    ($name:ident, $value:ty, $binding:ident, $key:expr, $state:ident) => {
        #[derive(Clone, Copy, Debug, Default)]
        pub struct $name;

        impl private::Sealed for $name {}

        impl MotionProperty for $name {
            type Value = $value;

            fn into_binding(self, binding: MotionBinding<Self::Value>) -> PropertyBinding {
                PropertyBinding::$binding(binding)
            }
        }

        impl StyleProperty for $name {
            type Value = $value;

            fn into_state_value(self, value: Self::Value) -> StylePropertyValue {
                StylePropertyValue {
                    key: $key,
                    value: StateValue::$state(value),
                }
            }
        }
    };
}

simple_property!(
    Transform,
    Transform2D,
    Transform,
    PropertyKey::Transform,
    Transform
);
simple_property!(
    BackgroundColor,
    Color,
    BackgroundColor,
    PropertyKey::BackgroundColor,
    BackgroundColor
);
simple_property!(
    BorderColor,
    Color,
    BorderColor,
    PropertyKey::BorderColor,
    BorderColor
);
simple_property!(
    BorderWidths,
    [f32; 4],
    BorderWidths,
    PropertyKey::BorderWidths,
    BorderWidths
);
simple_property!(
    CornerRadii,
    [f32; 4],
    CornerRadii,
    PropertyKey::CornerRadii,
    CornerRadii
);
simple_property!(Opacity, f32, Opacity, PropertyKey::Opacity, Opacity);

#[derive(Clone, Copy, Debug, Default)]
pub struct TextColor;

impl private::Sealed for TextColor {}

impl StyleProperty for TextColor {
    type Value = Color;

    fn into_state_value(self, value: Self::Value) -> StylePropertyValue {
        StylePropertyValue {
            key: PropertyKey::TextColor,
            value: StateValue::Color(value),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct VectorColor;

impl private::Sealed for VectorColor {}

impl StyleProperty for VectorColor {
    type Value = Color;

    fn into_state_value(self, value: Self::Value) -> StylePropertyValue {
        StylePropertyValue {
            key: PropertyKey::VectorColor,
            value: StateValue::Color(value),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Layout;

impl private::Sealed for Layout {}

impl StyleProperty for Layout {
    type Value = crate::LayoutStyle;

    fn into_state_value(self, value: Self::Value) -> StylePropertyValue {
        StylePropertyValue {
            key: PropertyKey::LayoutStyle,
            value: StateValue::LayoutStyle(Box::new(value)),
        }
    }
}
simple_property!(
    LayerOpacity,
    f32,
    LayerOpacity,
    PropertyKey::LayerOpacity,
    F32
);
simple_property!(
    LayerMaskRadii,
    [f32; 4],
    LayerMaskRadii,
    PropertyKey::LayerMaskRadii,
    CornerRadii
);
simple_property!(Scroll, Point, Scroll, PropertyKey::Scroll, Point);

#[derive(Clone, Copy, Debug, Default)]
pub struct Background;

impl private::Sealed for Background {}

impl StyleProperty for Background {
    type Value = Option<Fill>;

    fn into_state_value(self, value: Self::Value) -> StylePropertyValue {
        StylePropertyValue {
            key: PropertyKey::Background,
            value: StateValue::Background(value),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Border;

impl private::Sealed for Border {}

impl StyleProperty for Border {
    type Value = Option<argui_paint::Border>;

    fn into_state_value(self, value: Self::Value) -> StylePropertyValue {
        StylePropertyValue {
            key: PropertyKey::Border,
            value: StateValue::Border(value),
        }
    }
}

macro_rules! indexed_property {
    ($name:ident, $constructor:ident, $value:ty, $binding:ident, $key:ident, $state:ident) => {
        #[derive(Clone, Copy, Debug)]
        pub struct $name(usize);

        /// Creates a property selector for the item at `index`.
        ///
        /// * `index` — zero-based gradient stop or shadow index.
        #[must_use]
        pub const fn $constructor(index: usize) -> $name {
            $name(index)
        }

        impl private::Sealed for $name {}

        impl MotionProperty for $name {
            type Value = $value;

            fn into_binding(self, binding: MotionBinding<Self::Value>) -> PropertyBinding {
                PropertyBinding::$binding(self.0, binding)
            }
        }

        impl StyleProperty for $name {
            type Value = $value;

            fn into_state_value(self, value: Self::Value) -> StylePropertyValue {
                StylePropertyValue {
                    key: PropertyKey::$key(self.0),
                    value: StateValue::$state(value),
                }
            }
        }
    };
}

indexed_property!(
    GradientStopOffset,
    gradient_stop_offset,
    f32,
    GradientStopOffset,
    GradientStopOffset,
    F32
);
indexed_property!(
    GradientStopColor,
    gradient_stop_color,
    Color,
    GradientStopColor,
    GradientStopColor,
    Color
);
indexed_property!(
    ShadowOffset,
    shadow_offset,
    [f32; 2],
    ShadowOffset,
    ShadowOffset,
    Vec2
);
indexed_property!(ShadowBlur, shadow_blur, f32, ShadowBlur, ShadowBlur, F32);
indexed_property!(
    ShadowSpread,
    shadow_spread,
    f32,
    ShadowSpread,
    ShadowSpread,
    F32
);
indexed_property!(
    ShadowColor,
    shadow_color,
    Color,
    ShadowColor,
    ShadowColor,
    Color
);

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

        impl StyleProperty for $name {
            type Value = Point;

            fn into_state_value(self, value: Point) -> StylePropertyValue {
                StylePropertyValue {
                    key: PropertyKey::GradientPoint(GradientPointTarget::$target),
                    value: StateValue::Point(value),
                }
            }
        }
    };
}

gradient_point_property!(LinearGradientStart, LinearStart);
gradient_point_property!(LinearGradientEnd, LinearEnd);
gradient_point_property!(RadialGradientCenter, RadialCenter);
gradient_point_property!(RadialGradientRadius, RadialRadius);

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

        impl StyleProperty for $name {
            type Value = f32;

            fn into_state_value(self, value: f32) -> StylePropertyValue {
                StylePropertyValue {
                    key: PropertyKey::Layout(LayoutTarget::$target),
                    value: StateValue::F32(value),
                }
            }
        }
    };
}

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
    ($name:ident, $constructor:ident, $value:ty, $binding:ident, $key:ident, $state:ident) => {
        #[derive(Clone, Copy, Debug)]
        pub struct $name(EffectTarget);

        /// Creates a selector for an effect parameter.
        ///
        /// * `effect` — identity of the effect instance.
        /// * `parameter` — name of the parameter within that effect.
        #[must_use]
        pub const fn $constructor(effect: EffectId, parameter: &'static str) -> $name {
            $name(EffectTarget { effect, parameter })
        }

        impl private::Sealed for $name {}

        impl MotionProperty for $name {
            type Value = $value;

            fn into_binding(self, binding: MotionBinding<Self::Value>) -> PropertyBinding {
                PropertyBinding::Effect(EffectMotion::$binding(self.0, binding))
            }
        }

        impl StyleProperty for $name {
            type Value = $value;

            fn into_state_value(self, value: Self::Value) -> StylePropertyValue {
                let target = EffectPropertyKey {
                    effect: self.0.effect,
                    parameter: self.0.parameter,
                };
                StylePropertyValue {
                    key: PropertyKey::$key(target),
                    value: StateValue::$state(value),
                }
            }
        }
    };
}

effect_property!(EffectF32, effect_f32, f32, F32, EffectF32, F32);
effect_property!(
    EffectLogicalPixels,
    effect_logical_pixels,
    f32,
    LogicalPixels,
    EffectLogicalPixels,
    F32
);
effect_property!(EffectVec2, effect_vec2, [f32; 2], Vec2, EffectVec2, Vec2);
effect_property!(EffectVec3, effect_vec3, [f32; 3], Vec3, EffectVec3, Vec3);
effect_property!(EffectVec4, effect_vec4, [f32; 4], Vec4, EffectVec4, Vec4);
effect_property!(EffectMat3, effect_mat3, [f32; 9], Mat3, EffectMat3, Mat3);
effect_property!(EffectMat4, effect_mat4, [f32; 16], Mat4, EffectMat4, Mat4);
effect_property!(EffectColor, effect_color, Color, Color, EffectColor, Color);
