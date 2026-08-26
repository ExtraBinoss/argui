use argui_platform::PlatformEvent;
use argui_ui::UiEvent;

#[derive(Clone, Debug, PartialEq)]
pub enum RuntimeEvent {
    Platform(PlatformEvent),
    Ui(UiEvent),
    RendererReady,
    RendererFailed(String),
    LayoutFailed(String),
}
