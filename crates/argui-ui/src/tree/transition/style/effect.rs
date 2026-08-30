use argui_paint::{EffectValue, Filter, LayerStyle};

use crate::state::StateValue;
use crate::{EffectPropertyKey, PropertyKey, StatePropertyValue};

pub(super) fn effect_values(layer: &LayerStyle, values: &mut Vec<StatePropertyValue>) {
    for filter in layer.filters.iter().chain(&layer.backdrop_filters) {
        let Filter::Effect(effect) = filter else {
            continue;
        };
        for argument in &effect.parameters {
            let target = EffectPropertyKey {
                effect: effect.id,
                parameter: argument.name,
            };
            let property = match argument.value {
                EffectValue::F32(value) => {
                    state(PropertyKey::EffectF32(target), StateValue::F32(value))
                }
                EffectValue::LogicalPixels(value) => state(
                    PropertyKey::EffectLogicalPixels(target),
                    StateValue::F32(value),
                ),
                EffectValue::Vec2(value) => {
                    state(PropertyKey::EffectVec2(target), StateValue::Vec2(value))
                }
                EffectValue::Vec3(value) => {
                    state(PropertyKey::EffectVec3(target), StateValue::Vec3(value))
                }
                EffectValue::Vec4(value) => {
                    state(PropertyKey::EffectVec4(target), StateValue::Vec4(value))
                }
                EffectValue::Mat3(value) => {
                    state(PropertyKey::EffectMat3(target), StateValue::Mat3(value))
                }
                EffectValue::Mat4(value) => {
                    state(PropertyKey::EffectMat4(target), StateValue::Mat4(value))
                }
                EffectValue::Color(value) => {
                    state(PropertyKey::EffectColor(target), StateValue::Color(value))
                }
                EffectValue::I32(_) | EffectValue::U32(_) | EffectValue::Bool(_) => continue,
            };
            if !values.iter().any(|value| value.key == property.key) {
                values.push(property);
            }
        }
    }
}

pub(super) fn apply_effect(layer: &mut LayerStyle, key: PropertyKey, value: &StateValue) {
    let (target, effect_value) = match (key, value) {
        (PropertyKey::EffectF32(target), StateValue::F32(value)) => {
            (target, EffectValue::F32(*value))
        }
        (PropertyKey::EffectLogicalPixels(target), StateValue::F32(value)) => {
            (target, EffectValue::LogicalPixels(*value))
        }
        (PropertyKey::EffectVec2(target), StateValue::Vec2(value)) => {
            (target, EffectValue::Vec2(*value))
        }
        (PropertyKey::EffectVec3(target), StateValue::Vec3(value)) => {
            (target, EffectValue::Vec3(*value))
        }
        (PropertyKey::EffectVec4(target), StateValue::Vec4(value)) => {
            (target, EffectValue::Vec4(*value))
        }
        (PropertyKey::EffectMat3(target), StateValue::Mat3(value)) => {
            (target, EffectValue::Mat3(*value))
        }
        (PropertyKey::EffectMat4(target), StateValue::Mat4(value)) => {
            (target, EffectValue::Mat4(*value))
        }
        (PropertyKey::EffectColor(target), StateValue::Color(value)) => {
            (target, EffectValue::Color(*value))
        }
        _ => return,
    };
    for filter in layer.filters.iter_mut().chain(&mut layer.backdrop_filters) {
        if let Filter::Effect(effect) = filter
            && effect.id == target.effect
            && let Some(argument) = effect
                .parameters
                .iter_mut()
                .find(|argument| argument.name == target.parameter)
        {
            argument.value = effect_value.clone();
        }
    }
}

const fn state(key: PropertyKey, value: StateValue) -> StatePropertyValue {
    StatePropertyValue { key, value }
}
