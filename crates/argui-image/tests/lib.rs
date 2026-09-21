use argui_image::{ImageLibrary, decode};
use argui_paint::ImageId;
use image::ImageEncoder;

#[test]
fn decodes_png_to_renderer_neutral_rgba() {
    let mut png = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png)
        .write_image(&[255, 0, 0, 255], 1, 1, image::ExtendedColorType::Rgba8)
        .unwrap();
    let asset = decode(ImageId(7), &png).unwrap();
    assert_eq!((asset.id, asset.width, asset.height), (ImageId(7), 1, 1));
    assert_eq!(&*asset.rgba8, &[255, 0, 0, 255]);
}

/// WebP artwork decodes through the same renderer-neutral image pipeline.
#[test]
fn decodes_webp_to_renderer_neutral_rgba() {
    let mut webp = Vec::new();
    image::codecs::webp::WebPEncoder::new_lossless(&mut webp)
        .write_image(&[12, 34, 56, 255], 1, 1, image::ExtendedColorType::Rgba8)
        .unwrap();
    let asset = decode(ImageId(8), &webp).unwrap();
    assert_eq!((asset.id, asset.width, asset.height), (ImageId(8), 1, 1));
    assert_eq!(&*asset.rgba8, &[12, 34, 56, 255]);
}

#[test]
fn rejects_unknown_encoded_data() {
    assert!(decode(ImageId(1), b"not an image").is_err());
}

#[test]
fn library_generates_distinct_handles_and_retains_assets() {
    let mut png = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png)
        .write_image(&[0, 255, 0, 255], 1, 1, image::ExtendedColorType::Rgba8)
        .unwrap();
    let mut library = ImageLibrary::new();
    let first = library.insert(&png).unwrap();
    let second = library.insert(&png).unwrap();

    assert_ne!(first, second);
    assert_eq!(library.assets().len(), 2);
    assert_eq!(library.assets()[0].id, first);
    assert_eq!(library.assets()[1].id, second);
}
