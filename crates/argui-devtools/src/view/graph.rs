use super::{millis, text};
use argui_core::{Transform2D, TransformOrigin};
use argui_inspect::FrameRecord;
use argui_ui::{Element, Role, Semantics, Sides, StyleTransition, auto, length, percent};
use argui_widgets::WidgetTheme;

pub(super) fn graph(frames: &[FrameRecord], theme: &WidgetTheme) -> Element {
    // A fixed scale prevents the entire plot jumping when one slow frame arrives.
    const SCALE: f64 = 50.0;
    const HEIGHT: f32 = 90.0;
    let recent = &frames[frames.len().saturating_sub(60)..];
    let missing = 60 - recent.len();
    let bars = (0_usize..60).map(|slot| {
        let duration = slot
            .checked_sub(missing)
            .and_then(|index| recent.get(index))
            .map_or(0.0, |frame| {
                millis(frame.interval).max(millis(frame.total_cpu()))
            });
        Element::container([])
            .keyed(format!("__devtools-graph-bar-{slot}"))
            .grow(1.0)
            .min_width(length(0.0))
            .height(length(HEIGHT))
            .transform_origin(TransformOrigin::new(0.5, 1.0))
            .transform(Transform2D::IDENTITY.scale(1.0, (duration / SCALE).clamp(0.0, 1.0) as f32))
            .transition(StyleTransition::default())
            .background(if duration > 16.67 {
                theme.destructive
            } else {
                theme.primary
            })
    });
    let mut plot = Element::row(bars)
        .width(percent(1.0))
        .height(length(HEIGHT + 16.0))
        .shrink(0.0)
        .padding(Sides::length(8.0))
        .gap(2.0)
        .background(theme.card)
        .keyed("__devtools-profile-plot");
    for budget in [16.67, 33.33] {
        plot.children.push(
            Element::container([])
                .absolute(Sides {
                    left: length(8.0),
                    right: length(8.0),
                    top: auto(),
                    bottom: length(8.0 + (budget / SCALE) as f32 * HEIGHT),
                })
                .height(length(1.0))
                .background(theme.border)
                .semantic_hidden(true),
        );
    }
    Element::column([
        Element::text("Frame time · last 60 frames").text_style(text(12.0, theme.foreground)),
        plot,
        Element::text(if recent.is_empty() { "No recorded frames yet" }
            else { "0–50 ms · guides 16.7 / 33.3 ms · red: over budget" })
            .text_style(text(10.0, theme.muted_foreground)),
    ]).keyed("__devtools-profile-graph").gap(6.0).shrink(0.0)
        .semantics(Semantics::new(Role::Image).label("Frame timing graph")
            .description("Last 60 frames. Visual transitions are interpolated; recorded timings are unchanged. Values above 50 milliseconds are clipped."))
}
