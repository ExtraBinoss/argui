use argui::{
    accessibility::{Role, SemanticAction, SemanticValue, Semantics},
    core::{Color, Key, KeyState, Point, Rect, Size},
    paint::{Border, CornerRadii, QuadStyle},
    runtime::{Context, Render},
    text::{TextStyle, TextWrap},
    ui::{
        Axes, CursorIcon, CustomElement, CustomLayoutContext, CustomMeasurement,
        CustomPaintContext, Element, EventType, FocusPolicy, GestureCapture, GestureKind,
        GesturePhase, GestureSet, HitTestStyle, Interaction, Overflow, PanGesture, Sides,
        UiEventKind, UserSelect, length, percent, sides,
    },
    widgets::{Button, default_theme},
};

const DURATION: f32 = 20.0;
const PIXELS_PER_SECOND: f32 = 32.0;
const GUTTER: f32 = 76.0;
const RULER_HEIGHT: f32 = 32.0;
const TRACK_HEIGHT: f32 = 58.0;
const TIMELINE_WIDTH: f32 = GUTTER + DURATION * PIXELS_PER_SECOND + 48.0;
const TIME_LABELS: usize = 5;
const TRACK_LABELS: usize = 3;

#[derive(Clone, Copy, Debug)]
struct Clip {
    label: &'static str,
    start: f32,
    duration: f32,
    track: usize,
}

const CLIPS: [Clip; 5] = [
    Clip {
        label: "Intro",
        start: 0.0,
        duration: 5.0,
        track: 0,
    },
    Clip {
        label: "Interview",
        start: 5.0,
        duration: 9.0,
        track: 0,
    },
    Clip {
        label: "B-roll",
        start: 14.0,
        duration: 6.0,
        track: 0,
    },
    Clip {
        label: "Voiceover",
        start: 2.5,
        duration: 11.0,
        track: 1,
    },
    Clip {
        label: "Music bed",
        start: 0.0,
        duration: 20.0,
        track: 2,
    },
];

#[derive(Debug)]
struct EditorTimeline {
    background: Color,
    track: Color,
    grid: Color,
    playhead: Color,
    seconds: f32,
    clips: [Clip; 5],
}

impl CustomElement for EditorTimeline {
    type State = Vec<f32>;

    fn create_state(&self) -> Self::State {
        Vec::new()
    }

    fn layout_revision(&self) -> u64 {
        self.clips
            .iter()
            .fold(u64::from(self.seconds.to_bits()), |revision, clip| {
                revision
                    .wrapping_mul(31)
                    .wrapping_add(u64::from(clip.start.to_bits()))
                    .wrapping_mul(31)
                    .wrapping_add(clip.track as u64)
            })
    }

    fn paint_revision(&self) -> u64 {
        [self.background, self.track, self.grid, self.playhead]
            .into_iter()
            .fold(u64::from(self.seconds.to_bits()), |revision, color| {
                revision
                    .wrapping_mul(31)
                    .wrapping_add(u64::from(u32::from_le_bytes(color.to_srgba8())))
            })
    }

    fn layout(
        &self,
        _: &mut Self::State,
        cx: &mut dyn CustomLayoutContext,
    ) -> Result<CustomMeasurement, String> {
        for index in 0..TIME_LABELS {
            cx.place_child(
                index,
                Rect::new(
                    Point::new(GUTTER + index as f32 * 5.0 * PIXELS_PER_SECOND + 4.0, 7.0),
                    Size::new(42.0, 20.0),
                ),
            )?;
        }
        for index in 0..TRACK_LABELS {
            cx.place_child(
                TIME_LABELS + index,
                Rect::new(
                    Point::new(10.0, RULER_HEIGHT + index as f32 * TRACK_HEIGHT + 18.0),
                    Size::new(56.0, 20.0),
                ),
            )?;
        }
        for (index, clip) in self.clips.into_iter().enumerate() {
            cx.place_child(
                TIME_LABELS + TRACK_LABELS + index,
                Rect::new(
                    Point::new(
                        GUTTER + clip.start * PIXELS_PER_SECOND,
                        RULER_HEIGHT + clip.track as f32 * TRACK_HEIGHT + 6.0,
                    ),
                    Size::new(clip.duration * PIXELS_PER_SECOND, TRACK_HEIGHT - 12.0),
                ),
            )?;
        }
        cx.place_child(
            TIME_LABELS + TRACK_LABELS + CLIPS.len(),
            Rect::new(
                Point::new(GUTTER + self.seconds * PIXELS_PER_SECOND - 12.0, 0.0),
                Size::new(24.0, RULER_HEIGHT + TRACK_LABELS as f32 * TRACK_HEIGHT),
            ),
        )?;
        Ok(CustomMeasurement {
            size: Size::new(
                TIMELINE_WIDTH,
                RULER_HEIGHT + TRACK_LABELS as f32 * TRACK_HEIGHT,
            ),
            baseline: None,
        })
    }

