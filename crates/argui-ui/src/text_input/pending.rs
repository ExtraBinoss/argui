use std::collections::VecDeque;

use super::{TextInputFilter, TextInputState, TextPosition, grapheme_boundary};
use crate::{AppliedTextEdit, TextEdit};

/// One native value and the edit that produced it while awaiting an echo.
#[derive(Clone, PartialEq)]
pub(super) struct PendingEdit {
    /// UTF-8 byte length of the native value after this edit.
    pub(super) length: usize,
    /// Fast prefilter for equal-length acknowledgements; equality is verified.
    pub(super) checksum: u64,
    /// Replacement against the native value before this edit.
    pub(super) edit: TextEdit,
}

impl PendingEdit {
    /// Records the current `value`, cumulative `checksum`, and producing `edit`.
    pub(super) fn new(value: &str, checksum: u64, edit: &TextEdit) -> Self {
        Self {
            length: value.len(),
            checksum,
            edit: edit.clone(),
        }
    }
}

impl TextInputState {
    /// Reconciles a controlled property echo or external replacement with native edits.
    ///
    /// `value` is the authored text; the remaining arguments update the input's
    /// multiline, read-only and filter policy. A recognized older echo retains
    /// later native edits; an external value replaces them.
    pub fn sync(&mut self, value: &str, multiline: bool, read_only: bool, filter: TextInputFilter) {
        if self.multiline != multiline || self.filter != filter {
            self.history.clear();
            self.preedit = None;
        }
        self.multiline = multiline;
        self.read_only = read_only;
        self.filter = filter;
        if self.authored_value == value {
            return;
        }
        if self.value == value {
            self.authored_value = value.to_owned();
            self.pending_values.clear();
            self.pending_insert = None;
            self.pending_checkpoints.clear();
            self.content_checksum = 0;
            self.authored_checksum = 0;
            return;
        }
        if let Some((index, inserted_bytes)) = self
            .pending_insert
            .as_ref()
            .and_then(|run| run.acknowledged(&self.authored_value, value, &self.pending_values))
        {
            self.authored_checksum = self.pending_values[index].checksum;
            self.authored_value = value.to_owned();
            self.pending_values.drain(..=index);
            self.pending_checkpoints.acknowledge(index + 1);
            if let Some(run) = &mut self.pending_insert {
                run.consume(inserted_bytes);
            }
            if self.pending_values.is_empty() {
                self.pending_insert = None;
                self.pending_checkpoints.clear();
                self.content_checksum = 0;
                self.authored_checksum = 0;
            }
            return;
        }
        let acknowledged = acknowledged_edits(
            &self.authored_value,
            value,
            &self.pending_values,
            &self.pending_checkpoints,
            self.authored_checksum,
        );
        self.authored_value = value.to_owned();
        if let Some(index) = acknowledged {
            self.authored_checksum = self.pending_values[index].checksum;
            self.pending_values.drain(..=index);
            self.pending_insert = None;
            self.pending_checkpoints.acknowledge(index + 1);
            if self.pending_values.is_empty() {
                self.pending_checkpoints.clear();
                self.content_checksum = 0;
                self.authored_checksum = 0;
            }
            return;
        }
        self.pending_values.clear();
        self.pending_insert = None;
        self.pending_checkpoints.clear();
        self.content_checksum = 0;
        self.authored_checksum = 0;
        if self.value == value {
            return;
        }
        self.history.clear();
        self.value.clear();
        self.goal_x = None;
        self.value.push_str(value);
        self.cursor = grapheme_boundary(&self.value, self.cursor.min(self.value.len()));
        self.anchor = self.anchor.and_then(|anchor| {
            (anchor.index <= self.value.len()).then(|| {
                TextPosition::new(
                    grapheme_boundary(&self.value, anchor.index),
                    anchor.affinity,
                )
            })
        });
        self.preedit = None;
        self.reveal_cursor = true;
    }

