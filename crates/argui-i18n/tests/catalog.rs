use argui_i18n::{Catalog, FluentError, langid};

#[test]
fn catalogs_parse_one_or_several_non_overlapping_resources() {
    let catalog =
        Catalog::from_resources(langid!("fr"), ["save = Enregistrer", "cancel = Annuler"]).unwrap();

    assert_eq!(catalog.locale(), &langid!("fr"));
    assert!(format!("{catalog:?}").contains("fr"));
}

#[test]
fn syntax_and_duplicate_entries_report_their_locale_and_problems() {
    let syntax = Catalog::parse(langid!("fr"), "broken = { ").unwrap_err();
    assert_eq!(syntax.locale(), &langid!("fr"));
    assert!(!syntax.problems().is_empty());
    assert_eq!(syntax.to_string(), "invalid Fluent catalog for fr");

    let duplicate =
        Catalog::from_resources(langid!("en-US"), ["save = Save", "save = Store"]).unwrap_err();
    assert_eq!(duplicate.locale(), &langid!("en-US"));
    assert!(matches!(
        duplicate.problems(),
        [FluentError::Overriding { id, .. }] if id == "save"
    ));
}
