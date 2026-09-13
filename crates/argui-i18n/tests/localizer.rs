use argui_i18n::{
    Catalog, CharacterDirection, FluentArgs, Localizer, LocalizerError, TranslationError, langid,
};
use argui_ui::{Element, ElementKind, WritingDirection};

const ENGLISH: &str = r#"
hello = Hello
welcome = Welcome, { $name }!
inbox = { $count ->
    [one] One message
   *[other] { $count } messages
}
fallback-only = Available in English
dialog = Continue?
    .title = Confirmation
    .description = Continue as { $name }?
attribute-only =
    .label = An attribute
needs-name = Hello, { $name }.
"#;

const FRENCH: &str = r#"
hello = Bonjour
welcome = Bienvenue, { $name } !
inbox = { $count ->
    [one] Un message
   *[other] { $count } messages
}
dialog = Continuer ?
    .title = Confirmation
"#;

fn localizer() -> Localizer {
    Localizer::new(
        langid!("en-US"),
        [
            Catalog::parse(langid!("en-US"), ENGLISH).unwrap(),
            Catalog::parse(langid!("fr"), FRENCH).unwrap(),
        ],
    )
    .unwrap()
}

#[test]
fn locale_negotiation_switches_copy_and_keeps_a_per_message_fallback() {
    let mut localizer = localizer();
    assert_eq!(localizer.locale(), &langid!("en-US"));
    assert_eq!(localizer.fallback_locale(), &langid!("en-US"));
    assert!(!localizer.is_rtl());
    assert_eq!(localizer.text("hello").unwrap(), "Hello");

    assert!(localizer.select([langid!("fr-CA")]));
    assert_eq!(localizer.locale(), &langid!("fr"));
    assert_eq!(
        localizer.locales().cloned().collect::<Vec<_>>(),
        [langid!("fr"), langid!("en-US")]
    );
    assert_eq!(localizer.text("hello").unwrap(), "Bonjour");
    assert_eq!(
        localizer.text("fallback-only").unwrap(),
        "Available in English"
    );

    assert!(!localizer.select([langid!("fr-FR")]));
    assert!(!localizer.select([langid!("fr")]));
    assert!(localizer.select(std::iter::empty()));
    assert_eq!(localizer.locale(), &langid!("en-US"));
}

#[test]
fn fluent_variables_plurals_and_attributes_format_through_the_selected_catalogs() {
    let mut localizer = localizer();
    localizer.select([langid!("fr")]);

    let mut args = FluentArgs::new();
    args.set("name", "Ada");
    assert_eq!(
        localizer.format("welcome", &args).unwrap(),
        "Bienvenue, \u{2068}Ada\u{2069} !"
    );
    assert_eq!(
        localizer.attribute("dialog", "title").unwrap(),
        "Confirmation"
    );
    assert_eq!(
        localizer
            .format_attribute("dialog", "description", &args)
            .unwrap(),
        "Continue as \u{2068}Ada\u{2069}?"
    );

    args.set("count", 1);
    assert_eq!(localizer.format("inbox", &args).unwrap(), "Un message");
    args.set("count", 2);
    assert_eq!(
        localizer.format("inbox", &args).unwrap(),
        "\u{2068}2\u{2069} messages"
    );
}

#[test]
fn missing_parts_and_resolver_failures_are_explicit() {
    let mut localizer = localizer();
    localizer.select([langid!("fr")]);

    assert_eq!(
        localizer.text("absent"),
        Err(TranslationError::MissingMessage("absent".into()))
    );
    assert_eq!(
        localizer.text("attribute-only"),
        Err(TranslationError::MissingValue("attribute-only".into()))
    );
    assert_eq!(
        localizer.attribute("dialog", "accesskey"),
        Err(TranslationError::MissingAttribute {
            id: "dialog".into(),
            attribute: "accesskey".into(),
        })
    );
    assert_eq!(
        localizer.attribute("absent", "label"),
        Err(TranslationError::MissingMessage("absent".into()))
    );

    let error = localizer.text("needs-name").unwrap_err();
    assert!(matches!(
        error,
        TranslationError::Formatting { locale, id, problems }
            if locale == langid!("en-US") && id == "needs-name" && !problems.is_empty()
    ));

    let error = localizer
        .format_attribute("dialog", "description", &FluentArgs::new())
        .unwrap_err();
    assert!(matches!(
        error,
        TranslationError::Formatting { locale, id, problems }
            if locale == langid!("en-US") && id == "dialog" && !problems.is_empty()
    ));
}

#[test]
fn invalid_catalog_sets_are_rejected() {
    let missing = Localizer::new(
        langid!("en-US"),
        [Catalog::parse(langid!("fr"), FRENCH).unwrap()],
    )
    .unwrap_err();
    assert_eq!(missing, LocalizerError::MissingFallback(langid!("en-US")));
    assert!(missing.to_string().contains("en-US"));

    let duplicate = Localizer::new(
        langid!("en-US"),
        [
            Catalog::parse(langid!("en-US"), "one = One").unwrap(),
            Catalog::parse(langid!("en-US"), "two = Two").unwrap(),
        ],
    )
    .unwrap_err();
    assert_eq!(duplicate, LocalizerError::DuplicateLocale(langid!("en-US")));
}

#[test]
fn negotiated_direction_drives_a_real_argui_subtree() {
    let mut localizer = Localizer::new(
        langid!("en-US"),
        [
            Catalog::parse(langid!("en-US"), "hello = Hello").unwrap(),
            Catalog::parse(langid!("ar"), "hello = مرحبًا").unwrap(),
        ],
    )
    .unwrap();
    localizer.select([langid!("ar")]);
    assert_eq!(localizer.direction(), CharacterDirection::RTL);
    assert!(localizer.is_rtl());

    let direction = if localizer.is_rtl() {
        WritingDirection::Rtl
    } else {
        WritingDirection::Ltr
    };
    let root = Element::column([Element::text(localizer.text("hello").unwrap())])
        .direction_scope(direction);

    assert_eq!(root.direction_scope, Some(WritingDirection::Rtl));
    assert!(matches!(
        &root.children[0].kind,
        ElementKind::Text { content, .. } if content.as_str() == "مرحبًا"
    ));
}
