use argui_core::Color;
use argui_paint::{
    EffectArgument, EffectId, EffectInstance, EffectValue, Filter, ImageId, VectorId,
};
use argui_runtime::apply_visual_effect;
use argui_ui::{EffectScope, Element};

#[test]
fn authored_effect_attaches_to_each_visual_kind_without_special_routing() {
    let visuals = [
        Element::container([]).background(Color::BLACK),
        Element::text("label"),
        Element::image(ImageId::fresh()),
        Element::vector(VectorId::fresh()),
    ];
    for visual in visuals {
        let visual = apply_visual_effect(
            visual,
            EffectInstance::new(
                EffectId::new("dsl.glow"),
                [EffectArgument::new("intensity", EffectValue::F32(0.35))],
            ),
        );
        assert_eq!(visual.effects.len(), 1);
        assert_eq!(visual.effects[0].scope, EffectScope::WholeElement);
        assert!(matches!(
            visual.effects[0].layer.filters.as_slice(),
            [Filter::Effect(instance)]
                if instance.id.as_str() == "dsl.glow"
                    && instance.parameters[0].value == EffectValue::F32(0.35)
        ));
    }
}

#[test]
fn authored_effect_keeps_order_and_current_parameter_values() {
    let make = |intensity| {
        EffectInstance::new(
            EffectId::new("dsl.glow"),
            [EffectArgument::new(
                "intensity",
                EffectValue::F32(intensity),
            )],
        )
    };
    let visual = apply_visual_effect(
        apply_visual_effect(Element::container([]), make(0.25)),
        make(0.75),
    );
    assert_eq!(visual.effects.len(), 2);
    let values = visual
        .effects
        .iter()
        .map(|effect| match effect.layer.filters.as_slice() {
            [Filter::Effect(instance)] => instance.parameters[0].value.clone(),
            _ => panic!("expected one authored effect filter"),
        })
        .collect::<Vec<_>>();
    assert_eq!(values, [EffectValue::F32(0.25), EffectValue::F32(0.75)]);
}
