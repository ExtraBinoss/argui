use argui_core::{Color, ColorInterpolation, Point, Transform2D};
use argui_paint::{BilinearGradient, Border, CornerRadii, Fill, GradientStop, LinearGradient};
use argui_text::TextStyle;
use argui_ui::{
    AlignItems, Element, FocusPolicy, GestureCapture, GestureDelivery, GestureSet, Interaction,
    LengthPercentageAuto, PanGesture, Role, SemanticState, Semantics, Sides, StylePatch,
    UserSelect, VisualState, auto, length, percent,
};

use super::{ColorFormat, ColorPicker, values::hsv_to_rgb};
use crate::{Button, Input, RangeBehavior, RangePart, WidgetTheme};

impl ColorPicker<'_> {
    pub(super) fn view(&self, theme: &WidgetTheme) -> Element {
        let formats = ColorFormat::ALL.map(|format| {
            let mut style = if self.state.format == format {
                theme.secondary_button()
            } else {
                theme.ghost_button()
            };
            style.layout.padding = Sides::length(6.0);
            style.label.font_size = 11.0;
            Button::new(
                format!("{}::format::{}", self.key, format.label()),
                format.label(),
                style,
            )
            .enabled(self.state.enabled)
            .build()
            .grow(1.0)
            .min_width(length(0.0))
        });
        let inputs = self
            .state
            .fields()
            .into_iter()
            .enumerate()
            .map(|(index, (label, value))| {
                let draft = self.state.draft.as_ref().filter(|draft| draft.0 == index);
                let invalid = draft.is_some_and(|draft| draft.2);
                let mut style = theme.input();
                style.layout.padding = Sides::length(6.0);
                style.text.font_size = 12.0;
                let field = Input::new(
                    format!("{}::field::{index}", self.key),
                    draft.map_or(value, |draft| draft.1.clone()),
                    "",
                    style,
                )
                .label(format!("{} {label}", self.label))
                .description(if invalid {
                    "Enter a valid color value within the labeled range"
                } else {
                    "Changes apply immediately; Escape discards invalid input"
                })
                .invalid(invalid)
                .enabled(self.state.enabled)
                .build()
                .min_width(length(0.0));
                Element::column([label_text(label, theme.muted_foreground, 11.0), field])
                    .gap(4.0)
                    .grow(1.0)
                    .flex_basis(length(0.0))
                    .min_width(length(0.0))
            });
        let color = self.state.color();
        let [r, g, b, _] = color.to_srgba();
        let alpha = Fill::Linear(
            LinearGradient::new(
                Point::new(0.0, 0.0),
                Point::new(1.0, 0.0),
                ColorInterpolation::Srgb,
                [
                    GradientStop::new(0.0, Color::srgba(r, g, b, 0.0)),
                    GradientStop::new(1.0, Color::srgb(r, g, b)),
                ],
            )
            .expect("ordered alpha stops"),
        );
        let mut rows = vec![
            label_text(&self.label, theme.foreground, 13.0),
            self.pad(theme),
            self.track(
                self.state.range(&self.key, false),
                hue_gradient(),
                theme,
                false,
            ),
            self.track(self.state.range(&self.key, true), alpha, theme, true),
            Element::row(formats).gap(2.0).width(percent(1.0)),
            Element::row(inputs).gap(6.0).width(percent(1.0)),
        ];
        if self.state.draft.as_ref().is_some_and(|draft| draft.2) {
            rows.push(label_text(
                "Invalid value — the last valid color is preserved.",
                theme.destructive,
                11.0,
            ));
        }
        Element::column(rows)
            .keyed(self.key.clone())
            .gap(10.0)
            .width(percent(1.0))
            .min_width(length(0.0))
            .semantics(Semantics::new(Role::Group).label(self.label.clone()))
    }

    fn pad(&self, theme: &WidgetTheme) -> Element {
        let [h, s, v] = self.state.hsv;
        let [r, g, b] = hsv_to_rgb([h, 1.0, 1.0]);
        let marker = thumb(self.state.color(), 14.0)
            .absolute(position(s, 1.0 - v))
            .transform(Transform2D::IDENTITY.translate(-7.0, -7.0));
        Element::container([marker]).keyed(format!("{}::pad", self.key))
            .width(percent(1.0)).height(length(160.0))
            .fill(Fill::Bilinear(BilinearGradient::new(
                [Color::WHITE, Color::srgb(r, g, b), Color::BLACK, Color::BLACK], ColorInterpolation::Srgb,
            )))
            .radius(CornerRadii::all(6.0)).user_select(UserSelect::None)
            .interaction(Interaction::default().enabled(self.state.enabled)
                .focus_policy(if self.state.enabled { FocusPolicy::TabStop } else { FocusPolicy::None })
                .gestures(GestureSet::default().pan(PanGesture::default().immediate()
                    .capture(GestureCapture::OnPress).delivery(GestureDelivery::FrameCoalesced))))
            .semantics(Semantics::new(Role::Group)
                .label(format!("Saturation {:.0}%, brightness {:.0}%", s * 100.0, v * 100.0))
                .description("Left/right changes saturation; up/down changes brightness. Shift adjusts by 0.1%. Color channels are also editable below.")
                .state(SemanticState { disabled: !self.state.enabled, ..SemanticState::default() }))
            .when(VisualState::FocusVisible, StylePatch::new().set(argui_ui::property::BorderColor, theme.ring).set(argui_ui::property::BorderWidths, [2.0; 4]))
    }

    fn track(
        &self,
        behavior: RangeBehavior,
        gradient: Fill,
        theme: &WidgetTheme,
        alpha: bool,
    ) -> Element {
        let marker = thumb(theme.card, 16.0)
            .absolute(position(behavior.ratio(), 0.5))
            .transform(Transform2D::IDENTITY.translate(-8.0, -8.0));
        let background = if alpha {
            checkerboard()
        } else {
            Element::container([])
        };
        let track = behavior.decorate(
            RangePart::Track,
            Element::container([
                background,
                Element::container([])
                    .fill(gradient)
                    .absolute(Sides::length(0.0))
                    .radius(CornerRadii::all(4.0)),
            ])
            .width(percent(1.0))
            .height(length(16.0))
            .radius(CornerRadii::all(4.0)),
        );
        let control = behavior.decorate(
            RangePart::Control,
            Element::row([track, marker])
                .width(percent(1.0))
                .height(length(24.0))
                .align_items(AlignItems::CENTER)
                .when(
                    VisualState::FocusVisible,
                    StylePatch::new()
                        .set(argui_ui::property::BorderColor, theme.ring)
                        .set(argui_ui::property::BorderWidths, [2.0; 4]),
                ),
        );
        behavior.decorate(RangePart::Root, control)
    }
}

