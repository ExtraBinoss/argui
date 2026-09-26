use core::fmt;

use crate::ColorScheme;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
/// An RGBA color stored in linear-light sRGB coordinates.
pub struct Color([f32; 4]);

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
/// Color space used when interpolating color channels.
pub enum ColorInterpolation {
    /// Perceptually uniform Oklab coordinates.
    #[default]
    Oklab,
    /// Linear-light sRGB coordinates.
    LinearSrgb,
    /// Gamma-encoded sRGB coordinates.
    Srgb,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Error returned when parsing a supported color literal fails.
pub enum ParseColorError {
    /// The input does not start with `#`.
    MissingHash,
    /// The input has a number of hexadecimal digits other than 3, 4, 6, or 8.
    InvalidLength,
    /// One or more digits are not hexadecimal.
    InvalidDigit,
    /// An `oklch()` function has invalid channels or syntax.
    InvalidOklch,
    /// An `rgb()` or `rgba()` function has invalid channels or syntax.
    InvalidRgb,
}

impl fmt::Display for ParseColorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingHash => formatter.write_str("a hexadecimal color must start with '#'"),
            Self::InvalidLength => {
                formatter.write_str("a hexadecimal color must contain 3, 4, 6, or 8 digits")
            }
            Self::InvalidDigit => {
                formatter.write_str("a hexadecimal color contains a non-hexadecimal digit")
            }
            Self::InvalidOklch => {
                formatter.write_str("an oklch color has invalid channels or syntax")
            }
            Self::InvalidRgb => formatter.write_str("an rgb color has invalid channels or syntax"),
        }
    }
}

impl std::error::Error for ParseColorError {}

impl Color {
    /// Fully transparent black.
    pub const TRANSPARENT: Self = Self::linear_rgba(0.0, 0.0, 0.0, 0.0);
    /// Opaque black.
    pub const BLACK: Self = Self::linear_rgb(0.0, 0.0, 0.0);
    /// Opaque white.
    pub const WHITE: Self = Self::linear_rgb(1.0, 1.0, 1.0);

    /// Creates an opaque color from gamma-encoded sRGB channels.
    /// Each channel is expected in the conventional 0.0–1.0 range.
    /// * `red`, `green`, `blue` — gamma-encoded color channels.
    #[must_use]
    pub fn srgb(red: f32, green: f32, blue: f32) -> Self {
        Self::srgba(red, green, blue, 1.0)
    }

    /// Creates a color from gamma-encoded sRGB channels and alpha.
    /// Channels are expected in the conventional 0.0–1.0 range.
    /// * `red`, `green`, `blue` — gamma-encoded color channels; `alpha` — opacity.
    #[must_use]
    pub fn srgba(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self::linear_rgba(
            srgb_to_linear(red),
            srgb_to_linear(green),
            srgb_to_linear(blue),
            alpha,
        )
    }

    /// Creates a color from HSV coordinates and opacity.
    ///
    /// `hue` is an angle in degrees; `saturation`, `value`, and `alpha` are
    /// normalized to the 0–1 range. Nonfinite inputs use zero before clamping.
    /// Returns the resulting color in the library's linear-light storage.
    #[must_use]
    pub fn hsva(hue: f32, saturation: f32, value: f32, alpha: f32) -> Self {
        let finite = |channel: f32| if channel.is_finite() { channel } else { 0.0 };
        let hue = finite(hue).rem_euclid(360.0);
        let saturation = finite(saturation).clamp(0.0, 1.0);
        let value = finite(value).clamp(0.0, 1.0);
        let alpha = finite(alpha).clamp(0.0, 1.0);
        let chroma = value * saturation;
        let sector = hue / 60.0;
        let secondary = chroma * (1.0 - (sector % 2.0 - 1.0).abs());
        let [red, green, blue] = match sector as u32 {
            0 => [chroma, secondary, 0.0],
            1 => [secondary, chroma, 0.0],
            2 => [0.0, chroma, secondary],
            3 => [0.0, secondary, chroma],
            4 => [secondary, 0.0, chroma],
            _ => [chroma, 0.0, secondary],
        };
        let minimum = value - chroma;
        Self::srgba(red + minimum, green + minimum, blue + minimum, alpha)
    }

