mod controls;

use crate::app::text;
use argui::{
    core::{Color, ColorScheme, Point, Rect},
    paint::{
        Border, ColorInterpolation, CornerRadii, Fill, GradientStop, LayerMask, LayerStyle,
        LinearGradient, QuadStyle, Shadow,
    },
    runtime::{Context, LayoutSnapshot, Render},
    ui::{
        AlignItems, Axes, Element, EventType, FlexWrap, JustifyContent, Overflow, Role,
        ScrollConfig, Semantics, Sides, StylePatch, UiEventKind, auto, length, percent,
    },
    widgets::{Button, RangeState, TablerIcon, WidgetAssets, WidgetTheme, shadcn},
};
use argui_effects::LiquidGlass;

const INK: Color = Color::linear_rgb(0.0072, 0.0100, 0.0272);
const TABS: [(&str, &str, TablerIcon); 3] = [
    ("glass-explore", "Explore", TablerIcon::Search),
    ("glass-collections", "Collections", TablerIcon::Copy),
    ("glass-saved", "Saved", TablerIcon::Check),
];
const CARDS: [(&str, &str, [u8; 3], [u8; 3]); 8] = [
    (
        "Golden hour",
        "Warm tones · 12 colors",
        [255, 182, 66],
        [255, 92, 91],
    ),
    (
        "Electric blue",
        "Cool tones · 8 colors",
        [70, 209, 245],
        [84, 112, 245],
    ),
    (
        "Orchid garden",
        "Pastels · 16 colors",
        [235, 155, 248],
        [165, 122, 247],
    ),
    (
        "Freshly picked",
        "Nature · 10 colors",
        [207, 237, 114],
        [55, 203, 162],
    ),
    (
        "Cherry soda",
        "Bold tones · 6 colors",
        [255, 149, 161],
        [245, 87, 136],
    ),
    (
        "Coastal escape",
        "Summer · 14 colors",
        [100, 223, 211],
        [84, 173, 240],
    ),
    (
        "Peach sorbet",
        "Soft tones · 9 colors",
        [255, 217, 144],
        [252, 157, 127],
    ),
    (
        "After hours",
        "Nightfall · 11 colors",
        [168, 155, 255],
        [107, 136, 227],
    ),
];

pub(crate) struct GlassDemo {
    effect: LiquidGlass,
    ranges: [RangeState; controls::COUNT],
    tint: usize,
    enabled: bool,
    tab: usize,
    icons: WidgetAssets,
}

impl GlassDemo {
    const DEFAULT_EFFECT: LiquidGlass = LiquidGlass::new();

    pub(crate) fn new(icons: &WidgetAssets) -> Self {
        Self {
            effect: Self::DEFAULT_EFFECT,
            ranges: [RangeState::default(); controls::COUNT],
            tint: 0,
            enabled: true,
            tab: 0,
            icons: icons.clone(),
        }
    }
}

impl GlassDemo {
    fn feed(&self, theme: &WidgetTheme) -> Element {
        let (title, description) = [
            (
                "A little more color.",
                "Fresh palettes for everyday inspiration.",
            ),
            (
                "Made to go together.",
                "Color stories, curated into collections.",
            ),
            (
                "Keep the good ones.",
                "Your saved palettes, all in one place.",
            ),
        ][self.tab];
        let cards = CARDS
            .iter()
            .enumerate()
            .filter(|(index, _)| self.tab != 2 || index % 2 == 0)
            .map(|(index, (title, description, from, to))| {
                let color = |rgb: [u8; 3]| Color::from_srgb8(rgb[0], rgb[1], rgb[2]);
                let gradient = LinearGradient::new(
                    Point::default(),
                    Point::new(1.0, 1.0),
                    ColorInterpolation::Srgb,
                    [
                        GradientStop::new(0.0, color(*from)),
                        GradientStop::new(1.0, color(*to)),
                    ],
                )
                .expect("ordered palette stops");
                let swatches = Element::row((0..6).map(|step| {
                    let t = step as f32 / 5.0;
                    let rgb = std::array::from_fn(|i| {
                        (f32::from(from[i]) * (1.0 - t) + f32::from(to[i]) * t) as u8
                    });
                    Element::container([])
                        .grow(1.0)
                        .height(length(24.0))
                        .background(color(rgb))
                        .radius(CornerRadii::all(8.0))
                        .border(Border::all(1.0, Color::srgba(1.0, 1.0, 1.0, 0.4)))
                }))
                .gap(5.0);
                Element::column([
                    Element::row([
                        text(format!("{:02}", index + 1), 12.0, INK, 600),
                        text(
                            if self.tab == 1 {
                                "COLLECTION"
                            } else {
                                "COLOR STUDY"
                            },
                            10.0,
                            INK,
                            600,
                        ),
                    ])
                    .justify_content(JustifyContent::SPACE_BETWEEN),
                    Element::column([
                        text(*title, 23.0, INK, 700),
                        text(*description, 12.0, INK, 450),
                    ])
                    .gap(4.0),
                    swatches,
                ])
                .keyed(format!("glass-card-{index}"))
                .semantics(Semantics::new(Role::Group).label(*title))
                .height(length(170.0))
                .shrink(0.0)
                .padding(Sides::length(20.0))
                .justify_content(JustifyContent::SPACE_BETWEEN)
                .fill(Fill::Linear(gradient))
                .radius(CornerRadii::all(22.0))
            });
        Element::column([
            Element::column([
                text("CHROMATIC", 11.0, INK, 700),
                text(title, 28.0, INK, 700).keyed("glass-feed-title"),
                text(description, 12.0, INK, 400),
            ])
            .gap(8.0)
            .padding(Sides {
                bottom: length(8.0),
                ..Sides::length(0.0)
            }),
            Element::column(cards).gap(14.0),
            text("You're all caught up.", 12.0, INK, 500),
        ])
        .keyed("liquid-glass-feed")
        .semantics(Semantics::new(Role::Group).label("Scrollable palettes"))
        .height(percent(1.0))
        .width(percent(1.0))
        .min_height(length(0.0))
        .padding(Sides {
            left: length(18.0),
            right: length(18.0),
            top: length(24.0),
            bottom: length(116.0),
        })
        .gap(14.0)
        .overflow(Axes {
            x: Overflow::Hidden,
            y: Overflow::Auto,
        })
        .scroll_config(ScrollConfig::default().scrollbar(theme.scrollbar.clone()))
    }

