use argui_animation::{Duration, Transition, Tween};
use argui_core::{Color, Point, Transform2D};
use argui_paint::{Border, CornerRadii, Fill, GradientStop, LinearGradient, PaintStyle, QuadStyle};
use argui_ui::{
    Axes, Element, FlexWrap, Interaction, KeyboardActivation, Overflow, PropertyKey, ScrollConfig,
    ScrollbarPartStyle, ScrollbarStyle, Sides, StateScopeId, StateSelector, StylePatch,
    StyleTransition, TransitionDirection, TransitionRule, VisualState, length, percent, property,
    sides,
};
use argui_widgets::WidgetTheme;

use crate::text_style;

pub(crate) fn demo(theme: &WidgetTheme) -> Element {
    Element::column([
        Element::text("Automatic state transitions").text_style(text_style(
            18.0,
            theme.foreground,
            650,
            argui_text::TextWrap::Word,
        )),
        Element::text(
            "Hover, focus, press and scroll these retained examples. No application rebuild drives their animation.",
        )
        .text_style(text_style(
            14.0,
            theme.muted_foreground,
            400,
            argui_text::TextWrap::Word,
        )),
        Element::row([
            example(
                "Composed spring",
                "Hover + focus + press compose, including the inherited label transform.",
                spring_surface(theme),
                theme,
            ),
            example(
                "Directional tween",
                "The transform enters quickly and exits with a different duration.",
                directional_surface(theme),
                theme,
            ),
            example(
                "Incremental layout",
                "Width and padding animate through the existing Taffy node.",
                layout_surface(theme),
                theme,
            ),
            example(
                "Gradient properties",
                "Gradient geometry, stops and radii interpolate without replacing the fill.",
                gradient_surface(theme),
                theme,
            ),
            example(
                "Scrollable states",
                "Hover and drag the animated thumb; the retained rows transition independently.",
                scrollbar_surface(theme),
                theme,
            ),
        ])
        .flex_wrap(FlexWrap::Wrap)
        .gap(14.0),
    ])
    .keyed("state-style-examples")
    .gap(12.0)
    .padding(Sides::length(16.0))
    .background(theme.card)
    .border(Border::all(1.0, theme.border))
    .radius(CornerRadii::all(10.0))
}

fn spring_surface(theme: &WidgetTheme) -> Element {
    let scope = StateScopeId::new("state-example-spring");
    let label = Element::text("Hover, focus, press")
        .text_style(text_style(
            15.0,
            theme.primary_foreground,
            650,
            argui_text::TextWrap::None,
        ))
        .when(
            StateSelector::scope(scope, VisualState::Pressed),
            StylePatch::new().set(
                property::Transform,
                Transform2D::IDENTITY.translate(4.0, 0.0),
            ),
        )
        .transition(StyleTransition::new(Transition::spring()));
    Element::container([label])
        .keyed("state-example-spring")
        .state_scope(scope)
        .width(percent(1.0))
        .padding(sides(18.0, 14.0))
        .paint_style(PaintStyle::new(
            QuadStyle::solid(theme.primary)
                .radius(CornerRadii::all(10.0))
                .border(Border::all(1.0, theme.border)),
        ))
        .interaction(interactive())
        .when(
            VisualState::Hovered,
            StylePatch::new().set(property::CornerRadii, [18.0; 4]).set(
                property::Transform,
                Transform2D::IDENTITY
                    .translate(6.0, 0.0)
                    .scale(1.015, 1.015),
            ),
        )
        .when(
            VisualState::Focused,
            StylePatch::new()
                .set(property::BorderColor, theme.foreground)
                .set(property::BorderWidths, [3.0; 4]),
        )
        .when(
            VisualState::Pressed,
            StylePatch::new()
                .set(property::Opacity, 0.72)
                .set(property::Transform, Transform2D::IDENTITY.scale(0.97, 0.97)),
        )
        .transition(StyleTransition::new(Transition::spring()))
}

fn directional_surface(theme: &WidgetTheme) -> Element {
    let fast = Transition::tween(Tween::new(Duration::from_millis(90)));
    let slow = Transition::tween(Tween::new(Duration::from_millis(320)));
    let transition =
        StyleTransition::new(Transition::tween(Tween::new(Duration::from_millis(160))))
            .rule(
                TransitionRule::new(fast)
                    .property(PropertyKey::Transform)
                    .direction(TransitionDirection::Enter(VisualState::Hovered.into())),
            )
            .rule(
                TransitionRule::new(slow)
                    .property(PropertyKey::Transform)
                    .direction(TransitionDirection::Exit(VisualState::Hovered.into())),
            );
    Element::container([label("Fast in · slow out", theme.primary_foreground)])
        .keyed("state-example-directional")
        .width(percent(1.0))
        .padding(sides(18.0, 14.0))
        .background(theme.primary)
        .radius(CornerRadii::all(10.0))
        .interaction(interactive())
        .when(
            VisualState::Hovered,
            StylePatch::new()
                .set(
                    property::Transform,
                    Transform2D::IDENTITY.translate(18.0, 0.0).rotate(0.025),
                )
                .set(property::Opacity, 0.78),
        )
        .when(
            VisualState::Focused,
            StylePatch::new().set(property::Border, Some(Border::all(2.0, theme.foreground))),
        )
        .transition(transition)
}