    /// Records a native edit awaiting acknowledgement from controlled properties.
    ///
    /// `edit` is the accepted replacement against the previous native value.
    pub(crate) fn note_current_value(&mut self, edit: &AppliedTextEdit) {
        let replacement = &edit.edit;
        if self.pending_values.is_empty() {
            self.pending_insert = PendingInsert::new(replacement);
        } else if let Some(run) = &mut self.pending_insert
            && !run.push(replacement)
        {
            self.pending_insert = None;
        }
        self.content_checksum = self
            .content_checksum
            .wrapping_add(checksum(&replacement.replacement))
            .wrapping_sub(checksum(&edit.removed));
        self.pending_values.push_back(PendingEdit::new(
            &self.value,
            self.content_checksum,
            replacement,
        ));
        self.pending_checkpoints
            .record(self.pending_values.len(), &self.value);
    }
}

/// A contiguous run of controlled insertions after the last authored value.
#[derive(Clone, PartialEq)]
pub(super) struct PendingInsert {
    start: usize,
    text: String,
}

/// Sparse full values used to bound the replay span for mixed controlled edits.
#[derive(Clone, PartialEq)]
pub(super) struct PendingCheckpoints {
    values: VecDeque<(usize, String)>,
    interval: usize,
    next: usize,
    bytes: usize,
}

impl Default for PendingCheckpoints {
    /// Starts with a checkpoint every 256 edits and a 32 MiB value budget.
    fn default() -> Self {
        Self {
            values: VecDeque::new(),
            interval: 256,
            next: 256,
            bytes: 0,
        }
    }
}

impl PendingCheckpoints {
    /// Records `value` after `count` pending edits when a checkpoint is due.
    pub(super) fn record(&mut self, count: usize, value: &str) {
        if count != self.next || value.len() > 32 * 1024 * 1024 {
            return;
        }
        self.values.push_back((count, value.to_owned()));
        self.bytes += value.len();
        self.next = count.saturating_add(self.interval);
        while self.bytes > 32 * 1024 * 1024 && self.values.len() > 1 {
            let mut index = 0;
            self.values.retain(|(_, value)| {
                let keep = index % 2 == 1;
                index += 1;
                if !keep {
                    self.bytes -= value.len();
                }
                keep
            });
            self.interval = self.interval.saturating_mul(2);
            self.next = count.saturating_add(self.interval);
        }
    }

    /// Discards `count` acknowledged edits and rebases surviving checkpoints.
    pub(super) fn acknowledge(&mut self, count: usize) {
        while self
            .values
            .front()
            .is_some_and(|(index, _)| *index <= count)
        {
            if let Some((_, value)) = self.values.pop_front() {
                self.bytes -= value.len();
            }
        }
        for (index, _) in &mut self.values {
            *index -= count;
        }
        self.next -= count;
    }

    /// Restores the initial checkpoint schedule after an external or final echo.
    pub(super) fn clear(&mut self) {
        *self = Self::default();
    }

    /// Returns the newest checkpoint at or before pending edit `count`.
    fn before(&self, count: usize) -> Option<(usize, &str)> {
        self.values
            .iter()
            .rev()
            .find(|(index, _)| *index <= count)
            .map(|(index, value)| (*index, value.as_str()))
    }
}

impl PendingInsert {
    /// Starts a run for `edit` when it inserts nonempty text at one caret.
    pub(super) fn new(edit: &TextEdit) -> Option<Self> {
        (edit.range.is_empty() && !edit.replacement.is_empty()).then(|| Self {
            start: edit.range.start,
            text: edit.replacement.clone(),
        })
    }

    /// Extends the run with `edit` and returns whether it remains contiguous.
    pub(super) fn push(&mut self, edit: &TextEdit) -> bool {
        if !edit.range.is_empty()
            || edit.replacement.is_empty()
            || edit.range.start != self.start + self.text.len()
        {
            return false;
        }
        self.text.push_str(&edit.replacement);
        true
    }

    /// Finds a matching partial echo without replaying edits into a large string.
    ///
    /// `base` is the last authored value, `echo` is the new authored value, and
    /// `pending` contains lengths after each native edit. The result identifies
    /// the last acknowledged edit and the number of inserted UTF-8 bytes.
    pub(super) fn acknowledged(
        &self,
        base: &str,
        echo: &str,
        pending: &VecDeque<PendingEdit>,
    ) -> Option<(usize, usize)> {
        let inserted_bytes = echo.len().checked_sub(base.len())?;
        if inserted_bytes == 0
            || inserted_bytes > self.text.len()
            || echo.get(..self.start)? != base.get(..self.start)?
            || echo.get(self.start..self.start + inserted_bytes)?
                != self.text.get(..inserted_bytes)?
            || echo.get(self.start + inserted_bytes..)? != base.get(self.start..)?
        {
            return None;
        }
        let index = pending
            .iter()
            .position(|entry| entry.length == echo.len())?;
        Some((index, inserted_bytes))
    }

