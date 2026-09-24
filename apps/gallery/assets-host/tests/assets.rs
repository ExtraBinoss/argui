use argui_gallery_assets::{AssetInput, AssetKind, decode_assets, load};

#[test]
fn embedded_assets_decode_for_native_image_and_svg_primitives() {
    let assets = load().expect("every embedded asset should decode");
    assert_eq!(assets.images.len(), 2);
    assert_eq!(assets.vectors.len(), 9);
    assert!(assets.vectors.iter().any(|asset| asset.tintable));
    assert!(assets.vectors.iter().any(|asset| !asset.tintable));
    assert!(assets.images[0].width > 0);
    assert!(assets.images[0].height > 0);
}

#[test]
fn decode_reports_invalid_raster_with_its_source_name() {
    let error = decode_assets(&[AssetInput {
        key: "broken.png",
        kind: AssetKind::Image,
        id: 1,
        bytes: b"bad image",
    }])
    .err()
    .expect("invalid raster must fail");
    assert!(error.contains("broken.png"), "{error}");
}

#[test]
fn decode_reports_invalid_svg_with_its_source_name() {
    let error = decode_assets(&[AssetInput {
        key: "broken.svg",
        kind: AssetKind::Svg,
        id: 2,
        bytes: b"not svg",
    }])
    .err()
    .expect("invalid vector must fail");
    assert!(error.contains("broken.svg"), "{error}");
}
