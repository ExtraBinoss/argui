use core::fmt;

use argui_core::Affine2D;

use crate::{
    ClipChain, CompositorId, CompositorLayer, CompositorPatch, GpuCanvasPrimitive, ImagePrimitive,
    LayerStyle, Quad, VectorPrimitive,
};

#[derive(Clone, Debug, PartialEq)]
/// Ordered drawing commands submitted to a renderer.
pub enum DisplayCommand {
    Quad(Quad),
    Image(ImagePrimitive),
    GpuCanvas(GpuCanvasPrimitive),
    Vector(VectorPrimitive),
    Text {
        block: usize,
        transform: Affine2D,
        clips: ClipChain,
    },
    BeginLayer(LayerStyle),
    EndLayer,
    BeginCompositor(CompositorLayer),
    EndCompositor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Error returned when layer begin/end commands are unbalanced.
pub enum DisplayListError {
    UnexpectedLayerEnd { command: usize },
    UnclosedLayers { count: usize },
    MismatchedLayerEnd { command: usize },
}

impl fmt::Display for DisplayListError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedLayerEnd { command } => {
                write!(
                    formatter,
                    "layer ends without a matching begin at command {command}"
                )
            }
            Self::UnclosedLayers { count } => {
                write!(formatter, "display list has {count} unclosed layers")
            }
            Self::MismatchedLayerEnd { command } => {
                write!(
                    formatter,
                    "display list closes a different layer at command {command}"
                )
            }
        }
    }
}

impl std::error::Error for DisplayListError {}

#[derive(Clone, Debug, Default, PartialEq)]
/// Mutable sequence of paint commands with cached quad count.
pub struct DisplayList {
    commands: Vec<DisplayCommand>,
    quad_count: usize,
    gpu_canvas_count: usize,
}

