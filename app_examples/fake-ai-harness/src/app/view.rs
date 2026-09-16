use super::{AiHarness, FADE_TIME, GRAPH_SAMPLES, Phase, StreamChunk, TARGET_RATE, TARGET_TOKENS};
use crate::fake_model::SOURCE_TITLE;
use argui::{
    animation::Frame,
    core::{Color, Transform2D},
    paint::{Border, CornerRadii, Fill, Filter, LayerMask, LayerStyle, Shadow},
    runtime::{Context, Render},
    text::{TextColor, TextStyle, TextWrap},
    ui::{
        AlignItems, Element, JustifyContent, Position, ScrollPropagation, Sides, auto, length,
        percent,
    },
    widgets::{
        Badge, BadgeVariant, Button, Input, InputKind, TablerIcon, VList, WidgetTheme, shadcn,
    },
};

impl AiHarness {
    pub(super) fn view(&self, cx: &mut Context<Self>, theme: &WidgetTheme) -> Element {
        let latest_handler = cx.event_handler(|app, _, cx| {
            let request = app.message_scroll.latest("conversation");
            app.conversation_offset = app.scroll_maximum;
            cx.scroll(request);
            cx.notify();
        });
        let mut body_children = vec![
            self.conversation(cx, theme)
                .grow(1.0)
                .min_width(length(0.0)),
        ];
        if !self.compact {
            body_children.push(self.telemetry(theme));
        }
        let body = Element::row(body_children)
            .width(percent(1.0))
            .gap(if self.compact { 0.0 } else { 16.0 })
            .align_items(AlignItems::START);
        let mut body_layers = vec![body];
        body_layers.extend(self.latest_button(theme, latest_handler));
        let body = Element::container(body_layers)
            .width(percent(1.0))
            .min_width(length(0.0))
            .position(Position::Relative);
        Element::column([self.header(theme), body])
            .width(percent(1.0))
            .height(percent(1.0))
            .min_width(length(0.0))
            .padding(Sides::length(if self.compact { 12.0 } else { 20.0 }))
            .gap(if self.compact { 9.0 } else { 14.0 })
            .background(theme.background)
            .min_height(length(0.0))
    }

    fn header(&self, theme: &WidgetTheme) -> Element {
        let title = Element::column([
            text("Argui AI Harness", 25.0, theme.foreground, 700),
            text(
                "A local streaming workload for the complete UI pipeline",
                13.0,
                theme.muted_foreground,
                400,
            ),
        ])
        .gap(2.0);
        let observed = Badge::new(
            "rate-badge",
            format!("{:.0} TOK/S · OBSERVED", self.observed_rate()),
        )
        .variant(BadgeVariant::Primary)
        .build(theme);
        if self.compact {
            Element::row([
                text("Argui AI Harness", 20.0, theme.foreground, 700),
                observed,
            ])
            .gap(10.0)
            .align_items(AlignItems::CENTER)
            .justify_content(JustifyContent::SPACE_BETWEEN)
        } else {
            Element::row([title, observed])
                .gap(16.0)
                .align_items(AlignItems::CENTER)
                .justify_content(JustifyContent::SPACE_BETWEEN)
        }
    }