    fn prepare(&self, ticks: &mut Self::State, _: Size) {
        ticks.clear();
        ticks.extend(
            (0..=DURATION as usize).map(|second| GUTTER + second as f32 * PIXELS_PER_SECOND),
        );
    }

    fn paint(&self, ticks: &mut Self::State, cx: &mut CustomPaintContext<'_>) {
        cx.quad(
            Rect::new(Point::default(), cx.bounds.size),
            QuadStyle::solid(self.background),
        );
        for track in 0..TRACK_LABELS {
            cx.quad(
                Rect::new(
                    Point::new(0.0, RULER_HEIGHT + track as f32 * TRACK_HEIGHT + 2.0),
                    Size::new(cx.bounds.size.width, TRACK_HEIGHT - 4.0),
                ),
                QuadStyle::solid(self.track).radius(CornerRadii::all(6.0)),
            );
        }
        for (second, x) in ticks.iter().copied().enumerate() {
            cx.quad(
                Rect::new(
                    Point::new(
                        x,
                        if second.is_multiple_of(5) {
                            24.0
                        } else {
                            RULER_HEIGHT
                        },
                    ),
                    Size::new(
                        if second.is_multiple_of(5) { 1.5 } else { 1.0 },
                        cx.bounds.size.height - RULER_HEIGHT,
                    ),
                ),
                QuadStyle::solid(self.grid),
            );
        }
        let playhead_x = GUTTER + self.seconds * PIXELS_PER_SECOND;
        cx.quad(
            Rect::new(
                Point::new(playhead_x - 1.0, 10.0),
                Size::new(2.0, cx.bounds.size.height - 10.0),
            ),
            QuadStyle::solid(self.playhead),
        );
        cx.quad(
            Rect::new(Point::new(playhead_x - 6.0, 4.0), Size::new(12.0, 12.0)),
            QuadStyle::solid(self.playhead).radius(CornerRadii::all(3.0)),
        );
    }
}

pub struct Example {
    clips: [Clip; 5],
    selected_clip: usize,
    clip_drag_start: Clip,
    playhead: f32,
    playhead_drag_start: f32,
}

impl Default for Example {
    fn default() -> Self {
        Self {
            clips: CLIPS,
            selected_clip: 0,
            clip_drag_start: CLIPS[0],
            playhead: 7.5,
            playhead_drag_start: 7.5,
        }
    }
}

