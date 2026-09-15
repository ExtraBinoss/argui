use crate::{Easing, Interpolate, TimingError};

#[derive(Clone, Debug, PartialEq)]
/// Value and interpolation settings at a normalized timeline offset.
pub struct Keyframe<T> {
    /// Position in the normalized interval from zero to one.
    pub offset: f32,
    /// Value at this keyframe.
    pub value: T,
    /// Easing applied from this frame to the next.
    pub easing: Easing,
    /// Whether to hold this value until the next frame rather than interpolate.
    pub hold: bool,
}

impl<T> Keyframe<T> {
    /// Creates a keyframe with linear easing and no hold.
    /// * `offset` — normalized timeline position; `value` — value at that position.
    #[must_use]
    pub fn new(offset: f32, value: T) -> Self {
        Self {
            offset,
            value,
            easing: Easing::Linear,
            hold: false,
        }
    }

    /// Sets the easing used from this frame to its successor.
    #[must_use]
    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Holds this keyframe's value until the next keyframe.
    #[must_use]
    pub const fn hold(mut self) -> Self {
        self.hold = true;
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
/// Validated, ordered keyframes spanning the complete normalized interval.
pub struct Keyframes<T> {
    frames: Box<[Keyframe<T>]>,
}

impl<T> Keyframes<T> {
    /// Creates keyframes covering offsets zero through one in nondecreasing order.
    ///
    /// # Errors
    /// Returns [`TimingError::InvalidKeyframes`] if there are fewer than two frames,
    /// the endpoints are not zero and one, or offsets are invalid or unsorted.
    pub fn new(frames: impl Into<Vec<Keyframe<T>>>) -> Result<Self, TimingError> {
        let frames = frames.into();
        if frames.len() < 2
            || frames.first().is_none_or(|frame| frame.offset != 0.0)
            || frames.last().is_none_or(|frame| frame.offset != 1.0)
            || frames
                .iter()
                .any(|frame| !frame.offset.is_finite() || !(0.0..=1.0).contains(&frame.offset))
            || frames
                .windows(2)
                .any(|pair| pair[0].offset > pair[1].offset)
        {
            return Err(TimingError::InvalidKeyframes);
        }
        Ok(Self {
            frames: frames.into_boxed_slice(),
        })
    }

    /// Returns the validated keyframes in offset order.
    #[must_use]
    pub fn as_slice(&self) -> &[Keyframe<T>] {
        &self.frames
    }
}

impl<T: Clone + Interpolate> Keyframes<T> {
    /// Samples the keyframes at `progress`, clamped to the normalized interval.
    #[must_use]
    pub fn sample(&self, progress: f32) -> T {
        let progress = progress.clamp(0.0, 1.0);
        let upper = self
            .frames
            .partition_point(|frame| frame.offset <= progress);
        if upper == 0 {
            return self.frames[0].value.clone();
        }
        if upper == self.frames.len() {
            return self.frames[upper - 1].value.clone();
        }
        let from = &self.frames[upper - 1];
        let to = &self.frames[upper];
        if from.hold || from.offset == to.offset {
            return from.value.clone();
        }
        let local = (progress - from.offset) / (to.offset - from.offset);
        from.value
            .clone()
            .interpolate(to.value.clone(), from.easing.sample(local))
    }
}
