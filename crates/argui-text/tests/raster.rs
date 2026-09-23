use argui_core::{Point, Rect, Size};
use argui_text::{GlyphContent, TextBlock, TextEngine, TextScene};

const FONT: &[u8] = include_bytes!("../../argui-web-demo/assets/fonts/NotoSans-Regular.ttf");

/// Creates one glyph in a scene large enough to test physical raster variants.
fn scene(text: &str) -> TextScene {
    TextScene::new().with(TextBlock::new(
        text,
        Rect::new(Point::new(0.0, 0.0), Size::new(2000.0, 2000.0)),
    ))
}

/// Reuses CPU bitmaps and missing images, while enforcing the entry budget.
#[test]
fn raster_cache_hits_and_entry_eviction_are_bounded() {
    let mut engine = TextEngine::from_embedded_fonts([FONT], "Noto Sans", "Noto Sans", "Noto Sans");
    let scene = scene("M ");
    let original = engine.prepare(&scene, 1.0);
    let key = original.glyphs[0].key;
    let first = engine.rasterize(key).unwrap();
    assert_eq!(Some(first), engine.rasterize(key));
    let blank = original.glyphs.last().unwrap().key;
    assert_eq!(engine.rasterize(blank), engine.rasterize(blank));
    assert_eq!(engine.stats().raster_hits, 2);
    for step in 1..=1025 {
        let prepared = engine.prepare(&scene, 1.0 + step as f32 / 4096.0);
        engine.rasterize(prepared.glyphs[0].key);
    }
    assert_eq!(engine.stats().raster_entries, 1024);
    assert!(engine.stats().raster_bytes <= 8 * 1024 * 1024);
    let misses = engine.stats().raster_misses;
    engine.rasterize(key);
    assert_eq!(engine.stats().raster_misses, misses + 1);
    assert_eq!(engine.stats().layout_misses, 1);
}

/// Large glyphs evict by bitmap bytes before the entry-count limit is reached.
#[test]
fn raster_cache_enforces_its_byte_budget() {
    let mut engine = TextEngine::from_embedded_fonts([FONT], "Noto Sans", "Noto Sans", "Noto Sans");
    let scene = scene("M");
    for step in 0..100 {
        let prepared = engine.prepare(&scene, 40.0 + step as f32 / 16.0);
        engine.rasterize(prepared.glyphs[0].key);
    }
    assert!(engine.stats().raster_entries < 100);
    assert!(engine.stats().raster_entries > 0);
    assert!(engine.stats().raster_bytes <= 8 * 1024 * 1024);
}

/// Adds a single translucent red COLR layer to the existing Noto exclamation outline.
/// The runtime-built font avoids another binary font fixture and uses the existing font license.
fn translucent_color_font() -> &'static [u8] {
    let count = u16::from_be_bytes(FONT[4..6].try_into().unwrap()) as usize;
    let mut tables: Vec<([u8; 4], Vec<u8>)> = (0..count)
        .map(|index| {
            let record = &FONT[12 + index * 16..28 + index * 16];
            let start = u32::from_be_bytes(record[8..12].try_into().unwrap()) as usize;
            let length = u32::from_be_bytes(record[12..16].try_into().unwrap()) as usize;
            (
                record[..4].try_into().unwrap(),
                FONT[start..start + length].to_vec(),
            )
        })
        .collect();
    // COLR v0: base glyph 4 ('!'), one layer using the same outline and palette entry 0.
    tables.push((
        *b"COLR",
        vec![
            0, 0, 0, 1, 0, 0, 0, 14, 0, 0, 0, 20, 0, 1, 0, 4, 0, 0, 0, 1, 0, 4, 0, 0,
        ],
    ));
    // CPAL v0: a single BGRA entry with half alpha.
    tables.push((
        *b"CPAL",
        vec![0, 0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 14, 0, 0, 0, 0, 255, 128],
    ));
    tables.sort_by_key(|(tag, _)| *tag);
    let count = tables.len() as u16;
    let power = 1_u16 << (15 - count.leading_zeros());
    let mut bytes = vec![0; 12 + usize::from(count) * 16];
    bytes[..4].copy_from_slice(&FONT[..4]);
    bytes[4..6].copy_from_slice(&count.to_be_bytes());
    bytes[6..8].copy_from_slice(&(power * 16).to_be_bytes());
    bytes[8..10].copy_from_slice(&(power.trailing_zeros() as u16).to_be_bytes());
    bytes[10..12].copy_from_slice(&((count - power) * 16).to_be_bytes());
    let mut head = 0;
    for (index, (tag, mut data)) in tables.into_iter().enumerate() {
        let length = data.len();
        if tag == *b"head" {
            data[8..12].fill(0);
            head = bytes.len();
        }
        while !data.len().is_multiple_of(4) {
            data.push(0);
        }
        let checksum = checksum(&data);
        let start = 12 + index * 16;
        bytes[start..start + 4].copy_from_slice(&tag);
        bytes[start + 4..start + 8].copy_from_slice(&checksum.to_be_bytes());
        let offset = bytes.len() as u32;
        bytes[start + 8..start + 12].copy_from_slice(&offset.to_be_bytes());
        bytes[start + 12..start + 16].copy_from_slice(&(length as u32).to_be_bytes());
        bytes.extend(data);
    }
    let adjustment = 0xB1B0_AFBA_u32.wrapping_sub(checksum(&bytes));
    bytes[head + 8..head + 12].copy_from_slice(&adjustment.to_be_bytes());
    Box::leak(bytes.into_boxed_slice())
}

/// Computes the wrapping OpenType checksum for aligned font `bytes`.
fn checksum(bytes: &[u8]) -> u32 {
    bytes.as_chunks::<4>().0.iter().fold(0_u32, |sum, chunk| {
        sum.wrapping_add(u32::from_be_bytes(*chunk))
    })
}

/// COLR rasters expose straight RGB; applying alpha a second time must not darken edges.
#[test]
fn color_outline_rasters_are_straight_alpha_srgb() {
    let mut engine = TextEngine::from_embedded_fonts(
        [translucent_color_font()],
        "Noto Sans",
        "Noto Sans",
        "Noto Sans",
    );
    let prepared = engine.prepare(&scene("!"), 2.0);
    let image = engine.rasterize(prepared.glyphs[0].key).unwrap();
    assert_eq!(image.content, GlyphContent::Color);
    let visible: Vec<_> = image
        .data
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|pixel| pixel[3] > 8)
        .collect();
    assert!(!visible.is_empty());
    assert!(
        visible
            .iter()
            .all(|pixel| pixel[0] >= 250 && pixel[1] == 0 && pixel[2] == 0 && pixel[3] < 128)
    );
}