impl Render for Example {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = default_theme(cx.environment());
        let theme = themes.resolve(cx.environment().color_scheme);
        let text = |value: String, size: f32, weight: u16, color: Color| {
            Element::text(value).text_style(TextStyle {
                color,
                font_size: size,
                weight,
                ..TextStyle::default()
            })
        };
        let mut children: Vec<Element> = (0..TIME_LABELS)
            .map(|index| {
                text(
                    format!("0:{:02}", index * 5),
                    11.0,
                    600,
                    theme.muted_foreground,
                )
            })
            .collect();
        children.extend(
            ["VIDEO 1", "VIDEO 2", "AUDIO"]
                .into_iter()
                .map(|label| text(label.to_owned(), 11.0, 700, theme.muted_foreground)),
        );
        let clip_colors = [
            Color::from_srgb8(37, 99, 235),
            Color::from_srgb8(124, 58, 237),
            Color::from_srgb8(8, 145, 178),
            Color::from_srgb8(219, 39, 119),
            Color::from_srgb8(5, 150, 105),
        ];
        children.extend(self.clips.into_iter().zip(clip_colors).enumerate().map(
            |(index, (clip, color))| {
                let selected = self.selected_clip == index;
                Element::column([
                    Element::text(clip.label).text_style(TextStyle {
                        color: Color::WHITE,
                        font_size: 13.0,
                        line_height: 16.0,
                        weight: 700,
                        wrap: TextWrap::None,
                        ..TextStyle::default()
                    }),
                    Element::text(format!("{:.1}s", clip.duration)).text_style(TextStyle {
                        color: Color::WHITE.with_alpha(0.82),
                        font_size: 10.0,
                        line_height: 12.0,
                        weight: 500,
                        wrap: TextWrap::None,
                        ..TextStyle::default()
                    }),
                ])
                .keyed(format!("editor-clip-{index}"))
                .gap(2.0)
                .padding(sides(8.0, 4.0))
                .background(color)
                .border(Border::all(
                    if selected { 2.0 } else { 1.0 },
                    if selected {
                        theme.ring
                    } else {
                        Color::WHITE.with_alpha(0.25)
                    },
                ))
                .radius(CornerRadii::all(6.0))
                .overflow(Axes {
                    x: Overflow::Hidden,
                    y: Overflow::Hidden,
                })
                .z_index(i32::from(selected))
                .interaction(
                    Interaction::default()
                        .focus_policy(FocusPolicy::TabStop)
                        .cursor(CursorIcon::Grab)
                        .gestures(
                            GestureSet::EMPTY.pan(
                                PanGesture::default()
                                    .immediate()
                                    .capture(GestureCapture::OnPress),
                            ),
                        ),
                )
                .semantics(
                    Semantics::new(Role::Button)
                        .label(format!(
                            "{} clip at {:.1}s on track {}",
                            clip.label,
                            clip.start,
                            clip.track + 1
                        ))
                        .description(
                            "Drag horizontally or between tracks; arrow keys move the clip",
                        )
                        .action(SemanticAction::Focus),
                )
                .user_select(UserSelect::None)
                .on(cx.listener(EventType::Gesture, move |app, event, cx| {
                    let UiEventKind::Gesture(gesture) = event.kind else {
                        return;
                    };
                    let GestureKind::Pan { total, .. } = gesture.kind else {
                        return;
                    };
                    if gesture.phase == GesturePhase::Started {
                        app.selected_clip = index;
                        app.clip_drag_start = app.clips[index];
                    }
                    app.clips[index] = if gesture.phase == GesturePhase::Cancelled {
                        app.clip_drag_start
                    } else {
                        let mut next = app.clip_drag_start;
                        next.start = (next.start + total.x / PIXELS_PER_SECOND)
                            .clamp(0.0, DURATION - next.duration);
                        next.track = (next.track as f32 + total.y / TRACK_HEIGHT)
                            .round()
                            .clamp(0.0, (TRACK_LABELS - 1) as f32)
                            as usize;
                        next
                    };
                    event.stop_propagation();
                    cx.notify();
                }))
                .on(cx.listener(EventType::Key, move |app, event, cx| {
                    let UiEventKind::KeyInput(input) = &event.kind else {
                        return;
                    };
                    if input.state != KeyState::Pressed {
                        return;
                    }
                    let clip = &mut app.clips[index];
                    match input.key {
                        Key::ArrowLeft => clip.start = (clip.start - 0.5).max(0.0),
                        Key::ArrowRight => {
                            clip.start = (clip.start + 0.5).min(DURATION - clip.duration)
                        }
                        Key::ArrowUp => clip.track = clip.track.saturating_sub(1),
                        Key::ArrowDown => clip.track = (clip.track + 1).min(TRACK_LABELS - 1),
                        _ => return,
                    }
                    app.selected_clip = index;
                    event.stop_propagation();
                    cx.notify();
                }))
            },
        ));
        let playhead = Element::custom_region(
            "editor-playhead",
            Interaction::default()
                .focus_policy(FocusPolicy::TabStop)
                .cursor(CursorIcon::EwResize)
                .gestures(
                    GestureSet::EMPTY.pan(
                        PanGesture::default()
                            .immediate()
                            .capture(GestureCapture::OnPress),
                    ),
                ),
            Semantics::new(Role::Slider)
                .label("Timeline playhead")
                .description("Drag horizontally or use Left and Right to scrub")
                .value(SemanticValue::Number {
                    value: f64::from(self.playhead),
                    minimum: Some(0.0),
                    maximum: Some(f64::from(DURATION)),
                    step: Some(0.5),
                })
                .action(SemanticAction::Focus)
                .action(SemanticAction::Increment)
                .action(SemanticAction::Decrement)
                .action(SemanticAction::SetValue),
        )
        .hit_test(HitTestStyle::default().slop(Sides::length(8.0)))
        .user_select(UserSelect::None)
        .on(cx.listener(EventType::Gesture, |app, event, cx| {
            let UiEventKind::Gesture(gesture) = event.kind else {
                return;
            };
            let GestureKind::Pan { total, .. } = gesture.kind else {
                return;
            };
            if gesture.phase == GesturePhase::Started {
                app.playhead_drag_start = app.playhead;
            }
            app.playhead = if gesture.phase == GesturePhase::Cancelled {
                app.playhead_drag_start
            } else {
                (app.playhead_drag_start + total.x / PIXELS_PER_SECOND).clamp(0.0, DURATION)
            };
            event.stop_propagation();
            cx.notify();
        }))
        .on(cx.listener(EventType::Key, |app, event, cx| {
            let UiEventKind::KeyInput(input) = &event.kind else {
                return;
            };
            if input.state != KeyState::Pressed {
                return;
            }
            let delta = match input.key {
                Key::ArrowLeft => -0.5,
                Key::ArrowRight => 0.5,
                _ => return,
            };
            app.playhead = (app.playhead + delta).clamp(0.0, DURATION);
            event.stop_propagation();
            cx.notify();
        }))
        .on(cx.listener(EventType::SemanticAction, |app, event, cx| {
            let UiEventKind::SemanticAction { action, value } = &event.kind else {
                return;
            };
            app.playhead = match (action, value) {
                (SemanticAction::Increment, _) => app.playhead + 0.5,
                (SemanticAction::Decrement, _) => app.playhead - 0.5,
                (SemanticAction::SetValue, Some(SemanticValue::Number { value, .. })) => {
                    *value as f32
                }
                _ => return,
            }
            .clamp(0.0, DURATION);
            event.stop_propagation();
            cx.notify();
        }));
        children.push(playhead);

