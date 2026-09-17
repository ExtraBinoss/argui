use argui::{
    paint::{Border, CornerRadii, PaintStyle, QuadStyle},
    runtime::{Context, Render},
    text::TextStyle,
    ui::{
        AlignItems, Axes, ContinuousValuePhase, Element, EventType, JustifyContent, Overflow,
        RangeHandlerValue, Sides, StateSelector, StylePatch, StyleTransition, TextEdit,
        ValueHandler, VisualState, auto, length, percent, property,
    },
    widgets::{
        Button, RANGE_SCOPE, RangeAxis, RangeBehavior, RangeConfig, RangeDirection, RangePart,
        TextArea, WidgetTheme, default_theme,
    },
};

struct BigSlider {
    behavior: RangeBehavior,
    change_handlers: Vec<ValueHandler<f32>>,
    commit_handlers: Vec<ValueHandler<f32>>,
}

impl BigSlider {
    /// Creates the large custom slider used by this interaction example.
    fn new(key: &str, label: &str, value: f32, config: RangeConfig) -> Self {
        Self {
            behavior: RangeBehavior::new(key, label, value, config),
            change_handlers: Vec::new(),
            commit_handlers: Vec::new(),
        }
    }

    /// Adds a callback for each intermediate value produced while interacting.
    fn on_change(mut self, handler: ValueHandler<f32>) -> Self {
        self.change_handlers.push(handler);
        self
    }

    /// Adds a callback for the final value produced by an interaction.
    fn on_commit(mut self, handler: ValueHandler<f32>) -> Self {
        self.commit_handlers.push(handler);
        self
    }

    /// Builds a fully custom visual while preserving the standard range contract.
    fn build(self, theme: &WidgetTheme) -> Element {
        let ratio = self.behavior.ratio();
        let ticks = Element::row((0..9).map(|_| {
            Element::container([])
                .width(length(1.0))
                .height(length(12.0))
                .paint_style(PaintStyle::new(
                    QuadStyle::solid(theme.foreground).opacity(0.12),
                ))
                .when(
                    StateSelector::scope(RANGE_SCOPE, VisualState::Hovered),
                    StylePatch::new().set(property::Opacity, 0.42),
                )
                .when(
                    StateSelector::scope(RANGE_SCOPE, VisualState::Pressed),
                    StylePatch::new().set(property::Opacity, 0.7),
                )
                .transition(StyleTransition::default())
        }))
        .absolute(Sides {
            left: length(18.0),
            right: length(18.0),
            top: auto(),
            bottom: auto(),
        })
        .height(percent(1.0))
        .align_items(AlignItems::CENTER)
        .justify_content(JustifyContent::SPACE_BETWEEN);
        let fill = Element::container([])
            .absolute(Sides::length(0.0))
            .width(percent(1.0))
            .height(percent(1.0))
            .background(theme.primary.with_alpha(0.26));
        let thumb = Element::container([])
            .absolute(Sides {
                left: auto(),
                right: length(0.0),
                top: length(7.0),
                bottom: length(7.0),
            })
            .width(length(3.0))
            .background(theme.primary)
            .radius(CornerRadii::all(2.0));
        let progress = Element::container([fill, thumb])
            .absolute(Sides {
                left: length(0.0),
                right: auto(),
                top: length(0.0),
                bottom: length(0.0),
            })
            .width(percent(ratio));
        let track = self.behavior.decorate(
            RangePart::Track,
            Element::container([progress, ticks])
                .width(percent(1.0))
                .height(percent(1.0)),
        );
        let mut control = self.behavior.decorate(
            RangePart::Control,
            Element::container([track])
                .absolute(Sides::length(0.0))
                .width(percent(1.0))
                .height(percent(1.0)),
        );
        let config = self.behavior.config();
        for (phase, handlers) in [
            (ContinuousValuePhase::Change, &self.change_handlers),
            (ContinuousValuePhase::Commit, &self.commit_handlers),
        ] {
            let source = RangeHandlerValue::new(
                self.behavior.value(),
                config.minimum,
                config.maximum,
                config.step,
                config.axis == RangeAxis::Vertical,
                config.direction == RangeDirection::Reverse,
                phase,
            );
            for handler in handlers {
                for event in [
                    EventType::Key,
                    EventType::Gesture,
                    EventType::SemanticAction,
                ] {
                    control =
                        control.on(handler.direct_listener(event).range_handler_value(source));
                }
            }
        }
        self.behavior.decorate(
            RangePart::Root,
            Element::container([
                control,
                Element::text(format!("{:.0}%", self.behavior.value()))
                    .absolute(Sides {
                        left: auto(),
                        right: length(14.0),
                        top: length(18.0),
                        bottom: auto(),
                    })
                    .text_style(TextStyle {
                        color: theme.foreground,
                        weight: 700,
                        ..TextStyle::default()
                    })
                    .semantic_hidden(true),
            ])
            .width(percent(1.0))
            .height(length(58.0))
            .background(theme.card)
            .border(Border::all(1.0, theme.border))
            .radius(CornerRadii::all(12.0))
            .overflow(Axes {
                x: Overflow::Hidden,
                y: Overflow::Hidden,
            }),
        )
    }
}

