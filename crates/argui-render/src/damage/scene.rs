use argui_core::{Affine2D, Point, Rect, Size};
use argui_paint::{ClipChain, DisplayCommand, DisplayList, Filter, LayerStyle};
use argui_text::{PreparedDecoration, PreparedGlyph, PreparedText};
use std::sync::Arc;

use crate::{DamagePlan, DamageRegion, DamageTracking, EffectDamage, EffectRegistry};

#[derive(Clone, Debug, Default, PartialEq)]
struct TextVisual {
    glyphs: Vec<PreparedGlyph>,
    decorations: Vec<PreparedDecoration>,
}

#[derive(Clone, Debug, PartialEq)]
struct SceneItem {
    command: DisplayCommand,
    text: Option<Arc<TextVisual>>,
    bounds: Option<DamageRegion>,
}

#[derive(Clone, Debug, PartialEq)]
struct EffectLayer {
    style: LayerStyle,
    bounds: DamageRegion,
    dependency: DamageRegion,
}

/// Renderer-neutral scene signature used to detect changed pixels.
///
/// Snapshots retain paint command identities, prepared text visuals and their
/// conservative physical bounds. They do not own GPU resources.
#[derive(Clone, Debug, PartialEq)]
pub struct DamageSnapshot {
    items: Vec<SceneItem>,
    text_visuals: Vec<Arc<TextVisual>>,
    effect_layers: Vec<EffectLayer>,
    viewport: [u32; 2],
    scale_factor: f32,
}

impl DamageSnapshot {
    /// Captures command identities, text contents and conservative physical bounds.
    ///
    /// * `display_list` — ordered paint commands represented by the snapshot.
    /// * `text` — prepared text referenced by those commands.
    /// * `text_bounds` — physical bounds per prepared text block, or `None` for empty blocks.
    /// * `viewport` — physical viewport width and height.
    /// * `scale_factor` — logical-to-physical scale applied to paint commands.
    #[must_use]
    pub fn capture(
        display_list: &DisplayList,
        text: &PreparedText,
        text_bounds: &[Option<Rect>],
        viewport: [u32; 2],
        scale_factor: f32,
    ) -> Self {
        let text_visuals = text_visuals(text);
        Self::capture_with_visuals(
            display_list,
            &text_visuals,
            text_bounds,
            viewport,
            scale_factor,
        )
    }

    /// Recaptures composition-only changes while reusing prepared text visuals.
    ///
    /// This is valid when `display_list` differs from this snapshot only through
    /// compositor-layer transforms or opacity. It lets the renderer compute
    /// old and new damage bounds without reshaping or copying prepared text.
    ///
    /// * `display_list` — retained commands with updated compositor properties.
    /// * `text_bounds` — physical bounds retained from the last paint preparation.
    /// * `viewport` — physical viewport width and height.
    /// * `scale_factor` — logical-to-physical scale used by the retained scene.
    #[must_use]
    pub(crate) fn capture_composite(
        &self,
        display_list: &DisplayList,
        text_bounds: &[Option<Rect>],
        viewport: [u32; 2],
        scale_factor: f32,
    ) -> Self {
        Self::capture_with_visuals(
            display_list,
            &self.text_visuals,
            text_bounds,
            viewport,
            scale_factor,
        )
    }

    /// Captures a scene with text visuals already grouped by display-list block.
    ///
    /// * `display_list` — ordered paint commands represented by the snapshot.
    /// * `text_visuals` — retained glyphs and decorations indexed by text block.
    /// * `text_bounds` — physical bounds per prepared text block.
    /// * `viewport` — physical viewport width and height.
    /// * `scale_factor` — logical-to-physical scale applied to paint commands.
    fn capture_with_visuals(
        display_list: &DisplayList,
        text_visuals: &[Arc<TextVisual>],
        text_bounds: &[Option<Rect>],
        viewport: [u32; 2],
        scale_factor: f32,
    ) -> Self {
        let mut items = Vec::with_capacity(display_list.commands().len());
        let mut layers = Vec::new();
        let mut effect_layers = Vec::new();
        for command in display_list.commands() {
            let text = match command {
                DisplayCommand::Text { block, .. } => text_visuals.get(*block).cloned(),
                _ => None,
            };
            let bounds = command_bounds(command, text_bounds, viewport, scale_factor);
            if let Some(style) = command_layer_style(command, scale_factor)
                && layer_propagates_damage(&style)
                && let Some(bounds) = bounds
                && let Some(dependency) = effect_dependency(&style, viewport)
            {
                effect_layers.push(EffectLayer {
                    style,
                    bounds,
                    dependency,
                });
            }
            let index = items.len();
            match command {
                DisplayCommand::BeginLayer(_) | DisplayCommand::BeginCompositor(_) => {
                    layers.push((index, bounds));
                }
                DisplayCommand::EndLayer | DisplayCommand::EndCompositor => {
                    if let Some((_, layer_bounds)) = layers.pop() {
                        items.push(SceneItem {
                            command: command.clone(),
                            text,
                            bounds: layer_bounds,
                        });
                        continue;
                    }
                }
                _ => {}
            }
            items.push(SceneItem {
                command: command.clone(),
                text,
                bounds,
            });
        }
        Self {
            items,
            text_visuals: text_visuals.to_vec(),
            effect_layers,
            viewport,
            scale_factor,
        }
    }

