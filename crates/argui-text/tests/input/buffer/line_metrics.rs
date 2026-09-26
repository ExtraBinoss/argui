mod line_metrics {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/input/buffer/line_metrics.rs"
    ));
}

use line_metrics::LineMetrics;

fn next(seed: &mut u64) -> usize {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed as usize
}

#[test]
fn persistent_line_metrics_match_full_reindex_after_mixed_edits() {
    let mut text = String::from("\nA\tB\nمرحبا\ne\u{301}\n");
    let mut index = LineMetrics::new(&text);
    let mut seed = 0x789a_bcd1_2345_6789_u64;
    let insertions = ["", "x", "\n", "\t", "🌟", "م\n\t", "\n\n"];
    for step in 0..800 {
        let boundaries = text
            .char_indices()
            .map(|(offset, _)| offset)
            .chain(std::iter::once(text.len()))
            .collect::<Vec<_>>();
        let start_at = next(&mut seed) % boundaries.len();
        let end_at = (start_at + next(&mut seed) % 5).min(boundaries.len() - 1);
        let range = boundaries[start_at]..boundaries[end_at];
        let inserted = insertions[next(&mut seed) % insertions.len()];
        let mut updated = text.clone();
        updated.replace_range(range.clone(), inserted);
        index = index.edited(&text, &updated, range);
        let expected = LineMetrics::new(&updated);
        assert_eq!(index.len(), expected.len(), "line count at edit {step}");
        assert_eq!(
            index.max_width(),
            expected.max_width(),
            "max width at edit {step}"
        );
        for line in 0..index.len() {
            assert_eq!(
                index.get(line),
                expected.get(line),
                "start of line {line} at edit {step}"
            );
            assert_eq!(
                index.width(line),
                expected.width(line),
                "width of line {line} at edit {step}"
            );
        }
        for offset in [0, updated.len() / 2, updated.len()] {
            let actual = index.partition_point(|start| *start <= offset);
            let fresh = expected.partition_point(|start| *start <= offset);
            assert_eq!(actual, fresh, "line at byte {offset} after edit {step}");
        }
        text = updated;
    }
}
