use crate::{Button, ButtonBehavior, WidgetTheme};
use argui_core::{Color, Transform2D, TransformOrigin};
use argui_ui::{Element, Role, Semantics, Sides, auto, length};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ChartKind {
    #[default]
    Bar,
    Line,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ChartSeries {
    pub label: String,
    pub values: Vec<f64>,
    pub color: Color,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChartError {
    InvalidSize,
    InvalidValue,
    MismatchedSeries,
}

/// Cartesian chart with explicit size, zero baseline, legend, focusable points and tooltips.
/// The consumer owns data fetching and formats category/series labels before construction.
#[derive(Clone, Debug)]
pub struct Chart {
    pub key: String,
    pub label: String,
    pub categories: Vec<String>,
    pub series: Vec<ChartSeries>,
    pub kind: ChartKind,
    pub width: f32,
    pub height: f32,
}

impl Chart {
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        categories: impl IntoIterator<Item = String>,
        series: impl IntoIterator<Item = ChartSeries>,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            categories: categories.into_iter().collect(),
            series: series.into_iter().collect(),
            kind: ChartKind::Bar,
            width: 480.0,
            height: 240.0,
        }
    }

    pub fn domain(&self) -> Result<(f64, f64), ChartError> {
        if !self.width.is_finite()
            || !self.height.is_finite()
            || self.width < 80.0
            || self.height < 80.0
        {
            return Err(ChartError::InvalidSize);
        }
        let (mut minimum, mut maximum) = (0.0_f64, 0.0_f64);
        for series in &self.series {
            if series.values.len() != self.categories.len() {
                return Err(ChartError::MismatchedSeries);
            }
            for value in &series.values {
                if !value.is_finite() {
                    return Err(ChartError::InvalidValue);
                }
                minimum = minimum.min(*value);
                maximum = maximum.max(*value);
            }
        }
        if minimum == maximum {
            maximum = minimum + 1.0;
        }
        if !(maximum - minimum).is_finite() {
            return Err(ChartError::InvalidValue);
        }
        Ok((minimum, maximum))
    }

    /// Point activation identifies a source series and category; it never mutates the data.
    #[must_use]
    pub fn action(&self, event: &argui_ui::UiEvent) -> Option<(usize, usize)> {
        let suffix = event
            .target_key()?
            .strip_prefix(&format!("{}::point::", self.key))?;
        let (series, category) = suffix.split_once("::")?;
        let series = series.parse::<usize>().ok()?;
        let category = category.parse::<usize>().ok()?;
        self.series.get(series)?.values.get(category)?;
        ButtonBehavior::new(event.target_key()?, "point")
            .action(event)
            .map(|_| (series, category))
    }

    pub fn build(&self, theme: &WidgetTheme) -> Result<Element, ChartError> {
        let (minimum, maximum) = self.domain()?;
        let plot_height = self.height - 36.0;
        let plot_width = self.width - 48.0;
        let step = plot_width / self.categories.len().max(1) as f32;
        let y = |value: f64| {
            6.0 + ((maximum - value) / (maximum - minimum)) as f32 * (plot_height - 12.0)
        };
        let baseline = y(0.0);
        let mut marks = vec![position(
            Element::container([])
                .width(length(plot_width))
                .height(length(1.0))
                .background(theme.border)
                .semantic_hidden(true),
            40.0,
            baseline,
        )];
        for (index, value) in [(0, maximum), (1, minimum)] {
            marks.push(position(
                Element::text(format!("{value:.1}"))
                    .text_style(argui_text::TextStyle {
                        font_size: 11.0,
                        color: theme.muted_foreground,
                        ..Default::default()
                    })
                    .semantic_hidden(true),
                0.0,
                if index == 0 { 0.0 } else { plot_height - 16.0 },
            ));
        }
        for (series_index, series) in self.series.iter().enumerate() {
            let mut previous = None;
            for (index, value) in series.values.iter().enumerate() {
                let x = 40.0 + (index as f32 + 0.5) * step;
                let point_y = y(*value);
                if self.kind == ChartKind::Line
                    && let Some((px, py)) = previous
                {
                    let dx: f32 = x - px;
                    let dy: f32 = point_y - py;
                    marks.push(position(
                        Element::container([])
                            .width(length(dx.hypot(dy)))
                            .height(length(2.0))
                            .background(series.color)
                            .transform_origin(TransformOrigin::TOP_LEFT)
                            .transform(Transform2D::IDENTITY.rotate(dy.atan2(dx)))
                            .semantic_hidden(true),
                        px,
                        py,
                    ));
                }
                previous = Some((x, point_y));
                let label = format!("{} · {}: {value}", series.label, self.categories[index]);
                let (left, top, width, height) = if self.kind == ChartKind::Bar {
                    let width = step * 0.8 / self.series.len().max(1) as f32;
                    (
                        x - step * 0.4 + series_index as f32 * width,
                        baseline.min(point_y),
                        width.max(1.0),
                        (baseline - point_y).abs().max(2.0),
                    )
                } else {
                    (x - 5.0, point_y - 5.0, 10.0, 10.0)
                };
                let mut mark = Button::new(
                    format!("{}::point::{series_index}::{index}", self.key),
                    &label,
                    theme.ghost_button(),
                )
                .content(Element::container([]))
                .build()
                .width(length(width))
                .height(length(height))
                .padding(Sides::length(0.0))
                .background(series.color);
                mark.tooltip = Some(label);
                marks.push(position(mark, left, top));
            }
        }
        for (index, category) in self.categories.iter().enumerate() {
            marks.push(position(
                Element::text(category.clone())
                    .text_style(argui_text::TextStyle {
                        font_size: 11.0,
                        color: theme.muted_foreground,
                        ..Default::default()
                    })
                    .width(length(step))
                    .semantic_hidden(true),
                40.0 + index as f32 * step,
                plot_height + 8.0,
            ));
        }
        let plot = Element::container(marks)
            .width(length(self.width))
            .height(length(self.height));
        let legend = Element::row(self.series.iter().map(|series| {
            Element::row([
                Element::container([])
                    .width(length(10.0))
                    .height(length(10.0))
                    .background(series.color)
                    .semantic_hidden(true),
                Element::text(series.label.clone()).text_style(argui_text::TextStyle {
                    color: theme.foreground,
                    font_size: 13.0,
                    ..Default::default()
                }),
            ])
            .gap(6.0)
            .align_items(argui_ui::AlignItems::CENTER)
        }))
        .gap(16.0)
        .flex_wrap(argui_ui::FlexWrap::Wrap);
        Ok(Element::column([plot, legend])
            .keyed(&self.key)
            .gap(8.0)
            .semantics(Semantics::new(Role::Group).label(&self.label)))
    }
}

fn position(element: Element, x: f32, y: f32) -> Element {
    element.absolute(Sides {
        left: length(x),
        top: length(y),
        right: auto(),
        bottom: auto(),
    })
}
