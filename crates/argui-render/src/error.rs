use std::{error::Error, fmt};

use argui_paint::EffectId;

/// Describes one renderer configuration that failed during GPU initialization.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RendererAttemptFailure {
    /// Human-readable renderer configuration name.
    pub renderer: String,
    /// Error returned while initializing the configuration.
    pub error: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RendererError {
    SurfaceCreation(String),
    AdapterRequest(String),
    DeviceRequest(String),
    Initialization {
        attempts: Vec<RendererAttemptFailure>,
        fallback_enabled: bool,
    },
    UnsupportedSurface,
    UnsupportedSurfaceTransparency,
    GlyphAtlasFull,
    InvalidDisplayList(String),
    InvalidShader(String),
    MissingEffect(EffectId),
    DuplicateEffect(EffectId),
    InvalidEffectDefinition(String),
    InvalidEffectParameters {
        effect: EffectId,
        message: String,
    },
    EffectParametersTooLarge {
        provided: usize,
        maximum: usize,
    },
    GpuCanvasCapability {
        canvas: String,
        message: String,
    },
    IncompatibleGpuCanvasDevice {
        canvas: String,
        message: String,
    },
    MissingImage(u64),
    MissingVector(u64),
    VectorAtlasFull,
    VectorRasterization(String),
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
            Self::Initialization {
                attempts,
                fallback_enabled,
            } => {
                let plural = if attempts.len() == 1 { "" } else { "s" };
                writeln!(
                    formatter,
                    "renderer initialization failed after {} attempt{plural}:",
                    attempts.len()
                )?;
                for attempt in attempts {
                    writeln!(formatter, "- {}: {}", attempt.renderer, attempt.error)?;
                }
                if *fallback_enabled {
                    formatter.write_str(
                        "No compatible GPU renderer could be started. Update the graphics driver or choose a supported renderer configuration.",
                    )
                } else {
                    formatter.write_str(
                        "Renderer fallback is disabled; only DirectX 12 with DirectComposition was attempted.",
                    )
                }
            }
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
            Self::GpuCanvasCapability { canvas, message } => write!(
                formatter,
                "GPU canvas '{canvas}' cannot use this adapter: {message}"
            ),
            Self::IncompatibleGpuCanvasDevice { canvas, message } => write!(
                formatter,
                "GPU canvas '{canvas}' is incompatible with the shared renderer device: {message}"
            ),
            Self::MissingImage(id) => write!(formatter, "image {id} is not registered"),
            Self::MissingVector(id) => write!(formatter, "vector {id} is not registered"),
            Self::VectorAtlasFull => formatter.write_str("the bounded vector atlas is full"),
            Self::VectorRasterization(message) => {
                write!(formatter, "SVG rasterization failed: {message}")
            }
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
