use argui_paint::EffectId;
use argui_platform::WindowKey;
use argui_render::{EffectDefinition, EffectParameter, EffectPassDefinition};
use argui_runtime::{AppModel, Context, Entity, Render, SingleWindowModel};
use argui_ui::Element;

struct ShaderModel {
    revision: u64,
}

impl Render for ShaderModel {
    fn render(&mut self, _: &mut Context<Self>) -> Element {
        Element::container([])
    }

    fn effect_definitions(&self) -> Vec<EffectDefinition> {
        vec![EffectDefinition::new(
            EffectId::new("native.tint"),
            Vec::<EffectParameter>::new(),
            [EffectPassDefinition::fragment(
                "tint",
                "fn argui_effect(uv: vec2<f32>, source: vec4<f32>, backdrop: vec4<f32>) -> vec4<f32> { return vec4<f32>(source.rgb * 0.5, source.a); }",
            )],
        )
        .with_revision(self.revision)]
    }
}

#[test]
fn effect_definitions_follow_retained_model_revisions() {
    let entity = Entity::new(ShaderModel { revision: 1 });
    let model = SingleWindowModel::from_entity(entity.clone()).unwrap();
    let first = model.effect_definitions();
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].id.as_str(), "native.tint");
    assert_eq!(first[0].revision, 1);

    entity.update(|model, cx| {
        model.revision = 2;
        cx.notify();
    });
    assert_eq!(model.effect_definitions()[0].revision, 2);
    assert!(model.view(&WindowKey::main(), Default::default()).is_some());
}
