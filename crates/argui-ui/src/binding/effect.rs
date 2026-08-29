use argui_paint::{EffectValue, LayerStyle};

use super::{EffectMotion, EffectTarget, apply};

pub(super) fn resolve(layer: &mut LayerStyle, motion: &EffectMotion) {
    let target = motion.target();
    for filter in layer
        .filters
        .iter_mut()
        .chain(layer.backdrop_filters.iter_mut())
    {
        let argui_paint::Filter::Effect(instance) = filter else {
            continue;
        };
        if instance.id != target.effect {
            continue;
        }
        if let Some(argument) = instance
            .parameters
            .iter_mut()
            .find(|argument| argument.name == target.parameter)
        {
            motion.resolve(&mut argument.value);
        }
    }
}

impl EffectMotion {
    const fn target(&self) -> EffectTarget {
        match self {
            Self::F32(target, _)
            | Self::LogicalPixels(target, _)
            | Self::Vec2(target, _)
            | Self::Vec3(target, _)
            | Self::Vec4(target, _)
            | Self::Mat3(target, _)
            | Self::Mat4(target, _)
            | Self::Color(target, _) => *target,
        }
    }

    fn resolve(&self, value: &mut EffectValue) {
        match (self, value) {
            (Self::F32(_, binding), EffectValue::F32(base)) => *base = apply(*base, binding),
            (Self::LogicalPixels(_, binding), EffectValue::LogicalPixels(base)) => {
                *base = apply(*base, binding);
            }
            (Self::Vec2(_, binding), EffectValue::Vec2(base)) => *base = apply(*base, binding),
            (Self::Vec3(_, binding), EffectValue::Vec3(base)) => *base = apply(*base, binding),
            (Self::Vec4(_, binding), EffectValue::Vec4(base)) => *base = apply(*base, binding),
            (Self::Mat3(_, binding), EffectValue::Mat3(base)) => *base = apply(*base, binding),
            (Self::Mat4(_, binding), EffectValue::Mat4(base)) => *base = apply(*base, binding),
            (Self::Color(_, binding), EffectValue::Color(base)) => *base = apply(*base, binding),
            _ => {}
        }
    }
}