    /// Compares this snapshot with `current` and resolves an adaptive damage plan.
    ///
    /// * `current` — scene that will replace this snapshot.
    /// * `tracking` — region-count and area thresholds used by the decision.
    #[must_use]
    pub fn compare(&self, current: &Self, tracking: DamageTracking) -> DamagePlan {
        self.compare_regions(current, tracking, None)
    }

    /// Compares this snapshot with `current` and propagates changes through effect layers.
    ///
    /// Bounded custom effects expand intersecting changes to their complete
    /// layout-derived layer bounds. An unbounded or missing custom definition
    /// conservatively selects a full frame whenever the scene changes.
    ///
    /// * `current` — scene that will replace this snapshot.
    /// * `tracking` — region-count and area thresholds used by the decision.
    /// * `effects` — custom effect definitions used by the current scene.
    #[must_use]
    pub fn compare_with_effects(
        &self,
        current: &Self,
        tracking: DamageTracking,
        effects: &EffectRegistry,
    ) -> DamagePlan {
        self.compare_regions(current, tracking, Some(effects))
    }

    /// Resolves raw scene differences with optional effect dependency propagation.
    fn compare_regions(
        &self,
        current: &Self,
        tracking: DamageTracking,
        effects: Option<&EffectRegistry>,
    ) -> DamagePlan {
        if self.viewport != current.viewport || self.scale_factor != current.scale_factor {
            return DamagePlan::Full;
        }
        if !tracking.enabled {
            return DamagePlan::Full;
        }
        if self.items == current.items {
            return DamagePlan::Unchanged;
        }
        let mut regions = if self.items.len() == current.items.len() {
            self.items
                .iter()
                .zip(&current.items)
                .filter(|(old, new)| old != new)
                .flat_map(|(old, new)| [old.bounds, new.bounds])
                .flatten()
                .collect::<Vec<_>>()
        } else {
            changed_middle(self, current)
        };
        if let Some(effects) = effects {
            if current
                .effect_layers
                .iter()
                .any(|layer| contains_unbounded_effect(&layer.style, effects))
            {
                return DamagePlan::Full;
            }
            propagate_effect_bounds(&mut regions, &current.effect_layers);
        }
        DamagePlan::resolve(regions, current.viewport, tracking)
    }
}

/// Resolves changed scene items into an adaptive surface damage decision.
pub(crate) fn scene_damage(
    previous: Option<&DamageSnapshot>,
    current: &DamageSnapshot,
    tracking: DamageTracking,
    effects: &EffectRegistry,
) -> DamagePlan {
    previous.map_or(DamagePlan::Full, |previous| {
        previous.compare_with_effects(current, tracking, effects)
    })
}

/// Returns a scaled layer style for a layer-opening command.
fn command_layer_style(command: &DisplayCommand, scale: f32) -> Option<LayerStyle> {
    match command {
        DisplayCommand::BeginLayer(style) => Some(style.scaled(scale)),
        DisplayCommand::BeginCompositor(layer) => Some(layer.style().scaled(scale)),
        _ => None,
    }
}

/// Returns whether changes can spread beyond their original primitive bounds.
fn layer_propagates_damage(style: &LayerStyle) -> bool {
    !style.filters.is_empty() || !style.backdrop_filters.is_empty() || !style.shadows.is_empty()
}

/// Returns whether a layer contains a custom effect without finite dependency bounds.
fn contains_unbounded_effect(style: &LayerStyle, effects: &EffectRegistry) -> bool {
    style
        .filters
        .iter()
        .chain(&style.backdrop_filters)
        .any(|filter| match filter {
            Filter::Effect(effect) => effects
                .get(&effect.id)
                .is_none_or(|definition| definition.damage == EffectDamage::Unbounded),
            _ => false,
        })
}

