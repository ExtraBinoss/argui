use unicode_segmentation::UnicodeSegmentation;

/// Returns two byte offsets in ascending order.
pub(super) const fn ordered(left: usize, right: usize) -> (usize, usize) {
    if left <= right {
        (left, right)
    } else {
        (right, left)
    }
}

/// Returns the grapheme boundary immediately before `cursor`.
pub(super) fn previous_grapheme(value: &str, cursor: usize) -> usize {
    value[..cursor]
        .grapheme_indices(true)
        .next_back()
        .map_or(0, |(index, _)| index)
}

/// Returns the grapheme boundary immediately after `cursor`.
pub(super) fn next_grapheme(value: &str, cursor: usize) -> usize {
    value[cursor..]
        .grapheme_indices(true)
        .nth(1)
        .map_or(value.len(), |(index, _)| cursor + index)
}

/// Clamps `index` backward to a UTF-8 character boundary.
pub(super) fn char_boundary(value: &str, index: usize) -> usize {
    (0..=index)
        .rev()
        .find(|candidate| value.is_char_boundary(*candidate))
        .unwrap_or(0)
}

/// Clamps `index` backward to a Unicode grapheme boundary.
pub(super) fn grapheme_boundary(value: &str, index: usize) -> usize {
    if index >= value.len() {
        return value.len();
    }
    value
        .grapheme_indices(true)
        .take_while(|(boundary, _)| *boundary <= index)
        .last()
        .map_or(0, |(boundary, _)| boundary)
}

/// Returns the byte offset at the beginning of the cursor's logical line.
pub(super) fn line_start(value: &str, cursor: usize) -> usize {
    value[..cursor].rfind('\n').map_or(0, |index| index + 1)
}

/// Returns the byte offset at the end of the cursor's logical line.
pub(super) fn line_end(value: &str, cursor: usize) -> usize {
    value[cursor..]
        .find('\n')
        .map_or(value.len(), |index| cursor + index)
}
