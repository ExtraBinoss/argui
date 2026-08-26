use argui_core::{CaretAffinity, TextPosition};

#[test]
fn bidi_text_positions_keep_boundary_affinity() {
    assert_ne!(
        TextPosition::new(4, CaretAffinity::Before),
        TextPosition::new(4, CaretAffinity::After)
    );
}
