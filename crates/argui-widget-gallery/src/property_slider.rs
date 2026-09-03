use argui::{
    core::{Color, Key, KeyState, Transform2D, TransformOrigin},
    paint::{Border, CornerRadii, PaintStyle, QuadStyle},
    runtime::{Context, LayoutSnapshot},
    text::{TextAlign, TextStyle, TextWrap},
    ui::{
        AlignItems, Axes, Element, JustifyContent, Overflow, Sides, StateSelector, StylePatch,
        StyleTransition, TextSelection, UiEvent, UiEventKind, VisualState, auto, length, percent,
        property,
    },
    widgets::{
        ButtonBehavior, ButtonPart, Input, InputKind, RANGE_SCOPE, RangeBehavior, RangeConfig,
        RangeDetents, RangePart, TablerIcon, WidgetAssets, WidgetTheme,
    },
};

use crate::app::{WidgetGallery, text};

const KEY: &str = "property-slider";
const EDIT_KEY: &str = "property-slider::edit";
const INPUT_KEY: &str = "property-slider::input";
const RESET_KEY: &str = "property-slider::reset";
const DEFAULT_VALUE: f32 = 50.0;

pub(crate) fn render(
    gallery: &WidgetGallery,
    theme: &WidgetTheme,
    assets: &WidgetAssets,
) -> Element {
    let behavior = behavior(gallery);
    let ratio = behavior.ratio();
    let fill = Element::container([])
        .absolute(Sides::length(0.0))
        .width(percent(1.0))
        .height(percent(1.0))
        .background(theme.primary.with_alpha(0.28));
    let ticks = Element::row((0..9).map(|_| {
        Element::container([])
            .width(length(1.0))
            .height(length(8.0))
            .paint_style(PaintStyle::new(
                QuadStyle::solid(theme.foreground).opacity(0.0),
            ))
            .when(
                StateSelector::scope(RANGE_SCOPE, VisualState::Hovered),
                StylePatch::new().set(property::Opacity, 0.42),
            )
            .when(
                StateSelector::scope(RANGE_SCOPE, VisualState::Pressed),
                StylePatch::new().set(property::Opacity, 0.62),
            )
            .transition(StyleTransition::default())
    }))
    .absolute(Sides {
        left: length(14.0),
        right: length(14.0),
        top: auto(),
        bottom: auto(),
    })
    .height(percent(1.0))
    .align_items(AlignItems::CENTER)
    .justify_content(JustifyContent::SPACE_BETWEEN)
    .semantic_hidden(true);
    let thumb = Element::container([])
        .absolute(Sides {
            left: auto(),
            right: length(-1.0),
            top: length(4.0),
            bottom: length(4.0),
        })
        .width(length(2.0))
        .height(auto())
        .paint_style(PaintStyle::new(
            QuadStyle::solid(theme.foreground)
                .opacity(0.32)
                .radius(CornerRadii::all(1.0)),
        ))
        .transform(Transform2D::IDENTITY.scale(0.75, 0.72))
        .transform_origin(TransformOrigin::CENTER)
        .when(
            StateSelector::scope(RANGE_SCOPE, VisualState::Hovered),
            StylePatch::new()
                .set(property::Opacity, 1.0)
                .set(property::Transform, Transform2D::IDENTITY),
        )
        .when(
            StateSelector::scope(RANGE_SCOPE, VisualState::Pressed),
            StylePatch::new()
                .set(property::Opacity, 1.0)
                .set(property::Transform, Transform2D::IDENTITY.scale(2.0, 1.0)),
        )
        .transition(StyleTransition::default())
        .semantic_hidden(true);
    let progress = Element::container([fill, thumb])
        .absolute(Sides {
            left: length(0.0),
            right: auto(),
            top: length(0.0),
            bottom: length(0.0),
        })
        .width(percent(ratio))
        .height(percent(1.0))
        .semantic_hidden(true);
    let track = behavior.decorate(
        RangePart::Track,
        Element::container([progress, ticks])
            .width(percent(1.0))
            .height(percent(1.0)),
    );
    let control = behavior.decorate(
        RangePart::Control,
        Element::container([track])
            .absolute(Sides::length(0.0))
            .width(percent(1.0))
            .height(percent(1.0)),
    );
    let value = if gallery.slider_editing {
        let mut style = theme.input.clone();
        style.layout.padding = argui::ui::sides(8.0, 5.0);
        style.text.align = TextAlign::End;
        style.placeholder.align = TextAlign::End;
        Input::new(INPUT_KEY, &gallery.slider_edit_value, "Value", style)
            .kind(InputKind::Arithmetic)
            .label("Render scale value")
            .build()
            .width(length(82.0))
            .height(length(30.0))
    } else {
        action(
            EDIT_KEY,
            "Edit render scale",
            Element::text(format!("{:.0}%", gallery.slider)).text_style(TextStyle {
                font_size: 12.0,
                line_height: 18.0,
                color: theme.foreground,
                weight: 650,
                wrap: TextWrap::None,
                ..TextStyle::default()
            }),
            theme,
        )
    };
    let mut actions = Vec::new();
    if (gallery.slider - DEFAULT_VALUE).abs() > 0.001 && !gallery.slider_editing {
        actions.push(action(
            RESET_KEY,
            "Reset render scale",
            assets.icon(TablerIcon::Restore, 14.0),
            theme,
        ));
    }
    actions.push(value);
    let actions = Element::row(actions)
        .absolute(Sides {
            left: auto(),
            right: length(6.0),
            top: length(4.0),
            bottom: auto(),
        })
        .height(length(30.0))
        .gap(3.0)
        .align_items(AlignItems::CENTER);
    let root = behavior.decorate(
        RangePart::Root,
        Element::container([control, actions])
            .width(percent(1.0))
            .height(length(38.0))
            .background(theme.card)
            .border(Border::all(1.0, theme.border))
            .radius(CornerRadii::all(9.0))
            .overflow(Axes {
                x: Overflow::Hidden,
                y: Overflow::Hidden,
            }),
    );
    Element::row([
        text("Scale", 11.0, theme.muted_foreground, 700).width(length(92.0)),
        root.grow(1.0),
    ])
    .width(percent(1.0))
    .gap(12.0)
    .align_items(AlignItems::CENTER)
}

