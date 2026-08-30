use std::{error::Error, fmt};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RendererError {
    SurfaceCreation(String),
    AdapterRequest(String),
    DeviceRequest(String),
    UnsupportedSurface,
    UnsupportedSurfaceTransparency,
    GlyphAtlasFull,
    InvalidDisplayList(String),
    InvalidShader(String),
    MissingEffect(&'static str),
    DuplicateEffect(&'static str),
    InvalidEffectDefinition(String),
    InvalidEffectParameters {
        effect: &'static str,
        message: String,
    },
    EffectParametersTooLarge {
        provided: usize,
        maximum: usize,
    },
    MissingImage(u64),
    MissingVector(u64),
    ImageCacheFull {
        requested: usize,
        capacity: usize,
    },
    TooManyGradientStops {
        provided: usize,
        capacity: usize,
    },
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
            Self::UnsupportedSurfaceTransparency => formatter
                .write_str("the surface does not support premultiplied transparent composition"),
            Self::GlyphAtlasFull => formatter.write_str("the bounded glyph atlas is full"),
            Self::InvalidDisplayList(message) => {
                write!(formatter, "invalid display list: {message}")
            }
            Self::InvalidShader(message) => write!(formatter, "invalid effect shader: {message}"),
            Self::MissingEffect(id) => write!(formatter, "effect '{id}' is not registered"),
            Self::DuplicateEffect(id) => write!(formatter, "effect '{id}' is registered twice"),
            Self::InvalidEffectDefinition(message) => {
                write!(formatter, "invalid effect definition: {message}")
            }
            Self::InvalidEffectParameters { effect, message } => {
                write!(
                    formatter,
                    "invalid parameters for effect '{effect}': {message}"
                )
            }
            Self::EffectParametersTooLarge { provided, maximum } => write!(
                formatter,
                "effect parameters need {provided} bytes but this adapter allows {maximum}"
            ),
            Self::MissingImage(id) => write!(formatter, "image {id} is not registered"),
            Self::MissingVector(id) => write!(formatter, "vector {id} is not registered"),
            Self::ImageCacheFull {
                requested,
                capacity,
            } => write!(
                formatter,
                "image needs {requested} bytes but the GPU image cache allows {capacity}"
            ),
            Self::TooManyGradientStops { provided, capacity } => write!(
                formatter,
                "display list has {provided} gradient stops but the frame budget allows {capacity}"
            ),
            Self::Validation => formatter.write_str("surface validation failed"),
        }
    }
}

impl Error for RendererError {}
