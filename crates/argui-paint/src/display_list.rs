use core::fmt;

use argui_core::Affine2D;

use crate::{ClipChain, GpuCanvasPrimitive, ImagePrimitive, LayerStyle, Quad, VectorPrimitive};

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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Error returned when layer begin/end commands are unbalanced.
pub enum DisplayListError {
    UnexpectedLayerEnd { command: usize },
    UnclosedLayers { count: usize },
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
    /// [`DisplayListError::UnclosedLayers`] when the list ends inside a layer.
    pub fn validate(&self) -> Result<(), DisplayListError> {
        let mut depth = 0_usize;
        for (command, item) in self.commands.iter().enumerate() {
            match item {
                DisplayCommand::BeginLayer(_) => depth += 1,
                DisplayCommand::EndLayer if depth == 0 => {
                    return Err(DisplayListError::UnexpectedLayerEnd { command });
                }
                DisplayCommand::EndLayer => depth -= 1,
                DisplayCommand::Quad(_)
                | DisplayCommand::Image(_)
                | DisplayCommand::GpuCanvas(_)
                | DisplayCommand::Vector(_)
                | DisplayCommand::Text { .. } => {}
            }
        }
        if depth == 0 {
            Ok(())
        } else {
            Err(DisplayListError::UnclosedLayers { count: depth })
        }
    }
}
