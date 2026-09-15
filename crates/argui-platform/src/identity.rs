use std::sync::Arc;

use image::{ImageEncoder, codecs::png::PngEncoder};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
/// Validated reverse-DNS application identifier.
pub struct ApplicationId(String);

impl ApplicationId {
    /// Creates an application identifier after validating its reverse-DNS form.
    ///
    /// # Errors
    /// Returns an error when `value` has fewer than two valid segments or contains unsupported characters.
    pub fn new(value: impl Into<String>) -> Result<Self, ApplicationIdError> {
        let value = value.into();
        if valid_application_id(&value) {
            Ok(Self(value))
        } else {
            Err(ApplicationIdError(value))
        }
    }

    /// Returns the identifier's string value.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Invalid application identifier returned by [`ApplicationId::new`].
pub struct ApplicationIdError(String);

impl std::fmt::Display for ApplicationIdError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "invalid reverse-DNS application id: {}", self.0)
    }
}

impl std::error::Error for ApplicationIdError {}

fn valid_application_id(value: &str) -> bool {
    let mut segments = value.split('.');
    let Some(first) = segments.next() else {
        return false;
    };
    let Some(second) = segments.next() else {
        return false;
    };
    valid_segment(first) && valid_segment(second) && segments.all(valid_segment)
}

fn valid_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        && segment.as_bytes()[0].is_ascii_lowercase()
}

#[derive(Clone, Debug, PartialEq)]
/// Display name, identifiers, and icons shared with platform integrations.
pub struct ApplicationIdentity {
    /// Canonical reverse-DNS application ID.
    pub id: ApplicationId,
    /// Human-readable application name.
    pub display_name: String,
    /// Icons available for native windows and system integrations.
    pub icons: IconSet,
    linux_application_id: Option<String>,
}

impl ApplicationIdentity {
    /// Creates identity metadata shared by native windows and application integrations.
    /// `id` is the application identifier, `display_name` is user-visible, and `icons` supplies platform icon variants.
    #[must_use]
    pub fn new(id: ApplicationId, display_name: impl Into<String>, icons: IconSet) -> Self {
        Self {
            id,
            display_name: display_name.into(),
            icons,
            linux_application_id: None,
        }
    }

    /// Overrides the desktop-file id exposed to Wayland and X11.
    ///
    /// This should match the installed `<id>.desktop` file name without its
    /// `.desktop` suffix. It is useful when a packager derives that name from
    /// an executable instead of the reverse-DNS bundle identifier.
    /// `id` is the installed desktop-file name without the `.desktop` suffix.
    #[must_use]
    pub fn with_linux_application_id(mut self, id: impl Into<String>) -> Self {
        self.linux_application_id = Some(id.into());
        self
    }

    #[must_use]
    /// Returns the desktop-file ID used by Linux integrations.
    pub fn linux_application_id(&self) -> &str {
        self.linux_application_id
            .as_deref()
            .unwrap_or_else(|| self.id.as_str())
    }

