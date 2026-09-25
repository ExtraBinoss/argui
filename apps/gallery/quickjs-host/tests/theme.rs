use std::sync::{Arc, Mutex};

use argui_gallery_quickjs::QuickJsGallery;
use serde_json::Value;

#[test]
fn quickjs_theme_bridge_resolves_atomic_patch_and_rejects_bad_token() {
    let captured = Arc::new(Mutex::new(Vec::new()));
    let output = Arc::clone(&captured);
    let source = r#"
export function mountGallery(bridge) {
  const created = bridge.theme.create({
    tokens: {
      surface: { type: 'Color', default: '#ffffff', impact: 'Paint' },
      spacing: { type: 'Length', default: 8, impact: 'Layout' },
    },
    variants: { light: { surface: '#ffffff' }, dark: { surface: '#101116' } },
    initialVariant: 'light',
    systemVariants: { light: 'light', dark: 'dark' },
  })
  const changed = bridge.theme.update(created.id, {
    variant: 'dark', overrides: { spacing: 12 },
  })
  let invalid = ''
  try { bridge.theme.update(created.id, { overrides: { missing: 1 } }) }
  catch (error) { invalid = String(error) }
  bridge.control({ created, changed, invalid })
  return () => bridge.theme.dispose(created.id)
}
"#;
    let gallery = QuickJsGallery::new_with_control(
        source,
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
    .expect("theme module should mount");
    let result = captured.lock().unwrap()[0].clone();
    assert_eq!(result["created"]["snapshot"]["values"]["spacing"], 8);
    assert_eq!(result["changed"]["values"]["spacing"], 12);
    assert_eq!(result["changed"]["values"]["surface"], "#101116");
    assert_eq!(result["changed"]["change"]["impact"], "Layout");
    assert!(result["invalid"].as_str().unwrap().contains("missing"));
    gallery.dispose().unwrap();
    assert!(result["changed"]["revision"].as_u64().unwrap()
        > result["created"]["snapshot"]["revision"].as_u64().unwrap());
}
