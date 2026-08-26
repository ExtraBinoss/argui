//! One showcase shared by native and WebAssembly launchers.

use argui_animation::{
    CubicBezier, Direction, Duration, Easing, FillMode, Frame, Iterations, Keyframe, Keyframes,
    PlaybackState, Timeline, Timing,
};
use argui_paint::{Border, ClipBehavior, Color, CornerRadii, PaintStyle, QuadStyle};
use argui_runtime::{UiApp, ViewUpdate};
use argui_text::{TextColor, TextEngine, TextStyle, TextWrap};
use argui_ui::{
    Align, Button, ButtonStyle, Edges, Element, Inset, Interaction, Length, ScrollConfig,
    ScrollPolarity, ScrollbarStyle, TextInput, TextInputStyle, UiEvent, UiEventKind, VirtualList,
    Wrap,
};

pub struct StateShowcase {
    count: u32,
    warm: bool,
    reversed: bool,
    inverted_scroll: bool,
    virtual_offset: f32,
    animated_color: Color,
    animation: Timeline<Color>,
    animation_command: Option<AnimationCommand>,
}

#[derive(Clone, Copy)]
enum AnimationCommand {
    Restart,
    PauseOrResume,
    Reverse,
    Finish,
    Cancel,
}

impl Default for StateShowcase {
    fn default() -> Self {
        let animated_color = Color::rgb(0.20, 0.68, 0.94);
        Self {
            count: 0,
            warm: false,
            reversed: false,
            inverted_scroll: false,
            virtual_offset: 0.0,
            animated_color,
            animation: animation_timeline(animated_color),
            animation_command: None,
        }
    }
}

impl UiApp for StateShowcase {
    fn view(&self) -> Element {
        let accent = if self.warm {
            Color::rgb(0.96, 0.52, 0.26)
        } else {
            Color::rgb(0.20, 0.68, 0.94)
        };
        let items = if self.reversed {
            vec![chip("beta", "Stable beta"), chip("alpha", "Stable alpha")]
        } else {
            vec![chip("alpha", "Stable alpha"), chip("beta", "Stable beta")]
        };
        Element::column([Element::column([
            Element::text(format!("State updates: {}", self.count)).text_style(text_style(
                38.0,
                TextColor::WHITE,
                700,
                TextWrap::Word,
            )),
            Element::text(
                "Clicks mutate plain Rust state. The rebuilt tree decides whether it needs no work, a repaint, or a new layout.",
            )
            .text_style(text_style(
                19.0,
                TextColor::rgb(0.78, 0.84, 0.92),
                400,
                TextWrap::Word,
            )),
            text_input(
                "message",
                "Hello · مرحباً · שלום · 👋🏽",
                "Type in any language…",
                accent,
            ),
            text_input(
                "long-message",
                "This deliberately long editable line proves that the caret remains visible while the text scrolls horizontally.",
                "Long single-line input",
                accent,
            ),
            Element::row([
                button("increment", "Increment", accent),
                button("theme", "Toggle paint", accent),
                button("reorder", "Reorder keys", accent),
                button(
                    "polarity",
                    if self.inverted_scroll {
                        "Scroll: inverted"
                    } else {
                        "Scroll: normal"
                    },
                    accent,
                ),
            ])
            .wrap(Wrap::Wrap)
            .gap(12.0)
            .align(Align::Center),
            Element::row(items).wrap(Wrap::Wrap).gap(10.0),
            self.animation_demo(accent),
            self.virtual_list(accent),
            Element::text("OVERLAY · z-index 100")
                .text_style(text_style(
                    13.0,
                    TextColor::rgb(0.05, 0.08, 0.12),
                    700,
                    TextWrap::None,
                ))
                .padding(Edges::symmetric(11.0, 7.0))
                .background(accent)
                .radius(CornerRadii::all(9.0))
                .absolute(Inset::top_right(14.0, 14.0))
                .z_index(100),
        ])
        .padding(Edges::all(30.0))
        .gap(22.0)
        .background(if self.warm {
            Color::rgb(0.16, 0.09, 0.07)
        } else {
            Color::rgb(0.06, 0.08, 0.12)
        })
        .border(Border::all(1.5, accent))
        .radius(CornerRadii::all(22.0))
        .clip(ClipBehavior::Bounds)
        .shrink(0.0)])
        .keyed("page-scroll")
        .width(Length::Percent(1.0))
        .height(Length::Percent(1.0))
        .scrollable(ScrollConfig::default().scrollbar(self.scrollbar_style(accent)))
        .padding(Edges::symmetric(20.0, 24.0))
    }