    /// Creates an opaque color from linear-light sRGB channels.
    /// * `red`, `green`, `blue` — linear-light color channels.
    #[must_use]
    pub const fn linear_rgb(red: f32, green: f32, blue: f32) -> Self {
        Self::linear_rgba(red, green, blue, 1.0)
    }

    /// Creates a color from linear-light sRGB channels and alpha.
    /// * `red`, `green`, `blue` — linear-light color channels; `alpha` — opacity.
    #[must_use]
    pub const fn linear_rgba(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self([red, green, blue, alpha])
    }

    /// Creates an opaque color from 8-bit gamma-encoded sRGB channels.
    /// * `red`, `green`, `blue` — 8-bit gamma-encoded color channels.
    #[must_use]
    pub fn from_srgb8(red: u8, green: u8, blue: u8) -> Self {
        Self::from_srgba8(red, green, blue, u8::MAX)
    }

    /// Creates a color from 8-bit gamma-encoded sRGB channels and alpha.
    /// * `red`, `green`, `blue` — 8-bit gamma-encoded channels; `alpha` — 8-bit opacity.
    #[must_use]
    pub fn from_srgba8(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self::srgba(
            f32::from(red) / 255.0,
            f32::from(green) / 255.0,
            f32::from(blue) / 255.0,
            f32::from(alpha) / 255.0,
        )
    }

    /// Parses `#RGB`, `#RGBA`, `#RRGGBB`, or `#RRGGBBAA` notation.
    /// * `value` — hexadecimal color string to parse.
    ///
    /// # Errors
    /// Returns [`ParseColorError`] when the hash prefix, length, or digits are invalid.
    pub fn from_hex(value: &str) -> Result<Self, ParseColorError> {
        let digits = value
            .strip_prefix('#')
            .ok_or(ParseColorError::MissingHash)?;
        let channels = match digits.len() {
            3 => [
                repeated_nibble(digits, 0)?,
                repeated_nibble(digits, 1)?,
                repeated_nibble(digits, 2)?,
                u8::MAX,
            ],
            4 => [
                repeated_nibble(digits, 0)?,
                repeated_nibble(digits, 1)?,
                repeated_nibble(digits, 2)?,
                repeated_nibble(digits, 3)?,
            ],
            6 => [
                byte(digits, 0)?,
                byte(digits, 2)?,
                byte(digits, 4)?,
                u8::MAX,
            ],
            8 => [
                byte(digits, 0)?,
                byte(digits, 2)?,
                byte(digits, 4)?,
                byte(digits, 6)?,
            ],
            _ => return Err(ParseColorError::InvalidLength),
        };
        Ok(Self::from_srgba8(
            channels[0],
            channels[1],
            channels[2],
            channels[3],
        ))
    }

    /// Parses a hexadecimal color, CSS `oklch()`, `rgb()`, `rgba()`, or `transparent`.
    /// * `value` — color literal to parse; other named colors are unsupported.
    ///
    /// # Errors
    /// Returns [`ParseColorError`] for an unsupported or invalid literal.
    pub fn from_literal(value: &str) -> Result<Self, ParseColorError> {
        let value = value.trim();
        if value.eq_ignore_ascii_case("transparent") {
            Ok(Self::TRANSPARENT)
        } else if value.starts_with("oklch(") {
            parse_oklch(value)
        } else if value.starts_with("rgb(") || value.starts_with("rgba(") {
            parse_rgb(value)
        } else {
            Self::from_hex(value)
        }
    }

    #[must_use]
    /// Returns the stored linear-light RGBA channels.
    pub const fn to_linear_rgba(self) -> [f32; 4] {
        self.0
    }

    #[must_use]
    /// Converts the color to gamma-encoded sRGB channels and alpha.
    pub fn to_srgba(self) -> [f32; 4] {
        let [red, green, blue, alpha] = self.0;
        [
            linear_to_srgb(red),
            linear_to_srgb(green),
            linear_to_srgb(blue),
            alpha,
        ]
    }

