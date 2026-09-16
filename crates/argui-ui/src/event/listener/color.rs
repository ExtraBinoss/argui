use argui_core::{Color, Key, KeyState, Rect};

use crate::{GestureKind, GesturePhase, UiEventKind};

use super::{ContinuousValuePhase, RangeHandlerValue};

/// Text channel representation used by a color-value adapter.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ColorValueFormat {
    Hex,
    Rgb,
    Hsl,
    Hsv,
}

/// Pure color mapping used by color-picker listeners in normal event dispatch.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ColorHandlerValue {
    Pad {
        hsv: [f32; 3],
        alpha: f32,
    },
    Hue {
        hsv: [f32; 3],
        alpha: f32,
        phase: ContinuousValuePhase,
    },
    Alpha {
        hsv: [f32; 3],
        alpha: f32,
        phase: ContinuousValuePhase,
    },
    Field {
        hsv: [f32; 3],
        alpha: f32,
        format: ColorValueFormat,
        index: usize,
    },
}

impl ColorHandlerValue {
    /// Creates a saturation/brightness pad mapping from controlled HSV and alpha values.
    #[must_use]
    pub const fn pad(hsv: [f32; 3], alpha: f32) -> Self {
        Self::Pad { hsv, alpha }
    }

    /// Creates a hue-track mapping for the requested continuous delivery `phase`.
    #[must_use]
    pub const fn hue(hsv: [f32; 3], alpha: f32, phase: ContinuousValuePhase) -> Self {
        Self::Hue { hsv, alpha, phase }
    }

    /// Creates an alpha-track mapping for the requested continuous delivery `phase`.
    #[must_use]
    pub const fn alpha(hsv: [f32; 3], alpha: f32, phase: ContinuousValuePhase) -> Self {
        Self::Alpha { hsv, alpha, phase }
    }

    /// Creates a text-field mapping for one channel `index` in `format`.
    #[must_use]
    pub const fn field(hsv: [f32; 3], alpha: f32, format: ColorValueFormat, index: usize) -> Self {
        Self::Field {
            hsv,
            alpha,
            format,
            index,
        }
    }

    pub(super) fn event_value(self, kind: &UiEventKind, bounds: Option<Rect>) -> Option<Color> {
        let (hsv, alpha) = match self {
            Self::Pad { mut hsv, alpha } => {
                update_pad(&mut hsv, kind, bounds)?;
                (hsv, alpha)
            }
            Self::Hue {
                mut hsv,
                alpha,
                phase,
            } => {
                hsv[0] = RangeHandlerValue::new(hsv[0], 0.0, 360.0, 1.0, false, false, phase)
                    .event_value(kind, bounds)?;
                (hsv, alpha)
            }
            Self::Alpha { hsv, alpha, phase } => {
                let alpha =
                    RangeHandlerValue::new(alpha * 100.0, 0.0, 100.0, 1.0, false, false, phase)
                        .event_value(kind, bounds)?
                        / 100.0;
                (hsv, alpha)
            }
            Self::Field {
                mut hsv,
                mut alpha,
                format,
                index,
            } => {
                let text = match kind {
                    UiEventKind::TextChanged(text) | UiEventKind::Submitted(text) => text,
                    _ => return None,
                };
                update_field(&mut hsv, &mut alpha, format, index, text)?;
                (hsv, alpha)
            }
        };
        let [r, g, b] = hsv_to_rgb(hsv);
        Some(Color::srgba(r, g, b, alpha))
    }
}

