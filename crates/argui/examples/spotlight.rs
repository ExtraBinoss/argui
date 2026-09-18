use argui::{
    core::BackdropMaterial,
    platform::{
        ApplicationConfig, ApplicationIdentity, CloseBehavior, GlobalShortcut, TrayAction,
        TrayConfig, TrayItemId, TrayMenuItem, WindowConfig, WindowKey, WindowLevel,
    },
    render::RendererConfig,
    runtime::{RuntimeEvent, run_application_with_text_engine},
    text::TextEngine,
};
use argui_showcase::spotlight::{
    SPOTLIGHT_SHORTCUT_ID, SPOTLIGHT_WINDOW_HEIGHT, SPOTLIGHT_WINDOW_WIDTH, SpotlightShowcase,
};

const NOTO_SANS: &[u8] = include_bytes!("../../argui-web-demo/assets/fonts/NotoSans-Regular.ttf");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let text = TextEngine::from_embedded_fonts([NOTO_SANS], "Noto Sans", "Noto Sans", "Noto Sans");
    run_application_with_text_engine(
        ApplicationConfig::new(
            ApplicationIdentity::development("Argui Spotlight"),
            WindowConfig {
                title: "Argui Spotlight".into(),
                width: SPOTLIGHT_WINDOW_WIDTH,
                height: SPOTLIGHT_WINDOW_HEIGHT,
                decorations: false,
                resizable: true,
                transparent: true,
                desktop_backdrop: Some(BackdropMaterial::Glass),
                native_shadow: cfg!(any(target_os = "windows", target_os = "macos")),
                level: WindowLevel::Normal,
                close_behavior: CloseBehavior::Hide,
                ..WindowConfig::default()
            },
        )
        .with_global_shortcut(GlobalShortcut::new(
            SPOTLIGHT_SHORTCUT_ID,
            "CmdOrCtrl+Space",
        ))
        .with_tray(TrayConfig {
            tooltip: Some("Argui Spotlight — Cmd/Ctrl+Space".into()),
            menu: vec![
                TrayMenuItem::Action {
                    id: TrayItemId::new("show"),
                    label: "Show Spotlight".into(),
                    enabled: true,
                    action: TrayAction::FocusWindow(WindowKey::main()),
                },
                TrayMenuItem::Separator,
                TrayMenuItem::Action {
                    id: TrayItemId::new("quit"),
                    label: "Quit".into(),
                    enabled: true,
                    action: TrayAction::Quit,
                },
            ],
            ..TrayConfig::default()
        }),
        RendererConfig::default(),
        text,
        SpotlightShowcase::default(),
        |event| {
            if matches!(
                event,
                RuntimeEvent::CommandFailed(_)
                    | RuntimeEvent::RendererFailed(_)
                    | RuntimeEvent::GlobalShortcutsFailed(_)
                    | RuntimeEvent::GlobalShortcutsUnavailable(_)
                    | RuntimeEvent::TrayFailed(_)
                    | RuntimeEvent::TrayUnavailable(_)
            ) {
                eprintln!("{event:?}");
            }
        },
    )?;
    Ok(())
}
