use argui::{
    core::{Color, Point, Rect, Size},
    paint::QuadStyle,
    runtime::{Context, Render},
    ui::{
        CustomElement, CustomLayoutContext, CustomMeasurement, CustomPaintContext, Element, Sides,
        length, percent,
    },
    widgets::default_theme,
};

#[derive(Debug)]
struct Ruler {
    background: Color,
    ticks: Color,
}

impl CustomElement for Ruler {
    type State = Vec<f32>;
    fn create_state(&self) -> Self::State {
        Vec::new()
    }
    fn layout_revision(&self) -> u64 {
        0
    }
    fn paint_revision(&self) -> u64 {
        0
    }
    fn layout(
        &self,
        _: &mut Self::State,
        _: &mut dyn CustomLayoutContext,
    ) -> Result<CustomMeasurement, String> {
        Ok(CustomMeasurement {
            size: Size::new(720.0, 180.0),
            baseline: None,
        })
    }
    fn prepare(&self, ticks: &mut Self::State, size: Size) {
        ticks.clear();
        ticks.extend((0..=(size.width / 40.0) as usize).map(|tick| tick as f32 * 40.0));
    }
    fn paint(&self, ticks: &mut Self::State, cx: &mut CustomPaintContext<'_>) {
        cx.quad(
            Rect::new(Point::default(), cx.bounds.size),
            QuadStyle::solid(self.background),
        );
        for (index, x) in ticks.iter().copied().enumerate() {
            let height = if index.is_multiple_of(5) { 52.0 } else { 24.0 };
            cx.quad(
                Rect::new(Point::new(x, 0.0), Size::new(2.0, height)),
                QuadStyle::solid(self.ticks),
            );
        }
    }
}

pub struct Example;

impl Render for Example {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = default_theme(cx.environment());
        let theme = themes.resolve(cx.environment().color_scheme);
        Element::container([Element::custom(Ruler {
            background: theme.card,
            ticks: theme.primary,
        })
        .width(percent(1.0))
        .height(length(180.0))])
        .width(percent(1.0))
        .height(percent(1.0))
        .padding(Sides::length(24.0))
        .background(theme.background)
    }
}
