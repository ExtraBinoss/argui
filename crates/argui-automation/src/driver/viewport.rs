use serde::{Deserialize, Serialize};

const MAX_DIMENSION: u32 = 4096;

/// Logical viewport and pixel scale for a headless application.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Viewport {
    /// Width in logical pixels.
    pub width: u32,
    /// Height in logical pixels.
    pub height: u32,
    /// Physical pixels per logical pixel.
    #[serde(default = "default_scale")]
    pub scale: f32,
}

/// Returns the default one-to-one logical-to-physical scale.
fn default_scale() -> f32 {
    1.0
}

impl Default for Viewport {
    /// Selects an 800×600 logical viewport at one physical pixel per logical pixel.
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            scale: 1.0,
        }
    }
}

impl Viewport {
    /// Validates finite, bounded dimensions and returns physical pixel size.
    ///
    /// # Errors
    /// Returns a message if dimensions or scale cannot be rendered safely.
    pub fn physical_size(self) -> Result<(u32, u32), String> {
        if self.width == 0
            || self.height == 0
            || self.width > MAX_DIMENSION
            || self.height > MAX_DIMENSION
            || !self.scale.is_finite()
            || !(0.25..=4.0).contains(&self.scale)
        {
            return Err(
                "viewport must be 1–4096 logical pixels on each axis with scale 0.25–4".into(),
            );
        }
        let width = (self.width as f32 * self.scale).round() as u32;
        let height = (self.height as f32 * self.scale).round() as u32;
        if width > MAX_DIMENSION || height > MAX_DIMENSION {
            return Err("physical viewport exceeds 4096 pixels on one axis".into());
        }
        Ok((width.max(1), height.max(1)))
    }
}
