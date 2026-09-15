use argui::{
    i18n::{Catalog, FluentArgs, Localizer, langid},
    runtime::{Context, Render},
    text::TextStyle,
    ui::{Element, EventType, FlexWrap, Sides, UiEventKind, WritingDirection, percent},
    widgets::{Button, default_theme},
};

const ENGLISH: &str = "hello = Hello, { $name }!\nitems = { $count ->\n [one] One message\n*[other] { $count } messages\n}";
const FRENCH: &str = "hello = Bonjour, { $name } !\nitems = { $count ->\n [one] Un message\n*[other] { $count } messages\n}";
const ARABIC: &str = "hello = مرحبًا، { $name }!\nitems = { $count ->\n [one] رسالة واحدة\n*[other] { $count } رسائل\n}";

pub struct Example {
    localizer: Localizer,
    count: i32,
}

impl Default for Example {
    fn default() -> Self {
        let catalogs = [
            (langid!("en-US"), ENGLISH),
            (langid!("fr"), FRENCH),
            (langid!("ar"), ARABIC),
        ]
        .map(|(locale, source)| Catalog::parse(locale, source).expect("valid embedded Fluent"));
        Self {
            localizer: Localizer::new(langid!("en-US"), catalogs).expect("English fallback exists"),
            count: 1,
        }
    }
}

impl Render for Example {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = default_theme(cx.environment());
        let theme = themes.resolve(cx.environment().color_scheme);
        let mut args = FluentArgs::new();
        args.set("name", "Ada");
        args.set("count", self.count);
        let direction = if self.localizer.is_rtl() {
            WritingDirection::Rtl
        } else {
            WritingDirection::Ltr
        };
        let localized_text = |value| {
            Element::text(value).text_style(TextStyle {
                color: theme.foreground,
                ..TextStyle::default()
            })
        };
        let localized = Element::column([
            localized_text(
                self.localizer
                    .format("hello", &args)
                    .expect("known message"),
            ),
            localized_text(self.localizer.format("items", &args).expect("known plural")),
        ])
        .gap(8.0)
        .direction_scope(direction);
        Element::column([
            Element::row([
                Button::new("en", "English", theme.outline_button()).build(),
                Button::new("fr", "Français", theme.outline_button()).build(),
                Button::new("ar", "العربية", theme.outline_button()).build(),
            ])
            .gap(8.0)
            .flex_wrap(FlexWrap::Wrap),
            localized,
            Button::new("more", "Add message", theme.button()).build(),
        ])
        .width(percent(1.0))
        .height(percent(1.0))
        .padding(Sides::length(28.0))
        .gap(18.0)
        .background(theme.background)
        .on(cx.listener(EventType::Click, |app, event, cx| {
            if !matches!(event.kind, UiEventKind::Click(_)) {
                return;
            }
            match event.target_key() {
                Some("en") => {
                    app.localizer.select([langid!("en-US")]);
                }
                Some("fr") => {
                    app.localizer.select([langid!("fr")]);
                }
                Some("ar") => {
                    app.localizer.select([langid!("ar")]);
                }
                Some("more") => app.count += 1,
                _ => return,
            }
            cx.notify();
        }))
    }
}
