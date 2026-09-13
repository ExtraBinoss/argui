use argui_core::Color;

use super::{ColorFormat, ColorPickerState};

pub(super) fn hsv_to_rgb([h, s, v]: [f32; 3]) -> [f32; 3] {
    let chroma = v * s;
    let sector = h.rem_euclid(360.0) / 60.0;
    let x = chroma * (1.0 - (sector % 2.0 - 1.0).abs());
    let rgb = match sector as u32 {
        0 => [chroma, x, 0.0],
        1 => [x, chroma, 0.0],
        2 => [0.0, chroma, x],
        3 => [0.0, x, chroma],
        4 => [x, 0.0, chroma],
        _ => [chroma, 0.0, x],
    };
    rgb.map(|c| c + v - chroma)
}

pub(super) fn rgb_to_hsv([r, g, b]: [f32; 3], previous_hue: f32) -> [f32; 3] {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    let hue = if delta < f32::EPSILON {
        previous_hue
    } else if max == r {
        60.0 * ((g - b) / delta).rem_euclid(6.0)
    } else if max == g {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };
    [hue, if max == 0.0 { 0.0 } else { delta / max }, max]
}

impl ColorPickerState {
    pub(super) fn fields(&self) -> Vec<(&'static str, String)> {
        let [h, s, v] = self.hsv;
        let alpha = self.alpha * 100.0;
        let fields = match self.format {
            ColorFormat::Hex => {
                let [r, g, b, a] = self.color().to_srgba8();
                return vec![("HEX", format!("#{r:02X}{g:02X}{b:02X}{a:02X}"))];
            }
            ColorFormat::Rgb => {
                let [r, g, b] = hsv_to_rgb(self.hsv).map(|value| value * 255.0);
                [("R", r), ("G", g), ("B", b), ("A %", alpha)]
            }
            ColorFormat::Hsl => {
                let lightness = v * (1.0 - s * 0.5);
                let saturation = if lightness == 0.0 || lightness == 1.0 {
                    0.0
                } else {
                    (v - lightness) / lightness.min(1.0 - lightness)
                };
                [
                    ("H °", h),
                    ("S %", saturation * 100.0),
                    ("L %", lightness * 100.0),
                    ("A %", alpha),
                ]
            }
            ColorFormat::Hsv => [
                ("H °", h),
                ("S %", s * 100.0),
                ("V %", v * 100.0),
                ("A %", alpha),
            ],
        };
        fields
            .into_iter()
            .map(|(label, value)| (label, format_number(value)))
            .collect()
    }

    pub(super) fn parse_field(&mut self, index: usize, text: &str) -> bool {
        if self.format == ColorFormat::Hex {
            let Ok(color) = Color::from_hex(text.trim()) else {
                return false;
            };
            self.set_color(color);
            return true;
        }
        let Ok(value) = text.trim().parse::<f32>() else {
            return false;
        };
        let maximum = match (self.format, index) {
            (ColorFormat::Rgb, 0..=2) => 255.0,
            (ColorFormat::Hsl | ColorFormat::Hsv, 0) => 360.0,
            _ => 100.0,
        };
        if !value.is_finite() || !(0.0..=maximum).contains(&value) {
            return false;
        }
        if index == 3 {
            self.alpha = value / 100.0;
        } else {
            match self.format {
                ColorFormat::Rgb => {
                    let mut rgb = hsv_to_rgb(self.hsv);
                    rgb[index] = value / 255.0;
                    self.hsv = rgb_to_hsv(rgb, self.hsv[0]);
                }
                ColorFormat::Hsv => {
                    self.hsv[index] = if index == 0 { value } else { value / 100.0 }
                }
                ColorFormat::Hsl => {
                    let [h, s, v] = self.hsv;
                    let l = v * (1.0 - s * 0.5);
                    let sl = if l == 0.0 || l == 1.0 {
                        0.0
                    } else {
                        (v - l) / l.min(1.0 - l)
                    };
                    let mut hsl = [h, sl, l];
                    hsl[index] = if index == 0 { value } else { value / 100.0 };
                    let [h, s, l] = hsl;
                    let v = l + s * l.min(1.0 - l);
                    self.hsv = [
                        h,
                        if v == 0.0 {
                            self.hsv[1]
                        } else {
                            2.0 * (1.0 - l / v)
                        },
                        v,
                    ];
                }
                ColorFormat::Hex => unreachable!(),
            }
        }
        true
    }
}

fn format_number(value: f32) -> String {
    format!("{value:.1}")
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_owned()
}