    fn update(&mut self, event: &UiEvent) -> ViewUpdate {
        if let UiEventKind::Scrolled { offset, .. } = event.kind
            && event.key.as_deref() == Some("million-list")
        {
            let old = self.virtual_list_config().window(self.virtual_offset);
            self.virtual_offset = offset.y;
            let new = self.virtual_list_config().window(self.virtual_offset);
            return if old.range == new.range {
                ViewUpdate::None
            } else {
                ViewUpdate::Rebuild
            };
        }
        if event.kind != UiEventKind::Clicked {
            return ViewUpdate::None;
        }
        match event.key.as_deref() {
            Some("increment") => self.count += 1,
            Some("theme") => self.warm = !self.warm,
            Some("reorder") => self.reversed = !self.reversed,
            Some("polarity") => self.inverted_scroll = !self.inverted_scroll,
            Some("animation-play") => {
                self.animation_command = Some(AnimationCommand::Restart);
                return ViewUpdate::None;
            }
            Some("animation-pause") => {
                self.animation_command = Some(AnimationCommand::PauseOrResume);
                return ViewUpdate::None;
            }
            Some("animation-reverse") => {
                self.animation_command = Some(AnimationCommand::Reverse);
                return ViewUpdate::None;
            }
            Some("animation-finish") => {
                self.animation_command = Some(AnimationCommand::Finish);
                return ViewUpdate::None;
            }
            Some("animation-cancel") => {
                self.animation_command = Some(AnimationCommand::Cancel);
                return ViewUpdate::None;
            }
            _ => return ViewUpdate::None,
        }
        ViewUpdate::Rebuild
    }

    fn animation_frame(&mut self, frame: Frame) -> ViewUpdate {
        if let Some(command) = self.animation_command.take() {
            match command {
                AnimationCommand::Restart => self.animation.restart(frame.now),
                AnimationCommand::PauseOrResume
                    if self.animation.state() == PlaybackState::Paused =>
                {
                    self.animation.resume(frame.now);
                }
                AnimationCommand::PauseOrResume => self.animation.pause(frame.now),
                AnimationCommand::Reverse => self.animation.reverse(frame.now),
                AnimationCommand::Finish => self.animation.finish(),
                AnimationCommand::Cancel => self.animation.cancel(),
            }
        }
        let sample = self.animation.sample(frame.now);
        let color = sample.value.unwrap_or_else(|| self.accent());
        let changed = color != self.animated_color || sample.events != Default::default();
        self.animated_color = color;
        if changed {
            ViewUpdate::Rebuild
        } else {
            ViewUpdate::None
        }
    }

    fn wants_animation_frame(&self) -> bool {
        self.animation_command.is_some() || self.animation.needs_frame()
    }
}

impl StateShowcase {
    fn animation_demo(&self, accent: Color) -> Element {
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
                    .radius(CornerRadii::all(11.0)),
            ])
            .wrap(Wrap::Wrap)
            .gap(12.0)
            .align(Align::Center),
            Element::container([])
                .height(Length::Px(68.0))
                .width(Length::Percent(1.0))
                .background(self.animated_color)
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

    fn virtual_list_config(&self) -> VirtualList {
        let polarity = if self.inverted_scroll {
            ScrollPolarity::Inverted
        } else {
            ScrollPolarity::Normal
        };
        VirtualList::new(1_000_000, 36.0, 260.0)
            .overscan(3)
            .scroll_config(
                ScrollConfig::default()
                    .polarity(polarity)
                    .line_size(36.0)
                    .scrollbar(self.scrollbar_style(self.accent())),
            )
    }

    fn accent(&self) -> Color {
        if self.warm {
            Color::rgb(0.96, 0.52, 0.26)
        } else {
            Color::rgb(0.20, 0.68, 0.94)
        }
    }

    fn scrollbar_style(&self, accent: Color) -> ScrollbarStyle {
        ScrollbarStyle::new(
            QuadStyle::solid(Color::rgba(0.12, 0.16, 0.22, 0.72)).radius(CornerRadii::all(5.0)),
            QuadStyle::solid(accent).radius(CornerRadii::all(5.0)),
        )
        .width(10.0)
        .inset(5.0)
        .min_thumb(30.0)
    }

    fn virtual_list(&self, accent: Color) -> Element {
        self.virtual_list_config()
            .build("million-list", self.virtual_offset, |index| {
                let background = if index % 2 == 0 {
                    Color::rgb(0.075, 0.10, 0.15)
                } else {
                    Color::rgb(0.06, 0.08, 0.12)
                };
                Element::text(format!("Row #{index:07} / 1,000,000"))
                    .keyed(format!("row-{index}"))
                    .text_style(text_style(
                        15.0,
                        TextColor::rgb(0.78, 0.84, 0.92),
                        500,
                        TextWrap::None,
                    ))
                    .padding(Edges::symmetric(12.0, 8.0))
                    .background(background)
                    .interaction(
                        Interaction::default().hovered(
                            QuadStyle::solid(Color::rgb(0.12, 0.18, 0.25))
                                .border(Border::all(1.0, accent)),
                        ),
                    )
            })
            .background(Color::rgb(0.035, 0.045, 0.065))
            .border(Border::all(1.0, Color::rgb(0.18, 0.24, 0.32)))
            .radius(CornerRadii::all(12.0))
    }
}

