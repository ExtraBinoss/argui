/// A point in two-dimensional logical coordinates.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    /// Horizontal coordinate.
    pub x: f32,
    /// Vertical coordinate.
    pub y: f32,
}

impl Point {
    /// Creates a point from its horizontal and vertical coordinates.
    /// * `x`, `y` — horizontal and vertical coordinates.
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// A two-dimensional extent.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
    /// Width of the extent.
    pub width: f32,
    /// Height of the extent.
    pub height: f32,
}

impl Size {
    /// Creates an extent from its width and height.
    /// * `width`, `height` — dimensions of the extent.
    #[must_use]
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

/// An axis-aligned rectangle represented by an origin and size.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    /// Top-left origin of the rectangle.
    pub origin: Point,
    /// Width and height of the rectangle.
    pub size: Size,
}

impl Rect {
    /// Creates a rectangle from its origin and size.
    /// * `origin` — top-left point; `size` — width and height.
    #[must_use]
    pub const fn new(origin: Point, size: Size) -> Self {
        Self { origin, size }
    }

    /// Returns whether `point` lies within or on the rectangle's edges.
    #[must_use]
    pub fn contains(self, point: Point) -> bool {
        point.x >= self.origin.x
            && point.y >= self.origin.y
            && point.x <= self.origin.x + self.size.width
            && point.y <= self.origin.y + self.size.height
    }

    /// Returns the area shared with `other`, or `None` if they do not overlap
    /// with positive width and height.
    #[must_use]
    pub fn intersection(self, other: Self) -> Option<Self> {
        let left = self.origin.x.max(other.origin.x);
        let top = self.origin.y.max(other.origin.y);
        let right = (self.origin.x + self.size.width).min(other.origin.x + other.size.width);
        let bottom = (self.origin.y + self.size.height).min(other.origin.y + other.size.height);
        (right > left && bottom > top)
            .then(|| Self::new(Point::new(left, top), Size::new(right - left, bottom - top)))
    }

    /// Returns the four corners in clockwise order, starting at the origin.
    #[must_use]
    pub fn corners(self) -> [Point; 4] {
        let right = self.origin.x + self.size.width;
        let bottom = self.origin.y + self.size.height;
        [
            self.origin,
            Point::new(right, self.origin.y),
            Point::new(right, bottom),
            Point::new(self.origin.x, bottom),
        ]
    }
}

/// A decomposed, renderer-independent visual transform.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform2D {
    /// Translation applied after transforming around the origin.
    pub translation: Point,
    /// Scale on the horizontal and vertical axes.
    pub scale: Point,
    /// Rotation in radians.
    pub rotation: f32,
    /// Horizontal and vertical skew angles in radians.
    pub skew: Point,
}

impl Default for Transform2D {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Transform2D {
    /// Transform with no translation, rotation, or skew and unit scale.
    pub const IDENTITY: Self = Self {
        translation: Point::new(0.0, 0.0),
        scale: Point::new(1.0, 1.0),
        rotation: 0.0,
        skew: Point::new(0.0, 0.0),
    };

    /// Sets translation in logical units.
    /// * `x`, `y` — horizontal and vertical translation.
    #[must_use]
    pub const fn translate(mut self, x: f32, y: f32) -> Self {
        self.translation = Point::new(x, y);
        self
    }

    /// Sets horizontal and vertical scale factors.
    /// * `x`, `y` — scale factors for the corresponding axes.
    #[must_use]
    pub const fn scale(mut self, x: f32, y: f32) -> Self {
        self.scale = Point::new(x, y);
        self
    }

    /// Sets rotation in radians.
    #[must_use]
    pub const fn rotate(mut self, radians: f32) -> Self {
        self.rotation = radians;
        self
    }

    /// Sets horizontal and vertical skew angles in radians.
    /// * `x_radians`, `y_radians` — skew angles for the horizontal and vertical axes.
    #[must_use]
    pub const fn skew(mut self, x_radians: f32, y_radians: f32) -> Self {
        self.skew = Point::new(x_radians, y_radians);
        self
    }

