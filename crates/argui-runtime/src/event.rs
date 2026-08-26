use argui_platform::PlatformEvent;

#[derive(Clone, Debug, PartialEq)]
pub enum RuntimeEvent {
    Platform(PlatformEvent),
    RendererReady,
    RendererFailed(String),
}
