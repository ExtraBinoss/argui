mod controls;

use crate::app::text;
use argui::{
    core::{Color, Point, Rect},
    paint::{Border, CornerRadii, LayerMask, LayerStyle, Shadow},
    runtime::{Context, LayoutSnapshot, Render},
    ui::{
        Axes, CursorIcon, Element, EventType, GestureCapture, GestureKind, GesturePhase,
        GestureSet, Interaction, Overflow, PanGesture, Sides, UiEventKind, UserSelect, length,
        percent,
    },
    widgets::{RangeState, shadcn},
};
use argui_effects::LiquidGlass;

pub(crate) struct GlassDemo {
    position: Point,
    drag_origin: Point,
    effect: LiquidGlass,
    ranges: [RangeState; controls::COUNT],
    tint: usize,
    enabled: bool,
    max_x: f32,
    dragging: bool,
    noise: bool,
}

impl Default for GlassDemo {
    fn default() -> Self {
        Self {
            position: Point::new(48.0, 70.0),
            drag_origin: Point::default(),
            effect: LiquidGlass::new()
                .refraction(12.0)
                .blur(2.0)
                .tint([1.0, 1.0, 1.0, 0.12]),
            ranges: [RangeState::default(); controls::COUNT],
            tint: 0,
            enabled: true,
            max_x: 48.0,
            dragging: false,
            noise: false,
        }
    }
}

impl Render for GlassDemo {
    fn layout_changed(&mut self, layout: &LayoutSnapshot, cx: &mut Context<Self>) {
        self.layout_controls(layout);
        if let Some(bounds) = layout.bounds("liquid-glass-stage") {
            self.max_x = (bounds.size.width - 230.0).max(0.0);
            let x = self.position.x.min(self.max_x);
            if x != self.position.x {
                self.position.x = x;
                cx.notify();
            }
        }
    }
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = shadcn(cx.environment().primary);
        let theme = themes.resolve(cx.environment().color_scheme);
        let controls = self.controls(theme, cx);
        let mut pane = Element::column([Element::column([
            text("Liquid Glass", 16.0, Color::WHITE, 700),
            text("Grab anywhere to move", 11.0, Color::WHITE, 400),
        ])
        .gap(4.0)
        .padding(Sides::length(10.0))
        .background(Color::srgba(0.08, 0.12, 0.2, 0.85))
        .radius(CornerRadii::all(10.0))])
        .keyed("liquid-glass-pane")
        .width(length(230.0))
        .height(length(150.0))
        .padding(Sides::length(18.0))
        .gap(8.0)
        .absolute(Sides {
            left: length(self.position.x),
            top: length(self.position.y),
            right: argui::ui::auto(),
            bottom: argui::ui::auto(),
        })
        .background(Color::TRANSPARENT)
        .border(Border::all(1.0, Color::srgba(0.12, 0.2, 0.32, 0.5)))
        .radius(CornerRadii::all(24.0))
        .layer(
            LayerStyle::new(Rect::default())
                .mask(LayerMask::Rounded(CornerRadii::all(24.0)))
                .shadow(Shadow::drop(
                    [0.0, 8.0],
                    18.0,
                    Color::srgba(0.0, 0.0, 0.0, 0.28),
                )),
        )
        .hit_test(
            argui::ui::HitTestStyle::default()
                .shape(argui::ui::HitShape::RoundedRect(CornerRadii::all(24.0))),
        )
        .user_select(UserSelect::None)
        .interaction(
            Interaction::blocker()
                .cursor(if self.dragging {
                    CursorIcon::Grabbing
                } else {
                    CursorIcon::Grab
                })
                .gestures(
                    GestureSet::EMPTY.pan(
                        PanGesture::default()
                            .immediate()
                            .capture(GestureCapture::OnPress)
                            .delivery(argui::ui::GestureDelivery::FrameCoalesced),
                    ),
                ),
        )
        .on(cx.listener(EventType::Gesture, |demo, event, cx| {
            let UiEventKind::Gesture(gesture) = event.kind else {
                return;
            };
            let GestureKind::Pan { total, .. } = gesture.kind else {
                return;
            };
            if gesture.phase == GesturePhase::Started {
                demo.drag_origin = demo.position;
                demo.dragging = true;
            }
            if matches!(gesture.phase, GesturePhase::Ended | GesturePhase::Cancelled) {
                demo.dragging = false;
                cx.notify();
            }
            if matches!(gesture.phase, GesturePhase::Started | GesturePhase::Changed) {
                demo.position = Point::new(
                    (demo.drag_origin.x + total.x).clamp(0.0, demo.max_x),
                    (demo.drag_origin.y + total.y).clamp(0.0, 210.0),
                );
                cx.notify();
            }
        }));
        if self.enabled {
            let mut effect = self.effect;
            if !self.noise {
                effect.turbulence = 0.0;
            }
            pane = pane.backdrop_filter(effect.filter());
        }
        let background = Element::column((0..8).map(|row| {
            Element::row([
                Element::container([])
                    .width(length(12.0))
                    .height(length(30.0))
                    .background(if row % 2 == 0 {
                        Color::srgb(0.1, 0.4, 0.95)
                    } else {
                        Color::srgb(0.95, 0.15, 0.3)
                    }),
                text(
                    format!(
                        "{:02}  Refraction bends content · ABCDEFG · 0123456789",
                        row + 1
                    ),
                    21.0,
                    Color::BLACK,
                    600,
                ),
            ])
            .gap(10.0)
            .shrink(0.0)
        }))
        .padding(Sides::length(18.0))
        .gap(10.0);
        super::preview(
            "Liquid Glass",
            "Curved optical rim · Snell refraction and Fresnel reflections. Drag the glass across the text. Fractal noise is optional.",
            Element::column([
                controls,
                Element::container([background, pane])
                    .keyed("liquid-glass-stage")
                    .height(length(360.0))
                    .width(percent(1.0))
                    .background(Color::WHITE)
                    .radius(CornerRadii::all(12.0))
                    .overflow(Axes {
                        x: Overflow::Hidden,
                        y: Overflow::Hidden,
                    }),
            ])
            .gap(12.0)
            .width(percent(1.0)),
            theme,
        )
    }
}