    fn navbar(&self, theme: &WidgetTheme, cx: &mut Context<Self>) -> Element {
        let dark = cx.environment().color_scheme == ColorScheme::Dark;
        let foreground = if self.tint == 1 || !dark {
            INK
        } else {
            Color::WHITE
        };
        let buttons = TABS
            .into_iter()
            .enumerate()
            .map(|(index, (key, label, icon))| {
                let selected = self.tab == index;
                let mut style = theme.ghost_button();
                let quad = QuadStyle::solid(Color::srgba(
                    0.0,
                    0.0,
                    0.0,
                    if selected {
                        if dark { 0.352 } else { 0.092 }
                    } else {
                        0.0
                    },
                ))
                .radius(CornerRadii::all(28.0));
                style.paint.quad = quad.clone();
                style.hovered = StylePatch::from_quad(
                    QuadStyle::solid(Color::srgba(0.0, 0.0, 0.0, if dark { 0.4 } else { 0.14 }))
                        .radius(CornerRadii::all(28.0)),
                );
                style.pressed = StylePatch::from_quad(quad);
                Button::new(key, label, style)
                    .without_tooltip()
                    .content(
                        Element::column([
                            self.icons.icon(icon, 21.0).vector_color(foreground),
                            text(label, 10.0, foreground, if selected { 700 } else { 500 }),
                        ])
                        .align_items(AlignItems::CENTER)
                        .gap(4.0),
                    )
                    .build()
                    .grow(1.0)
                    .shrink(1.0)
                    .min_width(length(0.0))
                    .height(percent(1.0))
                    .padding(Sides::length(4.0))
                    .on(cx.listener(EventType::Click, move |demo, event, cx| {
                        if matches!(event.kind, UiEventKind::Click(_)) {
                            demo.tab = index;
                            cx.notify();
                        }
                    }))
            });
        let mut bar = Element::row(buttons)
            .keyed("liquid-glass-pane")
            .semantics(Semantics::new(Role::Group).label("Glass navigation"))
            .height(length(76.0))
            .gap(4.0)
            .padding(Sides::length(7.0))
            .absolute(Sides {
                left: length(18.0),
                right: length(18.0),
                top: auto(),
                bottom: length(18.0),
            })
            .background(Color::TRANSPARENT)
            .border(Border::all(1.0, Color::srgba(1.0, 1.0, 1.0, 0.16)))
            .radius(CornerRadii::all(38.0))
            .layer(
                LayerStyle::new(Rect::default())
                    .mask(LayerMask::Rounded(CornerRadii::all(38.0)))
                    .shadow(Shadow::drop(
                        [0.0, 6.0],
                        18.0,
                        Color::srgba(0.06, 0.08, 0.16, 0.18),
                    )),
            );
        if self.enabled {
            let mut effect = self.effect;
            if self.tint == 0 {
                effect.tint[..3].fill(if dark { 0.0 } else { 1.0 });
            }
            bar = bar.backdrop_filter(effect.filter());
        }
        bar
    }
}

impl Render for GlassDemo {
    fn layout_changed(&mut self, layout: &LayoutSnapshot, _cx: &mut Context<Self>) {
        self.layout_controls(layout);
    }
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = shadcn(cx.environment().primary);
        let theme = themes.resolve(cx.environment().color_scheme);
        let phone = Element::container([self.feed(theme), self.navbar(theme, cx)])
            .keyed("liquid-glass-stage")
            .height(length(540.0))
            .width(percent(1.0))
            .max_width(length(440.0))
            .min_width(length(0.0))
            .grow(1.0)
            .shrink(0.0)
            .background(Color::srgb(0.96, 0.96, 0.98))
            .border(Border::all(1.0, theme.border))
            .radius(CornerRadii::all(32.0))
            .layer(
                LayerStyle::new(Rect::default()).mask(LayerMask::Rounded(CornerRadii::all(32.0))),
            )
            .overflow(Axes {
                x: Overflow::Hidden,
                y: Overflow::Hidden,
            });
        Element::row([
            phone,
            self.controls(theme, cx).grow(1.0).width(length(300.0)),
        ])
        .keyed("liquid-glass-demo")
        .gap(24.0)
        .flex_wrap(FlexWrap::Wrap)
        .align_items(AlignItems::START)
        .min_width(length(0.0))
        .width(percent(1.0))
    }
}
