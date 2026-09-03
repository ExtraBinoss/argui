use argui_animation::{
    CubicBezier, Direction, Duration, Easing, FillMode, Iterations, Keyframe, Keyframes,
    PlaybackState, Timeline, Timing, Tween,
};
use argui_paint::{Border, Color, CornerRadii};
use argui_text::TextWrap;
use argui_ui::{AlignItems, Element, FlexWrap, Sides, length, percent, property};
use argui_widgets::WidgetTheme;

use crate::{StateShowcase, button, text_style};

impl StateShowcase {
    pub(super) fn animation_demo(&self, widgets: &WidgetTheme) -> Element {
        let status = match self.animation.state() {
            PlaybackState::Idle => "idle",
            PlaybackState::Running => "running",
            PlaybackState::Paused => "paused",
            PlaybackState::Finished => "finished",
            PlaybackState::Canceled => "canceled",
        };
        Element::column([
            Element::row([
                Element::text(format!("Typed keyframes · {status}")).text_style(text_style(
                    17.0,
                    widgets.foreground,
                    650,
                    TextWrap::None,
                )),
                Element::container([])
                    .width(length(46.0))
                    .height(length(22.0))
                    .background(self.animated_color)
                    .bind(
                        property::BackgroundColor,
                        self.animated_color_motion.clone(),
                    )
                    .radius(CornerRadii::all(11.0)),
            ])
            .flex_wrap(FlexWrap::Wrap)
            .gap(12.0)
            .align_items(AlignItems::CENTER),
            Element::container([])
                .height(length(68.0))
                .width(percent(1.0))
                .background(self.animated_color)
                .bind(
                    property::BackgroundColor,
                    self.animated_color_motion.clone(),
                )
                .border(Border::all(1.0, widgets.border))
                .radius(CornerRadii::all(18.0)),
            Element::row([
                button("animation-play", "Restart", widgets, false),
                button("animation-pause", "Pause / resume", widgets, false),
                button("animation-reverse", "Reverse", widgets, false),
                button("animation-finish", "Finish", widgets, false),
                button("animation-cancel", "Cancel", widgets, false),
            ])
            .flex_wrap(FlexWrap::Wrap)
            .gap(10.0),
        ])
        .gap(12.0)
        .padding(Sides::length(16.0))
        .background(widgets.card)
        .border(Border::all(1.0, widgets.border))
        .radius(CornerRadii::all(10.0))
    }
}

pub(super) fn animation_timeline(initial: Color) -> Timeline<Color> {
    let ease =
        CubicBezier::new(0.22, 1.0, 0.36, 1.0).expect("the showcase uses a valid cubic Bezier");
    let frames = Keyframes::new(vec![
        Keyframe::new(0.0, initial).easing(Easing::CubicBezier(ease)),
        Keyframe::new(0.38, Color::srgb(0.62, 0.32, 0.96)).easing(Easing::CubicBezier(ease)),
        Keyframe::new(0.72, Color::srgb(0.98, 0.48, 0.18)).easing(Easing::CubicBezier(ease)),
        Keyframe::new(1.0, Color::srgb(0.24, 0.84, 0.55)),
    ])
    .expect("the showcase keyframes are sorted and complete");
    Timeline::new(
        frames,
        Timing::new(Duration::from_millis(1_400))
            .iterations(Iterations::Finite(2.0))
            .direction(Direction::Alternate)
            .fill(FillMode::Forwards),
    )
    .expect("the showcase timing is valid")
}

pub(super) fn motion_tween() -> Tween {
    Tween::new(Duration::from_millis(420)).easing(Easing::CubicBezier(
        CubicBezier::new(0.22, 1.0, 0.36, 1.0).expect("the showcase uses a valid motion curve"),
    ))
}
