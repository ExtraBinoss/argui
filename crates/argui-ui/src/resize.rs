//! Native resize-handle configuration, independent of application callbacks.

/// Dimension controlled by a native resize handle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResizeAxis {
    /// Pointer x changes the target's width.
    Horizontal,
    /// Pointer y changes the target's height.
    Vertical,
}

/// A handle resizes one addressable element; flex siblings receive remaining space.
#[derive(Clone, Debug, PartialEq)]
pub struct ResizeHandle {
    /// Unique public key of the element whose preferred dimension changes.
    pub target: String,
    /// Controlled dimension and pointer coordinate.
    pub axis: ResizeAxis,
    /// A trailing pane grows when the pointer moves toward the leading edge.
    pub trailing: bool,
    /// Smallest allowed size in logical pixels.
    pub minimum: f32,
    /// Largest allowed size in logical pixels.
    pub maximum: f32,
}