pub(crate) fn update(
    gallery: &mut WidgetGallery,
    event: &UiEvent,
    cx: &mut Context<WidgetGallery>,
) -> bool {
    if event.key.as_deref() == Some(INPUT_KEY) {
        match &event.kind {
            UiEventKind::TextChanged(value) => gallery.slider_edit_value.clone_from(value),
            UiEventKind::Submitted(_) | UiEventKind::Blurred => finish_edit(gallery, cx),
            UiEventKind::KeyInput(input)
                if input.state == KeyState::Pressed && input.key == Key::Escape =>
            {
                gallery.slider_editing = false;
                cx.request_focus(KEY);
            }
            _ => return false,
        }
        cx.notify();
        return true;
    }
    if event.kind == UiEventKind::Clicked && event.key.as_deref() == Some(EDIT_KEY) {
        gallery.slider_editing = true;
        gallery.slider_edit_value = format!("{:.0}", gallery.slider);
        cx.request_focus(INPUT_KEY);
        cx.select_text(INPUT_KEY, TextSelection::All);
        cx.notify();
        return true;
    }
    if event.kind == UiEventKind::Clicked && event.key.as_deref() == Some(RESET_KEY) {
        gallery.slider = DEFAULT_VALUE;
        cx.notify();
        return true;
    }
    let behavior = behavior(gallery);
    if let Some(action) = gallery.slider_state.update(event, &behavior) {
        gallery.slider = action.value();
        gallery.slider_edit_value = format!("{:.0}", gallery.slider);
        cx.notify();
        return true;
    }
    false
}

pub(crate) fn layout_changed(gallery: &mut WidgetGallery, layout: &LayoutSnapshot) {
    let behavior = behavior(gallery);
    gallery.slider_state.layout_changed(layout, &behavior);
}

fn behavior(gallery: &WidgetGallery) -> RangeBehavior {
    let config = RangeConfig::default().detents(RangeDetents::new(10.0, 2.0, 220.0));
    RangeBehavior::new(KEY, "Render scale", gallery.slider, config).enabled(!gallery.slider_editing)
}

fn finish_edit(gallery: &mut WidgetGallery, cx: &mut Context<WidgetGallery>) {
    if let Some(value) = crate::numeric_expression::evaluate(&gallery.slider_edit_value) {
        gallery.slider = RangeConfig::default().clamp(value);
    }
    gallery.slider_edit_value = format!("{:.0}", gallery.slider);
    gallery.slider_editing = false;
    cx.request_focus(KEY);
}

fn action(key: &str, label: &str, content: Element, theme: &WidgetTheme) -> Element {
    ButtonBehavior::new(key, label).decorate(
        ButtonPart::Root,
        Element::row([content])
            .height(length(30.0))
            .padding(argui::ui::sides(6.0, 3.0))
            .align_items(AlignItems::CENTER)
            .justify_content(JustifyContent::CENTER)
            .paint_style(PaintStyle::new(
                QuadStyle::solid(Color::TRANSPARENT).radius(CornerRadii::all(6.0)),
            ))
            .when(
                VisualState::Hovered,
                QuadStyle::solid(theme.muted)
                    .radius(CornerRadii::all(6.0))
                    .into(),
            )
            .transition(StyleTransition::default()),
    )
}
