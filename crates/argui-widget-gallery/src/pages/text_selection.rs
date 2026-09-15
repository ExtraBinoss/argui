use argui::{
    core::{Color, ColorInterpolation, Point},
    paint::{Border, CornerRadii, Fill, GradientStop, LinearGradient},
    runtime::{Context, Render},
    text::{TextStyle, TextWrap},
    ui::{
        Element, EventType, FlexWrap, Sides, TextSelectionHighlight, TextSelectionStyle,
        UiEventKind, UserSelect, length, percent,
    },
    widgets::{Button, WidgetTheme, default_theme},
};

use crate::app::text;

/// Stateful selection showcase used by the Widget Gallery.
pub(crate) struct SelectionDemo {
    phase: f32,
    rainbow_running: bool,
    reduced_motion: bool,
}

impl Default for SelectionDemo {
    fn default() -> Self {
        Self {
            phase: 0.0,
            rainbow_running: true,
            reduced_motion: false,
        }
    }
}

impl SelectionDemo {
    /// Toggles the animated rainbow when its control is activated.
    fn event(&mut self, event: &argui::ui::UiEvent, cx: &mut Context<Self>) {
        if event.target_key() == Some("selection-rainbow-toggle")
            && matches!(event.kind, UiEventKind::Click(_))
        {
            self.rainbow_running = !self.rainbow_running;
            cx.notify();
        }
    }

    /// Builds the four visual highlight variants for the current animation phase.
    fn highlights(&self, theme: &WidgetTheme) -> Element {
        let default = selection_card(
            "selection-default",
            "Default",
            "A familiar solid highlight with square fragments.",
            TextSelectionStyle::default(),
            TextSelectionHighlight::default(),
            theme,
        );
        let rounded = selection_card(
            "selection-rounded",
            "Rounded solid",
            "Every visual line fragment can use a soft pill radius.",
            TextSelectionStyle {
                background: theme.primary.with_alpha(0.34),
                handle: theme.primary,
            },
            TextSelectionHighlight::solid(theme.primary.with_alpha(0.34)).radius(7.0),
            theme,
        );
        let gradient = selection_card(
            "selection-gradient",
            "Oklab gradient",
            "A gradient fill follows each selected line without replacing the text.",
            TextSelectionStyle {
                background: theme.primary.with_alpha(0.30),
                handle: theme.primary,
            },
            TextSelectionHighlight::new(static_gradient(theme)).radii(CornerRadii {
                top_left: 3.0,
                top_right: 10.0,
                bottom_right: 3.0,
                bottom_left: 10.0,
            }),
            theme,
        );
        let rainbow = selection_card(
            "selection-rainbow",
            "Animated rainbow",
            "The same public fill API can move smoothly through the full spectrum.",
            TextSelectionStyle {
                background: theme.primary.with_alpha(0.30),
                handle: theme.primary,
            },
            TextSelectionHighlight::new(rainbow_gradient(self.phase)).radius(8.0),
            theme,
        );
        Element::row([default, rounded, gradient, rainbow])
            .width(percent(1.0))
            .gap(16.0)
            .flex_wrap(FlexWrap::Wrap)
    }

    /// Builds cards demonstrating every document selection policy.
    fn policies(&self, theme: &WidgetTheme) -> Element {
        Element::row([
            policy_card(
                "Text",
                "Drag through individual characters.",
                UserSelect::Text,
                theme,
            ),
            policy_card(
                "All",
                "Touching this card selects it as one unit.",
                UserSelect::All,
                theme,
            ),
            policy_card(
                "Contain",
                "A drag that starts here stays inside this card.",
                UserSelect::Contain,
                theme,
            ),
            policy_card(
                "None",
                "This text deliberately cannot be selected.",
                UserSelect::None,
                theme,
            ),
        ])
        .width(percent(1.0))
        .gap(12.0)
        .flex_wrap(FlexWrap::Wrap)
    }
}

impl Render for SelectionDemo {
    fn wants_animation_frame(&self) -> bool {
        self.rainbow_running && !self.reduced_motion
    }

    fn animation_frame(&mut self, frame: argui::animation::Frame, cx: &mut Context<Self>) {
        if !self.rainbow_running || self.reduced_motion {
            return;
        }
        let elapsed = frame.elapsed.as_secs_f64().min(0.05) as f32;
        if elapsed > 0.0 {
            self.phase = (self.phase + elapsed / 2.4).rem_euclid(1.0);
            cx.notify();
        }
        cx.request_animation_frame();
    }

    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        self.reduced_motion = cx.environment().reduced_motion;
        if self.rainbow_running && !self.reduced_motion {
            cx.request_animation_frame();
        }
        let themes = default_theme(cx.environment());
        let theme = themes.resolve(cx.environment().color_scheme);
        let rainbow_label = if self.reduced_motion {
            "Rainbow paused by reduced motion"
        } else if self.rainbow_running {
            "Pause rainbow"
        } else {
            "Play rainbow"
        };
        super::preview(
            "Selection playground",
            "Drag across every sample. Selection styles inherit through subtrees and render through the same GPU paint pipeline as other fills.",
            Element::column([
                Element::row([
                    text(
                        "Highlights",
                        15.0,
                        theme.foreground,
                        650,
                    ),
                    Button::new("selection-rainbow-toggle", rainbow_label, theme.outline_button())
                        .enabled(!self.reduced_motion)
                        .build(),
                ])
                .width(percent(1.0))
                .gap(12.0)
                .flex_wrap(FlexWrap::Wrap),
                self.highlights(theme),
                Element::column([
                    text("UserSelect policies", 15.0, theme.foreground, 650),
                    text(
                        "Policies compose with any visual style and inherit until a child overrides them.",
                        13.0,
                        theme.muted_foreground,
                        400,
                    ),
                    self.policies(theme),
                ])
                .gap(12.0),
            ])
            .gap(22.0),
            theme,
        )
        .on(cx.listener(EventType::Click, Self::event))
    }
}

