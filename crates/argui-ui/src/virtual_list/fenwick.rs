#[derive(Clone, Debug, PartialEq)]
pub(super) struct Fenwick {
    tree: Vec<f32>,
}

impl Fenwick {
    pub(super) fn uniform(len: usize, value: f32) -> Self {
        Self::from_values(&vec![value; len])
    }

    pub(super) fn from_values(values: &[f32]) -> Self {
        let mut result = Self {
            tree: vec![0.0; values.len() + 1],
        };
        for (index, value) in values.iter().copied().enumerate() {
            result.add(index, value);
        }
        result
    }

    pub(super) fn add(&mut self, index: usize, delta: f32) {
        let mut cursor = index + 1;
        while cursor < self.tree.len() {
            self.tree[cursor] += delta;
            cursor += cursor & cursor.wrapping_neg();
        }
    }

    pub(super) fn push(&mut self, value: f32) {
        let position = self.tree.len();
        let width = position & position.wrapping_neg();
        let start = position - width;
        let previous = self.sum(position - 1) - self.sum(start);
        self.tree.push(previous + value);
    }

    pub(super) fn sum(&self, end: usize) -> f32 {
        let mut cursor = end.min(self.tree.len().saturating_sub(1));
        let mut sum = 0.0;
        while cursor != 0 {
            sum += self.tree[cursor];
            cursor &= cursor - 1;
        }
        sum
    }

    pub(super) fn total(&self) -> f32 {
        self.sum(self.tree.len().saturating_sub(1))
    }

    pub(super) fn lower_bound(&self, target: f32, count: usize) -> usize {
        if count == 0 || target <= 0.0 {
            return 0;
        }
        let mut index = 0;
        let mut accumulated = 0.0;
        let mut bit = count.next_power_of_two();
        while bit != 0 {
            let next = index + bit;
            if next <= count && accumulated + self.tree[next] <= target {
                index = next;
                accumulated += self.tree[next];
            }
            bit >>= 1;
        }
        index.min(count.saturating_sub(1))
    }
}
