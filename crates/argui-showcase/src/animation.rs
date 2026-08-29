use argui_animation::{
    CubicBezier, Direction, Duration, Easing, FillMode, Iterations, Keyframe, Keyframes,
    PlaybackState, Timeline, Timing, Tween,
};
use argui_paint::{Border, Color, CornerRadii};
use argui_text::{TextColor, TextWrap};
use argui_ui::{Align, Edges, Element, Length, Wrap, property};

use crate::{StateShowcase, button, text_style};

impl StateShowcase {
    pub(super) fn animation_demo(&self, accent: Color) -> Element {
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
                    TextColor::WHITE,
                    650,
                    TextWrap::None,
                )),
                Element::container([])
                    .width(Length::Px(46.0))
                    .height(Length::Px(22.0))
                    .background(self.animated_color)
                    .bind(
                        property::BackgroundColor,
                        self.animated_color_motion.clone(),
                    )
                    .radius(CornerRadii::all(11.0)),
            ])
            .wrap(Wrap::Wrap)
            .gap(12.0)
            .align(Align::Center),
            Element::container([])
                .height(Length::Px(68.0))
                .width(Length::Percent(1.0))
                .background(self.animated_color)
                .bind(
                    property::BackgroundColor,
                    self.animated_color_motion.clone(),
                )
                .border(Border::all(1.5, accent))
                .radius(CornerRadii::all(18.0)),
            Element::row([
                button("animation-play", "Restart", accent),
                button("animation-pause", "Pause / resume", accent),
                button("animation-reverse", "Reverse", accent),
                button("animation-finish", "Finish", accent),
                button("animation-cancel", "Cancel", accent),
            ])
            .wrap(Wrap::Wrap)
            .gap(10.0),
        ])
        .gap(12.0)
        .padding(Edges::all(16.0))
        .background(Color::rgb(0.045, 0.06, 0.09))
        .radius(CornerRadii::all(14.0))
    }
}

pub(super) fn animation_timeline(initial: Color) -> Timeline<Color> {
    let ease =
        CubicBezier::new(0.22, 1.0, 0.36, 1.0).expect("the showcase uses a valid cubic Bezier");
    let frames = Keyframes::new(vec![
        Keyframe::new(0.0, initial).easing(Easing::CubicBezier(ease)),
        Keyframe::new(0.38, Color::rgb(0.62, 0.32, 0.96)).easing(Easing::CubicBezier(ease)),
        Keyframe::new(0.72, Color::rgb(0.98, 0.48, 0.18)).easing(Easing::CubicBezier(ease)),
        Keyframe::new(1.0, Color::rgb(0.24, 0.84, 0.55)),
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

pub(super) fn effect_phase_timeline() -> Timeline<f32> {
    Timeline::new(
        Keyframes::new(vec![Keyframe::new(0.0, 0.0), Keyframe::new(1.0, 1.0)])
            .expect("effect phase keyframes are complete"),
        Timing::new(Duration::from_secs(1))
            .iterations(Iterations::Infinite)
            .fill(FillMode::Both),
    )
    .expect("effect phase timing is valid")
}

pub(super) fn motion_tween() -> Tween {
    Tween::new(Duration::from_millis(420)).easing(Easing::CubicBezier(
        CubicBezier::new(0.22, 1.0, 0.36, 1.0).expect("the showcase uses a valid motion curve"),
    ))
}
