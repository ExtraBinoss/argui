use argui_animation::{Duration, Iterations, Keyframe, Keyframes, Motion, Timeline, Timing};
use argui_core::Transform2D;
use argui_paint::VectorId;
use argui_runtime::{Context, Render};
use argui_ui::{Dimensions, Element, LayoutStyle, property};

#[derive(Clone, Debug)]
pub struct Spinner {
    vector: VectorId,
    size: f32,
    rotation: Option<Motion<Transform2D>>,
}

impl Spinner {
    /// Creates a decorative spinner from registered vector `vector` at `size` logical pixels.
    #[must_use]
    pub const fn new(vector: VectorId, size: f32) -> Self {
        Self {
            vector,
            size,
            rotation: None,
        }
    }
}

impl Render for Spinner {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let rotation = self
            .rotation
            .get_or_insert_with(|| Motion::new(Transform2D::IDENTITY));
        if cx.environment().reduced_motion {
            rotation.set(Transform2D::IDENTITY);
        } else if !rotation.is_active() {
            rotation.set(Transform2D::IDENTITY);
            rotation.play(rotation_timeline());
        }
        Element::vector(self.vector)
            .layout_style(LayoutStyle {
                size: Dimensions::length(self.size),
                ..LayoutStyle::default()
            })
            .bind(property::Transform, rotation.clone())
            .semantic_hidden(true)
    }
}

fn rotation_timeline() -> Timeline<Transform2D> {
    Timeline::new(
        Keyframes::new([
            Keyframe::new(0.0, Transform2D::IDENTITY),
            Keyframe::new(1.0, Transform2D::IDENTITY.rotate(std::f32::consts::TAU)),
        ])
        .expect("spinner keyframes cover one complete rotation"),
        Timing::new(Duration::from_secs(1)).iterations(Iterations::Infinite),
    )
    .expect("spinner timing is finite and non-zero")
}
