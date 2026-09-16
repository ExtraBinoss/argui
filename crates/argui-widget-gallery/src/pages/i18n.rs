use argui::{
    i18n::{Catalog, FluentArgs, Localizer, langid},
    paint::{Border, CornerRadii},
    runtime::{Context, Render},
    ui::{AlignItems, Element, FlexWrap, Sides, WritingDirection, length},
    widgets::{Button, shadcn},
};

const ENGLISH: &str = r#"
welcome = Welcome, { $name }!
inbox = { $count ->
    [one] You have one notification.
   *[other] You have { $count } notifications.
}
active-locale = Active catalog: { $locale }
less = Fewer
more = More
save-dialog =
    .confirm = Save changes
fallback-proof = This sentence comes from the English fallback catalog.
"#;

const FRENCH: &str = r#"
welcome = Bienvenue, { $name } !
inbox = { $count ->
    [one] Vous avez une notification.
   *[other] Vous avez { $count } notifications.
}
active-locale = Catalogue actif : { $locale }
less = Moins
more = Plus
save-dialog =
    .confirm = Enregistrer
"#;

const ARABIC: &str = r#"
welcome = مرحبًا، { $name }!
inbox = { $count ->
    [one] لديك إشعار واحد.
   *[other] لديك { $count } إشعارات.
}
active-locale = الكتالوج النشط: { $locale }
less = أقل
more = أكثر
save-dialog =
    .confirm = حفظ التغييرات
"#;

pub(crate) struct I18nDemo {
    localizer: Localizer,
    count: i32,
}

impl Default for I18nDemo {
    fn default() -> Self {
        let catalogs = [
            Catalog::parse(langid!("en-US"), ENGLISH),
            Catalog::parse(langid!("fr"), FRENCH),
            Catalog::parse(langid!("ar"), ARABIC),
        ]
        .map(|catalog| catalog.expect("embedded gallery Fluent catalogs must remain valid"));
        Self {
            localizer: Localizer::new(langid!("en-US"), catalogs)
                .expect("the gallery includes its declared fallback catalog"),
            count: 1,
        }
    }
}

impl I18nDemo {
    fn text(&self, id: &str) -> String {
        self.localizer
            .text(id)
            .expect("the embedded fallback catalog contains every plain message")
    }

    fn format(&self, id: &str, args: &FluentArgs<'_>) -> String {
        self.localizer
            .format(id, args)
            .expect("the embedded gallery messages accept the supplied variables")
    }
}

impl Render for I18nDemo {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = shadcn(cx.environment());
        let theme = themes.resolve(cx.environment().color_scheme);
        let mut args = FluentArgs::new();
        args.set("name", "Ada");
        args.set("count", self.count);
        args.set("locale", self.localizer.locale().to_string());
        let direction = if self.localizer.is_rtl() {
            WritingDirection::Rtl
        } else {
            WritingDirection::Ltr
        };
        let language = Element::row([
            Button::new("i18n-english", "English", theme.outline_button())
                .on_click(cx.callback(|demo| {
                    demo.localizer.select([langid!("en-US")]);
                }))
                .build(),
            Button::new("i18n-french", "Français", theme.outline_button())
                .on_click(cx.callback(|demo| {
                    demo.localizer.select([langid!("fr-CA")]);
                }))
                .build(),
            Button::new("i18n-arabic", "العربية", theme.outline_button())
                .on_click(cx.callback(|demo| {
                    demo.localizer.select([langid!("ar")]);
                }))
                .build(),
        ])
        .gap(8.0)
        .flex_wrap(FlexWrap::Wrap);
        let controls = Element::row([
            Button::new("i18n-less", self.text("less"), theme.outline_button())
                .on_click(cx.callback(|demo| demo.count = demo.count.saturating_sub(1).max(0)))
                .build(),
            Button::new("i18n-more", self.text("more"), theme.outline_button())
                .on_click(cx.callback(|demo| demo.count = demo.count.saturating_add(1).min(99)))
                .build(),
            Button::new(
                "i18n-save",
                self.localizer
                    .attribute("save-dialog", "confirm")
                    .expect("each embedded catalog contains the confirm attribute"),
                theme.button(),
            )
            .build(),
        ])
        .gap(8.0)
        .flex_wrap(FlexWrap::Wrap);
        let localized = Element::column([
            crate::app::text(self.format("welcome", &args), 24.0, theme.foreground, 700),
            crate::app::text(self.format("inbox", &args), 16.0, theme.foreground, 500),
            crate::app::text(
                self.format("active-locale", &args),
                13.0,
                theme.muted_foreground,
                450,
            ),
            controls,
            Element::column([
                crate::app::text("Fallback proof", 12.0, theme.muted_foreground, 600),
                crate::app::text(self.text("fallback-proof"), 14.0, theme.foreground, 450),
            ])
            .gap(4.0)
            .padding(Sides::length(12.0))
            .border(Border::all(1.0, theme.border))
            .radius(CornerRadii::all(8.0)),
        ])
        .keyed("i18n-content")
        .direction_scope(direction)
        .align_items(AlignItems::STRETCH)
        .padding(Sides::length(20.0))
        .gap(14.0)
        .width(length(620.0))
        .max_width(argui::ui::percent(1.0))
        .background(theme.card)
        .border(Border::all(1.0, theme.border))
        .radius(CornerRadii::all(12.0));
        super::preview(
            "Live Fluent catalogs",
            "Switch locale, exercise plural selection, inspect fallback copy and flip the subtree to RTL.",
            Element::column([language, localized]).gap(16.0),
            theme,
        )
    }
}
