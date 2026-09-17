use argui::{
    runtime::{Context, Render},
    text::TextStyle,
    ui::{Axes, Element, Overflow, ScrollConfig, Sides, length, percent},
    widgets::default_theme,
};

const PLATFORMS: [(&str, &str, &str); 6] = [
    (
        "Linux",
        "Supported · runtime-tested",
        "Native WGPU, input, accessibility, windows, and the complete gallery.",
    ),
    (
        "Windows",
        "Supported · CI-compiled",
        "Native WGPU with DirectX 12 and Vulkan fallback paths.",
    ),
    (
        "macOS",
        "Supported · CI-compiled",
        "Native AppKit windowing, Metal rendering, and desktop integration.",
    ),
    (
        "WebAssembly",
        "Supported · browser-tested",
        "WebGPU rendering, browser semantics, and the complete live gallery.",
    ),
    (
        "Android",
        "Preview",
        "Cross-compilation, APK/AAB packaging, IME, safe areas, and activity progress.",
    ),
    (
        "iOS",
        "Preview",
        "XCFramework/Simulator packaging, safe areas, IME, and ActivityKit progress.",
    ),
];

pub struct Example;

impl Render for Example {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = default_theme(cx.environment());
        let theme = themes.resolve(cx.environment().color_scheme);
        let cards = PLATFORMS.into_iter().map(|(platform, status, detail)| {
            Element::column([
                Element::text(platform).text_style(TextStyle {
                    color: theme.foreground,
                    font_size: 18.0,
                    weight: 700,
                    ..TextStyle::default()
                }),
                Element::text(status).text_style(TextStyle {
                    color: theme.primary,
                    font_size: 13.0,
                    weight: 650,
                    ..TextStyle::default()
                }),
                Element::text(detail).text_style(TextStyle {
                    color: theme.muted_foreground,
                    font_size: 13.0,
                    ..TextStyle::default()
                }),
            ])
            .padding(Sides::length(16.0))
            .gap(7.0)
            .background(theme.card)
            .border(argui::paint::Border::all(1.0, theme.border))
            .radius(argui::paint::CornerRadii::all(10.0))
        });
        Element::column([
            Element::text("Argui platform support").text_style(TextStyle {
                color: theme.foreground,
                font_size: 28.0,
                weight: 760,
                ..TextStyle::default()
            }),
            Element::text(
                "Desktop and Web are supported today. Android and iOS are explicit preview targets.",
            )
            .text_style(TextStyle {
                color: theme.muted_foreground,
                font_size: 14.0,
                ..TextStyle::default()
            }),
            Element::column(cards).gap(10.0),
        ])
        .width(percent(1.0))
        .height(percent(1.0))
        .min_height(length(0.0))
        .padding(Sides::length(24.0))
        .gap(16.0)
        .background(theme.background)
        .overflow(Axes {
            x: Overflow::Hidden,
            y: Overflow::Auto,
        })
        .scroll_config(ScrollConfig::default().scrollbar(theme.scrollbar.clone()))
    }
}