    #[must_use]
    /// Converts the color to 8-bit gamma-encoded sRGB and alpha channels.
    pub fn to_srgba8(self) -> [u8; 4] {
        self.to_srgba().map(unit_to_byte)
    }

    /// Returns the color as an eight-digit sRGB hexadecimal string.
    ///
    /// The return value has `#RRGGBBAA` spelling, including its alpha channel.
    #[must_use]
    pub fn to_hex_rgba(self) -> String {
        let [red, green, blue, alpha] = self.to_srgba8();
        format!("#{red:02X}{green:02X}{blue:02X}{alpha:02X}")
    }

    #[must_use]
    /// Returns this color with `alpha`, preserving its RGB channels.
    pub const fn with_alpha(self, alpha: f32) -> Self {
        Self([self.0[0], self.0[1], self.0[2], alpha])
    }

    #[must_use]
    /// Interpolates toward `target` by `progress` in the chosen color space.
    pub fn mix(self, target: Self, progress: f32, space: ColorInterpolation) -> Self {
        let alpha = lerp(self.0[3], target.0[3], progress);
        let left = coordinates(self, space);
        let right = coordinates(target, space);
        let mixed = if alpha.abs() <= f32::EPSILON {
            [0.0; 3]
        } else {
            [
                lerp(left[0] * self.0[3], right[0] * target.0[3], progress) / alpha,
                lerp(left[1] * self.0[3], right[1] * target.0[3], progress) / alpha,
                lerp(left[2] * self.0[3], right[2] * target.0[3], progress) / alpha,
            ]
        };
        from_coordinates(mixed, alpha, space)
    }

    #[must_use]
    /// Returns RGB coordinates in `space`, followed by the alpha channel.
    pub fn to_interpolation_components(self, space: ColorInterpolation) -> [f32; 4] {
        let coordinates = coordinates(self, space);
        [coordinates[0], coordinates[1], coordinates[2], self.0[3]]
    }

    #[must_use]
    /// Creates a color from RGB coordinates in `space` and an alpha channel.
    /// * `components` — red, green, blue, and alpha interpolation components.
    pub fn from_interpolation_components(components: [f32; 4], space: ColorInterpolation) -> Self {
        from_coordinates(
            [components[0], components[1], components[2]],
            components[3],
            space,
        )
    }

    #[must_use]
    /// Returns relative luminance using the linear-light sRGB coefficients.
    pub fn relative_luminance(self) -> f32 {
        0.2126 * self.0[0] + 0.7152 * self.0[1] + 0.0722 * self.0[2]
    }

    #[must_use]
    /// Returns the WCAG contrast ratio between this color and `other`.
    pub fn contrast_ratio(self, other: Self) -> f32 {
        let lighter = self.relative_luminance().max(other.relative_luminance());
        let darker = self.relative_luminance().min(other.relative_luminance());
        (lighter + 0.05) / (darker + 0.05)
    }

    #[must_use]
    /// Chooses the system-bar icon scheme with higher contrast against this background.
    ///
    /// A light background requests dark icons; a dark background requests light icons.
    pub fn preferred_contrast_scheme(self) -> ColorScheme {
        if self.contrast_ratio(Self::BLACK) >= self.contrast_ratio(Self::WHITE) {
            ColorScheme::Light
        } else {
            ColorScheme::Dark
        }
    }
}

