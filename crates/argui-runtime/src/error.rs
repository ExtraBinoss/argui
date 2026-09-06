use std::{error::Error, fmt};

use argui_layout::LayoutError;
use argui_platform::PlatformError;
use argui_render::RendererError;

#[derive(Debug)]
pub enum RuntimeError {
    Configuration(String),
    NativeHost(String),
    Platform(PlatformError),
    Layout(LayoutError),
    Renderer(RendererError),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Configuration(error) | Self::NativeHost(error) => formatter.write_str(error),
            Self::Platform(error) => error.fmt(formatter),
            Self::Layout(error) => error.fmt(formatter),
            Self::Renderer(error) => error.fmt(formatter),
        }
    }
}

impl Error for RuntimeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Configuration(_) | Self::NativeHost(_) => None,
            Self::Platform(error) => Some(error),
            Self::Layout(error) => Some(error),
            Self::Renderer(error) => Some(error),
        }
    }
}

impl From<LayoutError> for RuntimeError {
    fn from(error: LayoutError) -> Self {
        Self::Layout(error)
    }
}

impl From<PlatformError> for RuntimeError {
    fn from(error: PlatformError) -> Self {
        Self::Platform(error)
    }
}

impl From<RendererError> for RuntimeError {
    fn from(error: RendererError) -> Self {
        Self::Renderer(error)
    }
}