    /// Advances the run by `inserted_bytes` acknowledged UTF-8 bytes.
    pub(super) fn consume(&mut self, inserted_bytes: usize) {
        self.start += inserted_bytes;
        self.text.drain(..inserted_bytes);
    }
}

/// Finds a controlled echo using borrowed pieces instead of copying the full
/// document after every pending edit.
///
/// `base` is the last authored value, `echo` is the new authored value, and
/// `pending` holds ordered edits against successive native values.
/// `checkpoints` supply earlier full values to bound piece replay, and
/// `authored_checksum` rebases the incremental byte checksum after prior echoes.
/// Returns the index of the last acknowledged edit, or `None` for an external value.
pub(super) fn acknowledged_edits(
    base: &str,
    echo: &str,
    pending: &VecDeque<PendingEdit>,
    checkpoints: &PendingCheckpoints,
    authored_checksum: u64,
) -> Option<usize> {
    let checksum = authored_checksum
        .wrapping_add(checksum(echo))
        .wrapping_sub(checksum(base));
    for (index, _) in pending
        .iter()
        .enumerate()
        .filter(|(_, entry)| entry.length == echo.len() && entry.checksum == checksum)
    {
        let (skip, start_value) = checkpoints.before(index + 1).unwrap_or((0, base));
        let mut pieces = vec![start_value];
        for entry in pending.iter().skip(skip).take(index + 1 - skip) {
            let edit = &entry.edit;
            let start = split_piece(&mut pieces, edit.range.start)?;
            let end = split_piece(&mut pieces, edit.range.end)?;
            pieces.splice(
                start..end,
                (!edit.replacement.is_empty()).then_some(edit.replacement.as_str()),
            );
        }
        if pieces_equal(&pieces, echo) {
            return Some(index);
        }
    }
    None
}

/// Splits `pieces` at byte `offset` and returns the resulting piece boundary.
///
/// Returns `None` when the byte offset is out of bounds or splits UTF-8.
fn split_piece(pieces: &mut Vec<&str>, offset: usize) -> Option<usize> {
    let mut position = 0usize;
    for index in 0..pieces.len() {
        if offset == position {
            return Some(index);
        }
        let end = position.checked_add(pieces[index].len())?;
        if offset < end {
            let left = pieces[index].get(..offset - position)?;
            let right = pieces[index].get(offset - position..)?;
            pieces[index] = left;
            pieces.insert(index + 1, right);
            return Some(index + 1);
        }
        position = end;
    }
    (offset == position).then_some(pieces.len())
}

/// Compares `pieces` with `echo` without materializing a new full string.
fn pieces_equal(pieces: &[&str], echo: &str) -> bool {
    let mut position = 0usize;
    for piece in pieces {
        let Some(end) = position.checked_add(piece.len()) else {
            return false;
        };
        if echo.get(position..end) != Some(*piece) {
            return false;
        }
        position = end;
    }
    position == echo.len()
}

/// Sums bytes in eight-byte blocks for candidate filtering; pieces still
/// verify exact equality before an acknowledgement is accepted.
fn checksum(value: &str) -> u64 {
    let (chunks, remainder) = value.as_bytes().as_chunks::<8>();
    let mut sum = 0u64;
    for chunk in chunks {
        let word = u64::from_le_bytes(*chunk);
        let pair = (word & 0x00ff_00ff_00ff_00ff) + ((word >> 8) & 0x00ff_00ff_00ff_00ff);
        let quad = (pair & 0x0000_ffff_0000_ffff) + ((pair >> 16) & 0x0000_ffff_0000_ffff);
        sum = sum.wrapping_add((quad & 0xffff_ffff) + (quad >> 32));
    }
    for byte in remainder {
        sum = sum.wrapping_add(u64::from(*byte));
    }
    sum
}
