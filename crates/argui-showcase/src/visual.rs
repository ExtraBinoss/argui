use argui_core::Point;
use argui_paint::{
    ClipBehavior, Color, CornerRadii, Fill, GradientStop, GradientStops, ImageAsset, ImageFit,
    ImageId, LinearGradient, RadialGradient,
};
use argui_text::{TextColor, TextWrap};
use argui_theme::WidgetTheme;
use argui_ui::{Edges, Element, Length, Transform2D, TransformOrigin, Wrap};

use argui_ui::property;

use super::{StateShowcase, text_style};

pub(super) struct ShowcaseImages {
    library: argui_image::ImageLibrary,
    beam: ImageId,
    pia: ImageId,
}

impl ShowcaseImages {
    pub(super) fn embedded() -> Self {
        let mut library = argui_image::ImageLibrary::new();
        let beam = library
            .insert(include_bytes!("../assets/images/beam_transparent.png"))
            .expect("embedded Beam PNG is valid");
        let pia = library
            .insert(include_bytes!("../assets/images/PIA09784-orig.jpg"))
            .expect("embedded PIA JPEG is valid");
        Self { library, beam, pia }
    }

    pub(super) fn assets(&self) -> &[ImageAsset] {
        self.library.assets()
    }
}

impl StateShowcase {
    pub(super) fn visual_primitives(&self, widgets: &WidgetTheme) -> Element {
        let accent = widgets.primary;
        let linear = LinearGradient::with_stops(
            Point::new(0.0, 0.0),
            Point::new(1.0, 1.0),
            many_stops(accent),
        );
        let radial = RadialGradient::new(
            Point::new(0.35, 0.3),
            Point::new(0.7, 0.8),
            [
                GradientStop::new(0.0, Color::WHITE),
                GradientStop::new(0.25, accent),
                GradientStop::new(1.0, Color::rgb(0.03, 0.05, 0.12)),
            ],
        )
        .expect("showcase gradient is valid");
        Element::column([
            Element::text("Transforms, gradients and images")
                .text_style(text_style(18.0, widgets.foreground, 650, TextWrap::Word)),
            Element::row([
                Element::container([Element::text("linear · 12 stops").text_style(text_style(
                    15.0,
                    TextColor::WHITE,
                    650,
                    TextWrap::None,
                ))])
                .padding(Edges::all(14.0))
                .width(Length::Px(210.0))
                .height(Length::Px(92.0))
                .fill(Fill::Linear(linear))
                .radius(CornerRadii::all(18.0))
                .transform(self.visual_target())
                .bind(property::Transform, self.visual_motion.clone())
                .transform_origin(TransformOrigin::new(0.2, 0.8)),
                Element::container([])
                    .width(Length::Px(170.0))
                    .height(Length::Px(92.0))
                    .fill(Fill::Radial(radial))
                    .radius(CornerRadii::all(18.0)),
                Element::image(self.images.beam)
                    .image_fit(ImageFit::Contain)
                    .width(Length::Px(170.0))
                    .height(Length::Px(92.0))
                    .radius(CornerRadii::all(18.0))
                    .transform(Transform2D::IDENTITY.rotate(0.045)),
                Element::image(self.images.pia)
                    .image_fit(ImageFit::Cover)
                    .width(Length::Px(170.0))
                    .height(Length::Px(92.0))
                    .radius(CornerRadii::all(18.0)),
            ])
            .wrap(Wrap::Wrap)
            .gap(22.0)
            .padding(Edges::all(12.0)),
            Element::text(
                "The same fills work on containers, buttons and composed widgets; transforms do not relayout Taffy.",
            )
            .text_style(text_style(
                14.0,
                widgets.muted_foreground,
                400,
                TextWrap::Word,
            )),
        ])
        .keyed("visual-primitives")
        .gap(12.0)
        .padding(Edges::all(16.0))
        .background(widgets.card)
        .border(argui_paint::Border::all(1.0, widgets.border))
        .clip(ClipBehavior::Bounds)
        .radius(CornerRadii::all(10.0))
    }
}

fn many_stops(accent: Color) -> GradientStops {
    let colors = [
        Color::rgb(0.05, 0.25, 0.88),
        Color::rgb(0.05, 0.65, 0.98),
        accent,
        Color::rgb(0.18, 0.88, 0.72),
        Color::rgb(0.70, 0.92, 0.20),
        Color::rgb(1.0, 0.70, 0.10),
        Color::rgb(1.0, 0.32, 0.12),
        Color::rgb(0.92, 0.14, 0.45),
        Color::rgb(0.68, 0.18, 0.92),
        Color::rgb(0.35, 0.20, 0.95),
        Color::rgb(0.12, 0.42, 0.95),
        Color::rgb(0.05, 0.25, 0.88),
    ];
    let last = (colors.len() - 1) as f32;
    GradientStops::from_vec(
        colors
            .into_iter()
            .enumerate()
            .map(|(index, color)| GradientStop::new(index as f32 / last, color))
            .collect(),
    )
    .expect("generated stops are sorted")
}
