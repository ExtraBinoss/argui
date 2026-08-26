use argui_paint::{Border, BorderWidths, ClipBehavior, Color, CornerRadii, Fill, PaintStyle};

#[test]
fn paint_styles_are_explicit_and_theme_free() {
    let border = Border::all(2.0, Color::WHITE);
    let radii = CornerRadii::all(12.0);
    let style = PaintStyle {
        background: Some(Fill::Solid(Color::rgb(0.1, 0.2, 0.3))),
        border: Some(border),
        radii,
        opacity: 0.8,
        clip: ClipBehavior::Bounds,
    };

    assert!(style.is_visible());
    assert_eq!(border.widths, BorderWidths::all(2.0));
    assert_eq!(border.widths.as_array(), [2.0; 4]);
    assert_eq!(radii.as_array(), [12.0; 4]);
    assert!(!PaintStyle::default().is_visible());
}
