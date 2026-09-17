use argui::{
    runtime::{Context, Render},
    text::TextStyle,
    ui::{AlignItems, Axes, Element, JustifyContent, Overflow, ScrollConfig, Sides, percent},
    widgets::{Button, default_theme},
};

const SHIPPING: [(&str, &str); 4] = [
    ("Text input / IME", "Shipping"),
    ("Safe areas, keyboard, and system bars", "Shipping"),
    ("Live activity progress", "Shipping"),
    ("Background activity foundation", "Shipping"),
];

const PLANNED: [(&str, &str); 29] = [
    ("Accessibility device validation", "Highest priority"),
    ("Camera", "Medium priority"),
    ("Clipboard", "High priority"),
    ("Drag and drop", "High priority"),
    ("Haptics", "High priority"),
    ("File picker", "High priority"),
    ("Photo picker", "High priority"),
    ("Share sheet", "High priority"),
    ("Biometrics", "High priority"),
    ("Passkeys and credentials", "High priority"),
    ("Secure storage", "High priority"),
    ("Notifications", "High priority"),
    ("Home-screen widgets", "High priority"),
    ("Location", "Medium priority"),
    ("Motion and sensors", "Medium priority"),
    ("Bluetooth LE", "Medium priority"),
    ("NFC", "Medium priority"),
    ("UWB and ranging", "High priority"),
    ("Audio input and output", "High priority"),
    ("Video encode and decode", "Medium priority"),
    ("Mobile WebView", "High priority"),
    ("Deep links", "High priority"),
    ("Network status", "Medium priority"),
    ("Gamepads", "Medium priority"),
    ("Mouse and stylus validation", "High priority"),
    ("Store and in-app purchases", "Medium priority"),
    ("Speech and text to speech", "Later"),
    ("Contacts and calendar", "Later"),
    ("Health integrations", "Later"),
];

#[derive(Default)]
pub struct Example {
    expanded: bool,
}

impl Render for Example {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = default_theme(cx.environment());
        let theme = themes.resolve(cx.environment().color_scheme);
        let rows = SHIPPING
            .into_iter()
            .map(|(capability, status)| (capability, status, true))
            .chain(
                self.expanded
                    .then_some(PLANNED)
                    .into_iter()
                    .flatten()
                    .map(|(capability, priority)| (capability, priority, false)),
            )
            .map(|(capability, status, shipping)| {
                Element::row([
                    Element::text(capability).text_style(TextStyle {
                        color: theme.foreground,
                        weight: 600,
                        ..TextStyle::default()
                    }),
                    Element::text(status).text_style(TextStyle {
                        color: if shipping {
                            theme.primary
                        } else {
                            theme.muted_foreground
                        },
                        font_size: 12.0,
                        weight: 650,
                        ..TextStyle::default()
                    }),
                ])
                .align_items(AlignItems::CENTER)
                .justify_content(JustifyContent::SPACE_BETWEEN)
                .gap(12.0)
                .padding(Sides::length(12.0))
                .background(theme.card)
                .border(argui::paint::Border::all(1.0, theme.border))
                .radius(argui::paint::CornerRadii::all(8.0))
            });
        Element::column([
            Element::text("Mobile integration roadmap").text_style(TextStyle {
                color: theme.foreground,
                font_size: 28.0,
                weight: 760,
                ..TextStyle::default()
            }),
            Element::text(
                "Shipping means implemented now. Every other row is planned, not promised as available.",
            )
            .text_style(TextStyle {
                color: theme.muted_foreground,
                font_size: 14.0,
                ..TextStyle::default()
            }),
            Button::new(
                "toggle-roadmap",
                if self.expanded {
                    "Show shipping only"
                } else {
                    "Show the full planned roadmap"
                },
                theme.outline_button(),
            )
            .on_click(cx.callback(|app| app.expanded = !app.expanded))
            .build(),
            Element::column(rows).gap(8.0),
        ])
        .width(percent(1.0))
        .height(percent(1.0))
        .padding(Sides::length(24.0))
        .gap(14.0)
        .background(theme.background)
        .overflow(Axes {
            x: Overflow::Hidden,
            y: Overflow::Auto,
        })
        .scroll_config(ScrollConfig::default().scrollbar(theme.scrollbar.clone()))
    }
}
