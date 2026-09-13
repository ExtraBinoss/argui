use std::hash::{Hash, Hasher};

use argui_core::{Color, Point, Rect, Size};
use argui_paint::{Border, CornerRadii, QuadStyle};
use argui_ui::{CustomElement, CustomLayoutContext, CustomMeasurement, CustomPaintContext};

/// Decorations have no intrinsic size; their ordinary container supplies the bounds.
/// Moving a marker invalidates painting without changing layout or hit geometry.
#[derive(Debug)]
pub(super) enum Decoration {
    Checkerboard,
    Marker {
        color: Color,
        diameter: f32,
        position: Point,
    },
}

impl CustomElement for Decoration {
    type State = ();

    fn create_state(&self) {}

    fn layout_revision(&self) -> u64 {
        0
    }

    fn paint_revision(&self) -> u64 {
        let mut hash = std::hash::DefaultHasher::new();
        if let Self::Marker {
            color,
            diameter,
            position,
        } = self
        {
            for value in [position.x, position.y, *diameter]
                .into_iter()
                .chain(color.to_linear_rgba())
            {
                value.to_bits().hash(&mut hash);
            }
        }
        hash.finish()
    }

    fn prepare(&self, _: &mut (), _: Size) {}

    fn layout(
        &self,
        _: &mut (),
        _: &mut dyn CustomLayoutContext,
    ) -> Result<CustomMeasurement, String> {
        Ok(CustomMeasurement {
            size: Size::new(0.0, 0.0),
            baseline: None,
        })
    }

    fn paint(&self, _: &mut (), cx: &mut CustomPaintContext<'_>) {
        let size = cx.bounds.size;
        match *self {
            Self::Marker {
                color,
                diameter,
                position,
            } => {
                cx.quad(
                    Rect::new(
                        Point::new(
                            size.width * position.x - diameter / 2.0,
                            size.height * position.y - diameter / 2.0,
                        ),
                        Size::new(diameter, diameter),
                    ),
                    QuadStyle::solid(color)
                        .border(Border::all(2.0, Color::WHITE))
                        .radius(CornerRadii::all(999.0)),
                );
            }
            Self::Checkerboard => {
                for row in 0..2 {
                    for col in 0..16 {
                        let left = (size.width * col as f32 / 16.0).round();
                        let right = (size.width * (col + 1) as f32 / 16.0).round();
                        let shade = if (row + col) % 2 == 0 { 230 } else { 170 };
                        cx.quad(
                            Rect::new(
                                Point::new(left, row as f32 * 8.0),
                                Size::new(right - left, 8.0),
                            ),
                            QuadStyle::solid(Color::from_srgb8(shade, shade, shade)),
                        );
                    }
                }
            }
        }
    }
}
