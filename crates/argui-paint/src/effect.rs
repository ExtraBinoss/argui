use argui_core::{Affine2D, Color, Name, Point, Rect, Size};

use crate::CornerRadii;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ProfileDomain {
    Ui,
    Overlay,
    Engine,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RenderObjectId {
    pub domain: ProfileDomain,
    pub value: u64,
}

impl RenderObjectId {
    /// Creates an object identifier in the supplied profiling domain.
    /// * `value` — object identity within `domain`.
    #[must_use]
    pub const fn new(domain: ProfileDomain, value: u64) -> Self {
        Self { domain, value }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct EffectId(Name);

impl EffectId {
    /// Creates an effect identifier from a namespaced static name.
    /// * `namespaced_name` — stable namespaced effect name.
    #[must_use]
    pub const fn new(namespaced_name: &'static str) -> Self {
        Self(Name::from_static(namespaced_name))
    }

    /// Creates an effect identifier from dynamically loaded owned text.
    ///
    /// * `namespaced_name` — stable namespaced effect name owned by this identifier.
    #[must_use]
    pub fn from_owned(namespaced_name: String) -> Self {
        Self(Name::from_owned(namespaced_name))
    }

    /// Creates an effect identifier from an existing engine name.
    ///
    /// * `name` — shared engine-level name to wrap.
    #[must_use]
    pub fn from_name(name: Name) -> Self {
        Self(name)
    }

    /// Returns the namespaced effect name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl std::fmt::Display for EffectId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum EffectValue {
    F32(f32),
    I32(i32),
    U32(u32),
    Bool(bool),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Mat3([f32; 9]),
    Mat4([f32; 16]),
    Color(Color),
    LogicalPixels(f32),
}

impl EffectValue {
    /// Scales logical-pixel values by `factor`; leaves other value kinds unchanged.
    /// * `factor` — multiplier for logical-pixel values.
    #[must_use]
    pub fn scaled(&self, factor: f32) -> Self {
        match self {
            Self::LogicalPixels(value) => Self::LogicalPixels(value * factor),
            value => value.clone(),
        }
    }

    /// Appends this value's packed 32-bit representation to `words`.
    pub fn write_words(&self, words: &mut Vec<u32>) {
        match self {
            Self::F32(value) | Self::LogicalPixels(value) => words.push(value.to_bits()),
            Self::I32(value) => words.push(*value as u32),
            Self::U32(value) => words.push(*value),
            Self::Bool(value) => words.push(u32::from(*value)),
            Self::Vec2(values) => write_f32_words(words, values),
            Self::Vec3(values) => write_f32_words(words, values),
            Self::Vec4(values) => write_f32_words(words, values),
            Self::Color(value) => write_f32_words(words, &value.to_linear_rgba()),
            Self::Mat3(values) => write_f32_words(words, values),
            Self::Mat4(values) => write_f32_words(words, values),
        }
    }
}

fn write_f32_words(words: &mut Vec<u32>, values: &[f32]) {
    words.extend(values.iter().map(|value| value.to_bits()));
}

#[derive(Clone, Debug, PartialEq)]
pub struct EffectArgument {
    pub name: Name,
    pub value: EffectValue,
}

impl EffectArgument {
    /// Creates a named effect parameter.
    /// * `name` — parameter name; `value` — parameter value.
    #[must_use]
    pub const fn new(name: &'static str, value: EffectValue) -> Self {
        Self {
            name: Name::from_static(name),
            value,
        }
    }

    /// Creates a parameter from a dynamically loaded owned name.
    ///
    /// * `name` — parameter name owned by the argument; `value` — parameter value.
    #[must_use]
    pub fn from_owned(name: String, value: EffectValue) -> Self {
        Self {
            name: Name::from_owned(name),
            value,
        }
    }
}

impl From<(&'static str, EffectValue)> for EffectArgument {
    fn from((name, value): (&'static str, EffectValue)) -> Self {
        Self::new(name, value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EffectInstance {
    pub id: EffectId,
    pub parameters: Vec<EffectArgument>,
    pub expansion: f32,
}

impl EffectInstance {
    /// Creates an effect instance from an identifier and ordered parameter values.
    /// * `id` — effect identifier; `parameters` — ordered effect parameter values.
    #[must_use]
    pub fn new<I, A>(id: EffectId, parameters: I) -> Self
    where
        I: IntoIterator<Item = A>,
        A: Into<EffectArgument>,
    {
        Self {
            id,
            parameters: parameters.into_iter().map(Into::into).collect(),
            expansion: 0.0,
        }
    }

    /// Sets the effect's bounds expansion in logical pixels, clamped at zero.
    #[must_use]
    pub fn expansion(mut self, pixels: f32) -> Self {
        self.expansion = pixels.max(0.0);
        self
    }

    /// Packs parameter values into 32-bit words in parameter order.
    #[must_use]
    pub fn packed_words(&self) -> Vec<u32> {
        let mut words = Vec::new();
        for parameter in &self.parameters {
            parameter.value.write_words(&mut words);
        }
        words
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Refraction {
    pub strength: f32,
    pub chromatic_aberration: f32,
    pub edge: f32,
}

impl Refraction {
    /// Creates a refraction effect with the given strength.
    #[must_use]
    pub const fn new(strength: f32) -> Self {
        Self {
            strength,
            chromatic_aberration: 0.0,
            edge: 0.15,
        }
    }

    /// Sets chromatic aberration amount.
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
    Effect(EffectInstance),
}

impl Filter {
    /// Returns the bounds expansion required by this filter.
    #[must_use]
    pub fn expansion(&self) -> f32 {
        match self {
            Self::Blur(radius) => radius.max(0.0) * 3.0,
            Self::Effect(effect) => effect.expansion,
            _ => 0.0,
        }
    }

    /// Scales logical dimensions while preserving dimensionless filter values.
    /// * `factor` — multiplier applied to logical dimensions.
    #[must_use]
    pub fn scaled(&self, factor: f32) -> Self {
        match self {
            Self::Blur(radius) => Self::Blur(radius * factor),
            Self::Effect(effect) => Self::Effect(EffectInstance {
                id: effect.id.clone(),
                parameters: effect
                    .parameters
                    .iter()
                    .map(|argument| EffectArgument {
                        name: argument.name.clone(),
                        value: argument.value.scaled(factor),
                    })
                    .collect(),
                expansion: effect.expansion * factor,
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
    /// Creates an offset drop shadow with zero spread.
    /// * `offset` — horizontal and vertical displacement; `blur` — blur radius; `color` — shadow color.
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

    /// Creates a centered glow shadow.
    /// * `blur` — blur radius; `color` — glow color.
    #[must_use]
    pub const fn glow(blur: f32, color: Color) -> Self {
        Self::drop([0.0, 0.0], blur, color)
    }

    /// Sets the blur radius.
    #[must_use]
    pub const fn blur(mut self, radius: f32) -> Self {
        self.blur = radius;
        self
    }

    /// Sets the spread radius.
    #[must_use]
    pub const fn spread(mut self, radius: f32) -> Self {
        self.spread = radius;
        self
    }

    /// Sets whether the shadow is inset.
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
    /// Composition-time transform applied after the layer content is rendered.
    pub transform: Affine2D,
    /// Whether the renderer should retain this layer even at identity presentation.
    pub retained: bool,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub filters: Vec<Filter>,
    pub backdrop_filters: Vec<Filter>,
    pub shadows: Vec<Shadow>,
    pub mask: LayerMask,
    pub profile: Option<RenderObjectId>,
}

impl LayerStyle {
    /// Creates a layer covering `bounds` with default compositing settings.
    #[must_use]
    pub const fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            transform: Affine2D::IDENTITY,
            retained: false,
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
            filters: Vec::new(),
            backdrop_filters: Vec::new(),
            shadows: Vec::new(),
            mask: LayerMask::None,
            profile: None,
        }
    }

    /// Adds a foreground filter.
    #[must_use]
    pub fn filter(mut self, filter: Filter) -> Self {
        self.filters.push(filter);
        self
    }

    /// Adds a filter applied to the backdrop behind the layer.
    #[must_use]
    pub fn backdrop(mut self, filter: Filter) -> Self {
        self.backdrop_filters.push(filter);
        self
    }

    /// Adds a shadow to the layer.
    #[must_use]
    pub fn shadow(mut self, shadow: Shadow) -> Self {
        self.shadows.push(shadow);
        self
    }

    /// Sets layer opacity.
    #[must_use]
    pub const fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }

    /// Sets the composition-time transform applied to the completed layer.
    ///
    /// * `transform` — affine transform in logical surface coordinates.
    #[must_use]
    pub const fn transform(mut self, transform: Affine2D) -> Self {
        self.transform = transform;
        self
    }

    /// Marks this layer as retained compositor content.
    ///
    /// Retained layers receive an offscreen surface even while their current
    /// transform and opacity are at identity, avoiding a promotion hitch when
    /// an animation begins.
    #[must_use]
    pub const fn retained(mut self, retained: bool) -> Self {
        self.retained = retained;
        self
    }

    /// Sets the blend mode used when compositing the layer.
    /// * `blend_mode` — compositing blend mode.
    #[must_use]
    pub const fn blend(mut self, blend_mode: BlendMode) -> Self {
        self.blend_mode = blend_mode;
        self
    }

    /// Sets the mask applied to the layer.
    #[must_use]
    pub const fn mask(mut self, mask: LayerMask) -> Self {
        self.mask = mask;
        self
    }

    /// Associates a stable object identifier for profiling.
    /// * `profile` — stable profiling identity for the layer.
    #[must_use]
    pub const fn profile(mut self, profile: RenderObjectId) -> Self {
        self.profile = Some(profile);
        self
    }

    /// Returns whether the style requires an offscreen compositing pass.
    #[must_use]
    pub fn requires_offscreen(&self) -> bool {
        self.retained
            || self.transform != Affine2D::IDENTITY
            || self.opacity != 1.0
            || self.blend_mode != BlendMode::Normal
            || !self.filters.is_empty()
            || !self.backdrop_filters.is_empty()
            || !self.shadows.is_empty()
            || self.mask != LayerMask::None
    }

    /// Returns layer bounds expanded for foreground filters and shadows.
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

    /// Returns expanded layer bounds after applying the composition transform.
    #[must_use]
    pub fn transformed_bounds(&self) -> Rect {
        self.transform.transform_rect(self.expanded_bounds())
    }

    /// Returns bounds expanded for foreground filters only.
    #[must_use]
    pub fn foreground_expansion(&self) -> f32 {
        self.filters.iter().map(Filter::expansion).sum()
    }

    /// Returns bounds expanded for foreground filters only.
    #[must_use]
    pub fn foreground_bounds(&self) -> Rect {
        outset(self.bounds, self.foreground_expansion())
    }

    /// Returns a copy with logical dimensions and effects scaled by `factor`.
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
        scaled.transform = self.transform.scaled(factor);
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
