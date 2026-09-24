use std::sync::{Arc, Mutex};

use argui_gallery_quickjs::QuickJsGallery;
use serde_json::{Value, json};

const SOURCE: &str = r#"
export function mountGallery(bridge) {
  const before = bridge.i18n.load({
    fallback: 'en-US',
    catalogs: {
      'en-US': { greeting: 'Hello, { $name }!', onlyEnglish: 'Settings' },
      fr: { greeting: '__FRENCH__' }
    }
  })
  const english = bridge.i18n.tr('greeting', { name: 'Ada' })
  const selected = bridge.i18n.select('fr-CA')
  const french = bridge.i18n.tr('greeting', { name: 'Ada' })
  const fallback = bridge.i18n.tr('onlyEnglish')
  bridge.control({ before, selected, english, french, fallback })
  return () => {}
}
"#;

#[test]
fn json_catalogs_translate_through_the_native_quickjs_bridge_after_reload() {
    for (translation, expected) in [
        ("Bonjour, { $name } !", "Bonjour, \u{2068}Ada\u{2069} !"),
        ("Salut, { $name } !", "Salut, \u{2068}Ada\u{2069} !"),
    ] {
        let captured = Arc::new(Mutex::new(Vec::new()));
        let output = Arc::clone(&captured);
        let source = SOURCE.replace("__FRENCH__", translation);
        let gallery = QuickJsGallery::new_with_control(
            &source,
            r#"{"abiHash":"test","natives":[]}"#,
            "mountGallery",
            |_| String::new(),
            move |json| {
                output
                    .lock()
                    .unwrap()
                    .push(serde_json::from_str::<Value>(&json).unwrap());
                String::new()
            },
        )
        .unwrap();
        let result = captured.lock().unwrap()[0].clone();
        assert_eq!(result["before"], json!({"locale":"en-US","rtl":false}));
        assert_eq!(result["selected"], json!({"locale":"fr","rtl":false}));
        assert_eq!(result["english"], "Hello, \u{2068}Ada\u{2069}!");
        assert_eq!(result["french"], expected);
        assert_eq!(result["fallback"], "Settings");
        gallery.dispose().unwrap();
    }
}

#[test]
fn malformed_json_catalog_reports_an_error_to_tsx() {
    let source = r#"
export function mountGallery(bridge) {
  bridge.i18n.load({ fallback: 'en-US', catalogs: { 'en-US': { 'bad.key': 'No' } } })
  return () => {}
}
"#;
    let error = QuickJsGallery::new_with_control(
        source,
        r#"{"abiHash":"test","natives":[]}"#,
        "mountGallery",
        |_| String::new(),
        |_| String::new(),
    )
    .err()
    .expect("invalid catalog should fail the mount");
    assert!(error.contains("invalid Fluent message ID"), "{error}");
}
