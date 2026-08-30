use crate::{Easing, Interpolate, TimingError};

#[derive(Clone, Debug, PartialEq)]
pub struct Keyframe<T> {
    pub offset: f32,
    pub value: T,
    pub easing: Easing,
    pub hold: bool,
}

impl<T> Keyframe<T> {
    #[must_use]
    pub fn new(offset: f32, value: T) -> Self {
        Self {
            offset,
            value,
            easing: Easing::Linear,
            hold: false,
        }
    }

    #[must_use]
    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    #[must_use]
    pub const fn hold(mut self) -> Self {
        self.hold = true;
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Keyframes<T> {
    frames: Box<[Keyframe<T>]>,
}

impl<T> Keyframes<T> {
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

    #[must_use]
    pub fn as_slice(&self) -> &[Keyframe<T>] {
        &self.frames
    }
}

impl<T: Clone + Interpolate> Keyframes<T> {
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
