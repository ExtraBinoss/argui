use argui_core::{Point, Rect, Size};
use argui_ui::{
    CustomConstraints, CustomElement, CustomLayoutContext, CustomMeasurement, CustomPaintContext,
};

/// Only variable-width transitions need layout on each frame. Measure ordinary
/// text children so inserted/removed graphemes use the host's actual font metrics.
#[derive(Debug)]
pub(super) struct ReelLayout {
    pub position: f32,
}

impl CustomElement for ReelLayout {
    type State = ();

    fn create_state(&self) {}

    fn layout_revision(&self) -> u64 {
        u64::from(self.position.to_bits())
    }

    fn paint_revision(&self) -> u64 {
        0
    }

    fn prepare(&self, _: &mut (), _: Size) {}

    fn layout(
        &self,
        _: &mut (),
        context: &mut dyn CustomLayoutContext,
    ) -> Result<CustomMeasurement, String> {
        let mut first = 0.0;
        let mut last = 0.0;
        let mut height = 0.0;
        for index in 0..context.child_count() {
            let measured = context.measure_child(
                index,
                CustomConstraints {
                    width: None,
                    height: None,
                },
            )?;
            context.place_child(index, Rect::new(Point::new(0.0, height), measured.size))?;
            if index == 0 {
                first = measured.size.width;
            }
            last = measured.size.width;
            height += measured.size.height;
        }
        Ok(CustomMeasurement {
            size: Size::new(first + (last - first) * self.position, height),
            baseline: None,
        })
    }

    fn paint(&self, _: &mut (), _: &mut CustomPaintContext<'_>) {}
}