/// Parses CSS comma or space separated RGB channels, with an optional alpha.
///
/// `value` includes the function name. Numeric channels use 0–255 and
/// percentage channels use 0–100%. Alpha is normalized or a percentage.
/// Returns an error for malformed, nonfinite, or out-of-range input.
fn parse_rgb(value: &str) -> Result<Color, ParseColorError> {
    let (body, legacy_alpha) = if let Some(body) = value.strip_prefix("rgb(") {
        (body, false)
    } else if let Some(body) = value.strip_prefix("rgba(") {
        (body, true)
    } else {
        return Err(ParseColorError::InvalidRgb);
    };
    let body = body.strip_suffix(')').ok_or(ParseColorError::InvalidRgb)?;
    let (channels, alpha) = if body.contains(',') {
        if body.contains('/') {
            return Err(ParseColorError::InvalidRgb);
        }
        let parts = body.split(',').map(str::trim).collect::<Vec<_>>();
        let expected = if legacy_alpha { 4 } else { 3 };
        if parts.len() != expected {
            return Err(ParseColorError::InvalidRgb);
        }
        ([parts[0], parts[1], parts[2]], parts.get(3).copied())
    } else {
        let (channels, alpha) = body
            .split_once('/')
            .map_or((body, None), |(channels, alpha)| {
                (channels, Some(alpha.trim()))
            });
        let parts = channels.split_whitespace().collect::<Vec<_>>();
        if parts.len() != 3 || (legacy_alpha && alpha.is_none()) {
            return Err(ParseColorError::InvalidRgb);
        }
        ([parts[0], parts[1], parts[2]], alpha)
    };
    let parse_channel = |part: &str| -> Result<f32, ParseColorError> {
        let (number, maximum) = part
            .strip_suffix('%')
            .map_or((part, 255.0), |number| (number, 100.0));
        let value = number
            .trim()
            .parse::<f32>()
            .map_err(|_| ParseColorError::InvalidRgb)?;
        if !value.is_finite() || !(0.0..=maximum).contains(&value) {
            return Err(ParseColorError::InvalidRgb);
        }
        Ok(value / maximum)
    };
    let parse_alpha = |part: &str| -> Result<f32, ParseColorError> {
        let (number, maximum) = part
            .strip_suffix('%')
            .map_or((part, 1.0), |number| (number, 100.0));
        let value = number
            .trim()
            .parse::<f32>()
            .map_err(|_| ParseColorError::InvalidRgb)?;
        if !value.is_finite() || !(0.0..=maximum).contains(&value) {
            return Err(ParseColorError::InvalidRgb);
        }
        Ok(value / maximum)
    };
    Ok(Color::srgba(
        parse_channel(channels[0])?,
        parse_channel(channels[1])?,
        parse_channel(channels[2])?,
        alpha.map(parse_alpha).transpose()?.unwrap_or(1.0),
    ))
}

/// Parses the whitespace-separated CSS `oklch(L C H / alpha)` notation.
/// `value` includes the function name and parentheses; percentage alpha is
/// accepted. Returns an error for malformed, nonfinite, or out-of-range input.
fn parse_oklch(value: &str) -> Result<Color, ParseColorError> {
    let body = value
        .strip_prefix("oklch(")
        .and_then(|body| body.strip_suffix(')'))
        .ok_or(ParseColorError::InvalidOklch)?;
    let (channels, alpha) = body.split_once('/').unwrap_or((body, "1"));
    let mut parts = channels.split_whitespace();
    let parse = |part: Option<&str>| {
        part.and_then(|part| part.parse::<f32>().ok())
            .filter(|value| value.is_finite())
            .ok_or(ParseColorError::InvalidOklch)
    };
    let lightness_part = parts.next().ok_or(ParseColorError::InvalidOklch)?;
    let lightness = if let Some(percent) = lightness_part.strip_suffix('%') {
        parse(Some(percent))? / 100.0
    } else {
        parse(Some(lightness_part))?
    };
    let chroma = parse(parts.next())?;
    let hue_part = parts.next().ok_or(ParseColorError::InvalidOklch)?;
    let hue = if let Some(degrees) = hue_part.strip_suffix("deg") {
        parse(Some(degrees))?
    } else if let Some(turns) = hue_part.strip_suffix("turn") {
        parse(Some(turns))? * 360.0
    } else if let Some(radians) = hue_part.strip_suffix("rad") {
        parse(Some(radians))?.to_degrees()
    } else {
        parse(Some(hue_part))?
    };
    if parts.next().is_some()
        || !(0.0..=1.0).contains(&lightness)
        || chroma < 0.0
        || !hue.is_finite()
    {
        return Err(ParseColorError::InvalidOklch);
    }
    let alpha = alpha.trim();
    let alpha = if let Some(percent) = alpha.strip_suffix('%') {
        parse(Some(percent.trim()))? / 100.0
    } else {
        parse(Some(alpha))?
    };
    if !(0.0..=1.0).contains(&alpha) {
        return Err(ParseColorError::InvalidOklch);
    }
    let radians = hue.to_radians();
    let (sin, cos) = radians.sin_cos();
    Ok(Color::from_interpolation_components(
        [lightness, chroma * cos, chroma * sin, alpha],
        ColorInterpolation::Oklab,
    ))
}

