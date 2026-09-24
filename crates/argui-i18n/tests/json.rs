use argui_i18n::{Catalog, FluentArgs, JsonCatalogError, Localizer, langid};

#[test]
fn json_catalogs_format_and_fallback_through_the_native_localizer() {
    let english = Catalog::from_json(
        langid!("en-US"),
        r#"{"greeting":"Hello, { $name }!","onlyEnglish":"Settings"}"#,
    )
    .unwrap();
    let french =
        Catalog::from_json(langid!("fr"), r#"{"greeting":"Bonjour, { $name } !"}"#).unwrap();
    let mut localizer = Localizer::new(langid!("en-US"), [english, french]).unwrap();
    assert!(localizer.select([langid!("fr-CA")]));
    let mut args = FluentArgs::new();
    args.set("name", "Ada");
    assert_eq!(
        localizer.format("greeting", &args).unwrap(),
        "Bonjour, \u{2068}Ada\u{2069} !"
    );
    assert_eq!(localizer.text("onlyEnglish").unwrap(), "Settings");
}

#[test]
fn json_catalog_rejects_values_that_cannot_become_messages() {
    assert!(matches!(
        Catalog::from_json(langid!("en-US"), r#"{"nested":{"label":"Hi"}}"#),
        Err(JsonCatalogError::InvalidJson(_))
    ));
    assert_eq!(
        Catalog::from_json(langid!("en-US"), r#"{"bad.key":"Hi"}"#).unwrap_err(),
        JsonCatalogError::InvalidId("bad.key".into())
    );
    assert!(matches!(
        Catalog::from_json(langid!("en-US"), r#"{"2bad":"Hi"}"#),
        Err(JsonCatalogError::InvalidId(_))
    ));
    assert!(matches!(
        Catalog::from_json(langid!("en-US"), r#"{"broken":"{ $name"}"#),
        Err(JsonCatalogError::Fluent(_))
    ));
}

#[test]
fn json_catalog_accepts_empty_and_multiline_patterns() {
    let catalog = Catalog::from_json(
        langid!("en-US"),
        r#"{"empty":"","paragraph":"First line\nSecond line"}"#,
    )
    .unwrap();
    let localizer = Localizer::new(langid!("en-US"), [catalog]).unwrap();
    assert_eq!(localizer.text("empty").unwrap(), "");
    assert_eq!(
        localizer.text("paragraph").unwrap(),
        "First line\nSecond line"
    );
}
