use argui_core::Color;
use argui_paint::{Fill, QuadStyle};

/// Resolves the opaque solid color directly behind descendant text.
///
/// `parent` is the nearest already-resolved backdrop, `quad` is the current element's paint,
/// and `unknown` indicates that a native backdrop or background effect changes the pixels after
/// ordinary quad painting. The return value is `None` whenever an exact opaque color cannot be
/// established.
pub(super) fn resolve(parent: Option<Color>, quad: &QuadStyle, unknown: bool) -> Option<Color> {
    if unknown {
        return None;
    }
    let Some(fill) = &quad.background else {
        return parent;
    };
    let Fill::Solid(source) = fill else {
        return None;
    };
    let [source_red, source_green, source_blue, source_alpha] = source.to_linear_rgba();
    let alpha = (source_alpha * quad.opacity).clamp(0.0, 1.0);
    if alpha <= f32::EPSILON {
        return parent;
    }
    if alpha >= 1.0 - f32::EPSILON {
        return Some(Color::linear_rgb(source_red, source_green, source_blue));
    }
    let [backdrop_red, backdrop_green, backdrop_blue, backdrop_alpha] = parent?.to_linear_rgba();
    if backdrop_alpha < 1.0 - f32::EPSILON {
        return None;
    }
    Some(Color::linear_rgb(
        source_red * alpha + backdrop_red * (1.0 - alpha),
        source_green * alpha + backdrop_green * (1.0 - alpha),
        source_blue * alpha + backdrop_blue * (1.0 - alpha),
    ))
}
