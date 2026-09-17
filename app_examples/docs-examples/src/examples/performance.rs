use argui::{
    accessibility::{Role, Semantics},
    core::{Color, Point, Transform2D},
    paint::{Border, CornerRadii},
    runtime::{Context, Render},
    text::TextStyle,
    ui::{
        AlignItems, CursorIcon, Element, EventType, FocusPolicy, GestureCapture, GestureDelivery,
        GestureKind, GesturePhase, GestureSet, Interaction, JustifyContent, PanGesture, Sides,
        UiEventKind, UserSelect, length, percent,
    },
    widgets::default_theme,
};

#[derive(Default)]
pub struct Example {
    offset: Point,
    velocity: Point,
    delivered_updates: u64,
    dragging: bool,
}

impl Render for Example {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = default_theme(cx.environment());
        let theme = themes.resolve(cx.environment().color_scheme);
        let orb = Element::row([
            Element::text(if self.dragging { "HOLDING" } else { "DRAG" }).text_style(TextStyle {
                color: Color::WHITE,
                font_size: 11.0,
                line_height: 15.0,
                weight: 760,
                ..TextStyle::default()
            }),
        ])
        .absolute(Sides {
            left: length(220.0),
            right: argui::ui::auto(),
            top: length(72.0),
            bottom: argui::ui::auto(),
        })
        .width(length(88.0))
        .height(length(56.0))
        .align_items(AlignItems::CENTER)
        .justify_content(JustifyContent::CENTER)
        .background(theme.primary)
        .radius(CornerRadii::all(14.0))
        .transform(Transform2D::IDENTITY.translate(self.offset.x, self.offset.y));
        let pad = Element::container([orb])
            .keyed("frame-coalesced-pad")
            .width(percent(1.0))
            .height(length(200.0))
            .background(theme.muted)
            .border(Border::all(1.0, theme.border))
            .radius(CornerRadii::all(14.0))
            .user_select(UserSelect::None)
            .interaction(
                Interaction::default()
                    .focus_policy(FocusPolicy::TabStop)
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
                                .delivery(GestureDelivery::FrameCoalesced),
                        ),
                    ),
            )
            .semantics(
                Semantics::new(Role::Group)
                    .label("Frame-coalesced drag pad")
                    .description("Drag to observe one visual update per available frame"),
            )
            .on(cx.listener(EventType::Gesture, |example, event, cx| {
                let UiEventKind::Gesture(gesture) = event.kind else {
                    return;
                };
                let GestureKind::Pan {
                    total, velocity, ..
                } = gesture.kind
                else {
                    return;
                };
                if gesture.phase == GesturePhase::Started {
                    example.delivered_updates = 0;
                    example.dragging = true;
                }
                if gesture.phase == GesturePhase::Changed {
                    example.delivered_updates = example.delivered_updates.saturating_add(1);
                }
                example.offset =
                    Point::new(total.x.clamp(-210.0, 210.0), total.y.clamp(-68.0, 68.0));
                example.velocity = velocity;
                if matches!(gesture.phase, GesturePhase::Ended | GesturePhase::Cancelled) {
                    example.dragging = false;
                }
                event.stop_propagation();
                cx.notify();
            }));
        let status = format!(
            "Delivered frame updates: {} · velocity {:.0}, {:.0} px/s",
            self.delivered_updates, self.velocity.x, self.velocity.y
        );

        Element::column([
            Element::text("Frame-coalesced continuous input").text_style(TextStyle {
                color: theme.foreground,
                font_size: 18.0,
                line_height: 24.0,
                weight: 700,
                ..TextStyle::default()
            }),
            Element::text(
                "Move quickly: position, velocity and this status are delivered together at most once per available display frame.",
            )
            .text_style(TextStyle {
                color: theme.muted_foreground,
                ..TextStyle::default()
            }),
            pad,
            Element::text(status.clone())
                .text_style(TextStyle {
                    color: theme.muted_foreground,
                    font_size: 13.0,
                    line_height: 18.0,
                    weight: 550,
                    ..TextStyle::default()
                })
                .semantics(Semantics::new(Role::Status).label(status)),
        ])
        .width(percent(1.0))
        .height(percent(1.0))
        .padding(Sides::length(28.0))
        .gap(14.0)
        .background(theme.background)
    }
}
