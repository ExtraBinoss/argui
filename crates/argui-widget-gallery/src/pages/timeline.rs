use crate::app::text;
use argui::{
    accessibility::{Role, Semantics},
    core::{Color, Point, Rect, Size},
    paint::QuadStyle,
    runtime::{Context, Entity, ModelRuntime, Render},
    ui::{
        CursorIcon, CustomElement, CustomLayoutContext, CustomMeasurement, CustomPaintContext,
        Element, EventType, GestureCapture, GestureKind, GesturePhase, GestureSet, PanGesture,
        UiEventKind, UserSelect, length, percent,
    },
    widgets::{Button, shadcn},
};

mod regions;

#[derive(Debug)]
struct Ruler {
    zoom: f32,
    color: Color,
    clips: [Clip; 3],
}

impl CustomElement for Ruler {
    type State = Vec<f32>;
    fn create_state(&self) -> Self::State {
        Vec::new()
    }
    fn layout_revision(&self) -> u64 {
        self.clips
            .iter()
            .fold(u64::from(self.zoom.to_bits()), |revision, clip| {
                revision
                    .wrapping_mul(31)
                    .wrapping_add(u64::from(clip.start.to_bits()))
                    .wrapping_mul(31)
                    .wrapping_add(u64::from(clip.duration.to_bits()))
            })
    }
    fn paint_revision(&self) -> u64 {
        u64::from(u32::from_le_bytes(self.color.to_srgba8()))
    }
    fn layout(
        &self,
        _: &mut Self::State,
        cx: &mut dyn CustomLayoutContext,
    ) -> Result<CustomMeasurement, String> {
        for (index, clip) in self.clips.iter().enumerate() {
            let x = clip.start * 40.0 * self.zoom;
            let y = 28.0 + index as f32 * 62.0;
            let width = clip.duration * 40.0 * self.zoom;
            cx.place_child(
                index * 2,
                Rect::new(Point::new(x, y), Size::new(width, 40.0)),
            )?;
            cx.place_child(
                index * 2 + 1,
                Rect::new(Point::new(x + width - 12.0, y), Size::new(12.0, 40.0)),
            )?;
        }
        for index in 6..cx.child_count() {
            cx.place_child(
                index,
                Rect::new(
                    Point::new((index - 6) as f32 * 80.0 * self.zoom + 3.0, 0.0),
                    Size::new(60.0 * self.zoom, 22.0),
                ),
            )?;
        }
        Ok(CustomMeasurement {
            size: Size::new(800.0 * self.zoom, 230.0),
            baseline: None,
        })
    }
    fn prepare(&self, ticks: &mut Self::State, size: Size) {
        let count = (size.width / (40.0 * self.zoom)).ceil() as usize;
        ticks.clear();
        ticks.extend((0..count).map(|tick| tick as f32 * 40.0 * self.zoom));
    }
    fn paint(&self, ticks: &mut Self::State, cx: &mut CustomPaintContext<'_>) {
        for &x in ticks.iter() {
            cx.quad(
                Rect::new(Point::new(x, 0.0), Size::new(1.0, cx.bounds.size.height)),
                QuadStyle::solid(self.color),
            );
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Clip {
    start: f32,
    duration: f32,
}

pub(crate) struct Timeline {
    views: [Entity<TimelineView>; 2],
}

struct TimelineView {
    clips: Entity<[Clip; 3]>,
    prefix: &'static str,
    zoom: f32,
    selected: usize,
    drag_start: Clip,
}

impl Default for Timeline {
    fn default() -> Self {
        let runtime = ModelRuntime::default();
        let clips = runtime.entity([
            Clip {
                start: 1.0,
                duration: 4.0,
            },
            Clip {
                start: 3.0,
                duration: 6.0,
            },
            Clip {
                start: 10.0,
                duration: 5.0,
            },
        ]);
        Self {
            views: ["timeline", "timeline-secondary"].map(|prefix| {
                runtime.entity(TimelineView {
                    clips: clips.clone(),
                    prefix,
                    zoom: 1.0,
                    selected: 0,
                    drag_start: Clip {
                        start: 0.0,
                        duration: 1.0,
                    },
                })
            }),
        }
    }
}

impl Render for Timeline {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        Element::column(self.views.iter().map(|view| cx.entity(view)))
            .gap(28.0)
            .width(percent(1.0))
    }
}

impl TimelineView {
    fn key(&self, suffix: &str) -> String {
        format!("{}-{suffix}", self.prefix)
    }
    fn change_clip(&self, index: usize, edit: impl FnOnce(&mut Clip)) {
        self.clips.update(|clips, cx| {
            edit(&mut clips[index]);
            cx.notify();
        });
    }
}

impl Render for TimelineView {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = shadcn(cx.environment());
        let theme = themes.resolve(cx.environment().color_scheme);
        let clips = cx.read(&self.clips, |clips| *clips);
        let zoom_out = Button::new(self.key("zoom-out"), "Zoom −", theme.ghost_button())
            .build()
            .on(cx.listener(EventType::Click, |this, _, cx| {
                this.zoom = (this.zoom / 1.25).max(0.5);
                cx.notify();
            }));
        let zoom_in = Button::new(self.key("zoom-in"), "Zoom +", theme.ghost_button())
            .build()
            .on(cx.listener(EventType::Click, |this, _, cx| {
                this.zoom = (this.zoom * 1.25).min(4.0);
                cx.notify();
            }));
        let mut content = Vec::new();
        for (index, clip) in clips.iter().copied().enumerate() {
            let selected = self.selected == index;
            let mut block = Button::new(
                self.key(&format!("clip-{index}")),
                format!("Clip {} · {:.1}s", index + 1, clip.duration),
                if selected {
                    theme.button()
                } else {
                    theme.secondary_button()
                },
            )
            .build()
            .on(cx.listener(EventType::Click, move |this, _, cx| {
                this.selected = index;
                cx.notify();
            }))
            .width(length(clip.duration * 40.0 * self.zoom))
            .height(length(40.0))
            .user_select(UserSelect::None);
            if let Some(semantics) = &mut block.semantics {
                semantics.state.selected = selected;
                semantics.description =
                    Some("Left/Right moves the clip; Shift+Left/Right changes duration".into());
            }
            let drag = cx.listener(EventType::Gesture, move |this, event, cx| {
                let UiEventKind::Gesture(gesture) = event.kind else {
                    return;
                };
                let GestureKind::Pan { total, .. } = gesture.kind else {
                    return;
                };
                if gesture.phase == GesturePhase::Started {
                    this.selected = index;
                    this.drag_start = this.clips.read(|clips| clips[index]);
                }
                if gesture.phase == GesturePhase::Cancelled {
                    this.change_clip(index, |clip| {
                        clip.start = this.drag_start.start.min(20.0 - clip.duration);
                    });
                } else {
                    this.change_clip(index, |clip| {
                        clip.start = (this.drag_start.start + total.x / (40.0 * this.zoom))
                            .clamp(0.0, 20.0 - clip.duration);
                    });
                }
                cx.notify();
            });
            let interaction = block.interaction.clone().expect("button interaction");
            block = block
                .interaction(
                    interaction.cursor(CursorIcon::Grab).gestures(
                        GestureSet::EMPTY.pan(
                            PanGesture::default()
                                .immediate()
                                .capture(GestureCapture::OnPress),
                        ),
                    ),
                )
                .on(drag)
                .on(cx.listener(EventType::Key, move |this, event, cx| {
                    let UiEventKind::KeyInput(key) = &event.kind else {
                        return;
                    };
                    if key.state != argui::core::KeyState::Pressed {
                        return;
                    }
                    let delta = match key.key {
                        argui::core::Key::ArrowLeft => -0.5,
                        argui::core::Key::ArrowRight => 0.5,
                        _ => return,
                    };
                    this.selected = index;
                    if key.modifiers.shift {
                        this.change_clip(index, |clip| {
                            clip.duration = (clip.duration + delta).clamp(1.0, 20.0 - clip.start)
                        });
                    } else {
                        this.change_clip(index, |clip| {
                            clip.start = (clip.start + delta).clamp(0.0, 20.0 - clip.duration)
                        });
                    }
                    event.stop_propagation();
                    cx.notify();
                }));
            content.push(block);
            content.push(self.resize_region(cx, index, clip, theme.foreground, theme.ring));
        }
        content.extend((0..10).map(|index| {
            text(format!("{}s", index * 2), 12.0, theme.muted_foreground, 400)
                .keyed(self.key(&format!("tick-{index}")))
        }));
        let selected = self.selected;
        let shorten = Button::new(
            self.key("shorten"),
            "Shorten selected",
            theme.ghost_button(),
        )
        .build()
        .on(cx.listener(EventType::Click, move |this, _, cx| {
            this.change_clip(selected, |clip| {
                clip.duration = (clip.duration - 0.5).max(1.0)
            });
            cx.notify();
        }));
        let extend = Button::new(self.key("extend"), "Extend selected", theme.ghost_button())
            .build()
            .on(cx.listener(EventType::Click, move |this, _, cx| {
                this.change_clip(selected, |clip| {
                    clip.duration = (clip.duration + 0.5).min(20.0 - clip.start)
                });
                cx.notify();
            }));
        let details = clips.iter().enumerate().map(|(index, clip)| {
            text(
                format!(
                    "Clip {}    start {:.1}s    duration {:.1}s",
                    index + 1,
                    clip.start,
                    clip.duration
                ),
                14.0,
                theme.foreground,
                400,
            )
            .keyed(self.key(&format!("detail-{index}")))
        });
        Element::column([
            text(if self.prefix == "timeline" { "Custom Timeline" } else { "Shared Timeline · independent view" }, 24.0, theme.foreground, 600),
            text("Custom layout and painted ruler. Drag clips to move; drag their right handles to resize. Keyboard and accessible actions use the same model.", 14.0, theme.muted_foreground, 400),
            Element::row([zoom_out, zoom_in, shorten, extend]).gap(8.0),
            Element::container([Element::custom_container(Ruler {
                zoom: self.zoom, color: theme.border, clips,
            }, content).keyed(self.key("ruler"))])
                .width(percent(1.0)).height(length(250.0))
                .overflow(argui::ui::Axes { x: argui::ui::Overflow::Scroll, y: argui::ui::Overflow::Hidden })
                .scroll_config(argui::ui::ScrollConfig::default().axes(argui::ui::ScrollAxes::Horizontal)),
            Element::column(details).gap(8.0),
        ]).gap(16.0).width(percent(1.0))
            .semantics(Semantics::new(Role::Group).label(if self.prefix == "timeline" {
                "Primary timeline"
            } else {
                "Shared timeline, independent presentation"
            }))
    }
}
