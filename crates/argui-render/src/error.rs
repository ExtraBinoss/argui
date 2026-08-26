use std::{error::Error, fmt};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RendererError {
    SurfaceCreation(String),
    AdapterRequest(String),
    DeviceRequest(String),
    UnsupportedSurface,
    Validation,
}

impl fmt::Display for RendererError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SurfaceCreation(message) => {
                write!(formatter, "surface creation failed: {message}")
            }
            Self::AdapterRequest(message) => write!(formatter, "adapter request failed: {message}"),
            Self::DeviceRequest(message) => write!(formatter, "device request failed: {message}"),
            Self::UnsupportedSurface => {
                formatter.write_str("the adapter cannot present to this surface")
            }
            Self::Validation => formatter.write_str("surface validation failed"),
        }
    }
}

impl Error for RendererError {}