    #[must_use]
    /// Creates development identity metadata with no icons.
    /// `display_name` is shown to users.
    pub fn development(display_name: impl Into<String>) -> Self {
        Self {
            id: ApplicationId("dev.argui.application".into()),
            display_name: display_name.into(),
            icons: IconSet::new(),
            linux_application_id: None,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
/// Application icons ordered by pixel count.
pub struct IconSet {
    icons: Vec<AppIcon>,
}

impl IconSet {
    /// Creates an empty icon set.
    #[must_use]
    pub const fn new() -> Self {
        Self { icons: Vec::new() }
    }

    #[must_use]
    /// Creates an icon set containing `icon`.
    /// `icon` is the sole image variant in the returned set.
    pub fn single(icon: AppIcon) -> Self {
        Self { icons: vec![icon] }
    }

    #[must_use]
    /// Adds or replaces an icon with the same dimensions.
    /// `icon` replaces any existing image with matching dimensions.
    pub fn with(mut self, icon: AppIcon) -> Self {
        self.icons
            .retain(|candidate| candidate.width != icon.width || candidate.height != icon.height);
        self.icons.push(icon);
        self.icons.sort_by_key(AppIcon::pixel_count);
        self
    }

    #[must_use]
    /// Returns the icons in this set, ordered by pixel count.
    pub fn icons(&self) -> &[AppIcon] {
        &self.icons
    }

    #[must_use]
    /// Returns the icon whose largest dimension is closest to `target`.
    pub fn best_square(&self, target: u32) -> Option<&AppIcon> {
        self.icons.iter().min_by_key(|icon| {
            let size = icon.width.max(icon.height);
            size.abs_diff(target)
        })
    }

    #[must_use]
    /// Returns whether this set contains no icons.
    pub fn is_empty(&self) -> bool {
        self.icons.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq)]
/// Encoded PNG and decoded RGBA representations of an application icon.
pub struct AppIcon {
    /// Icon width in pixels.
    pub width: u32,
    /// Icon height in pixels.
    pub height: u32,
    /// Row-major 8-bit RGBA pixels.
    pub rgba8: Arc<[u8]>,
    /// PNG-encoded icon bytes.
    pub png: Arc<[u8]>,
}

impl AppIcon {
    /// Decodes a PNG and stores its RGBA pixels and original encoded bytes.
    /// `encoded` contains PNG data.
    ///
    /// # Errors
    /// Returns an error if decoding fails or the resulting image has invalid dimensions.
    pub fn from_png(encoded: impl AsRef<[u8]>) -> Result<Self, AppIconError> {
        let encoded = encoded.as_ref();
        let decoded =
            image::load_from_memory_with_format(encoded, image::ImageFormat::Png)?.into_rgba8();
        Self::from_parts(
            decoded.width(),
            decoded.height(),
            decoded.into_raw(),
            encoded.to_vec(),
        )
    }

    /// Encodes tightly packed 8-bit RGBA pixels as a PNG icon.
    ///
    /// # Arguments
    /// * `width` — image width in pixels.
    /// * `height` — image height in pixels.
    /// * `rgba8` — row-major RGBA bytes.
    ///
    /// # Errors
    /// Returns an error if dimensions are zero, the byte count is invalid, or PNG encoding fails.
    pub fn from_rgba8(
        width: u32,
        height: u32,
        rgba8: impl Into<Vec<u8>>,
    ) -> Result<Self, AppIconError> {
        let rgba8 = rgba8.into();
        validate_rgba(width, height, rgba8.len())?;
        let mut png = Vec::new();
        PngEncoder::new(&mut png).write_image(
            &rgba8,
            width,
            height,
            image::ExtendedColorType::Rgba8,
        )?;
        Self::from_parts(width, height, rgba8, png)
    }

    fn from_parts(
        width: u32,
        height: u32,
        rgba8: Vec<u8>,
        png: Vec<u8>,
    ) -> Result<Self, AppIconError> {
        validate_rgba(width, height, rgba8.len())?;
        Ok(Self {
            width,
            height,
            rgba8: rgba8.into(),
            png: png.into(),
        })
    }

    fn pixel_count(&self) -> u64 {
        u64::from(self.width) * u64::from(self.height)
    }
}

fn validate_rgba(width: u32, height: u32, actual: usize) -> Result<(), AppIconError> {
    let expected = usize::try_from(width)
        .ok()
        .and_then(|width| usize::try_from(height).ok().map(|height| width * height))
        .and_then(|pixels| pixels.checked_mul(4))
        .filter(|_| width != 0 && height != 0)
        .ok_or(AppIconError::InvalidDimensions)?;
    if expected == actual {
        Ok(())
    } else {
        Err(AppIconError::InvalidByteLength { expected, actual })
    }
}

#[derive(Debug)]
/// Failure while decoding or validating application icon data.
pub enum AppIconError {
    /// PNG decoding failed.
    Decode(image::ImageError),
    /// Image dimensions are zero or cannot be represented.
    InvalidDimensions,
    /// Pixel buffer length differs from the dimensions' required length.
    InvalidByteLength {
        /// Number of bytes required by the dimensions.
        expected: usize,
        /// Number of bytes supplied.
        actual: usize,
    },
}

impl std::fmt::Display for AppIconError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Decode(error) => write!(formatter, "icon decoding failed: {error}"),
            Self::InvalidDimensions => formatter.write_str("invalid icon dimensions"),
            Self::InvalidByteLength { expected, actual } => {
                write!(
                    formatter,
                    "invalid icon byte length: expected {expected}, got {actual}"
                )
            }
        }
    }
}

impl std::error::Error for AppIconError {}

impl From<image::ImageError> for AppIconError {
    fn from(error: image::ImageError) -> Self {
        Self::Decode(error)
    }
}
