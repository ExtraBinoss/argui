use argui_animation::{Duration, FillMode, Iterations, Keyframe, Keyframes, Timeline, Timing};
use argui_paint::{Border, Color, CornerRadii, Filter, LayerMask, LayerStyle, Refraction, Shadow};
use argui_text::{TextColor, TextWrap};
use argui_ui::{Edges, Element, Inset, Length, Wrap};

use super::{StateShowcase, button, chip, text_style};

impl StateShowcase {
    pub(super) fn popover_demo(&self, accent: Color) -> Element {
        let mut children = vec![button(
            "popover-toggle",
            if self.popover_open {
                "Close effects popover"
            } else {
                "Open effects popover"
            },
            accent,
        )];
        if self.popover_open {
            children.push(self.popover_content(accent));
        }
        Element::container(children)
            .keyed("popover-anchor")
            .width(Length::Percent(1.0))
            .z_index(400)
    }

    fn popover_content(&self, accent: Color) -> Element {
        Element::column([
            Element::text("GPU effects popover").text_style(text_style(
                22.0,
                TextColor::WHITE,
                700,
                TextWrap::Word,
            )),
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
            button("popover-close", "Done", accent),
        ])
        .keyed("effects-popover")
        .width(Length::Percent(0.92))
        .max_width(Length::Px(380.0))
        .padding(Edges::all(20.0))
        .gap(14.0)
        .background(Color::rgba(0.055, 0.075, 0.115, 0.82))
        .border(Border::all(1.0, Color::rgba(0.7, 0.88, 1.0, 0.62)))
        .radius(CornerRadii::all(20.0))
        .absolute(Inset::top_right(58.0, 22.0))
        .z_index(500)
        .layer(self.popover_layer())
    }

    fn popover_layer(&self) -> LayerStyle {
        let [red, green, blue, _] = self.shadow_color.as_array();
        LayerStyle::new(Default::default())
            .backdrop(Filter::Blur(11.0))
            .backdrop(Filter::Saturation(1.35))
            .backdrop(Filter::Refraction(
                Refraction::new(0.16).chromatic_aberration(0.06),
            ))
            .shadow(Shadow::glow(30.0, Color::rgba(red, green, blue, 0.28)))
            .mask(LayerMask::Rounded(CornerRadii::all(20.0)))
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
