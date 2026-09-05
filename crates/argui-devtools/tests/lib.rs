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
fn host_can_disable_or_replace_edge_effects() {
    fn find<'a>(root: &'a argui_ui::Element, key: &str) -> Option<&'a argui_ui::Element> {
        if root.key.as_deref() == Some(key) {
            Some(root)
        } else {
            root.children.iter().find_map(|child| find(child, key))
        }
    }
    let mut host = argui_devtools::DevtoolsHost::new(argui_showcase::StateShowcase::default())
        .open(true)
        .scroll_effect(None);
    assert!(
        find(&host.view(), "__devtools-tree")
            .unwrap()
            .scroll
            .as_ref()
            .unwrap()
            .effects
            .is_empty()
    );
    let effect = argui_effects::EdgeFade::new(40.0).scroll();
    host = host.scroll_effect(Some(effect.clone()));
    assert_eq!(
        find(&host.view(), "__devtools-tree")
            .unwrap()
            .scroll
            .as_ref()
            .unwrap()
            .effects,
        vec![effect]
    );
    let app = argui_devtools::DevtoolsApp::new(argui_runtime::SingleWindowModel::new(
        argui_showcase::StateShowcase::default(),
    ))
    .scroll_effect(None);
    assert!(!app.inspector().enabled());
}
