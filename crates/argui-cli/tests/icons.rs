#![cfg(not(target_arch = "wasm32"))]

use argui_cli::run_in;
use image::{ImageBuffer, Rgba};

#[test]
/// One PNG produces the expected desktop icon formats.
fn generates_png_iconset_and_ico() {
    let temp = tempfile::tempdir().unwrap();
    ImageBuffer::from_pixel(512, 512, Rgba([30u8, 90, 180, 255]))
        .save(temp.path().join("source.png"))
        .unwrap();
    run_in(temp.path(), &["icon".into(), "source.png".into()]).unwrap();
    assert_eq!(
        image::open(temp.path().join("icons/icon-32.png"))
            .unwrap()
            .width(),
        32
    );
    assert_eq!(
        &std::fs::read(temp.path().join("icons/app.ico")).unwrap()[..4],
        &[0, 0, 1, 0]
    );
}

#[test]
/// A non-square source is rejected before creating outputs.
fn rejects_non_square_source() {
    let temp = tempfile::tempdir().unwrap();
    ImageBuffer::from_pixel(512, 256, Rgba([0u8, 0, 0, 255]))
        .save(temp.path().join("source.png"))
        .unwrap();
    assert!(run_in(temp.path(), &["icon".into(), "source.png".into()]).is_err());
}

#[test]
/// Invalid source paths and occupied output paths produce useful errors.
fn icon_errors_name_the_source_or_output() {
    let temp = tempfile::tempdir().unwrap();
    assert!(
        run_in(temp.path(), &["icon".into(), "source.jpg".into()])
            .unwrap_err()
            .contains("PNG")
    );
    assert!(
        run_in(temp.path(), &["icon".into(), "missing.png".into()])
            .unwrap_err()
            .contains("missing.png")
    );
    ImageBuffer::from_pixel(512, 512, Rgba([30u8, 90, 180, 255]))
        .save(temp.path().join("source.png"))
        .unwrap();
    std::fs::write(temp.path().join("icons"), "occupied").unwrap();
    assert!(run_in(temp.path(), &["icon".into(), "source.png".into()]).is_err());
}

#[test]
/// Occupied output files cannot be overwritten during icon generation.
fn icon_generation_reports_output_conflicts() {
    let temp = tempfile::tempdir().unwrap();
    ImageBuffer::from_pixel(512, 512, Rgba([30u8, 90, 180, 255]))
        .save(temp.path().join("source.png"))
        .unwrap();
    let icons = temp.path().join("icons");
    std::fs::create_dir_all(icons.join("app.iconset/icon_16x16.png")).unwrap();
    let error = run_in(temp.path(), &["icon".into(), "source.png".into()]).unwrap_err();
    assert!(!error.is_empty());
    std::fs::remove_dir(icons.join("app.iconset/icon_16x16.png")).unwrap();
    std::fs::remove_file(icons.join("icon-16.png")).unwrap();
    std::fs::create_dir(icons.join("icon-16.png")).unwrap();
    let error = run_in(temp.path(), &["icon".into(), "source.png".into()]).unwrap_err();
    assert!(!error.is_empty());
    std::fs::remove_dir(icons.join("icon-16.png")).unwrap();
    std::fs::create_dir(icons.join("app.ico")).unwrap();
    let error = run_in(temp.path(), &["icon".into(), "source.png".into()]).unwrap_err();
    assert!(!error.is_empty());
}