fn layout_surface(theme: &WidgetTheme) -> Element {
    Element::container([label("Hover to grow", theme.primary_foreground)])
        .keyed("state-example-layout")
        .width(length(164.0))
        .height(length(54.0))
        .padding(sides(14.0, 10.0))
        .background(theme.primary)
        .radius(CornerRadii::all(10.0))
        .interaction(interactive())
        .when(
            VisualState::Hovered,
            StylePatch::new()
                .set(property::WidthPx, 260.0)
                .set(property::PaddingLeft, 28.0)
                .set(property::PaddingRight, 28.0)
                .set(property::CornerRadii, [18.0; 4]),
        )
        .when(
            VisualState::Pressed,
            StylePatch::new().set(property::HeightPx, 46.0),
        )
        .transition(StyleTransition::new(Transition::tween(Tween::new(
            Duration::from_millis(220),
        ))))
}

fn gradient_surface(theme: &WidgetTheme) -> Element {
    let gradient = LinearGradient::new(
        Point::new(0.0, 0.0),
        Point::new(1.0, 1.0),
        [
            GradientStop::new(0.0, theme.primary),
            GradientStop::new(0.5, Color::rgb(0.18, 0.82, 0.74)),
            GradientStop::new(1.0, Color::rgb(0.74, 0.24, 0.92)),
        ],
    )
    .expect("showcase gradient is valid");
    Element::container([label("Animated gradient", Color::WHITE)])
        .keyed("state-example-gradient")
        .width(percent(1.0))
        .height(length(80.0))
        .padding(Sides::length(14.0))
        .fill(Fill::Linear(gradient))
        .radius(CornerRadii::all(10.0))
        .interaction(interactive())
        .when(
            VisualState::Hovered,
            StylePatch::new()
                .set(property::LinearGradientStart, Point::new(0.0, 1.0))
                .set(property::LinearGradientEnd, Point::new(1.0, 0.0))
                .set(property::gradient_stop_offset(1), 0.72)
                .set(
                    property::gradient_stop_color(0),
                    Color::rgb(0.98, 0.32, 0.18),
                )
                .set(property::CornerRadii, [24.0; 4]),
        )
        .transition(StyleTransition::new(Transition::tween(Tween::new(
            Duration::from_millis(280),
        ))))
}

fn scrollbar_surface(theme: &WidgetTheme) -> Element {
    let rows = (1..=10).map(|index| {
        Element::container([
            Element::text(format!("Retained row {index:02}")).text_style(text_style(
                14.0,
                theme.foreground,
                550,
                argui_text::TextWrap::None,
            )),
        ])
        .padding(sides(12.0, 9.0))
        .background(theme.muted)
        .border(Border::all(1.0, theme.border))
        .radius(CornerRadii::all(8.0))
        .interaction(Interaction::default())
        .when(
            VisualState::Hovered,
            StylePatch::new()
                .set(property::BorderColor, theme.primary)
                .set(property::CornerRadii, [14.0; 4])
                .set(
                    property::Transform,
                    Transform2D::IDENTITY.translate(5.0, 0.0),
                ),
        )
        .transition(StyleTransition::new(Transition::tween(Tween::new(
            Duration::from_millis(130),
        ))))
    });
    let scrollbar = ScrollbarStyle::new(
        ScrollbarPartStyle::new(
            QuadStyle::solid(theme.muted)
                .radius(CornerRadii::all(999.0))
                .opacity(0.45),
        )
        .when(
            VisualState::Hovered,
            StylePatch::new().set(property::Opacity, 1.0),
        )
        .transition(StyleTransition::new(Transition::tween(Tween::new(
            Duration::from_millis(160),
        )))),
        ScrollbarPartStyle::new(QuadStyle::solid(theme.primary).radius(CornerRadii::all(999.0)))
            .when(
                VisualState::Hovered,
                StylePatch::new()
                    .set(property::BackgroundColor, theme.foreground)
                    .set(property::CornerRadii, [5.0; 4]),
            )
            .when(
                VisualState::Pressed,
                StylePatch::new()
                    .set(property::BackgroundColor, theme.primary)
                    .set(property::Opacity, 0.72),
            )
            .transition(StyleTransition::new(Transition::tween(Tween::new(
                Duration::from_millis(180),
            )))),
    )
    .width(10.0)
    .insets(sides(5.0, 6.0))
    .min_thumb(30.0);
    Element::column(rows)
        .keyed("state-example-scrollbar")
        .width(percent(1.0))
        .height(length(170.0))
        .padding(Sides {
            left: length(4.0),
            right: length(20.0),
            top: length(4.0),
            bottom: length(4.0),
        })
        .gap(7.0)
        .overflow(Axes {
            x: Overflow::Hidden,
            y: Overflow::Auto,
        })
        .scroll_config(ScrollConfig::default().scrollbar(scrollbar))
}

fn example(title: &str, description: &str, content: Element, theme: &WidgetTheme) -> Element {
    Element::column([
        Element::text(title).text_style(text_style(
            15.0,
            theme.foreground,
            650,
            argui_text::TextWrap::Word,
        )),
        Element::text(description).text_style(text_style(
            13.0,
            theme.muted_foreground,
            400,
            argui_text::TextWrap::Word,
        )),
        content,
    ])
    .width(length(320.0))
    .padding(Sides::length(14.0))
    .gap(10.0)
    .background(theme.background)
    .border(Border::all(1.0, theme.border))
    .radius(CornerRadii::all(10.0))
}

fn label(text: &str, color: Color) -> Element {
    Element::text(text).text_style(text_style(15.0, color, 650, argui_text::TextWrap::None))
}

fn interactive() -> Interaction {
    Interaction::default()
        .focusable(true)
        .keyboard_activation(KeyboardActivation::EnterOrSpace)
}