        let back = Button::new("playhead-back", "Back 1s", theme.outline_button())
            .on_click(cx.callback(|app| app.playhead = (app.playhead - 1.0).max(0.0)))
            .build();
        let forward = Button::new("playhead-forward", "Forward 1s", theme.outline_button())
            .on_click(cx.callback(|app| {
                app.playhead = (app.playhead + 1.0).min(DURATION);
            }))
            .build();
        let timeline = Element::custom_container(
            EditorTimeline {
                background: theme.card,
                track: theme.muted,
                grid: theme.border,
                playhead: theme.destructive,
                seconds: self.playhead,
                clips: self.clips,
            },
            children,
        )
        .keyed("video-editor-timeline")
        .width(percent(1.0))
        .height(length(RULER_HEIGHT + TRACK_LABELS as f32 * TRACK_HEIGHT));

        Element::column([
            Element::row([
                Element::column([
                    text(
                        "Product launch edit".to_owned(),
                        22.0,
                        750,
                        theme.foreground,
                    ),
                    text(
                        "Drag clips to retime them or move tracks; drag the red playhead to scrub."
                            .to_owned(),
                        13.0,
                        400,
                        theme.muted_foreground,
                    ),
                ])
                .gap(3.0)
                .grow(1.0),
                text(
                    format!("00:{:04.1} / 00:20.0", self.playhead),
                    13.0,
                    700,
                    theme.foreground,
                ),
            ]),
            Element::row([back, forward]).gap(8.0),
            text(
                format!(
                    "Selected: {} · {:.1}s · track {}",
                    self.clips[self.selected_clip].label,
                    self.clips[self.selected_clip].start,
                    self.clips[self.selected_clip].track + 1
                ),
                12.0,
                600,
                theme.muted_foreground,
            ),
            timeline,
        ])
        .width(percent(1.0))
        .height(percent(1.0))
        .padding(Sides::length(24.0))
        .gap(14.0)
        .background(theme.background)
    }
}
