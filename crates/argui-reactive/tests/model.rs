use std::{cell::Cell, rc::Rc};

use argui_reactive::{Model, ModelChange, Property};

#[derive(Clone, Debug, Eq, PartialEq)]
struct Row {
    name: &'static str,
    score: i32,
}

/// Row IDs remain attached to their rows through insertion, removal, movement, and update.
#[test]
fn model_row_identity_survives_edits() {
    let mut model = Model::new([
        Row {
            name: "alpha",
            score: 2,
        },
        Row {
            name: "beta",
            score: 9,
        },
        Row {
            name: "gamma",
            score: 5,
        },
    ]);
    let alpha = model.row_id(0).unwrap();
    let beta = model.row_id(1).unwrap();
    let gamma = model.row_id(2).unwrap();

    let inserted = model.insert(
        1,
        Row {
            name: "delta",
            score: 7,
        },
    );
    assert_ne!(inserted, alpha);
    assert_ne!(inserted, beta);
    assert_ne!(inserted, gamma);
    assert_eq!(
        model.last_change(),
        Some(ModelChange::Insert {
            id: inserted,
            index: 1
        })
    );
    assert_eq!(model.row_id(0), Some(alpha));
    assert_eq!(model.row_id(2), Some(beta));
    assert_eq!(model.row_id(3), Some(gamma));

    assert_eq!(
        model.remove(2),
        Some((
            beta,
            Row {
                name: "beta",
                score: 9
            }
        ))
    );
    assert_eq!(
        model.last_change(),
        Some(ModelChange::Remove { id: beta, index: 2 })
    );
    assert_eq!(model.row_id(0), Some(alpha));
    assert_eq!(model.row_id(1), Some(inserted));
    assert_eq!(model.row_id(2), Some(gamma));

    assert!(model.move_row(2, 0));
    assert_eq!(
        model.last_change(),
        Some(ModelChange::Move {
            id: gamma,
            from: 2,
            to: 0
        })
    );
    assert_eq!(model.row_id(0), Some(gamma));
    assert_eq!(model.row_id(1), Some(alpha));
    assert_eq!(model.row_id(2), Some(inserted));

    assert!(model.update(1, |row| {
        row.score = 3;
        true
    }));
    assert_eq!(
        model.last_change(),
        Some(ModelChange::Update {
            id: alpha,
            index: 1
        })
    );
    assert_eq!(model.row_id(1), Some(alpha));
    assert_eq!(model[1].score, 3);
    assert_eq!(model.revision(), 4);
}

/// Filtered and sorted projections contain source indices and leave row identity unchanged.
#[test]
fn model_projection_filters_and_sorts_without_reordering_source_rows() {
    let model = Model::new([
        Row {
            name: "alpha",
            score: 2,
        },
        Row {
            name: "beta",
            score: 9,
        },
        Row {
            name: "gamma",
            score: 5,
        },
        Row {
            name: "delta",
            score: 7,
        },
    ]);
    let ids = (0..model.len())
        .map(|index| model.row_id(index).unwrap())
        .collect::<Vec<_>>();
    let mut compare = |left: &Row, right: &Row| left.score.cmp(&right.score);

    let projection = model.project(|row| row.score >= 5, Some(&mut compare));

    assert_eq!(projection, [2, 3, 1]);
    assert_eq!(
        projection
            .iter()
            .map(|index| model[*index].name)
            .collect::<Vec<_>>(),
        ["gamma", "delta", "beta"]
    );
    assert_eq!(
        (0..model.len())
            .map(|index| model.row_id(index).unwrap())
            .collect::<Vec<_>>(),
        ids
    );
    assert_eq!(
        model.iter().map(|row| row.name).collect::<Vec<_>>(),
        ["alpha", "beta", "gamma", "delta"]
    );
    assert_eq!(model.revision(), 0);
    assert_eq!(model.last_change(), None);
}

struct CloneTrackedRow {
    value: i32,
    clones: Rc<Cell<usize>>,
}

impl Clone for CloneTrackedRow {
    fn clone(&self) -> Self {
        self.clones.set(self.clones.get() + 1);
        Self {
            value: self.value,
            clones: Rc::clone(&self.clones),
        }
    }
}

impl PartialEq for CloneTrackedRow {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

/// A row edit through `Property::mutate` preserves the backing vector and clones no rows.
#[test]
fn property_model_mutation_updates_a_row_without_copying_the_collection() {
    let clones = Rc::new(Cell::new(0));
    let rows = (0..4096)
        .map(|value| CloneTrackedRow {
            value,
            clones: Rc::clone(&clones),
        })
        .collect::<Vec<_>>();
    let property = Property::new(Model::new(rows));
    let before = property.with(|model| model.as_ptr());

    let changed = property.mutate(|model| {
        model.update(2048, |row| {
            row.value += 1;
            true
        })
    });

    assert!(changed);
    assert_eq!(clones.get(), 0, "a row update cloned model contents");
    assert!(property.with(|model| std::ptr::eq(before, model.as_ptr())));
    assert_eq!(property.with(|model| model[2048].value), 2049);
    assert_eq!(property.revision(), 1);
}

/// Imported identities reject malformed rows and preserve revision and ordering when valid.
#[test]
fn imported_model_rows_validate_identity_and_exhaustion() {
    use std::hint::black_box;

    assert!(Model::from_identified_rows([(black_box(0), "zero")], 3).is_none());
    assert!(Model::from_identified_rows([(black_box(7), "first"), (7, "again")], 3).is_none());
    assert!(Model::from_identified_rows([(black_box(u64::MAX), "last")], 3).is_none());

    let restored = Model::from_identified_rows([(black_box(12), "alpha"), (4, "beta")], 9)
        .expect("distinct nonzero identities should restore");
    assert_eq!(restored.row_ids(), &[12, 4]);
    assert_eq!(restored.revision(), 9);
    assert_eq!(&*restored, &["alpha", "beta"]);
}

/// Rejected moves preserve snapshot identity; no-op edits preserve values and revision.
#[test]
fn rejected_model_edits_leave_snapshot_and_revision_unchanged() {
    use std::hint::black_box;

    let mut model = Model::new(["alpha", "beta"]);
    let snapshot = model.clone();
    for (from, to) in [(2, 0), (0, 2), (1, 1)] {
        assert!(!model.move_row(black_box(from), black_box(to)));
    }
    assert!(!model.update(black_box(2), |_| panic!("out-of-range edit ran")));
    assert_eq!(model, snapshot);
    assert!(!model.update(black_box(0), |_| false));
    assert_ne!(model, snapshot);
    assert_eq!(&*model, &*snapshot);
    assert_eq!(model.revision(), 0);
    assert_eq!(model.row_ids(), snapshot.row_ids());
    assert_eq!(model.project(|_| true, None), [0, 1]);
}
