//! Compares incremental `TextEdit` delivery with complete-value delivery.

use std::{hint::black_box, time::Instant};

use argui::ui::TextEdit;

const DEFAULT_DOCUMENT_BYTES: usize = 1_048_576;
const DEFAULT_EDITS: usize = 2_000;
const RUNS: usize = 5;

/// Reads one positive integer argument or returns `default` when it is absent.
fn positive_argument(index: usize, default: usize, name: &str) -> usize {
    let Some(value) = std::env::args().nth(index) else {
        return default;
    };
    let value = value
        .parse::<usize>()
        .unwrap_or_else(|error| panic!("invalid {name} '{value}': {error}"));
    assert!(value > 0, "{name} must be positive");
    value
}

/// Returns one deterministic ASCII replacement at `iteration`.
fn replacement(iteration: usize) -> &'static str {
    if iteration.is_multiple_of(2) {
        "b"
    } else {
        "a"
    }
}

/// Profiles the delta path used by `on_edit` and returns elapsed nanoseconds.
fn incremental_sample(base: &str, edits: usize) -> u128 {
    let mut engine_value = base.to_owned();
    let mut controlled_value = base.to_owned();
    let started = Instant::now();
    for iteration in 0..edits {
        let offset = iteration.wrapping_mul(7_919) % base.len();
        let edit = TextEdit::new(offset..offset + 1, replacement(iteration));
        edit.apply_to(&mut engine_value)
            .expect("the benchmark range is an ASCII boundary");
        edit.apply_to(&mut controlled_value)
            .expect("the controlled value follows the engine revision");
        black_box(&controlled_value);
    }
    let elapsed = started.elapsed().as_nanos();
    assert_eq!(engine_value, controlled_value);
    elapsed
}

/// Profiles complete-value callback delivery and returns elapsed nanoseconds.
fn complete_value_sample(base: &str, edits: usize) -> u128 {
    let mut engine_value = base.to_owned();
    let mut controlled_value = base.to_owned();
    black_box(&controlled_value);
    let started = Instant::now();
    for iteration in 0..edits {
        let offset = iteration.wrapping_mul(7_919) % base.len();
        engine_value.replace_range(offset..offset + 1, replacement(iteration));
        controlled_value = engine_value.clone();
        black_box(&controlled_value);
    }
    let elapsed = started.elapsed().as_nanos();
    assert_eq!(engine_value, controlled_value);
    elapsed
}

/// Returns the median value from the non-empty sample set `values`.
fn median(values: &mut [u128]) -> u128 {
    assert!(!values.is_empty(), "a profile needs at least one sample");
    values.sort_unstable();
    values[values.len() / 2]
}

/// Runs both delivery strategies in alternating order and prints one JSON record.
fn main() {
    let document_bytes = positive_argument(1, DEFAULT_DOCUMENT_BYTES, "document bytes");
    let edits = positive_argument(2, DEFAULT_EDITS, "edit count");
    let base = "a".repeat(document_bytes);
    let mut incremental = Vec::with_capacity(RUNS);
    let mut complete = Vec::with_capacity(RUNS);

    for run in 0..RUNS {
        if run.is_multiple_of(2) {
            incremental.push(incremental_sample(&base, edits));
            complete.push(complete_value_sample(&base, edits));
        } else {
            complete.push(complete_value_sample(&base, edits));
            incremental.push(incremental_sample(&base, edits));
        }
    }

    let incremental_ns = median(&mut incremental);
    let complete_ns = median(&mut complete);
    let speedup = complete_ns as f64 / incremental_ns.max(1) as f64;
    println!(
        concat!(
            "{{\"document_bytes\":{},\"edits\":{},\"runs\":{},",
            "\"incremental_median_ms\":{:.3},\"complete_value_median_ms\":{:.3},",
            "\"delivery_speedup\":{:.2},\"complete_value_payload_mib\":{:.2}}}"
        ),
        document_bytes,
        edits,
        RUNS,
        incremental_ns as f64 / 1_000_000.0,
        complete_ns as f64 / 1_000_000.0,
        speedup,
        document_bytes.saturating_mul(edits) as f64 / 1_048_576.0,
    );
}
