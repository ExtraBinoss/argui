use argui_core::{Affine2D, Point, Rect, Size};
use argui_paint::{ClipChain, DisplayCommand, DisplayList};
use argui_text::{PreparedDecoration, PreparedGlyph, PreparedText};

use crate::{DamagePlan, DamageRegion, DamageTracking};

#[derive(Clone, Debug, Default, PartialEq)]
struct TextVisual {
    glyphs: Vec<PreparedGlyph>,
    decorations: Vec<PreparedDecoration>,
}

#[derive(Clone, Debug, PartialEq)]
struct SceneItem {
    command: DisplayCommand,
    text: Option<TextVisual>,
    bounds: Option<DamageRegion>,
}

/// Renderer-neutral scene signature used to detect changed pixels.
///
/// Snapshots retain paint command identities, prepared text visuals and their
/// conservative physical bounds. They do not own GPU resources.
#[derive(Clone, Debug, PartialEq)]
pub struct DamageSnapshot {
    items: Vec<SceneItem>,
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
        let mut items = Vec::with_capacity(display_list.commands().len());
        let mut layers = Vec::new();
        for command in display_list.commands() {
            let text = match command {
                DisplayCommand::Text { block, .. } => text_visuals.get(*block).cloned(),
                _ => None,
            };
            let bounds = command_bounds(command, text_bounds, viewport, scale_factor);
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
        if self.viewport != current.viewport || self.scale_factor != current.scale_factor {
            return DamagePlan::Full;
        }
        if self.items == current.items {
            return DamagePlan::Unchanged;
        }
        let regions = if self.items.len() == current.items.len() {
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
        DamagePlan::resolve(regions, current.viewport, tracking)
    }
}

/// Resolves changed scene items into an adaptive surface damage decision.
pub(crate) fn scene_damage(
    previous: Option<&DamageSnapshot>,
    current: &DamageSnapshot,
    tracking: DamageTracking,
) -> DamagePlan {
    previous.map_or(DamagePlan::Full, |previous| {
        previous.compare(current, tracking)
    })
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
fn text_visuals(text: &PreparedText) -> Vec<TextVisual> {
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
    visuals
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