pub struct Example {
    live_volume: f32,
    committed_volume: f32,
    saves: u32,
    source: String,
    edit_count: u32,
    last_edit: String,
}

impl Default for Example {
    fn default() -> Self {
        Self {
            live_volume: 35.0,
            committed_volume: 35.0,
            saves: 0,
            source: "fn main() {\n    println!(\"fast edits\");\n}".into(),
            edit_count: 0,
            last_edit: "No edits delivered yet".into(),
        }
    }
}

impl Render for Example {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = default_theme(cx.environment());
        let theme = themes.resolve(cx.environment().color_scheme);
        let label = |value: String, color, weight| {
            Element::text(value).text_style(TextStyle {
                color,
                weight,
                ..TextStyle::default()
            })
        };

        let live_value = cx.value_callback(|app, value| app.live_volume = value);
        let committed_value = cx.value_callback(|app, value| {
            app.live_volume = value;
            app.committed_volume = value;
        });
        let save = cx.callback(|app| app.saves = app.saves.saturating_add(1));
        let edit_source = cx.edit_callback(|app, edit: TextEdit| {
            let summary = format!(
                "bytes {}..{} → {} byte(s)",
                edit.range.start,
                edit.range.end,
                edit.replacement.len()
            );
            if edit.apply_to(&mut app.source).is_ok() {
                app.edit_count = app.edit_count.saturating_add(1);
                app.last_edit = summary;
            } else {
                app.last_edit = "Rejected stale edit".into();
            }
        });

        Element::column([
            label("Choose callbacks by intent".into(), theme.foreground, 700),
            label(
                "on_change previews continuously; on_commit stores the final value.".into(),
                theme.muted_foreground,
                450,
            ),
            Element::column([
                BigSlider::new(
                    "volume",
                    "Preview volume",
                    self.live_volume,
                    RangeConfig::new(0.0, 100.0, 1.0),
                )
                .on_change(live_value)
                .on_commit(committed_value)
                .build(theme),
                label(
                    format!("Live value: {:.0}%", self.live_volume),
                    theme.foreground,
                    600,
                ),
                label(
                    format!("Committed value: {:.0}%", self.committed_volume),
                    theme.muted_foreground,
                    500,
                ),
            ])
            .padding(Sides::length(18.0))
            .gap(10.0)
            .background(theme.card)
            .border(Border::all(1.0, theme.border))
            .radius(CornerRadii::all(10.0)),
            Element::column([
                label(
                    "Use deltas for document-sized text".into(),
                    theme.foreground,
                    650,
                ),
                label(
                    "on_edit sends one UTF-8 range replacement instead of cloning the whole value."
                        .into(),
                    theme.muted_foreground,
                    450,
                ),
                TextArea::new(
                    "incremental-source",
                    &self.source,
                    "Paste or type Rust…",
                    theme.input(),
                )
                .on_edit(edit_source)
                .build()
                .height(length(118.0)),
                label(
                    format!(
                        "Incremental edits: {} · {}",
                        self.edit_count, self.last_edit
                    ),
                    theme.muted_foreground,
                    500,
                ),
            ])
            .padding(Sides::length(18.0))
            .gap(10.0)
            .background(theme.card)
            .border(Border::all(1.0, theme.border))
            .radius(CornerRadii::all(10.0)),
            Button::new("save", "Save preset", theme.button())
                .on_click(save)
                .build(),
            label(
                format!("Saved {} time(s)", self.saves),
                theme.muted_foreground,
                500,
            ),
        ])
        .width(percent(1.0))
        .height(percent(1.0))
        .padding(Sides::length(28.0))
        .gap(16.0)
        .background(theme.background)
    }
}