    /// Converts this transform to an affine matrix around `origin` in `bounds`.
    #[must_use]
    pub fn affine(self, bounds: Rect, origin: TransformOrigin) -> Affine2D {
        let pivot = Point::new(
            bounds.origin.x + bounds.size.width * origin.x,
            bounds.origin.y + bounds.size.height * origin.y,
        );
        let (sin, cos) = self.rotation.sin_cos();
        let skew_x = self.skew.x.tan();
        let skew_y = self.skew.y.tan();
        let linear = Affine2D {
            matrix: [
                self.scale.x * (cos - sin * skew_y),
                self.scale.x * (sin + cos * skew_y),
                self.scale.y * (cos * skew_x - sin),
                self.scale.y * (sin * skew_x + cos),
            ],
            translation: Point::default(),
        };
        Affine2D::translation(self.translation.x + pivot.x, self.translation.y + pivot.y)
            * linear
            * Affine2D::translation(-pivot.x, -pivot.y)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransformOrigin {
    /// Horizontal pivot fraction within the bounds.
    pub x: f32,
    /// Vertical pivot fraction within the bounds.
    pub y: f32,
}

impl TransformOrigin {
    /// Origin at the top-left corner of the bounds.
    pub const TOP_LEFT: Self = Self { x: 0.0, y: 0.0 };
    /// Origin at the center of the bounds.
    pub const CENTER: Self = Self { x: 0.5, y: 0.5 };

    /// Creates a pivot from normalized horizontal and vertical coordinates.
    /// * `x`, `y` — normalized pivot coordinates within the bounds.
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl Default for TransformOrigin {
    fn default() -> Self {
        Self::CENTER
    }
}

/// An affine matrix laid out as `[m11, m12, m21, m22]` plus translation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Affine2D {
    /// Linear matrix coefficients in `[m11, m12, m21, m22]` order.
    pub matrix: [f32; 4],
    /// Translation component.
    pub translation: Point,
}

impl Default for Affine2D {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Affine2D {
    /// Identity matrix with no translation.
    pub const IDENTITY: Self = Self {
        matrix: [1.0, 0.0, 0.0, 1.0],
        translation: Point::new(0.0, 0.0),
    };

    /// Creates a pure translation transform.
    /// * `x`, `y` — horizontal and vertical translation.
    #[must_use]
    pub const fn translation(x: f32, y: f32) -> Self {
        Self {
            matrix: [1.0, 0.0, 0.0, 1.0],
            translation: Point::new(x, y),
        }
    }

    /// Transforms `point` by this matrix and its translation.
    #[must_use]
    pub fn transform_point(self, point: Point) -> Point {
        Point::new(
            self.matrix[0] * point.x + self.matrix[2] * point.y + self.translation.x,
            self.matrix[1] * point.x + self.matrix[3] * point.y + self.translation.y,
        )
    }

    /// Transforms all rectangle corners and returns their axis-aligned bounds.
    /// * `rect` — rectangle whose corners are transformed.
    #[must_use]
    pub fn transform_rect(self, rect: Rect) -> Rect {
        let corners = rect.corners().map(|point| self.transform_point(point));
        let left = corners.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
        let top = corners.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
        let right = corners
            .iter()
            .map(|p| p.x)
            .fold(f32::NEG_INFINITY, f32::max);
        let bottom = corners
            .iter()
            .map(|p| p.y)
            .fold(f32::NEG_INFINITY, f32::max);
        Rect::new(Point::new(left, top), Size::new(right - left, bottom - top))
    }

    /// Computes the inverse matrix, or returns `None` when it is singular.
    #[must_use]
    pub fn inverse(self) -> Option<Self> {
        let determinant = self.matrix[0] * self.matrix[3] - self.matrix[1] * self.matrix[2];
        if determinant.abs() <= f32::EPSILON {
            return None;
        }
        let inverse = [
            self.matrix[3] / determinant,
            -self.matrix[1] / determinant,
            -self.matrix[2] / determinant,
            self.matrix[0] / determinant,
        ];
        let translation = Point::new(
            -(inverse[0] * self.translation.x + inverse[2] * self.translation.y),
            -(inverse[1] * self.translation.x + inverse[3] * self.translation.y),
        );
        Some(Self {
            matrix: inverse,
            translation,
        })
    }

    /// Scales the translation while leaving the linear matrix unchanged.
    /// * `factor` — multiplier applied to both translation coordinates.
    #[must_use]
    pub const fn scaled(self, factor: f32) -> Self {
        Self {
            matrix: self.matrix,
            translation: Point::new(self.translation.x * factor, self.translation.y * factor),
        }
    }
}

impl core::ops::Mul for Affine2D {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            matrix: [
                self.matrix[0] * rhs.matrix[0] + self.matrix[2] * rhs.matrix[1],
                self.matrix[1] * rhs.matrix[0] + self.matrix[3] * rhs.matrix[1],
                self.matrix[0] * rhs.matrix[2] + self.matrix[2] * rhs.matrix[3],
                self.matrix[1] * rhs.matrix[2] + self.matrix[3] * rhs.matrix[3],
            ],
            translation: self.transform_point(rhs.translation),
        }
    }
}