fn position(x: f32, y: f32) -> Sides<LengthPercentageAuto> {
    Sides {
        left: LengthPercentageAuto::percent(x),
        top: LengthPercentageAuto::percent(y),
        right: auto(),
        bottom: auto(),
    }
}

fn thumb(color: Color, size: f32) -> Element {
    Element::container([])
        .width(length(size))
        .height(length(size))
        .background(color)
        .border(Border::all(2.0, Color::WHITE))
        .radius(CornerRadii::all(999.0))
        .semantic_hidden(true)
}

fn label_text(value: &str, color: Color, size: f32) -> Element {
    Element::text(value).text_style(TextStyle {
        font_size: size,
        color,
        ..TextStyle::default()
    })
}

fn checkerboard() -> Element {
    Element::column((0..2).map(|row| {
        Element::row((0..16).map(|col| {
            Element::container([])
                .grow(1.0)
                .height(length(8.0))
                .background(if (row + col) % 2 == 0 {
                    Color::from_srgb8(230, 230, 230)
                } else {
                    Color::from_srgb8(170, 170, 170)
                })
        }))
        .width(percent(1.0))
    }))
    .width(percent(1.0))
    .semantic_hidden(true)
}

fn hue_gradient() -> Fill {
    static HUE: std::sync::OnceLock<Fill> = std::sync::OnceLock::new();
    HUE.get_or_init(|| {
        Fill::Linear(
            LinearGradient::new(
                Point::default(),
                Point::new(1.0, 0.0),
                ColorInterpolation::Srgb,
                std::array::from_fn::<_, 7, _>(|index| {
                    let [r, g, b] = hsv_to_rgb([index as f32 * 60.0, 1.0, 1.0]);
                    GradientStop::new(index as f32 / 6.0, Color::srgb(r, g, b))
                }),
            )
            .expect("ordered hue stops"),
        )
    })
    .clone()
}