    fn conversation(&self, cx: &mut Context<Self>, theme: &WidgetTheme) -> Element {
        let input_handler = cx.input_callback(|app, prompt| app.prompt = prompt);
        let submit_handler = cx.submit_event_handler(|app, _, _, cx| app.start(cx));
        let run_handler = cx.event_handler(|app, _, cx| app.start(cx));
        let messages = if self.submitted_prompt.is_empty() {
            Element::column([
                text("Send a message", 20.0, theme.foreground, 650),
                text(
                    "Press Enter or Send. A bundled Wikipedia adaptation will stream locally so the benchmark measures Argui instead of the network.",
                    14.0,
                    theme.muted_foreground,
                    400,
                ),
            ])
            .height(length(self.conversation_viewport))
            .padding(Sides::length(18.0))
            .gap(7.0)
        } else {
            VList::variable(
                "conversation",
                &self.message_heights,
                self.conversation_offset,
            )
            .propagation(ScrollPropagation::Contain)
            .build(self.message_heights.item_count(), theme, |index| {
                if index == 0 {
                    return message(
                        Some("YOU"),
                        &self.submitted_prompt,
                        theme.muted,
                        theme.foreground,
                        theme,
                    )
                    .keyed("message-user");
                }
                cx.entity(&self.chunks[index - 1])
            })
        };
        let controls = Element::row([
            Input::new(
                "prompt",
                &self.prompt,
                "Ask the fake agent anything…",
                theme.input(),
            )
            .kind(InputKind::Text)
            .label("Prompt")
            .on_input(input_handler)
            .on_submit(submit_handler)
            .build()
            .width(auto())
            .flex_basis(percent(0.8))
            .grow(0.0)
            .shrink(1.0)
            .min_width(length(0.0)),
            Button::new(
                "run",
                if self.phase == Phase::Streaming {
                    "Restart"
                } else {
                    "Send"
                },
                theme.button(),
            )
            .enabled(!self.prompt.trim().is_empty())
            .on_click(run_handler)
            .build()
            .width(auto())
            .flex_basis(percent(0.2))
            .grow(0.0)
            .shrink(0.0),
        ])
        .gap(8.0)
        .align_items(AlignItems::CENTER);
        let content =
            Element::column([messages, controls]).gap(if self.compact { 7.0 } else { 10.0 });
        panel("Conversation", content, theme)
            .padding(Sides::length(if self.compact { 10.0 } else { 15.0 }))
            .gap(if self.compact { 8.0 } else { 13.0 })
            .width(percent(1.0))
            .min_width(length(0.0))
    }

    fn latest_button(
        &self,
        theme: &WidgetTheme,
        handler: argui::ui::EventHandler,
    ) -> Option<Element> {
        (!self.message_scroll.following).then(|| {
            let mut button = Button::icon(
                "conversation::latest",
                "Go to latest message",
                self.assets
                    .icon(TablerIcon::ArrowDown, 17.0)
                    .vector_color(theme.foreground),
                theme.outline_button(),
            )
            .on_click(handler)
            .tooltip(format!(
                "Go to latest message · {} unread",
                self.message_scroll.unread
            ))
            .build()
            .width(length(38.0))
            .height(length(38.0))
            .padding(Sides::length(0.0))
            .radius(CornerRadii::all(19.0))
            .layer(
                LayerStyle::new(Default::default())
                    .backdrop(Filter::Blur(10.0))
                    .shadow(Shadow::glow(6.0, Color::BLACK.with_alpha(0.10)))
                    .mask(LayerMask::Rounded(CornerRadii::all(19.0))),
            );
            button.override_background(Some(Fill::Solid(theme.card.with_alpha(0.58))));
            Element::row([button])
                .height(length(38.0))
                .justify_content(JustifyContent::CENTER)
                .absolute(Sides {
                    left: length(0.0),
                    right: length(0.0),
                    top: auto(),
                    bottom: length(62.0),
                })
        })
    }

