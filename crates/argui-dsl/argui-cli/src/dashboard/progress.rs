//! True-color progress rendering for observable live-update phases.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
};

/// A bounded, source-stage progress bar with a gradient across filled cells.
pub(super) struct ProgressBar {
    pub ratio: f64,
    pub color: Color,
}

impl ProgressBar {
    /// Renders the stage, colored bar, and current status within `area`.
    ///
    /// The dashboard supplies an area at least 28 columns wide and four rows high.
    pub(super) fn render(
        self,
        frame: &mut Frame<'_>,
        area: Rect,
        phase: &str,
        generation: &str,
        status: &str,
    ) {
        frame.render_widget(
            Block::default()
                .title(" Live update ")
                .borders(Borders::ALL),
            area,
        );
        let inner = Rect::new(
            area.x.saturating_add(1),
            area.y.saturating_add(1),
            area.width.saturating_sub(2),
            area.height.saturating_sub(2),
        );
        let label = format!("{phase} · gen {generation} · {status}");
        frame.render_widget(
            Paragraph::new(label)
                .style(Style::default().fg(self.color))
                .wrap(Wrap { trim: true }),
            Rect::new(inner.x, inner.y, inner.width, 1),
        );
        let filled = (f64::from(inner.width) * self.ratio.clamp(0.0, 1.0)).round() as u16;
        for index in 0..inner.width {
            let cell = frame
                .buffer_mut()
                .cell_mut((inner.x + index, inner.y + 1))
                .expect("progress bar cell is inside its frame");
            if index < filled {
                cell.set_symbol("━")
                    .set_fg(gradient_color(index, inner.width, self.color));
            } else {
                cell.set_symbol("─").set_fg(Color::DarkGray);
            }
        }
    }
}

/// Blends the cool start accent into the stage's semantic destination color.
fn gradient_color(index: u16, width: u16, destination: Color) -> Color {
    let (end_red, end_green, end_blue) = match destination {
        Color::Rgb(red, green, blue) => (red, green, blue),
        Color::Red => (239, 68, 68),
        Color::Green => (34, 197, 94),
        Color::Cyan => (6, 182, 212),
        _ => (156, 163, 175),
    };
    let fraction = f64::from(index) / f64::from(width.max(1));
    let mix = |start: u8, end: u8| {
        (f64::from(start) * (1.0 - fraction) + f64::from(end) * fraction).round() as u8
    };
    Color::Rgb(mix(45, end_red), mix(212, end_green), mix(191, end_blue))
}