fn update_pad(hsv: &mut [f32; 3], kind: &UiEventKind, bounds: Option<Rect>) -> Option<()> {
    match kind {
        UiEventKind::Gesture(gesture) if gesture.phase != GesturePhase::Cancelled => {
            let position = match gesture.kind {
                GestureKind::Tap { position } | GestureKind::Pan { position, .. } => position,
                _ => return None,
            };
            let bounds = bounds?;
            if bounds.size.width <= 0.0 || bounds.size.height <= 0.0 {
                return None;
            }
            hsv[1] = ((position.x - bounds.origin.x) / bounds.size.width).clamp(0.0, 1.0);
            hsv[2] = (1.0 - (position.y - bounds.origin.y) / bounds.size.height).clamp(0.0, 1.0);
        }
        UiEventKind::KeyInput(input)
            if input.state == KeyState::Pressed
                && !input.modifiers.command()
                && !input.modifiers.alt =>
        {
            let step = if input.modifiers.shift { 0.001 } else { 0.01 };
            match input.key {
                Key::ArrowLeft => hsv[1] = (hsv[1] - step).max(0.0),
                Key::ArrowRight => hsv[1] = (hsv[1] + step).min(1.0),
                Key::ArrowDown => hsv[2] = (hsv[2] - step).max(0.0),
                Key::ArrowUp => hsv[2] = (hsv[2] + step).min(1.0),
                Key::Home => {
                    hsv[1] = 0.0;
                    hsv[2] = 1.0;
                }
                Key::End => {
                    hsv[1] = 1.0;
                    hsv[2] = 0.0;
                }
                _ => return None,
            }
        }
        _ => return None,
    }
    Some(())
}

fn update_field(
    hsv: &mut [f32; 3],
    alpha: &mut f32,
    format: ColorValueFormat,
    index: usize,
    text: &str,
) -> Option<()> {
    if format == ColorValueFormat::Hex {
        let color = Color::from_hex(text.trim()).ok()?;
        let [r, g, b, next_alpha] = color.to_srgba();
        *hsv = rgb_to_hsv([r, g, b], hsv[0]);
        *alpha = next_alpha;
        return Some(());
    }
    let value = text.trim().parse::<f32>().ok()?;
    let maximum = match (format, index) {
        (ColorValueFormat::Rgb, 0..=2) => 255.0,
        (ColorValueFormat::Hsl | ColorValueFormat::Hsv, 0) => 360.0,
        _ => 100.0,
    };
    if !value.is_finite() || !(0.0..=maximum).contains(&value) {
        return None;
    }
    if index == 3 {
        *alpha = value / 100.0;
        return Some(());
    }
    match format {
        ColorValueFormat::Rgb => {
            let mut rgb = hsv_to_rgb(*hsv);
            *rgb.get_mut(index)? = value / 255.0;
            *hsv = rgb_to_hsv(rgb, hsv[0]);
        }
        ColorValueFormat::Hsv => {
            *hsv.get_mut(index)? = if index == 0 { value } else { value / 100.0 };
        }
        ColorValueFormat::Hsl => {
            let [h, s, v] = *hsv;
            let lightness = v * (1.0 - s * 0.5);
            let saturation = if lightness == 0.0 || lightness == 1.0 {
                0.0
            } else {
                (v - lightness) / lightness.min(1.0 - lightness)
            };
            let mut hsl = [h, saturation, lightness];
            *hsl.get_mut(index)? = if index == 0 { value } else { value / 100.0 };
            let [h, s, l] = hsl;
            let v = l + s * l.min(1.0 - l);
            *hsv = [
                h,
                if v == 0.0 {
                    hsv[1]
                } else {
                    2.0 * (1.0 - l / v)
                },
                v,
            ];
        }
        ColorValueFormat::Hex => unreachable!("hex is handled before numeric parsing"),
    }
    Some(())
}

fn hsv_to_rgb([h, s, v]: [f32; 3]) -> [f32; 3] {
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
    rgb.map(|channel| channel + v - chroma)
}

fn rgb_to_hsv([r, g, b]: [f32; 3], previous_hue: f32) -> [f32; 3] {
    let maximum = r.max(g).max(b);
    let minimum = r.min(g).min(b);
    let delta = maximum - minimum;
    let hue = if delta < f32::EPSILON {
        previous_hue
    } else if maximum == r {
        60.0 * ((g - b) / delta).rem_euclid(6.0)
    } else if maximum == g {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };
    [
        hue,
        if maximum == 0.0 { 0.0 } else { delta / maximum },
        maximum,
    ]
}