fn coordinates(color: Color, space: ColorInterpolation) -> [f32; 3] {
    let [red, green, blue, _] = color.0;
    match space {
        ColorInterpolation::LinearSrgb => [red, green, blue],
        ColorInterpolation::Srgb => [
            linear_to_srgb(red),
            linear_to_srgb(green),
            linear_to_srgb(blue),
        ],
        ColorInterpolation::Oklab => linear_srgb_to_oklab([red, green, blue]),
    }
}

fn from_coordinates(value: [f32; 3], alpha: f32, space: ColorInterpolation) -> Color {
    let [red, green, blue] = match space {
        ColorInterpolation::LinearSrgb => value,
        ColorInterpolation::Srgb => value.map(srgb_to_linear),
        ColorInterpolation::Oklab => oklab_to_linear_srgb(value),
    };
    Color::linear_rgba(red, green, blue, alpha)
}

fn linear_srgb_to_oklab([red, green, blue]: [f32; 3]) -> [f32; 3] {
    let l = 0.412_221_46 * red + 0.536_332_55 * green + 0.051_445_995 * blue;
    let m = 0.211_903_5 * red + 0.680_699_5 * green + 0.107_396_96 * blue;
    let s = 0.088_302_46 * red + 0.281_718_85 * green + 0.629_978_7 * blue;
    let [l, m, s] = [l.cbrt(), m.cbrt(), s.cbrt()];
    [
        0.210_454_26 * l + 0.793_617_8 * m - 0.004_072_047 * s,
        1.977_998_5 * l - 2.428_592_2 * m + 0.450_593_7 * s,
        0.025_904_037 * l + 0.782_771_77 * m - 0.808_675_77 * s,
    ]
}

fn oklab_to_linear_srgb([lightness, a, b]: [f32; 3]) -> [f32; 3] {
    let l = (lightness + 0.396_337_78 * a + 0.215_803_76 * b).powi(3);
    let m = (lightness - 0.105_561_346 * a - 0.063_854_17 * b).powi(3);
    let s = (lightness - 0.089_484_18 * a - 1.291_485_5 * b).powi(3);
    [
        4.076_741_7 * l - 3.307_711_6 * m + 0.230_969_94 * s,
        -1.268_438 * l + 2.609_757_4 * m - 0.341_319_38 * s,
        -0.004_196_086_3 * l - 0.703_418_6 * m + 1.707_614_7 * s,
    ]
}

fn srgb_to_linear(value: f32) -> f32 {
    let sign = value.signum();
    let magnitude = value.abs();
    sign * if magnitude <= 0.04045 {
        magnitude / 12.92
    } else {
        ((magnitude + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(value: f32) -> f32 {
    let sign = value.signum();
    let magnitude = value.abs();
    sign * if magnitude <= 0.003_130_8 {
        magnitude * 12.92
    } else {
        1.055 * magnitude.powf(1.0 / 2.4) - 0.055
    }
}

fn lerp(left: f32, right: f32, progress: f32) -> f32 {
    left + (right - left) * progress
}

fn unit_to_byte(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn repeated_nibble(value: &str, offset: usize) -> Result<u8, ParseColorError> {
    let nibble = digit(value.as_bytes()[offset])?;
    Ok(nibble * 17)
}

fn byte(value: &str, offset: usize) -> Result<u8, ParseColorError> {
    Ok(digit(value.as_bytes()[offset])? * 16 + digit(value.as_bytes()[offset + 1])?)
}

fn digit(value: u8) -> Result<u8, ParseColorError> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        b'A'..=b'F' => Ok(value - b'A' + 10),
        _ => Err(ParseColorError::InvalidDigit),
    }
}
