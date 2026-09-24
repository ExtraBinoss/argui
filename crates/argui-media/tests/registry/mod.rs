use std::io::Cursor;

use argui_media::{AssetHandle, AssetKey, AssetRecord, AssetRegistry, AssetRegistryError};
use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};

fn png(color: [u8; 4]) -> Vec<u8> {
    let image = DynamicImage::ImageRgba8(RgbaImage::from_pixel(1, 1, Rgba(color)));
    let mut bytes = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
        .unwrap();
    bytes
}

#[test]
fn keys_are_platform_independent_and_reject_escaping_paths() {
    assert_eq!(
        AssetKey::new("./assets\\icons//logo.svg").unwrap().as_str(),
        "assets/icons/logo.svg"
    );
    for invalid in [
        "",
        ".",
        "././",
        "../secret.png",
        "assets/../secret.png",
        "/root.png",
        "\\root.png",
        "C:\\root.png",
        "z:relative.png",
        "bad\0name.png",
    ] {
        assert!(matches!(
            AssetKey::new(invalid),
            Err(AssetRegistryError::InvalidKey(_))
        ));
    }
}

#[test]
fn records_can_be_ordered_removed_and_reused_without_leaking_kinds() {
    let mut registry = AssetRegistry::new();
    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
    let vector = AssetKey::new("a.svg").unwrap();
    let image = AssetKey::new("z.png").unwrap();
    let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" width="1" height="1"/>"#;
    let first_vector = registry.upsert_vector(vector.clone(), svg).unwrap();
    let same_vector = registry.upsert_vector(vector.clone(), svg).unwrap();
    assert!(!same_vector.changed);
    assert_eq!(first_vector.revision, same_vector.revision);
    assert_eq!(first_vector.handle, same_vector.handle);
    registry
        .upsert_image(image.clone(), &png([1, 2, 3, 255]))
        .unwrap();
    assert_eq!(registry.len(), 2);
    assert_eq!(
        registry
            .records()
            .map(|record| record.key().as_str())
            .collect::<Vec<_>>(),
        ["a.svg", "z.png"]
    );
    assert!(matches!(
        registry.upsert_image(vector.clone(), &png([1, 2, 3, 255])),
        Err(AssetRegistryError::KindMismatch { .. })
    ));
    assert!(registry.remove(&vector).is_some());
    assert!(registry.remove(&vector).is_none());
    assert!(registry.remove(&image).is_some());
    assert!(registry.is_empty());
    assert_eq!(
        AssetKey::new("folder//icon.svg").unwrap().to_string(),
        "folder/icon.svg"
    );
}

#[test]
fn image_updates_keep_the_handle_and_advance_only_for_new_valid_content() {
    let key = AssetKey::new("assets/pixel.png").unwrap();
    let mut registry = AssetRegistry::new();
    let first = registry
        .upsert_image(key.clone(), &png([255, 0, 0, 255]))
        .unwrap();
    let unchanged = registry
        .upsert_image(key.clone(), &png([255, 0, 0, 255]))
        .unwrap();
    assert!(!unchanged.changed);
    assert_eq!(unchanged.revision, first.revision);
    assert_eq!(unchanged.handle, first.handle);

    let second = registry
        .upsert_image(key.clone(), &png([0, 0, 255, 255]))
        .unwrap();
    assert!(second.changed);
    assert_eq!(second.revision.get(), first.revision.get() + 1);
    assert_eq!(second.handle, first.handle);

    assert!(registry.upsert_image(key.clone(), b"not an image").is_err());
    let current = registry.get(&key).unwrap();
    assert_eq!(current.revision(), second.revision);
    assert_eq!(current.handle(), second.handle);
}

#[test]
fn vector_updates_keep_the_handle_and_failed_edits_are_transactional() {
    let key = AssetKey::new("assets/logo.svg").unwrap();
    let mut registry = AssetRegistry::new();
    let first_svg = br#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><rect width="10" height="10"/></svg>"#;
    let second_svg = br#"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="10"><rect width="20" height="10" fill="currentColor"/></svg>"#;
    let first = registry.upsert_vector(key.clone(), first_svg).unwrap();
    let second = registry.upsert_vector(key.clone(), second_svg).unwrap();
    assert_eq!(first.handle, second.handle);
    assert_eq!(second.revision.get(), 2);
    assert!(registry.upsert_vector(key.clone(), b"<svg").is_err());
    let AssetRecord::Vector { asset, .. } = registry.get(&key).unwrap() else {
        panic!("expected vector record");
    };
    assert_eq!(asset.size.width, 20.0);
    assert!(asset.tintable);
}

#[test]
fn one_source_key_cannot_change_asset_kind() {
    let key = AssetKey::new("assets/content.bin").unwrap();
    let mut registry = AssetRegistry::new();
    registry
        .upsert_image(key.clone(), &png([0, 0, 0, 255]))
        .unwrap();
    let error = registry
        .upsert_vector(
            key,
            br#"<svg xmlns="http://www.w3.org/2000/svg" width="1" height="1"/>"#,
        )
        .unwrap_err();
    assert!(matches!(error, AssetRegistryError::KindMismatch { .. }));
    assert!(matches!(
        registry.records().next().unwrap().handle(),
        AssetHandle::Image(_)
    ));
}
