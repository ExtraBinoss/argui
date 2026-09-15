use argui_animation::Frame;
use argui_core::Transform2D;
use argui_paint::VectorId;
use argui_runtime::{Context, Render};
use argui_ui::{Dimensions, Element, LayoutStyle};

#[derive(Clone, Debug)]
pub struct Spinner {
    vector: VectorId,
    size: f32,
    radians: f32,
    reduced_motion: bool,
}

impl Spinner {
    /// Creates a decorative spinner from registered vector `vector` at `size` logical pixels.
    #[must_use]
    pub const fn new(vector: VectorId, size: f32) -> Self {
        Self {
            vector,
            size,
            radians: 0.0,
            reduced_motion: false,
        }
    }
}

impl Render for Spinner {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        self.reduced_motion = cx.environment().reduced_motion;
        Element::vector(self.vector)
            .layout_style(LayoutStyle {
                size: Dimensions::length(self.size),
                ..LayoutStyle::default()
            })
            .transform(Transform2D::IDENTITY.rotate(self.radians))
            .semantic_hidden(true)
    }

    fn animation_frame(&mut self, frame: Frame, cx: &mut Context<Self>) {
        if !self.reduced_motion {
            self.radians = (self.radians
                + frame.elapsed.as_secs_f64() as f32 * std::f32::consts::TAU)
                % std::f32::consts::TAU;
            cx.notify();
        }
    }

    fn wants_animation_frame(&self) -> bool {
        !self.reduced_motion
    }
}