/// Builds one selectable card with independent color and highlight configuration.
fn selection_card(
    key: &'static str,
    label: &'static str,
    copy: &'static str,
    style: TextSelectionStyle,
    highlight: TextSelectionHighlight,
    theme: &WidgetTheme,
) -> Element {
    Element::column([
        text(label, 14.0, theme.foreground, 700),
        Element::text(copy).text_style(body_style(theme)),
        text(
            "Select this sentence to reveal the highlight.",
            13.0,
            theme.muted_foreground,
            500,
        ),
    ])
    .keyed(key)
    .width(length(390.0))
    .min_width(length(0.0))
    .grow(1.0)
    .padding(Sides::length(16.0))
    .gap(8.0)
    .background(theme.card)
    .border(Border::all(1.0, theme.border))
    .radius(CornerRadii::all(12.0))
    .selection_style(style)
    .selection_highlight(highlight)
}

/// Builds one compact card governed by `policy`.
fn policy_card(
    label: &'static str,
    copy: &'static str,
    policy: UserSelect,
    theme: &WidgetTheme,
) -> Element {
    Element::column([
        text(label, 13.0, theme.foreground, 700),
        text(copy, 13.0, theme.muted_foreground, 400),
    ])
    .keyed(format!("selection-policy-{}", label.to_lowercase()))
    .width(length(195.0))
    .min_width(length(0.0))
    .grow(1.0)
    .padding(Sides::length(14.0))
    .gap(5.0)
    .background(theme.muted)
    .radius(CornerRadii::all(9.0))
    .user_select(policy)
    .selection_style(TextSelectionStyle {
        background: theme.primary.with_alpha(0.30),
        handle: theme.primary,
    })
    .selection_highlight(TextSelectionHighlight::solid(theme.primary.with_alpha(0.30)).radius(5.0))
}

/// Returns the readable paragraph style shared by highlight samples.
fn body_style(theme: &WidgetTheme) -> TextStyle {
    TextStyle {
        font_size: 16.0,
        line_height: 24.0,
        color: theme.foreground,
        wrap: TextWrap::Word,
        ..TextStyle::default()
    }
}

/// Creates the fixed Oklab selection gradient derived from the active theme.
fn static_gradient(theme: &WidgetTheme) -> Fill {
    Fill::Linear(
        LinearGradient::new(
            Point::new(0.0, 0.0),
            Point::new(1.0, 1.0),
            ColorInterpolation::Oklab,
            [
                GradientStop::new(0.0, theme.primary.with_alpha(0.42)),
                GradientStop::new(0.52, Color::from_srgb8(168, 85, 247).with_alpha(0.42)),
                GradientStop::new(1.0, Color::from_srgb8(20, 184, 166).with_alpha(0.42)),
            ],
        )
        .expect("the static selection gradient uses sorted normalized stops"),
    )
}

/// Creates one seamless rainbow fill at normalized animation `phase`.
fn rainbow_gradient(phase: f32) -> Fill {
    let stop = |offset| GradientStop::new(offset, rainbow_color(phase + offset).with_alpha(0.42));
    Fill::Linear(
        LinearGradient::new(
            Point::new(0.0, 0.5),
            Point::new(1.0, 0.5),
            ColorInterpolation::Oklab,
            [
                stop(0.0),
                stop(0.2),
                stop(0.4),
                stop(0.6),
                stop(0.8),
                stop(1.0),
            ],
        )
        .expect("the animated selection gradient uses sorted normalized stops"),
    )
}

/// Samples the cyclic rainbow palette at normalized `position`.
fn rainbow_color(position: f32) -> Color {
    let palette = [
        Color::from_srgb8(244, 63, 94),
        Color::from_srgb8(249, 115, 22),
        Color::from_srgb8(234, 179, 8),
        Color::from_srgb8(34, 197, 94),
        Color::from_srgb8(6, 182, 212),
        Color::from_srgb8(59, 130, 246),
        Color::from_srgb8(168, 85, 247),
    ];
    let cursor = position.rem_euclid(1.0) * palette.len() as f32;
    let index = cursor.floor() as usize % palette.len();
    palette[index].mix(
        palette[(index + 1) % palette.len()],
        cursor.fract(),
        ColorInterpolation::Oklab,
    )
}
