//! Deterministic desktop icon sizes from one square PNG source.

use image::{DynamicImage, ImageFormat, imageops::FilterType};
use std::{fs, io::Cursor, path::Path};

/// Generates PNG sizes, a Windows ICO, and a macOS iconset in `cwd/icons`.
/// `source` points to a square PNG; relative paths resolve against `cwd`.
///
/// # Errors
/// Returns an error if the input is invalid or output files cannot be written.
pub fn generate(cwd: &Path, source: &Path) -> Result<(), String> {
    let input = if source.is_absolute() {
        source.to_path_buf()
    } else {
        cwd.join(source)
    };
    if input.extension().and_then(|value| value.to_str()) != Some("png") {
        return Err("icon source must be a PNG file".into());
    }
    let image = image::open(&input).map_err(|error| format!("{}: {error}", input.display()))?;
    if image.width() != image.height() || image.width() < 512 {
        return Err("icon source must be a square PNG at least 512×512".into());
    }
    let target = cwd.join("icons");
    let iconset = target.join("app.iconset");
    fs::create_dir_all(&iconset).map_err(|error| error.to_string())?;
    let mut ico_images = Vec::new();
    for size in [16, 32, 48, 64, 128, 256, 512, 1024] {
        let resized = image.resize_exact(size, size, FilterType::Lanczos3);
        let bytes = png(&resized)?;
        fs::write(target.join(format!("icon-{size}.png")), &bytes)
            .map_err(|error| error.to_string())?;
        let iconset_names: &[&str] = match size {
            16 => &["icon_16x16.png"],
            32 => &["icon_16x16@2x.png", "icon_32x32.png"],
            64 => &["icon_32x32@2x.png"],
            128 => &["icon_128x128.png"],
            256 => &["icon_128x128@2x.png", "icon_256x256.png"],
            512 => &["icon_256x256@2x.png", "icon_512x512.png"],
            1024 => &["icon_512x512@2x.png"],
            _ => &[],
        };
        for name in iconset_names {
            fs::write(iconset.join(name), &bytes).map_err(|error| error.to_string())?;
        }
        if size <= 256 {
            ico_images.push((size, bytes));
        }
    }
    fs::write(target.join("app.ico"), ico(&ico_images)).map_err(|error| error.to_string())?;
    #[cfg(target_os = "macos")]
    {
        let result = std::process::Command::new("iconutil")
            .arg("-c")
            .arg("icns")
            .arg(&iconset)
            .arg("-o")
            .arg(target.join("app.icns"))
            .status()
            .map_err(|error| format!("iconutil: {error}"))?;
        if !result.success() {
            return Err(format!("iconutil exited with {result}"));
        }
    }
    println!("Generated desktop icons in {}", target.display());
    Ok(())
}

/// Encodes `image` as PNG bytes and returns the encoded data.
///
/// # Errors
/// Returns an error if the image encoder fails.
fn png(image: &DynamicImage) -> Result<Vec<u8>, String> {
    let mut output = Cursor::new(Vec::new());
    image
        .write_to(&mut output, ImageFormat::Png)
        .map_err(|error| error.to_string())?;
    Ok(output.into_inner())
}

/// Encodes the six generated PNG sizes into a Windows ICO file.
/// `images` contains the fixed 16 through 256 pixel payloads made by `generate`.
/// Returns the complete ICO bytes.
fn ico(images: &[(u32, Vec<u8>)]) -> Vec<u8> {
    let count = images.len() as u16;
    let mut output = vec![0, 0, 1, 0];
    output.extend_from_slice(&count.to_le_bytes());
    let mut offset = 6 + 16 * images.len();
    for (size, bytes) in images {
        output.extend_from_slice(&[
            if *size == 256 { 0 } else { *size as u8 },
            if *size == 256 { 0 } else { *size as u8 },
            0,
            0,
        ]);
        output.extend_from_slice(&1u16.to_le_bytes());
        output.extend_from_slice(&32u16.to_le_bytes());
        output.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        output.extend_from_slice(&(offset as u32).to_le_bytes());
        offset += bytes.len();
    }
    for (_, bytes) in images {
        output.extend_from_slice(bytes);
    }
    output
}
