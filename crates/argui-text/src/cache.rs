use std::collections::VecDeque;

use argui_core::Size;

use crate::{GlyphKey, TextBlock, TextStyle};

const CACHE_CAPACITY: usize = 512;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MeasureKey {
    text: String,
    style: TextStyle,
    width: Option<f32>,
}

impl MeasureKey {
    pub(crate) fn new(text: &str, style: &TextStyle, width: Option<f32>) -> Self {
        Self {
            text: text.to_owned(),
            style: style.clone(),
            width,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ShapeKey {
    text: String,
    style: TextStyle,
    size: Size,
    scale_factor: f32,
    subpixel_origin: [f32; 2],
}

impl ShapeKey {
    pub(crate) fn new(block: &TextBlock, scale_factor: f32) -> Self {
        let pixel = [
            block.bounds.origin.x * scale_factor,
            block.bounds.origin.y * scale_factor,
        ];
        Self {
            text: block.text.clone(),
            style: block.style.clone(),
            size: block.bounds.size,
            scale_factor,
            subpixel_origin: [pixel[0] - pixel[0].round(), pixel[1] - pixel[1].round()],
        }
    }

    pub(crate) const fn subpixel_origin(&self) -> [f32; 2] {
        self.subpixel_origin
    }
}

#[derive(Clone, Copy)]
pub(crate) struct CachedGlyph {
    pub(crate) key: GlyphKey,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) rtl: bool,
    pub(crate) local: [i32; 2],
}

#[derive(Default)]
pub(crate) struct TextCache {
    measurements: VecDeque<(MeasureKey, Size)>,
    shapes: VecDeque<(ShapeKey, Vec<CachedGlyph>)>,
}

impl TextCache {
    pub(crate) fn measurement(&self, key: &MeasureKey) -> Option<Size> {
        self.measurements
            .iter()
            .find_map(|(candidate, size)| (candidate == key).then_some(*size))
    }

    pub(crate) fn insert_measurement(&mut self, key: MeasureKey, size: Size) {
        insert_bounded(&mut self.measurements, (key, size));
    }

    pub(crate) fn shape(&self, key: &ShapeKey) -> Option<Vec<CachedGlyph>> {
        self.shapes
            .iter()
            .find_map(|(candidate, glyphs)| (candidate == key).then(|| glyphs.clone()))
    }

    pub(crate) fn insert_shape(&mut self, key: ShapeKey, glyphs: Vec<CachedGlyph>) {
        insert_bounded(&mut self.shapes, (key, glyphs));
    }

    pub(crate) fn clear(&mut self) {
        self.measurements.clear();
        self.shapes.clear();
    }
}

fn insert_bounded<T>(entries: &mut VecDeque<T>, value: T) {
    if entries.len() == CACHE_CAPACITY {
        entries.pop_front();
    }
    entries.push_back(value);
}

#[cfg(test)]
mod tests {
    use super::{CACHE_CAPACITY, MeasureKey, TextCache};
    use crate::TextStyle;
    use argui_core::Size;

    #[test]
    fn cache_hits_and_stays_bounded() {
        let mut cache = TextCache::default();
        for index in 0..=CACHE_CAPACITY {
            cache.insert_measurement(
                MeasureKey::new(&index.to_string(), &TextStyle::default(), None),
                Size::new(index as f32, 1.0),
            );
        }
        assert!(
            cache
                .measurement(&MeasureKey::new("0", &TextStyle::default(), None))
                .is_none()
        );
        assert_eq!(
            cache.measurement(&MeasureKey::new(
                &CACHE_CAPACITY.to_string(),
                &TextStyle::default(),
                None
            )),
            Some(Size::new(CACHE_CAPACITY as f32, 1.0))
        );
        cache.clear();
        assert!(cache.measurements.is_empty());
    }
}
