use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

const BLOCK_LINES: usize = 256;
static NEXT_NODE: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy)]
struct Line {
    bytes: usize,
    columns: usize,
}

/// Persistent line index with byte prefix sums and maximum line widths.
#[derive(Clone)]
pub(in crate::input) struct LineMetrics(Option<Arc<Node>>);

struct Node {
    left: Option<Arc<Node>>,
    right: Option<Arc<Node>>,
    block: Vec<Line>,
    priority: u64,
    lines: usize,
    bytes: usize,
    max_columns: usize,
}

/// Returns a reproducible, dispersed priority for one new treap block.
fn random_priority() -> u64 {
    let mut value = NEXT_NODE.fetch_add(1, Ordering::Relaxed);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

/// Returns the number of lines in `root`, or zero for an empty subtree.
fn count(root: &Option<Arc<Node>>) -> usize {
    root.as_ref().map_or(0, |node| node.lines)
}

/// Returns the total UTF-8 byte length of all lines in `root`.
fn total_bytes(root: &Option<Arc<Node>>) -> usize {
    root.as_ref().map_or(0, |node| node.bytes)
}

/// Returns the widest line in `root`, or zero for an empty subtree.
fn max_columns(root: &Option<Arc<Node>>) -> usize {
    root.as_ref().map_or(0, |node| node.max_columns)
}

impl Node {
    /// Builds an immutable node from `left`, `block`, and `right` using `priority`.
    ///
    /// Returns a node with refreshed line, byte, and maximum-width summaries.
    fn new(
        left: Option<Arc<Self>>,
        block: Vec<Line>,
        right: Option<Arc<Self>>,
        priority: u64,
    ) -> Arc<Self> {
        let lines = count(&left) + block.len() + count(&right);
        let bytes = total_bytes(&left)
            + block.iter().map(|line| line.bytes).sum::<usize>()
            + total_bytes(&right);
        let max_columns = max_columns(&left)
            .max(block.iter().map(|line| line.columns).max().unwrap_or(0))
            .max(max_columns(&right));
        Arc::new(Self {
            left,
            right,
            block,
            priority,
            lines,
            bytes,
            max_columns,
        })
    }

    /// Copies the path to `index` and replaces its byte length and columns with `line`.
    ///
    /// Returns the updated node. Panics if `index` is outside this subtree.
    fn updated(&self, index: usize, line: Line) -> Arc<Self> {
        let left_len = count(&self.left);
        let (left, block, right) = if index < left_len {
            (
                Some(
                    self.left
                        .as_ref()
                        .expect("line lies in left subtree")
                        .updated(index, line),
                ),
                self.block.clone(),
                self.right.clone(),
            )
        } else if index < left_len + self.block.len() {
            let mut block = self.block.clone();
            block[index - left_len] = line;
            (self.left.clone(), block, self.right.clone())
        } else {
            (
                self.left.clone(),
                self.block.clone(),
                Some(
                    self.right
                        .as_ref()
                        .expect("line lies in right subtree")
                        .updated(index - left_len - self.block.len(), line),
                ),
            )
        };
        Self::new(left, block, right, self.priority)
    }
}

/// Merges ordered `left` and `right` treaps, copying only boundary paths.
///
/// Returns the merged tree, or an empty tree when both inputs are empty.
fn merge(left: Option<Arc<Node>>, right: Option<Arc<Node>>) -> Option<Arc<Node>> {
    match (left, right) {
        (None, right) => right,
        (left, None) => left,
        (Some(left), Some(right)) if left.priority >= right.priority => Some(Node::new(
            left.left.clone(),
            left.block.clone(),
            merge(left.right.clone(), Some(right)),
            left.priority,
        )),
        (Some(left), Some(right)) => Some(Node::new(
            merge(Some(left), right.left.clone()),
            right.block.clone(),
            right.right.clone(),
            right.priority,
        )),
    }
}

/// Splits `root` before line `index`.
///
/// Returns the prefix and suffix; `index` may equal the line count.
fn split(root: Option<Arc<Node>>, index: usize) -> (Option<Arc<Node>>, Option<Arc<Node>>) {
    let Some(root) = root else {
        return (None, None);
    };
    let left_len = count(&root.left);
    let block_end = left_len + root.block.len();
    if index < left_len {
        let (before, after) = split(root.left.clone(), index);
        return (
            before,
            Some(Node::new(
                after,
                root.block.clone(),
                root.right.clone(),
                root.priority,
            )),
        );
    }
    if index > block_end {
        let (before, after) = split(root.right.clone(), index - block_end);
        return (
            Some(Node::new(
                root.left.clone(),
                root.block.clone(),
                before,
                root.priority,
            )),
            after,
        );
    }
    let offset = index - left_len;
    if offset == 0 {
        return (
            root.left.clone(),
            Some(Node::new(
                None,
                root.block.clone(),
                root.right.clone(),
                root.priority,
            )),
        );
    }
    if offset == root.block.len() {
        return (
            Some(Node::new(
                root.left.clone(),
                root.block.clone(),
                None,
                root.priority,
            )),
            root.right.clone(),
        );
    }
    let before = Node::new(None, root.block[..offset].to_vec(), None, random_priority());
    let after = Node::new(None, root.block[offset..].to_vec(), None, random_priority());
    (
        merge(root.left.clone(), Some(before)),
        merge(Some(after), root.right.clone()),
    )
}

/// Converts `text` into line summaries, adding a final empty line when
/// `includes_document_end` is true and the text ends with a newline.
///
/// Returns byte lengths and approximate columns for every hard line.
fn lines(text: &str, includes_document_end: bool) -> Vec<Line> {
    let mut result = text
        .split_inclusive('\n')
        .map(|part| Line {
            bytes: part.len(),
            columns: part
                .chars()
                .filter(|character| *character != '\n')
                .map(|character| if character == '\t' { 4 } else { 1 })
                .sum(),
        })
        .collect::<Vec<_>>();
    if result.is_empty() || (includes_document_end && text.ends_with('\n')) {
        result.push(Line {
            bytes: 0,
            columns: 0,
        });
    }
    result
}

/// Returns approximate columns in `text`, expanding each tab to four columns.
fn columns(text: &str) -> usize {
    text.chars()
        .map(|character| if character == '\t' { 4 } else { 1 })
        .sum()
}

/// Builds a treap from `lines` in bounded blocks.
///
/// Returns an empty tree when there are no lines.
fn build(lines: Vec<Line>) -> Option<Arc<Node>> {
    lines.chunks(BLOCK_LINES).fold(None, |root, block| {
        merge(
            root,
            Some(Node::new(None, block.to_vec(), None, random_priority())),
        )
    })
}

impl LineMetrics {
    /// Indexes every hard line in `text`, including a final empty line.
    pub(in crate::input) fn new(text: &str) -> Self {
        Self(build(lines(text, true)))
    }

    /// Returns the number of hard lines, including a final empty line.
    pub(in crate::input) fn len(&self) -> usize {
        count(&self.0)
    }

    /// Returns the byte start of hard line `index`, or `None` past the final line.
    pub(in crate::input) fn get(&self, index: usize) -> Option<usize> {
        if index >= self.len() {
            return None;
        }
        let (mut node, mut remaining, mut bytes) = (self.0.as_ref()?, index, 0);
        loop {
            let left_len = count(&node.left);
            if remaining < left_len {
                node = node.left.as_ref()?;
                continue;
            }
            bytes += total_bytes(&node.left);
            remaining -= left_len;
            if remaining < node.block.len() {
                bytes += node.block[..remaining]
                    .iter()
                    .map(|line| line.bytes)
                    .sum::<usize>();
                return Some(bytes);
            }
            bytes += node.block.iter().map(|line| line.bytes).sum::<usize>();
            remaining -= node.block.len();
            node = node.right.as_ref()?;
        }
    }

    /// Returns the approximate width of hard line `index`.
    ///
    /// Panics if `index` is outside the document.
    pub(in crate::input) fn width(&self, mut index: usize) -> usize {
        let mut node = self.0.as_ref().expect("document always contains a line");
        loop {
            let left_len = count(&node.left);
            if index < left_len {
                node = node.left.as_ref().expect("line lies in left subtree");
            } else if index < left_len + node.block.len() {
                return node.block[index - left_len].columns;
            } else {
                index -= left_len + node.block.len();
                node = node.right.as_ref().expect("line lies in right subtree");
            }
        }
    }

    /// Returns the largest approximate width across all hard lines.
    pub(in crate::input) fn max_width(&self) -> usize {
        max_columns(&self.0)
    }

    /// Finds the first line start for which `predicate` is false.
    ///
    /// `predicate` must be monotonic over increasing byte starts. Returns the
    /// number of lines when it remains true for every line.
    pub(in crate::input) fn partition_point(
        &self,
        mut predicate: impl FnMut(&usize) -> bool,
    ) -> usize {
        let (mut low, mut high) = (0, self.len());
        while low < high {
            let mid = low + (high - low) / 2;
            if predicate(&self.get(mid).expect("mid lies in line index")) {
                low = mid + 1;
            } else {
                high = mid;
            }
        }
        low
    }

    /// Applies one contiguous edit from `old` to `new` over the old byte `range`.
    ///
    /// Returns a new index after rescanning only affected lines. The caller must
    /// provide a UTF-8 boundary range whose replacement produces `new`.
    pub(in crate::input) fn edited(
        &self,
        old: &str,
        new: &str,
        range: std::ops::Range<usize>,
    ) -> Self {
        let first = self.partition_point(|offset| *offset <= range.start) - 1;
        let last = self.partition_point(|offset| *offset <= range.end) - 1;
        let byte_start = self.get(first).expect("first affected line exists");
        let old_end = self.get(last + 1).unwrap_or(old.len());
        let delta = new.len() as isize - old.len() as isize;
        let inserted_len = new.len() + range.len() - old.len();
        let inserted = &new[range.start..range.start + inserted_len];
        let removed = &old[range.clone()];
        if first == last && !inserted.contains('\n') && !removed.contains('\n') {
            let line = Line {
                bytes: (old_end - byte_start).saturating_add_signed(delta),
                columns: self.width(first) + columns(inserted) - columns(removed),
            };
            let root = self.0.as_ref().expect("document always contains a line");
            return Self(Some(root.updated(first, line)));
        }
        let new_end = old_end.saturating_add_signed(delta);
        let replacement = lines(&new[byte_start..new_end], last + 1 == self.len());
        let (before, rest) = split(self.0.clone(), first);
        let (_, after) = split(rest, last - first + 1);
        Self(merge(merge(before, build(replacement)), after))
    }
}
