use argui::{
    core::{ColorInterpolation, Transform2D, TransformOrigin},
    paint::{Border, CornerRadii, LayerStyle, Shadow, VectorAsset, VectorId},
    runtime::{Context, LayoutSnapshot, Render},
    text::TextStyle,
    ui::{
        AlignItems, Axes, Element, FlexWrap, JustifyContent, Overflow, ScrollConfig, Sides, length,
        percent,
    },
    vector::VectorLibrary,
    widgets::{Button, WidgetTheme, default_theme},
};

const ORBIT_SVG: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
  <circle cx="12" cy="12" r="8" fill="none" stroke="currentColor" stroke-width="3"/>
  <circle cx="12" cy="4" r="2.5" fill="currentColor"/>
</svg>"#;
const SPARK_SVG: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
  <path fill="currentColor" d="M12 1.5l2.7 7.1 7.3 2.7-7.3 2.7L12 21.5 9.3 14 2 11.3l7.3-2.7z"/>
</svg>"#;

pub struct Example {
    elapsed: f32,
    running: bool,
    reduced_motion: bool,
    compact: bool,
    vectors: VectorLibrary,
    orbit: VectorId,
    spark: VectorId,
}

impl Default for Example {
    fn default() -> Self {
        let mut vectors = VectorLibrary::new();
        let orbit = vectors.insert_svg(ORBIT_SVG).expect("valid orbit SVG");
        let spark = vectors.insert_svg(SPARK_SVG).expect("valid spark SVG");
        Self {
            elapsed: 0.0,
            running: true,
            reduced_motion: false,
            compact: false,
            vectors,
            orbit,
            spark,
        }
    }
}

impl Example {
    fn text(
        &self,
        value: impl Into<String>,
        size: f32,
        weight: u16,
        color: argui::core::Color,
    ) -> Element {
        Element::text(value.into()).text_style(TextStyle {
            color,
            font_size: size,
            line_height: size * 1.35,
            weight,
            ..TextStyle::default()
        })
    }

    fn stage(&self, art: Element, theme: &WidgetTheme) -> Element {
        Element::container([art])
            .width(percent(1.0))
            .height(length(132.0))
            .padding(Sides::length(16.0))
            .align_items(AlignItems::CENTER)
            .justify_content(JustifyContent::CENTER)
            .background(theme.muted)
            .border(Border::all(1.0, theme.border))
            .radius(CornerRadii::all(12.0))
            .overflow(Axes {
                x: Overflow::Hidden,
                y: Overflow::Hidden,
            })
    }

    fn card(
        &self,
        title: &'static str,
        detail: &'static str,
        art: Element,
        theme: &WidgetTheme,
    ) -> Element {
        Element::column([
            self.text(title, 16.0, 700, theme.foreground),
            self.text(detail, 12.0, 450, theme.muted_foreground),
            self.stage(art, theme),
        ])
        .width(if self.compact {
            percent(1.0)
        } else {
            length(330.0)
        })
        .grow(1.0)
        .padding(Sides::length(16.0))
        .gap(9.0)
        .background(theme.card)
        .border(Border::all(1.0, theme.border))
        .radius(CornerRadii::all(14.0))
    }

