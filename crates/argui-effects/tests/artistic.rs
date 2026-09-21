#![cfg(feature = "artistic")]

use argui_effects::{
    ANIMATED_GRADIENT_ID, AnimatedGradient, WORLEY_BORDER_FIRE_ID, WorleyBorderFire,
};
use argui_paint::{EffectValue, Filter};

fn parameters(filter: Filter) -> (argui_paint::EffectId, Vec<(String, EffectValue)>, f32) {
    let Filter::Effect(effect) = filter else {
        panic!("artistic presets must produce effect instances");
    };
    (
        effect.id,
        effect
            .parameters
            .into_iter()
            .map(|argument| (argument.name.as_str().to_owned(), argument.value))
            .collect(),
        effect.expansion,
    )
}

#[test]
fn artistic_presets_expose_stable_ids_and_typed_parameters() {
    let (id, values, expansion) = parameters(
        AnimatedGradient::new(0.5)
            .frequency(2.0)
            .intensity(0.75)
            .filter(),
    );
    assert_eq!(id, ANIMATED_GRADIENT_ID);
    assert_eq!(values[0].0, "phase");
    assert_eq!(values[0].1, EffectValue::F32(0.5));
    assert_eq!(values[1].0, "frequency");
    assert_eq!(values[1].1, EffectValue::F32(2.0));
    assert_eq!(values[2].0, "intensity");
    assert_eq!(values[2].1, EffectValue::F32(0.75));
    assert_eq!(expansion, 0.0);

    let (id, values, expansion) = parameters(
        WorleyBorderFire::new(0.2)
            .intensity(0.6)
            .frequency(10.0)
            .expansion(4.0)
            .filter(),
    );
    assert_eq!(id, WORLEY_BORDER_FIRE_ID);
    assert_eq!(values.len(), 4);
    assert_eq!(values[3].0, "expansion");
    assert_eq!(values[3].1, EffectValue::LogicalPixels(4.0));
    assert_eq!(expansion, 4.0);
}