/// Expands intersecting changes to complete effect outputs until nesting stabilizes.
fn propagate_effect_bounds(regions: &mut Vec<DamageRegion>, layers: &[EffectLayer]) {
    loop {
        let mut changed = false;
        for layer in layers {
            if regions
                .iter()
                .copied()
                .any(|region| regions_overlap(region, layer.dependency))
                && !regions.contains(&layer.bounds)
            {
                regions.push(layer.bounds);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
}

/// Returns the conservative input footprint for one effect layer.
fn effect_dependency(style: &LayerStyle, viewport: [u32; 2]) -> Option<DamageRegion> {
    let backdrop_expansion = style
        .backdrop_filters
        .iter()
        .map(Filter::expansion)
        .sum::<f32>()
        .max(0.0);
    let mut bounds = style.expanded_bounds();
    bounds.origin.x -= backdrop_expansion;
    bounds.origin.y -= backdrop_expansion;
    bounds.size.width += backdrop_expansion * 2.0;
    bounds.size.height += backdrop_expansion * 2.0;
    DamageRegion::from_rect(style.transform.transform_rect(bounds), viewport)
}

/// Returns whether two non-empty physical regions overlap.
const fn regions_overlap(left: DamageRegion, right: DamageRegion) -> bool {
    left.x < right.right()
        && right.x < left.right()
        && left.y < right.bottom()
        && right.y < left.bottom()
}

/// Returns bounds from the changed middle after equal prefixes and suffixes are removed.
fn changed_middle(previous: &DamageSnapshot, current: &DamageSnapshot) -> Vec<DamageRegion> {
    let prefix = previous
        .items
        .iter()
        .zip(&current.items)
        .take_while(|(old, new)| old == new)
        .count();
    let remaining_old = previous.items.len().saturating_sub(prefix);
    let remaining_new = current.items.len().saturating_sub(prefix);
    let suffix = previous.items[prefix..]
        .iter()
        .rev()
        .zip(current.items[prefix..].iter().rev())
        .take_while(|(old, new)| old == new)
        .count()
        .min(remaining_old)
        .min(remaining_new);
    previous.items[prefix..previous.items.len() - suffix]
        .iter()
        .chain(&current.items[prefix..current.items.len() - suffix])
        .filter_map(|item| item.bounds)
        .collect()
}

/// Groups prepared glyphs and decorations by their text block.
fn text_visuals(text: &PreparedText) -> Vec<Arc<TextVisual>> {
    let mut visuals = vec![TextVisual::default(); text.blocks];
    for glyph in &text.glyphs {
        if let Some(block) = visuals.get_mut(glyph.block) {
            block.glyphs.push(*glyph);
        }
    }
    for decoration in &text.decorations {
        if let Some(block) = visuals.get_mut(decoration.block) {
            block.decorations.push(*decoration);
        }
    }
    visuals.into_iter().map(Arc::new).collect()
}

/// Resolves one display command to conservative physical-pixel bounds.
fn command_bounds(
    command: &DisplayCommand,
    text_bounds: &[Option<Rect>],
    viewport: [u32; 2],
    scale: f32,
) -> Option<DamageRegion> {
    let bounds = match command {
        DisplayCommand::Quad(quad) => {
            transformed_bounds(quad.bounds, quad.transform, &quad.clips, scale)
        }
        DisplayCommand::Image(image) => {
            transformed_bounds(image.bounds, image.transform, &image.clips, scale)
        }
        DisplayCommand::GpuCanvas(canvas) => {
            transformed_bounds(canvas.bounds, canvas.transform, &canvas.clips, scale)
        }
        DisplayCommand::Vector(vector) => {
            transformed_bounds(vector.bounds, vector.transform, &vector.clips, scale)
        }
        DisplayCommand::Text { block, .. } => text_bounds.get(*block).copied().flatten(),
        DisplayCommand::BeginLayer(layer) => Some(layer.scaled(scale).transformed_bounds()),
        DisplayCommand::BeginCompositor(layer) => {
            Some(layer.style().scaled(scale).transformed_bounds())
        }
        DisplayCommand::EndLayer | DisplayCommand::EndCompositor => None,
    }?;
    DamageRegion::from_rect(bounds, viewport)
}

/// Scales, transforms and clips a logical primitive rectangle.
fn transformed_bounds(
    bounds: Rect,
    transform: Affine2D,
    clips: &ClipChain,
    scale: f32,
) -> Option<Rect> {
    let mut bounds = transform
        .scaled(scale)
        .transform_rect(scale_rect(bounds, scale));
    for clip in clips.regions() {
        let clip_bounds = clip
            .transform
            .scaled(scale)
            .transform_rect(scale_rect(clip.bounds, scale));
        bounds = bounds.intersection(clip_bounds)?;
    }
    Some(bounds)
}

/// Converts `rect` from logical to physical coordinates.
fn scale_rect(rect: Rect, scale: f32) -> Rect {
    Rect::new(
        Point::new(rect.origin.x * scale, rect.origin.y * scale),
        Size::new(rect.size.width * scale, rect.size.height * scale),
    )
}