    fn cards(&self, theme: &WidgetTheme) -> Element {
        let cycle = if self.reduced_motion {
            0.35
        } else {
            (self.elapsed / 2.0).fract()
        };
        let angle = cycle * std::f32::consts::TAU;
        let wave = angle.sin() * 0.5 + 0.5;
        let travel = angle.sin() * if self.compact { 52.0 } else { 82.0 };
        let combined = Element::container([])
            .width(length(58.0))
            .height(length(58.0))
            .background(theme.primary)
            .radius(CornerRadii::all(12.0 + wave * 12.0))
            .transform(
                Transform2D::IDENTITY
                    .translate(travel, 0.0)
                    .rotate(angle)
                    .scale(0.78 + wave * 0.42, 0.78 + wave * 0.42),
            )
            .transform_origin(TransformOrigin::CENTER);

        let aura = self
            .text("Production ready", 17.0, 700, theme.foreground)
            .padding(Sides::length(18.0))
            .background(theme.card)
            .border(Border::all(1.0 + wave * 2.0, theme.primary))
            .radius(CornerRadii::all(12.0))
            .layer(
                LayerStyle::new(Default::default()).shadow(
                    Shadow::glow(
                        10.0 + wave * 24.0,
                        theme.primary.with_alpha(0.18 + wave * 0.28),
                    )
                    .spread(wave * 4.0),
                ),
            );

        let morph_color = theme
            .primary
            .mix(theme.destructive, wave, ColorInterpolation::Oklab);
        let morph = Element::container([])
            .width(length(76.0 + wave * 112.0))
            .height(length(58.0 + wave * 28.0))
            .background(morph_color)
            .radius(CornerRadii {
                top_left: 8.0 + wave * 34.0,
                top_right: 38.0 - wave * 28.0,
                bottom_right: 8.0 + wave * 34.0,
                bottom_left: 38.0 - wave * 28.0,
            });

        let orbit = Element::vector(self.orbit)
            .vector_color(theme.primary)
            .paint_opacity(1.0 - wave)
            .width(length(72.0))
            .height(length(72.0))
            .transform(
                Transform2D::IDENTITY
                    .rotate(angle)
                    .scale(1.0 - wave * 0.28, 1.0 - wave * 0.28),
            );
        let spark = Element::vector(self.spark)
            .vector_color(theme.destructive)
            .paint_opacity(wave)
            .width(length(72.0))
            .height(length(72.0))
            .transform(
                Transform2D::IDENTITY
                    .rotate(-angle * 0.35)
                    .scale(0.72 + wave * 0.28, 0.72 + wave * 0.28),
            );
        let vector_transition = Element::container([
            orbit.absolute(Sides::length(0.0)),
            spark.absolute(Sides::length(0.0)),
        ])
        .width(length(72.0))
        .height(length(72.0));

        let loader = Element::row((0..5).map(|index| {
            let local =
                ((cycle + index as f32 * 0.14).fract() * std::f32::consts::TAU).sin() * 0.5 + 0.5;
            Element::container([])
                .width(length(12.0))
                .height(length(28.0 + local * 48.0))
                .background(
                    theme
                        .primary
                        .mix(theme.destructive, local, ColorInterpolation::Oklab),
                )
                .radius(CornerRadii::all(999.0))
        }))
        .height(length(84.0))
        .gap(9.0)
        .align_items(AlignItems::CENTER);

        let pulse = 0.82 + wave * 0.18;
        let notification = Element::row([
            Element::container([])
                .width(length(12.0))
                .height(length(12.0))
                .background(theme.primary)
                .radius(CornerRadii::all(999.0))
                .layer(LayerStyle::new(Default::default()).shadow(Shadow::glow(
                    6.0 + wave * 18.0,
                    theme.primary.with_alpha(0.34),
                ))),
            self.text("Sync complete", 15.0, 650, theme.foreground),
        ])
        .padding(Sides::length(16.0))
        .gap(10.0)
        .align_items(AlignItems::CENTER)
        .background(theme.card)
        .border(Border::all(1.0, theme.border))
        .radius(CornerRadii::all(999.0))
        .transform(Transform2D::IDENTITY.scale(pulse, pulse));

        Element::row([
            self.card(
                "Composed transform",
                "Translate, rotate, scale, and radius run in one two-second loop.",
                combined,
                theme,
            ),
            self.card(
                "Text aura",
                "A practical focus treatment combines border width and GPU glow.",
                aura,
                theme,
            ),
            self.card(
                "Layout morph",
                "Size, asymmetric radii, and Oklab color interpolate together.",
                morph,
                theme,
            ),
            self.card(
                "SVG transition",
                "Two retained vector assets crossfade, rotate, and scale.",
                vector_transition,
                theme,
            ),
            self.card(
                "Sequenced loader",
                "Phase offsets create a continuous multi-element loading rhythm.",
                loader,
                theme,
            ),
            self.card(
                "Status pulse",
                "A restrained scale and aura draw attention without moving layout.",
                notification,
                theme,
            ),
        ])
        .width(percent(1.0))
        .padding(Sides {
            left: length(16.0),
            right: length(16.0),
            top: length(16.0),
            bottom: length(28.0),
        })
        .gap(12.0)
        .flex_wrap(FlexWrap::Wrap)
    }
}

impl Render for Example {
    fn wants_animation_frame(&self) -> bool {
        self.running && !self.reduced_motion
    }

    fn animation_frame(&mut self, frame: argui::animation::Frame, cx: &mut Context<Self>) {
        if self.running && !self.reduced_motion {
            self.elapsed = (self.elapsed + frame.elapsed.as_secs_f64().min(0.05) as f32) % 2.0;
            cx.notify();
        }
    }

    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let environment = cx.environment();
        self.reduced_motion = environment.reduced_motion;
        let themes = default_theme(&environment);
        let theme = themes.resolve(environment.color_scheme);
        let status = if self.reduced_motion {
            "Reduced motion is active"
        } else if self.running {
            "Six loops · 2 seconds"
        } else {
            "Animations paused"
        };
        let controls = Element::row([
            Element::column([
                self.text("Animation lab", 21.0, 760, theme.foreground),
                self.text(status, 12.0, 500, theme.muted_foreground),
            ])
            .gap(2.0)
            .grow(1.0),
            Button::new(
                "toggle-animation",
                if self.running {
                    "Pause animations"
                } else {
                    "Resume animations"
                },
                theme.button(),
            )
            .enabled(!self.reduced_motion)
            .on_click(cx.callback(|app| app.running = !app.running))
            .build(),
        ])
        .width(percent(1.0))
        .padding(Sides::length(14.0))
        .gap(12.0)
        .flex_wrap(FlexWrap::Wrap)
        .align_items(AlignItems::CENTER)
        .background(theme.card)
        .border(Border {
            widths: argui::paint::BorderWidths {
                bottom: 1.0,
                ..argui::paint::BorderWidths::default()
            },
            color: theme.border,
        });
        let gallery = self
            .cards(theme)
            .keyed("animation-gallery")
            .grow(1.0)
            .min_height(length(0.0))
            .overflow(Axes {
                x: Overflow::Hidden,
                y: Overflow::Auto,
            })
            .scroll_config(ScrollConfig::default().scrollbar(theme.scrollbar.clone()));

        Element::column([controls, gallery])
            .width(percent(1.0))
            .height(percent(1.0))
            .min_height(length(0.0))
            .background(theme.background)
    }

    fn layout_changed(&mut self, layout: &LayoutSnapshot, cx: &mut Context<Self>) {
        let compact = layout.viewport_size().width < 620.0;
        if self.compact != compact {
            self.compact = compact;
            cx.notify();
        }
    }

    fn vector_assets(&self) -> Vec<VectorAsset> {
        self.vectors.assets().to_vec()
    }
}
