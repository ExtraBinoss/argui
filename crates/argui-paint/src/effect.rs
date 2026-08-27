use argui_core::{Color, Point, Rect, Size};

use crate::CornerRadii;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct ShaderEffectId(pub u64);

#[derive(Clone, Debug, PartialEq)]
pub struct CustomEffect {
    pub shader: ShaderEffectId,
    pub parameters: Vec<f32>,
    pub expansion: f32,
    pub pixel_parameters: u32,
}

impl CustomEffect {
    #[must_use]
    pub fn new(shader: ShaderEffectId, parameters: impl Into<Vec<f32>>) -> Self {
        Self {
            shader,
            parameters: parameters.into(),
            expansion: 0.0,
            pixel_parameters: 0,
        }
    }

    #[must_use]
    pub fn expansion(mut self, pixels: f32) -> Self {
        self.expansion = pixels.max(0.0);
        self
    }

    /// Marks one parameter as a logical-pixel value that follows DPI scaling.
    #[must_use]
    pub fn pixel_parameter(mut self, index: usize) -> Self {
        if index < 24 {
            self.pixel_parameters |= 1 << index;
        }
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Refraction {
    pub strength: f32,
    pub chromatic_aberration: f32,
    pub edge: f32,
}

impl Refraction {
    #[must_use]
    pub const fn new(strength: f32) -> Self {
        Self {
            strength,
            chromatic_aberration: 0.0,
            edge: 0.15,
        }
    }

    #[must_use]
    pub const fn chromatic_aberration(mut self, amount: f32) -> Self {
        self.chromatic_aberration = amount;
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Filter {
    Blur(f32),
    Brightness(f32),
    Contrast(f32),
    Saturation(f32),
    HueRotate(f32),
    Opacity(f32),
    ColorMatrix([f32; 20]),
    Refraction(Refraction),
    Custom(CustomEffect),
}

impl Filter {
    #[must_use]
    pub fn expansion(&self) -> f32 {
        match self {
            Self::Blur(radius) => radius.max(0.0) * 3.0,
            Self::Custom(effect) => effect.expansion,
            _ => 0.0,
        }
    }

    #[must_use]
    pub fn scaled(&self, factor: f32) -> Self {
        match self {
            Self::Blur(radius) => Self::Blur(radius * factor),
            Self::Custom(effect) => Self::Custom(CustomEffect {
                shader: effect.shader,
                parameters: effect
                    .parameters
                    .iter()
                    .enumerate()
                    .map(|(index, value)| {
                        if effect.pixel_parameters & (1 << index) != 0 {
                            value * factor
                        } else {
                            *value
                        }
                    })
                    .collect(),
                expansion: effect.expansion * factor,
                pixel_parameters: effect.pixel_parameters,
            }),
            other => other.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BlendMode {
    #[default]
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    Difference,
    Exclusion,
    PlusLighter,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shadow {
    pub offset: [f32; 2],
    pub blur: f32,
    pub spread: f32,
    pub color: Color,
    pub inset: bool,
}

impl Shadow {
    #[must_use]
    pub const fn drop(offset: [f32; 2], blur: f32, color: Color) -> Self {
        Self {
            offset,
            blur,
            spread: 0.0,
            color,
            inset: false,
        }
    }

    #[must_use]
    pub const fn glow(blur: f32, color: Color) -> Self {
        Self::drop([0.0, 0.0], blur, color)
    }

    #[must_use]
    pub const fn blur(mut self, radius: f32) -> Self {
        self.blur = radius;
        self
    }

    #[must_use]
    pub const fn spread(mut self, radius: f32) -> Self {
        self.spread = radius;
        self
    }

    #[must_use]
    pub const fn inset(mut self, inset: bool) -> Self {
        self.inset = inset;
        self
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum LayerMask {
    #[default]
    None,
    Bounds,
    Rounded(CornerRadii),
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayerStyle {
    pub bounds: Rect,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub filters: Vec<Filter>,
    pub backdrop_filters: Vec<Filter>,
    pub shadows: Vec<Shadow>,
    pub mask: LayerMask,
}

impl LayerStyle {
    #[must_use]
    pub const fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
            filters: Vec::new(),
            backdrop_filters: Vec::new(),
            shadows: Vec::new(),
            mask: LayerMask::None,
        }
    }

    #[must_use]
    pub fn filter(mut self, filter: Filter) -> Self {
        self.filters.push(filter);
        self
    }

    #[must_use]
    pub fn backdrop(mut self, filter: Filter) -> Self {
        self.backdrop_filters.push(filter);
        self
    }

    #[must_use]
    pub fn shadow(mut self, shadow: Shadow) -> Self {
        self.shadows.push(shadow);
        self
    }

    #[must_use]
    pub const fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }

    #[must_use]
    pub const fn blend(mut self, blend_mode: BlendMode) -> Self {
        self.blend_mode = blend_mode;
        self
    }

    #[must_use]
    pub const fn mask(mut self, mask: LayerMask) -> Self {
        self.mask = mask;
        self
    }

    #[must_use]
    pub fn requires_offscreen(&self) -> bool {
        self.opacity != 1.0
            || self.blend_mode != BlendMode::Normal
            || !self.filters.is_empty()
            || !self.backdrop_filters.is_empty()
            || !self.shadows.is_empty()
            || self.mask != LayerMask::None
    }

    #[must_use]
    pub fn expanded_bounds(&self) -> Rect {
        let mut expansion = self.foreground_expansion();
        for shadow in &self.shadows {
            expansion = expansion.max(
                shadow.blur.max(0.0) * 3.0
                    + shadow.spread.max(0.0)
                    + shadow.offset[0].abs().max(shadow.offset[1].abs()),
            );
        }
        Rect::new(
            Point::new(
                self.bounds.origin.x - expansion,
                self.bounds.origin.y - expansion,
            ),
            Size::new(
                self.bounds.size.width + expansion * 2.0,
                self.bounds.size.height + expansion * 2.0,
            ),
        )
    }

    #[must_use]
    pub fn foreground_expansion(&self) -> f32 {
        self.filters.iter().map(Filter::expansion).sum()
    }

    #[must_use]
    pub fn foreground_bounds(&self) -> Rect {
        outset(self.bounds, self.foreground_expansion())
    }

    #[must_use]
    pub fn scaled(&self, factor: f32) -> Self {
        let mut scaled = self.clone();
        scaled.bounds = Rect::new(
            Point::new(self.bounds.origin.x * factor, self.bounds.origin.y * factor),
            Size::new(
                self.bounds.size.width * factor,
                self.bounds.size.height * factor,
            ),
        );
        scaled.filters = self
            .filters
            .iter()
            .map(|filter| filter.scaled(factor))
            .collect();
        scaled.backdrop_filters = self
            .backdrop_filters
            .iter()
            .map(|filter| filter.scaled(factor))
            .collect();
        for shadow in &mut scaled.shadows {
            shadow.offset[0] *= factor;
            shadow.offset[1] *= factor;
            shadow.blur *= factor;
            shadow.spread *= factor;
        }
        if let LayerMask::Rounded(radii) = scaled.mask {
            scaled.mask = LayerMask::Rounded(radii.scaled(factor));
        }
        scaled
    }
}

fn outset(rect: Rect, amount: f32) -> Rect {
    Rect::new(
        Point::new(rect.origin.x - amount, rect.origin.y - amount),
        Size::new(
            rect.size.width + amount * 2.0,
            rect.size.height + amount * 2.0,
        ),
    )
}