impl DisplayList {
    /// Allocated command capacity, excluding shared and nested command payloads.
    #[must_use]
    pub fn storage_bytes(&self) -> usize {
        self.commands.capacity() * size_of::<DisplayCommand>()
    }
    /// Creates an empty display list.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            commands: Vec::new(),
            quad_count: 0,
            gpu_canvas_count: 0,
        }
    }

    /// Appends a quad primitive.
    pub fn push_quad(&mut self, quad: Quad) {
        self.commands.push(DisplayCommand::Quad(quad));
        self.quad_count += 1;
    }

    /// Appends an image primitive.
    pub fn push_image(&mut self, image: ImagePrimitive) {
        self.commands.push(DisplayCommand::Image(image));
    }

    /// Appends a retained GPU-canvas primitive.
    pub fn push_gpu_canvas(&mut self, canvas: GpuCanvasPrimitive) {
        self.commands.push(DisplayCommand::GpuCanvas(canvas));
        self.gpu_canvas_count += 1;
    }

    /// Appends a vector primitive.
    pub fn push_vector(&mut self, vector: VectorPrimitive) {
        self.commands.push(DisplayCommand::Vector(vector));
    }

    /// Appends a text block with identity transform and no clipping.
    pub fn push_text(&mut self, block: usize) {
        self.push_text_transformed(block, Affine2D::IDENTITY, ClipChain::default());
    }

    /// Appends a text block with an affine transform and clip chain.
    /// * `clips` — clipping regions applied to the text block.
    pub fn push_text_transformed(&mut self, block: usize, transform: Affine2D, clips: ClipChain) {
        self.commands.push(DisplayCommand::Text {
            block,
            transform,
            clips,
        });
    }

    /// Begins a layer using `style`; pair it with [`Self::end_layer`].
    pub fn begin_layer(&mut self, style: LayerStyle) {
        self.commands.push(DisplayCommand::BeginLayer(style));
    }

    /// Ends the most recently begun layer.
    pub fn end_layer(&mut self) {
        self.commands.push(DisplayCommand::EndLayer);
    }

    /// Begins a retained compositor group; pair it with [`Self::end_compositor`].
    ///
    /// * `layer` — stable compositor metadata for the enclosed commands.
    pub fn begin_compositor(&mut self, layer: CompositorLayer) {
        self.commands.push(DisplayCommand::BeginCompositor(layer));
    }

    /// Ends the most recently begun compositor group.
    pub fn end_compositor(&mut self) {
        self.commands.push(DisplayCommand::EndCompositor);
    }

    /// Applies composition-only patches without modifying retained paint commands.
    ///
    /// * `patches` — latest transform and opacity for retained layer identities.
    ///
    /// Returns the number of compositor entries whose presentation changed.
    pub fn apply_compositor_patches(&mut self, patches: &[CompositorPatch]) -> usize {
        let patches = patches
            .iter()
            .map(|patch| (patch.id, (patch.transform, patch.opacity)))
            .collect::<std::collections::HashMap<_, _>>();
        let mut changed = 0;
        for command in &mut self.commands {
            let DisplayCommand::BeginCompositor(layer) = command else {
                continue;
            };
            let Some((transform, opacity)) = patches.get(&layer.id) else {
                continue;
            };
            changed += usize::from(layer.update(*transform, *opacity));
        }
        changed
    }

    /// Returns retained compositor layers in display order.
    pub fn compositor_layers(&self) -> impl Iterator<Item = &CompositorLayer> {
        self.commands.iter().filter_map(|command| match command {
            DisplayCommand::BeginCompositor(layer) => Some(layer),
            _ => None,
        })
    }

    /// Updates the source bounds for one retained compositor layer.
    ///
    /// * `id` — layer whose painted bounds changed.
    /// * `bounds` — new surface-space source bounds.
    ///
    /// Returns whether the layer was found.
    pub fn set_compositor_bounds(&mut self, id: CompositorId, bounds: argui_core::Rect) -> bool {
        self.commands.iter_mut().any(|command| {
            let DisplayCommand::BeginCompositor(layer) = command else {
                return false;
            };
            if layer.id != id {
                return false;
            }
            layer.bounds = bounds;
            true
        })
    }

    /// Expands every compositor source to include its retained descendant primitives.
    ///
    /// Text is covered by the compositor's initial element bounds. Quad, image,
    /// vector, effect, and nested compositor bounds expand that conservative base.
    pub fn resolve_compositor_bounds(&mut self) {
        let mut stack = Vec::<(CompositorId, argui_core::Rect)>::new();
        let mut resolved = Vec::new();
        for command in &self.commands {
            match command {
                DisplayCommand::BeginCompositor(layer) => stack.push((layer.id, layer.bounds)),
                DisplayCommand::EndCompositor => {
                    let Some((id, bounds)) = stack.pop() else {
                        continue;
                    };
                    resolved.push((id, bounds));
                    if let Some((_, parent)) = stack.last_mut() {
                        *parent = union(*parent, bounds);
                    }
                }
                DisplayCommand::Quad(quad) => {
                    expand_current(&mut stack, quad.transform.transform_rect(quad.bounds));
                }
                DisplayCommand::Image(image) => {
                    expand_current(&mut stack, image.transform.transform_rect(image.bounds));
                }
                DisplayCommand::GpuCanvas(canvas) => {
                    expand_current(&mut stack, canvas.transform.transform_rect(canvas.bounds));
                }
                DisplayCommand::Vector(vector) => {
                    expand_current(&mut stack, vector.transform.transform_rect(vector.bounds));
                }
                DisplayCommand::BeginLayer(layer) => {
                    expand_current(&mut stack, layer.transformed_bounds());
                }
                DisplayCommand::Text { .. } | DisplayCommand::EndLayer => {}
            }
        }
        for (id, bounds) in resolved {
            self.set_compositor_bounds(id, bounds);
        }
    }

    /// Removes all commands and resets the cached quad count.
    pub fn clear(&mut self) {
        self.commands.clear();
        self.quad_count = 0;
        self.gpu_canvas_count = 0;
    }

    /// Appends commands from an iterator and updates the quad count.
    pub fn extend(&mut self, commands: impl IntoIterator<Item = DisplayCommand>) {
        for command in commands {
            if matches!(command, DisplayCommand::Quad(_)) {
                self.quad_count += 1;
            }
            if matches!(command, DisplayCommand::GpuCanvas(_)) {
                self.gpu_canvas_count += 1;
            }
            self.commands.push(command);
        }
    }

    /// Returns the number of commands.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.commands.len()
    }

    /// Returns whether there are no commands.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Returns commands in submission order.
    #[must_use]
    pub fn commands(&self) -> &[DisplayCommand] {
        &self.commands
    }

    /// Returns the number of quad commands.
    #[must_use]
    pub const fn quad_count(&self) -> usize {
        self.quad_count
    }

    /// Returns the number of GPU-canvas commands.
    #[must_use]
    pub const fn gpu_canvas_count(&self) -> usize {
        self.gpu_canvas_count
    }

    /// Checks that every layer end has a matching begin.
    ///
    /// # Errors
    /// Returns [`DisplayListError::UnexpectedLayerEnd`] for an unmatched end or
    /// [`DisplayListError::MismatchedLayerEnd`] for the wrong end kind, or
    /// [`DisplayListError::UnclosedLayers`] when the list ends inside a layer.
    pub fn validate(&self) -> Result<(), DisplayListError> {
        #[derive(Clone, Copy, Eq, PartialEq)]
        enum LayerKind {
            Effect,
            Compositor,
        }
        let mut stack = Vec::new();
        for (command, item) in self.commands.iter().enumerate() {
            match item {
                DisplayCommand::BeginLayer(_) => stack.push(LayerKind::Effect),
                DisplayCommand::BeginCompositor(_) => stack.push(LayerKind::Compositor),
                DisplayCommand::EndLayer | DisplayCommand::EndCompositor if stack.is_empty() => {
                    return Err(DisplayListError::UnexpectedLayerEnd { command });
                }
                DisplayCommand::EndLayer => {
                    if stack.pop() != Some(LayerKind::Effect) {
                        return Err(DisplayListError::MismatchedLayerEnd { command });
                    }
                }
                DisplayCommand::EndCompositor => {
                    if stack.pop() != Some(LayerKind::Compositor) {
                        return Err(DisplayListError::MismatchedLayerEnd { command });
                    }
                }
                DisplayCommand::Quad(_)
                | DisplayCommand::Image(_)
                | DisplayCommand::GpuCanvas(_)
                | DisplayCommand::Vector(_)
                | DisplayCommand::Text { .. } => {}
            }
        }
        if stack.is_empty() {
            Ok(())
        } else {
            Err(DisplayListError::UnclosedLayers { count: stack.len() })
        }
    }
}

/// Expands the innermost open compositor layer to include `bounds`.
fn expand_current(stack: &mut [(CompositorId, argui_core::Rect)], bounds: argui_core::Rect) {
    if let Some((_, current)) = stack.last_mut() {
        *current = union(*current, bounds);
    }
}

/// Returns the conservative axis-aligned union of two rectangles.
fn union(left: argui_core::Rect, right: argui_core::Rect) -> argui_core::Rect {
    let x = left.origin.x.min(right.origin.x);
    let y = left.origin.y.min(right.origin.y);
    let right_edge = (left.origin.x + left.size.width).max(right.origin.x + right.size.width);
    let bottom = (left.origin.y + left.size.height).max(right.origin.y + right.size.height);
    argui_core::Rect::new(
        argui_core::Point::new(x, y),
        argui_core::Size::new((right_edge - x).max(0.0), (bottom - y).max(0.0)),
    )
}
