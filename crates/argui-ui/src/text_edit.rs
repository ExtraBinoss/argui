use std::{fmt, ops::Range};

/// One accepted text-buffer replacement expressed with UTF-8 byte offsets.
///
/// Engine-generated edits always point at character boundaries in the value
/// that existed immediately before the edit. Applying edits instead of copying
/// the complete value keeps large controlled editors responsive while typing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextEdit {
    /// Half-open UTF-8 byte range removed from the previous value.
    pub range: Range<usize>,
    /// Text inserted at the beginning of [`Self::range`].
    pub replacement: String,
}

impl TextEdit {
    /// Creates a text replacement without applying it.
    ///
    /// * `range` — half-open UTF-8 byte range in the value before the edit.
    /// * `replacement` — text that replaces the range.
    #[must_use]
    pub fn new(range: Range<usize>, replacement: impl Into<String>) -> Self {
        Self {
            range,
            replacement: replacement.into(),
        }
    }

    /// Applies this replacement to `value`.
    ///
    /// Returns an error when the range is reversed, outside the current value,
    /// or does not lie on UTF-8 character boundaries. The value is unchanged on
    /// error.
    pub fn apply_to(&self, value: &mut String) -> Result<(), TextEditError> {
        validate_range(value, &self.range)?;
        value.replace_range(self.range.clone(), &self.replacement);
        Ok(())
    }
}

/// Reason a [`TextEdit`] could not be applied to a particular string.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextEditError {
    /// The start offset was greater than the end offset.
    ReversedRange {
        /// Invalid starting byte offset.
        start: usize,
        /// Invalid ending byte offset.
        end: usize,
    },
    /// The end offset exceeded the current text length.
    OutOfBounds {
        /// Requested ending byte offset.
        end: usize,
        /// Current value length in bytes.
        length: usize,
    },
    /// An offset split a UTF-8 code point.
    NotCharBoundary {
        /// Byte offset that was not a character boundary.
        index: usize,
    },
}

impl fmt::Display for TextEditError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReversedRange { start, end } => {
                write!(
                    formatter,
                    "text edit range starts at {start} after its end at {end}"
                )
            }
            Self::OutOfBounds { end, length } => {
                write!(
                    formatter,
                    "text edit ends at {end}, beyond value length {length}"
                )
            }
            Self::NotCharBoundary { index } => {
                write!(
                    formatter,
                    "text edit offset {index} splits a UTF-8 character"
                )
            }
        }
    }
}

impl std::error::Error for TextEditError {}

/// Validates that `range` can safely address `value`.
fn validate_range(value: &str, range: &Range<usize>) -> Result<(), TextEditError> {
    if range.start > range.end {
        return Err(TextEditError::ReversedRange {
            start: range.start,
            end: range.end,
        });
    }
    if range.end > value.len() {
        return Err(TextEditError::OutOfBounds {
            end: range.end,
            length: value.len(),
        });
    }
    for index in [range.start, range.end] {
        if !value.is_char_boundary(index) {
            return Err(TextEditError::NotCharBoundary { index });
        }
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AppliedTextEdit {
    pub edit: TextEdit,
    pub removed: String,
}

impl AppliedTextEdit {
    /// Replaces a validated range and retains the removed bytes for undo history.
    pub(crate) fn apply(value: &mut String, edit: TextEdit) -> Self {
        debug_assert!(validate_range(value, &edit.range).is_ok());
        let removed = value[edit.range.clone()].to_owned();
        value.replace_range(edit.range.clone(), &edit.replacement);
        Self { edit, removed }
    }

    /// Describes the smallest replacement that turns `before` into `after`.
    pub(crate) fn between(before: &str, after: &str) -> Self {
        let mut start = before
            .bytes()
            .zip(after.bytes())
            .take_while(|(left, right)| left == right)
            .count();
        while !before.is_char_boundary(start) || !after.is_char_boundary(start) {
            start -= 1;
        }
        let mut suffix = before[start..]
            .bytes()
            .rev()
            .zip(after[start..].bytes().rev())
            .take_while(|(left, right)| left == right)
            .count();
        while !before.is_char_boundary(before.len() - suffix)
            || !after.is_char_boundary(after.len() - suffix)
        {
            suffix -= 1;
        }
        Self {
            edit: TextEdit::new(
                start..before.len() - suffix,
                &after[start..after.len() - suffix],
            ),
            removed: before[start..before.len() - suffix].to_owned(),
        }
    }
}
