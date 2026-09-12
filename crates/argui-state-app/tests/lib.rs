use std::time::Duration;

use argui::{
    platform::{TrayAction, TrayMenuItem, WindowKey},
    runtime::{AnimationProfile, RuntimeEvent, WindowRuntimeEvent},
    ui::TreeUpdate,
};
use argui_state_app::{application_config, profile_message, renderer_config};

#[test]
fn packaged_application_configuration_has_its_identity_window_icons_and_quit_item() {
    let config = application_config().unwrap();

    assert_eq!(config.identity.id.as_str(), "dev.argui.state");
    assert_eq!(config.identity.display_name, "Argui state showcase");
    assert_eq!(config.identity.linux_application_id(), "argui-state");
    assert_eq!(
        config
            .identity
            .icons
            .icons()
            .iter()
            .map(|icon| (icon.width, icon.height))
            .collect::<Vec<_>>(),
        [(32, 32), (128, 128), (256, 256)]
    );

    assert_eq!(config.windows.len(), 1);
    assert_eq!(config.windows[0].key, WindowKey::main());
    assert_eq!(config.windows[0].window.title, "Argui state showcase");
    assert_eq!(config.windows[0].window.width, 1050.0);
    assert_eq!(config.windows[0].window.height, 680.0);

    let tray = config.tray.as_ref().unwrap();
    assert_eq!(tray.tooltip.as_deref(), Some("Argui state showcase"));
    assert_eq!(tray.menu.len(), 1);
    assert!(matches!(
        &tray.menu[0],
        TrayMenuItem::Action {
            id,
            label,
            enabled: true,
            action: TrayAction::Quit,
        } if id.as_str() == "quit" && label == "Quit Argui"
    ));
    assert!(config.validate().is_ok());
}

#[test]
fn renderer_configuration_preserves_the_profile_switch_and_devtools_effects() {
    for profiling in [false, true] {
        let config = renderer_config(profiling).unwrap();
        assert_eq!(config.profiling, profiling);
        assert!(config.effects.definitions().len() >= 2);
    }
}

#[test]
fn profile_messages_cover_animation_render_and_unhandled_events() {
    let animation = AnimationProfile {
        frame_interval: Duration::from_millis(16),
        model_time: Duration::from_millis(2),
        tree_time: Duration::from_millis(3),
        paint_time: Duration::from_millis(4),
        tree_update: TreeUpdate::Paint,
    };
    assert_eq!(
        profile_message(true, RuntimeEvent::AnimationProfile(animation)),
        Some("[argui][ui] frame=16.00ms model=2.00ms tree=3.00ms paint=4.00ms update=Paint".into())
    );
    assert!(
        profile_message(
            true,
            RuntimeEvent::Window {
                window: WindowKey::new("secondary"),
                event: WindowRuntimeEvent::AnimationProfile(animation),
            }
        )
        .unwrap()
        .starts_with("[argui][ui]")
    );

    let render = argui::render::RenderProfile {
        cpu_time: Duration::from_millis(5),
        effects: argui::render::EffectGraphStats {
            offscreen_layers: 2,
            layers: 5,
            filter_passes: 3,
            offscreen_pixels: 4_096,
            ..argui::render::EffectGraphStats::default()
        },
        texture_pool: argui::render::TexturePoolStats {
            textures: 7,
            reused_this_frame: 4,
            allocated_bytes: 2 * 1024 * 1024,
            ..argui::render::TexturePoolStats::default()
        },
        ..argui::render::RenderProfile::default()
    };
    assert_eq!(
        profile_message(true, RuntimeEvent::RenderProfile(Box::new(render.clone()))),
        Some(
            "[argui][gpu] cpu=5.00ms layers=2/5 passes=3 pixels=4096 textures=7 reused=4 memory=2.0MiB"
                .into()
        )
    );
    assert!(
        profile_message(
            true,
            RuntimeEvent::Window {
                window: WindowKey::new("secondary"),
                event: WindowRuntimeEvent::RenderProfile(Box::new(render)),
            }
        )
        .unwrap()
        .starts_with("[argui][gpu]")
    );
    assert_eq!(profile_message(true, RuntimeEvent::RendererReady), None);
    assert_eq!(
        profile_message(false, RuntimeEvent::AnimationProfile(animation)),
        None
    );
}
