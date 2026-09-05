use core::fmt;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Color([f32; 4]);

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ColorInterpolation {
    #[default]
    Oklab,
    LinearSrgb,
    Srgb,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParseColorError {
    MissingHash,
    InvalidLength,
    InvalidDigit,
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
        }
    }
}

impl std::error::Error for ParseColorError {}

impl Color {
    pub const TRANSPARENT: Self = Self::linear_rgba(0.0, 0.0, 0.0, 0.0);
    pub const BLACK: Self = Self::linear_rgb(0.0, 0.0, 0.0);
    pub const WHITE: Self = Self::linear_rgb(1.0, 1.0, 1.0);

    #[must_use]
    pub fn srgb(red: f32, green: f32, blue: f32) -> Self {
        Self::srgba(red, green, blue, 1.0)
    }

    #[must_use]
    pub fn srgba(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self::linear_rgba(
            srgb_to_linear(red),
            srgb_to_linear(green),
            srgb_to_linear(blue),
            alpha,
        )
    }

    #[must_use]
    pub const fn linear_rgb(red: f32, green: f32, blue: f32) -> Self {
        Self::linear_rgba(red, green, blue, 1.0)
    }

    #[must_use]
    pub const fn linear_rgba(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self([red, green, blue, alpha])
    }

    #[must_use]
    pub fn from_srgb8(red: u8, green: u8, blue: u8) -> Self {
        Self::from_srgba8(red, green, blue, u8::MAX)
    }

    #[must_use]
    pub fn from_srgba8(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self::srgba(
            f32::from(red) / 255.0,
            f32::from(green) / 255.0,
            f32::from(blue) / 255.0,
            f32::from(alpha) / 255.0,
        )
    }

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

    #[must_use]
    pub const fn to_linear_rgba(self) -> [f32; 4] {
        self.0
    }

    #[must_use]
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
    pub fn to_srgba8(self) -> [u8; 4] {
        self.to_srgba().map(unit_to_byte)
    }

    #[must_use]
    pub const fn with_alpha(self, alpha: f32) -> Self {
        Self([self.0[0], self.0[1], self.0[2], alpha])
    }

    #[must_use]
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
    pub fn to_interpolation_components(self, space: ColorInterpolation) -> [f32; 4] {
        let coordinates = coordinates(self, space);
        [coordinates[0], coordinates[1], coordinates[2], self.0[3]]
    }

    #[must_use]
    pub fn from_interpolation_components(components: [f32; 4], space: ColorInterpolation) -> Self {
        from_coordinates(
            [components[0], components[1], components[2]],
            components[3],
            space,
        )
    }

    #[must_use]
    pub fn relative_luminance(self) -> f32 {
        0.2126 * self.0[0] + 0.7152 * self.0[1] + 0.0722 * self.0[2]
    }

    #[must_use]
    pub fn contrast_ratio(self, other: Self) -> f32 {
        let lighter = self.relative_luminance().max(other.relative_luminance());
        let darker = self.relative_luminance().min(other.relative_luminance());
        (lighter + 0.05) / (darker + 0.05)
    }
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