    fn telemetry(&self, theme: &WidgetTheme) -> Element {
        let mut bars = Vec::with_capacity(GRAPH_SAMPLES);
        let missing = GRAPH_SAMPLES.saturating_sub(self.recent_batches.len());
        for index in 0..GRAPH_SAMPLES {
            let tokens = if index < missing {
                0
            } else {
                self.recent_batches[index - missing]
            };
            let height = 4.0 + (tokens as f32 / 20.0).clamp(0.0, 1.0) * 40.0;
            bars.push(
                Element::container([])
                    .width(length(3.0))
                    .height(length(height))
                    .background(if tokens == 0 {
                        theme.muted
                    } else {
                        theme.primary
                    })
                    .radius(CornerRadii::all(2.0)),
            );
        }
        let panel = panel(
            "Live telemetry",
            Element::column([
                Element::row([
                    tiny_metric(
                        &format!("{:.0} tok/s", self.observed_rate()),
                        "Observed throughput",
                        theme,
                    ),
                    tiny_metric(
                        &self.first_token.map_or_else(
                            || "—".into(),
                            |value| format!("{:.1} ms", value.as_secs_f64() * 1_000.0),
                        ),
                        "First token",
                        theme,
                    ),
                ])
                .gap(12.0),
                Element::row([
                    tiny_metric(
                        &format!("{} / {TARGET_TOKENS}", self.emitted()),
                        "Generated context",
                        theme,
                    ),
                    tiny_metric(&self.batches.to_string(), "UI batches", theme),
                ])
                .gap(12.0),
                text("TOKENS PER UI BATCH", 10.0, theme.muted_foreground, 650),
                Element::row(bars)
                    .height(length(48.0))
                    .gap(2.0)
                    .align_items(AlignItems::END),
                divider(theme),
                text(
                    format!(
                        "{} tokens last frame · display-synced · {TARGET_RATE} tok/s target",
                        self.last_batch
                    ),
                    11.0,
                    theme.muted_foreground,
                    400,
                ),
            ])
            .gap(11.0),
            theme,
        )
        .shrink(0.0);
        if self.compact {
            panel.width(percent(1.0))
        } else {
            panel.width(length(224.0))
        }
    }
}

impl Render for StreamChunk {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let environment = cx.environment();
        self.reduced_motion = environment.reduced_motion;
        let themes = shadcn(&environment);
        let theme = themes.resolve(environment.color_scheme);
        let progress = if self.reduced_motion {
            1.0
        } else {
            (self.born.elapsed().as_secs_f32() / FADE_TIME.as_secs_f32()).clamp(0.0, 1.0)
        };
        self.fade_complete = progress >= 1.0;
        let eased = progress * progress * (3.0 - 2.0 * progress);
        message(
            self.source_label.then_some(SOURCE_TITLE),
            &self.text,
            theme.primary.with_alpha(0.10),
            theme.foreground,
            theme,
        )
        .keyed(self.key.clone())
        .opacity(eased)
        .transform(Transform2D::IDENTITY.translate(0.0, 4.0 * (1.0 - eased)))
    }

    fn wants_animation_frame(&self) -> bool {
        !self.reduced_motion && !self.fade_complete
    }

    fn animation_frame(&mut self, _frame: Frame, cx: &mut Context<Self>) {
        cx.notify();
    }
}

fn panel(title: &str, content: Element, theme: &WidgetTheme) -> Element {
    Element::column([text(title, 15.0, theme.foreground, 650), content])
        .width(percent(1.0))
        .min_width(length(0.0))
        .padding(Sides::length(15.0))
        .gap(13.0)
        .background(theme.card)
        .border(Border::all(1.0, theme.border))
        .radius(CornerRadii::all(12.0))
}

fn tiny_metric(value: &str, label: &str, theme: &WidgetTheme) -> Element {
    Element::column([
        text(value, 13.0, theme.foreground, 700),
        text(label, 9.0, theme.muted_foreground, 600),
    ])
    .grow(1.0)
    .min_width(length(0.0))
    .gap(2.0)
}

fn message(
    label: Option<&str>,
    body: &str,
    background: Color,
    foreground: Color,
    theme: &WidgetTheme,
) -> Element {
    Element::column(
        label
            .into_iter()
            .map(|label| text(label, 11.0, theme.muted_foreground, 700))
            .chain([text(body, 14.0, foreground, 400)]),
    )
    .width(percent(1.0))
    .padding(Sides::length(14.0))
    .gap(if label.is_some() { 7.0 } else { 0.0 })
    .background(background)
    .radius(CornerRadii::all(10.0))
}

fn divider(theme: &WidgetTheme) -> Element {
    Element::container([])
        .width(percent(1.0))
        .height(length(1.0))
        .background(theme.border)
}

fn text(value: impl Into<String>, size: f32, color: TextColor, weight: u16) -> Element {
    Element::text(value.into()).text_style(TextStyle {
        font_size: size,
        line_height: size * 1.4,
        color,
        weight,
        wrap: TextWrap::Word,
        ..TextStyle::default()
    })
}
