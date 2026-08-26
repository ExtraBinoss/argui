use cosmic_text::FontSystem;

pub struct TextEngine {
    fonts: FontSystem,
}

impl Default for TextEngine {
    fn default() -> Self {
        Self {
            fonts: FontSystem::new(),
        }
    }
}

impl TextEngine {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fonts_mut(&mut self) -> &mut FontSystem {
        &mut self.fonts
    }
}
