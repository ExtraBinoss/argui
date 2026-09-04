use argui_animation::{
    Duration, FillMode, Iterations, Keyframe, Keyframes, Spring, SpringConfig, Timeline, Timing,
};
use argui_core::Transform2D;
use argui_paint::{Border, Color, CornerRadii, Filter, LayerMask, LayerStyle, Shadow};
use argui_text::TextWrap;
use argui_ui::{
    Axes, Element, FlexWrap, FloatingPlacement, FocusScope, InitialFocus, Interaction, Overflow,
    Placement, ScrollConfig, ScrollPropagation, Sides, WindowLayer, length, percent, property,
    sides,
};
use argui_widgets::WidgetTheme;

use super::{StateShowcase, button, chip, text_style};

pub(super) fn popover_spring() -> Spring<f32> {
    Spring::new(
        0.0,
        0.0,
        0.0,
        SpringConfig {
            mass: 1.0,
            stiffness: 480.0,
            damping: 40.0,
            rest_speed: 0.004,
            rest_delta: 0.002,
        },
    )
    .expect("popover spring is physical")
}

impl StateShowcase {
    pub(super) fn popover_demo(&self, widgets: &WidgetTheme) -> Element {
        let mut children = vec![
            Element::row([
                button("popover-toggle", "Effects popover", widgets, false),
                button("tooltip-anchor", "Hover for tooltip", widgets, false),
            ])
            .flex_wrap(FlexWrap::Wrap)
            .gap(10.0),
        ];
        if self.popover_progress > 0.0 {
            children.push(self.popover_content(widgets));
        }
        if self.tooltip_visible {
            children.push(self.tooltip_content(widgets));
        }
        Element::container(children)
            .keyed("popover-anchor")
            .width(percent(1.0))
            .z_index(400)
    }

    fn popover_content(&self, widgets: &WidgetTheme) -> Element {
        let progress = self.popover_progress.clamp(0.0, 1.0);
        let content = Element::column([
            Element::text("GPU effects popover")
                .text_style(text_style(
                    22.0,
                    widgets.foreground,
                    700,
                    TextWrap::Word,
                )),
            Element::text(
                "A composed overlay with nested clipping, constant backdrop blur and a continuous shadow.",
            )
            .text_style(text_style(
                15.0,
                widgets.muted_foreground,
                400,
                TextWrap::Word,
            )),
            Element::row([
                chip("popover-native", "Native WGPU", widgets),
                chip("popover-wasm", "Same WASM tree", widgets),
            ])
            .flex_wrap(FlexWrap::Wrap)
            .gap(8.0),
            Element::text("Scrollable content constrained by the available viewport space.")
                .text_style(text_style(
                    14.0,
                    widgets.muted_foreground,
                    500,
                    TextWrap::Word,
                )),
            Element::column((0..12).map(|index| {
                Element::text(format!("Popover row {:02} · retained and clipped", index + 1))
                    .padding(sides(10.0, 7.0))
                    .background(if index % 2 == 0 {
                        widgets.muted
                    } else {
                        widgets.card
                    })
                    .radius(CornerRadii::all(8.0))
                    .text_style(text_style(
                        13.0,
                        widgets.foreground,
                        400,
                        TextWrap::Word,
                    ))
            }))
            .gap(5.0),
            button("popover-close", "Done", widgets, true),
        ]);
        content
            .keyed("effects-popover")
            .focus_scope(FocusScope::trapped(InitialFocus::Target(
                "popover-close".into(),
            )))
            .width(length(400.0))
            .height(length(430.0))
            .padding(Sides::length(20.0))
            .gap(14.0)
            .background(widgets.card)
            .border(Border::all(1.0, widgets.border))
            .radius(CornerRadii::all(20.0))
            .anchored_portal(
                WindowLayer::Popover,
                "popover-toggle",
                FloatingPlacement::new(Placement::BottomEnd)
                    .offset(12.0)
                    .viewport_padding(14.0),
            )
            .z_index(500)
            .interaction(Interaction::blocker().enabled(progress > 0.0))
            .overflow(Axes {
                x: Overflow::Hidden,
                y: Overflow::Auto,
            })
            .scroll_config(
                ScrollConfig::default()
                    .enabled(progress > 0.0)
                    .propagation(ScrollPropagation::Contain)
                    .scrollbar(self.scrollbar_style(widgets)),
            )
            .transform(entry_transform(progress))
            .layer(self.popover_layer(progress))
            .bind(property::shadow_color(0), self.shadow_motion.clone())
    }

    fn tooltip_content(&self, widgets: &WidgetTheme) -> Element {
        Element::text("Placed automatically after a 450 ms delay")
            .keyed("delayed-tooltip")
            .width(length(250.0))
            .height(length(54.0))
            .padding(sides(11.0, 8.0))
            .background(widgets.card)
            .border(Border::all(1.0, widgets.border))
            .radius(CornerRadii::all(9.0))
            .text_style(text_style(13.0, widgets.foreground, 550, TextWrap::Word))
            .anchored_portal(
                WindowLayer::Popover,
                "tooltip-anchor",
                FloatingPlacement::new(Placement::Top)
                    .offset(8.0)
                    .viewport_padding(10.0),
            )
            .z_index(700)
    }

    fn popover_layer(&self, progress: f32) -> LayerStyle {
        LayerStyle::new(Default::default())
            .filter(Filter::Blur((1.0 - progress) * 5.0))
            .backdrop(Filter::Blur(14.0))
            .shadow(Shadow::glow(30.0, self.shadow_motion.value()))
            .mask(LayerMask::Rounded(CornerRadii::all(20.0)))
    }
}

fn entry_transform(progress: f32) -> Transform2D {
    let distance = (1.0 - progress) * 14.0;
    let transform = Transform2D::IDENTITY.scale(0.96 + progress * 0.04, 0.96 + progress * 0.04);
    transform.translate(0.0, -distance)
}

pub(super) fn shadow_timeline(initial: Color) -> Timeline<Color> {
    let frames = Keyframes::new(vec![
        Keyframe::new(0.0, initial),
        Keyframe::new(0.25, Color::srgba(0.68, 0.28, 1.0, 0.28)),
        Keyframe::new(0.5, Color::srgba(1.0, 0.30, 0.48, 0.28)),
        Keyframe::new(0.75, Color::srgba(0.20, 0.92, 0.66, 0.28)),
        Keyframe::new(1.0, initial),
    ])
    .expect("the popover shadow keyframes are sorted and complete");
    Timeline::new(
        frames,
        Timing::new(Duration::from_millis(2_800))
            .iterations(Iterations::Infinite)
            .fill(FillMode::Both),
    )
    .expect("the popover shadow timing is valid")
}
