use argui_devtools::configure_renderer;
use argui_render::RendererConfig;

#[test]
fn renderer_configuration_registers_scroll_effects_once_and_preserves_settings() {
    let config = RendererConfig::default().profiling(true);
    let configured = configure_renderer(config).unwrap();
    assert!(configured.profiling);
    assert_eq!(configured.effects.definitions().len(), 2);
    let again = configure_renderer(configured).unwrap();
    assert_eq!(again.effects.definitions().len(), 2);
    for definition in again.effects.definitions() {
        definition.validate().unwrap();
    }
}

#[test]
fn renderer_configuration_adds_only_missing_scroll_effects() {
    let available = argui_effects::registry().unwrap();
    for id in [argui_effects::EDGE_FADE_ID, argui_effects::EDGE_SHADOW_ID] {
        let existing = available.get(&id).unwrap().clone();
        let config = RendererConfig::default()
            .effects(argui_render::EffectRegistry::new([existing.clone()]).unwrap());
        let configured = configure_renderer(config).unwrap();
        assert_eq!(configured.effects.definitions().len(), 2);
        assert_eq!(configured.effects.definitions().first(), Some(&existing));
        assert_eq!(configured.effects.get(&id), Some(&existing));
    }
}

#[test]
fn host_can_disable_or_replace_edge_effects() {
    fn find<'a>(root: &'a argui_ui::Element, key: &str) -> Option<&'a argui_ui::Element> {
        if root.key.as_deref() == Some(key) {
            Some(root)
        } else {
            root.children.iter().find_map(|child| find(child, key))
        }
    }
    let effect = argui_effects::EdgeFade::new(40.0).scroll();
    for effects in [None, Some(effect)] {
        let host = argui_runtime::Entity::new(
            argui_devtools::DevtoolsHost::new(argui_showcase::StateShowcase::default())
                .open(true)
                .scroll_effect(effects.clone()),
        )
        .mount()
        .unwrap();
        let root = host.render(Default::default()).unwrap();
        assert_eq!(
            find(&root, "__devtools-tree")
                .unwrap()
                .scroll
                .as_ref()
                .unwrap()
                .effects,
            effects.into_iter().collect::<Vec<_>>()
        );
    }
    let app = argui_devtools::DevtoolsApp::new(argui_runtime::SingleWindowModel::new(
        argui_showcase::StateShowcase::default(),
    ))
    .scroll_effect(None);
    assert!(!app.inspector().enabled());
}
