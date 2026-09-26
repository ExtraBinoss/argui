use argui_gallery_assets::load_manifest;
use std::{fs, path::PathBuf};

/// Creates one isolated app with a generated-style manifest and a custom SVG.
fn app_fixture() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "argui-assets-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("assets/mine")).unwrap();
    fs::write(
        root.join("assets/mine/star.svg"),
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M12 2 15 9 22 9 17 14 19 22 12 17 5 22 7 14 2 9 9 9Z" fill="currentColor"/></svg>"#,
    )
    .unwrap();
    root
}

#[test]
/// App-local SVGs decode with their declared IDs before the first frame.
fn loads_app_owned_svg() {
    let root = app_fixture();
    fs::write(
        root.join("assets.generated.json"),
        r#"{"assets":[{"key":"mine/star.svg","kind":"svg","id":123,"path":"mine/star.svg"}]}"#,
    )
    .unwrap();
    let assets = load_manifest(&root.join("assets.generated.json")).unwrap();
    assert_eq!(assets.vectors.len(), 1);
    assert_eq!(assets.vectors[0].id.0, 123);
    assert!(assets.images.is_empty());
    fs::remove_dir_all(root).unwrap();
}

#[test]
/// A modified manifest cannot read a file outside the app's assets directory.
fn rejects_parent_traversal() {
    let root = app_fixture();
    fs::write(
        root.join("assets.generated.json"),
        r#"{"assets":[{"key":"mine/star.svg","kind":"svg","id":123,"path":"../secret.svg"}]}"#,
    )
    .unwrap();
    let error = load_manifest(&root.join("assets.generated.json"))
        .err()
        .unwrap();
    assert!(error.contains("unsafe asset path"), "{error}");
    fs::remove_dir_all(root).unwrap();
}
