use argui_core::Rect;
use argui_paint::{Filter, ShaderEffectId};

use crate::{RendererError, effect_graph::EffectGraph};

use super::SurfaceRenderer;

pub(super) struct EffectPass {
    pub(super) mode: u32,
    pub(super) data: [f32; 4],
    pub(super) matrix: Option<[f32; 20]>,
    pub(super) bounds: Rect,
    pub(super) shader: Option<ShaderEffectId>,
    pub(super) target_size: Option<[u32; 2]>,
}

impl EffectPass {
    pub(super) const fn new(mode: u32, data: [f32; 4], bounds: Rect) -> Self {
        Self {
            mode,
            data,
            matrix: None,
            bounds,
            shader: None,
            target_size: None,
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl SurfaceRenderer {
    pub(super) fn validate_custom_effects(&self, graph: &EffectGraph) -> Result<(), RendererError> {
        for effect in graph.custom_effects() {
            if effect.parameters.len() > 24 {
                return Err(RendererError::TooManyEffectParameters {
                    provided: effect.parameters.len(),
                    maximum: 24,
                });
            }
            if !self.effect.contains(effect.shader) {
                return Err(RendererError::MissingShader(effect.shader.0));
            }
        }
        Ok(())
    }

    pub(super) fn apply_filters(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        source: usize,
        filters: &[Filter],
        viewport: [f32; 2],
        bounds: Rect,
    ) -> usize {
        let mut current = source;
        for filter in filters {
            let (mode, data, matrix) = match filter {
                Filter::Blur(radius) => {
                    let downsample = blur_downsample(*radius);
                    if downsample > 1 {
                        let mut pass = EffectPass::new(99, [0.0; 4], bounds);
                        pass.target_size = Some([
                            (viewport[0] as u32 / downsample).max(1),
                            (viewport[1] as u32 / downsample).max(1),
                        ]);
                        current = self.effect_pass(encoder, current, viewport, pass);
                    }
                    let sample_radius = *radius / downsample as f32 * 0.35;
                    current = self.effect_pass(encoder, current, viewport, {
                        let mut pass = EffectPass::new(1, [sample_radius, 0.0, 0.0, 0.0], bounds);
                        if downsample > 1 {
                            pass.target_size = Some([
                                (viewport[0] as u32 / downsample).max(1),
                                (viewport[1] as u32 / downsample).max(1),
                            ]);
                        }
                        pass
                    });
                    let mut vertical = EffectPass::new(2, [sample_radius, 0.0, 0.0, 0.0], bounds);
                    if downsample > 1 {
                        vertical.target_size = Some([
                            (viewport[0] as u32 / downsample).max(1),
                            (viewport[1] as u32 / downsample).max(1),
                        ]);
                    }
                    current = self.effect_pass(encoder, current, viewport, vertical);
                    if downsample > 1 {
                        current = self.effect_pass(
                            encoder,
                            current,
                            viewport,
                            EffectPass::new(99, [0.0; 4], bounds),
                        );
                    }
                    continue;
                }
                Filter::Brightness(value) => (3, [*value, 0.0, 0.0, 0.0], None),
                Filter::Contrast(value) => (4, [*value, 0.0, 0.0, 0.0], None),
                Filter::Saturation(value) => (5, [*value, 0.0, 0.0, 0.0], None),
                Filter::HueRotate(value) => (6, [*value, 0.0, 0.0, 0.0], None),
                Filter::Opacity(value) => (7, [*value, 0.0, 0.0, 0.0], None),
                Filter::ColorMatrix(value) => (8, [0.0; 4], Some(*value)),
                Filter::Refraction(value) => (
                    9,
                    [value.strength, value.chromatic_aberration, value.edge, 0.0],
                    None,
                ),
                Filter::Custom(effect) => {
                    let mut values = [0.0; 24];
                    values[..effect.parameters.len()].copy_from_slice(&effect.parameters);
                    let mut matrix = [0.0; 20];
                    matrix.copy_from_slice(&values[4..]);
                    let mut pass = EffectPass::new(
                        99,
                        values[..4].try_into().expect("four custom parameters"),
                        bounds,
                    );
                    pass.matrix = Some(matrix);
                    pass.shader = Some(effect.shader);
                    current = self.effect_pass(encoder, current, viewport, pass);
                    continue;
                }
            };
            let mut pass = EffectPass::new(mode, data, bounds);
            pass.matrix = matrix;
            current = self.effect_pass(encoder, current, viewport, pass);
        }
        current
    }
}

fn blur_downsample(radius: f32) -> u32 {
    if radius >= 12.0 {
        4
    } else if radius >= 6.0 {
        2
    } else {
        1
    }
}
