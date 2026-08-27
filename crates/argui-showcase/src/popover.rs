use argui_animation::{
    Duration, FillMode, Iterations, Keyframe, Keyframes, Spring, SpringConfig, Timeline, Timing,
};
use argui_core::Transform2D;
use argui_effects::{self as effects, AnimatedGradient, LiquidGlass, WorleyBorderFire};
use argui_paint::{Border, Color, CornerRadii, Filter, LayerMask, LayerStyle, Shadow};
use argui_runtime::EffectShader;
use argui_text::{TextColor, TextWrap};
use argui_ui::{
    Edges, Element, Interaction, Length, OverlayAlign, OverlayPlacement, PlacementSide,
    ScrollChaining, ScrollConfig, Wrap,
};

use super::{StateShowcase, button, chip, text_style};

pub(super) const EFFECT_SHADERS: &[EffectShader] = effects::SHADERS;

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
    pub(super) fn popover_demo(&self, accent: Color) -> Element {
        let mut children = vec![
            Element::row([
                button("popover-toggle", "Effects popover", accent),
                button("tooltip-anchor", "Hover for tooltip", accent),
            ])
            .wrap(Wrap::Wrap)
            .gap(10.0),
        ];
        children.push(self.popover_content(accent));
        if self.tooltip_visible {
            children.push(self.tooltip_content());
        }
        Element::container(children)
            .keyed("popover-anchor")
            .width(Length::Percent(1.0))
            .z_index(400)
    }

    fn popover_content(&self, accent: Color) -> Element {
        let progress = self.popover_progress.clamp(0.0, 1.0);
        let content = Element::column([
            Element::text("GPU effects popover")
                .text_style(text_style(
                    22.0,
                    TextColor::WHITE,
                    700,
                    TextWrap::Word,
                ))
                .text_effect(
                    LayerStyle::new(Default::default())
                        .filter(AnimatedGradient::new(self.effect_phase).filter()),
                ),
            Element::text(
                "A real composed overlay: nested clipping, backdrop blur, refraction and a continuously animated shadow.",
            )
            .text_style(text_style(
                15.0,
                TextColor::rgb(0.82, 0.88, 0.96),
                400,
                TextWrap::Word,
            )),
            Element::row([
                chip("popover-native", "Native WGPU"),
                chip("popover-wasm", "Same WASM tree"),
            ])
            .wrap(Wrap::Wrap)
            .gap(8.0),
            Element::text("Scrollable content constrained by the available viewport space.")
                .text_style(text_style(
                    14.0,
                    TextColor::rgb(0.76, 0.84, 0.94),
                    500,
                    TextWrap::Word,
                )),
            Element::column((0..12).map(|index| {
                Element::text(format!("Popover row {:02} · retained and clipped", index + 1))
                    .padding(Edges::symmetric(10.0, 7.0))
                    .background(if index % 2 == 0 {
                        Color::rgba(0.12, 0.18, 0.28, 0.72)
                    } else {
                        Color::rgba(0.08, 0.12, 0.20, 0.72)
                    })
                    .radius(CornerRadii::all(8.0))
                    .text_style(text_style(
                        13.0,
                        TextColor::rgb(0.82, 0.88, 0.96),
                        400,
                        TextWrap::Word,
                    ))
            }))
            .gap(5.0),
            button("popover-close", "Done", accent),
        ]);
        content
            .keyed("effects-popover")
            .width(Length::Px(400.0))
            .height(Length::Px(430.0))
            .padding(Edges::all(20.0))
            .gap(14.0)
            .background(Color::rgba(0.055, 0.075, 0.115, 0.82))
            .border(Border::all(1.0, Color::rgba(0.7, 0.88, 1.0, 0.62)))
            .radius(CornerRadii::all(20.0))
            .anchored_to(
                "popover-toggle",
                OverlayPlacement::new(PlacementSide::Bottom)
                    .align(OverlayAlign::End)
                    .gap(12.0)
                    .margin(14.0),
            )
            .z_index(500)
            .interaction(Interaction::blocker().enabled(progress > 0.0))
            .scrollable(
                ScrollConfig::default()
                    .enabled(progress > 0.0)
                    .chaining(ScrollChaining::Contain)
                    .scrollbar(self.scrollbar_style(accent)),
            )
            .transform(entry_transform(PlacementSide::Bottom, progress))
            .layer(self.popover_layer(progress))
    }

    fn tooltip_content(&self) -> Element {
        Element::text("Placed automatically after a 450 ms delay")
            .keyed("delayed-tooltip")
            .width(Length::Px(250.0))
            .height(Length::Px(54.0))
            .padding(Edges::symmetric(11.0, 8.0))
            .background(Color::rgba(0.025, 0.035, 0.06, 0.96))
            .border(Border::all(1.0, Color::rgba(0.6, 0.82, 1.0, 0.5)))
            .radius(CornerRadii::all(9.0))
            .text_style(text_style(13.0, TextColor::WHITE, 550, TextWrap::Word))
            .anchored_to(
                "tooltip-anchor",
                OverlayPlacement::new(PlacementSide::Top)
                    .gap(8.0)
                    .margin(10.0),
            )
            .z_index(700)
    }

    fn popover_layer(&self, progress: f32) -> LayerStyle {
        let [red, green, blue, _] = self.shadow_color.as_array();
        LayerStyle::new(Default::default())
            .opacity(progress)
            .filter(WorleyBorderFire::new(self.effect_phase).filter())
            .filter(Filter::Blur((1.0 - progress) * 5.0))
            .backdrop(
                LiquidGlass::new()
                    .refraction(9.0)
                    .chromatic_aberration(1.4)
                    .blur(3.5)
                    .highlight(0.24)
                    .edge_width(20.0)
                    .saturation(1.35)
                    .filter(),
            )
            .shadow(Shadow::glow(30.0, Color::rgba(red, green, blue, 0.28)))
            .mask(LayerMask::Rounded(CornerRadii::all(20.0)))
    }
}

fn entry_transform(side: PlacementSide, progress: f32) -> Transform2D {
    let distance = (1.0 - progress) * 14.0;
    let transform = Transform2D::IDENTITY.scale(0.96 + progress * 0.04, 0.96 + progress * 0.04);
    match side {
        PlacementSide::Top => transform.translate(0.0, distance),
        PlacementSide::Bottom => transform.translate(0.0, -distance),
        PlacementSide::Left => transform.translate(distance, 0.0),
        PlacementSide::Right => transform.translate(-distance, 0.0),
    }
}

pub(super) fn shadow_timeline(initial: Color) -> Timeline<Color> {
    let frames = Keyframes::new(vec![
        Keyframe::new(0.0, initial),
        Keyframe::new(0.25, Color::rgb(0.68, 0.28, 1.0)),
        Keyframe::new(0.5, Color::rgb(1.0, 0.30, 0.48)),
        Keyframe::new(0.75, Color::rgb(0.20, 0.92, 0.66)),
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