fn animation_timeline(initial: Color) -> Timeline<Color> {
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

#[must_use]
pub fn text_engine() -> TextEngine {
    TextEngine::from_embedded_fonts(
        [
            include_bytes!("../../argui-web-demo/assets/fonts/NotoSans-Regular.ttf").as_slice(),
            include_bytes!("../../argui-web-demo/assets/fonts/NotoSansArabic.ttf").as_slice(),
            include_bytes!("../../argui-web-demo/assets/fonts/NotoSansHebrew.ttf").as_slice(),
            include_bytes!("../../argui-web-demo/assets/fonts/NotoEmoji-Regular.ttf").as_slice(),
            include_bytes!("../../argui-web-demo/assets/fonts/FiraMono-Medium.ttf").as_slice(),
        ],
        "Noto Sans",
        "Noto Sans",
        "Fira Mono",
    )
}

fn text_style(size: f32, color: TextColor, weight: u16, wrap: TextWrap) -> TextStyle {
    TextStyle {
        font_size: size,
        line_height: size * 1.25,
        color,
        weight,
        wrap,
        ..TextStyle::default()
    }
}

fn button(key: &str, label: &str, accent: Color) -> Element {
    let radius = CornerRadii::all(12.0);
    let rest = QuadStyle::solid(Color::rgb(0.10, 0.14, 0.20))
        .border(Border::all(1.0, Color::rgb(0.28, 0.36, 0.48)))
        .radius(radius);
    let active = QuadStyle::solid(Color::rgb(0.14, 0.22, 0.30))
        .border(Border::all(1.0, accent))
        .radius(radius);
    Button::new(
        key,
        label,
        ButtonStyle::new(
            PaintStyle::new(rest),
            text_style(17.0, TextColor::WHITE, 600, TextWrap::None),
        )
        .hovered(active)
        .pressed(active.opacity(0.72))
        .focused(rest.border(Border::all(2.0, accent))),
    )
    .build()
}

fn text_input(key: &str, value: &str, placeholder: &str, accent: Color) -> Element {
    let [red, green, blue, _] = accent.as_array();
    let radius = CornerRadii::all(11.0);
    let rest = QuadStyle::solid(Color::rgb(0.035, 0.05, 0.075))
        .border(Border::all(1.0, Color::rgb(0.20, 0.27, 0.36)))
        .radius(radius);
    let active = rest.border(Border::all(1.5, accent));
    TextInput::new(
        key,
        value,
        placeholder,
        TextInputStyle::new(
            PaintStyle::new(rest).clip(ClipBehavior::Bounds),
            text_style(17.0, TextColor::WHITE, 400, TextWrap::None),
        )
        .hovered(active)
        .focused(active)
        .selection(Color::rgba(red, green, blue, 0.38))
        .caret(accent),
    )
    .build()
}

fn chip(key: &str, label: &str) -> Element {
    Element::text(label)
        .keyed(key)
        .text_style(text_style(
            15.0,
            TextColor::rgb(0.66, 0.88, 0.72),
            600,
            TextWrap::None,
        ))
        .padding(Edges::symmetric(12.0, 7.0))
        .shrink(0.0)
        .background(Color::rgb(0.08, 0.20, 0.14))
        .radius(CornerRadii::all(9.0))
}
