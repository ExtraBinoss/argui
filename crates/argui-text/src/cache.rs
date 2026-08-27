use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use argui_core::Size;

use crate::{FontFamily, GlyphKey, TextBlock, TextStyle, TextWrap};

const CACHE_CAPACITY: usize = 512;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct MeasureKey {
    text: String,
    style: StyleKey,
    width: Option<u32>,
}

impl MeasureKey {
    pub(crate) fn new(text: &str, style: &TextStyle, width: Option<f32>) -> Self {
        Self {
            text: text.to_owned(),
            style: StyleKey::new(style),
            width: width.map(f32::to_bits),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct ShapeKey {
    text: String,
    style: StyleKey,
    size: [u32; 2],
    scale_factor: u32,
    subpixel_origin: [u32; 2],
}

impl ShapeKey {
    pub(crate) fn new(block: &TextBlock, scale_factor: f32) -> Self {
        let pixel = [
            block.bounds.origin.x * scale_factor,
            block.bounds.origin.y * scale_factor,
        ];
        Self {
            text: block.text.clone(),
            style: StyleKey::new(&block.style),
            size: [
                block.bounds.size.width.to_bits(),
                block.bounds.size.height.to_bits(),
            ],
            scale_factor: scale_factor.to_bits(),
            subpixel_origin: [
                (pixel[0] - pixel[0].round()).to_bits(),
                (pixel[1] - pixel[1].round()).to_bits(),
            ],
        }
    }

    pub(crate) fn subpixel_origin(&self) -> [f32; 2] {
        self.subpixel_origin.map(f32::from_bits)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct StyleKey {
    font_size: u32,
    line_height: u32,
    family: FontFamily,
    weight: u16,
    wrap: TextWrap,
}

impl StyleKey {
    fn new(style: &TextStyle) -> Self {
        Self {
            font_size: style.font_size.to_bits(),
            line_height: style.line_height.to_bits(),
            family: style.family.clone(),
            weight: style.weight,
            wrap: style.wrap,
        }
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
    measurements: HashMap<MeasureKey, Size>,
    measurement_order: VecDeque<MeasureKey>,
    shapes: HashMap<ShapeKey, Arc<[CachedGlyph]>>,
    shape_order: VecDeque<ShapeKey>,
}

impl TextCache {
    pub(crate) fn measurement(&self, key: &MeasureKey) -> Option<Size> {
        self.measurements.get(key).copied()
    }

    pub(crate) fn insert_measurement(&mut self, key: MeasureKey, size: Size) {
        insert_bounded(
            &mut self.measurements,
            &mut self.measurement_order,
            key,
            size,
        );
    }

    pub(crate) fn shape(&self, key: &ShapeKey) -> Option<Arc<[CachedGlyph]>> {
        self.shapes.get(key).cloned()
    }

    pub(crate) fn insert_shape(
        &mut self,
        key: ShapeKey,
        glyphs: Vec<CachedGlyph>,
    ) -> Arc<[CachedGlyph]> {
        let glyphs = Arc::from(glyphs);
        insert_bounded(
            &mut self.shapes,
            &mut self.shape_order,
            key,
            Arc::clone(&glyphs),
        );
        glyphs
    }

    pub(crate) fn clear(&mut self) {
        self.measurements.clear();
        self.measurement_order.clear();
        self.shapes.clear();
        self.shape_order.clear();
    }
}

fn insert_bounded<K, V>(entries: &mut HashMap<K, V>, order: &mut VecDeque<K>, key: K, value: V)
where
    K: Clone + Eq + std::hash::Hash,
{
    if let std::collections::hash_map::Entry::Occupied(mut entry) = entries.entry(key.clone()) {
        entry.insert(value);
        return;
    }
    if entries.len() == CACHE_CAPACITY
        && let Some(oldest) = order.pop_front()
    {
        entries.remove(&oldest);
    }
    order.push_back(key.clone());
    entries.insert(key, value);
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
