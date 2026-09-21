use argui_core::Point;
use argui_paint::{
    Border, BorderWidths, Color, CornerRadii, Fill, GradientError, PaintStyle, QuadStyle,
};

/// All gradient families preserve arbitrary ordered stops and validate malformed input.
#[test]
fn gradient_families_accept_variable_stop_counts() {
    let colors = (0..12)
        .map(|index| Color::srgb(index as f32 / 12.0, 0.2, 0.8))
        .collect::<Vec<_>>();
    let offsets = (0..12).map(|index| index as f32 / 11.0).collect::<Vec<_>>();
    let linear = Fill::linear_gradient(&colors, &offsets, 40.0, "oklab").unwrap();
    let radial = Fill::radial_gradient(
        &colors,
        &offsets,
        Point::new(0.5, 0.5),
        Point::new(0.7, 0.7),
        "srgb",
    )
    .unwrap();
    let conic =
        Fill::conic_gradient(&colors, &offsets, Point::new(0.5, 0.5), 90.0, "linear-srgb").unwrap();
    assert!(matches!(linear, Fill::Linear(value) if value.stops.as_slice().len() == 12));
    assert!(matches!(radial, Fill::Radial(value) if value.stops.as_slice().len() == 12));
    assert!(
        matches!(conic, Fill::Conic(value) if value.stops.as_slice().len() == 12 && value.start_angle == 90.0)
    );
    assert_eq!(
        Fill::linear_gradient(&colors, &offsets[..11], 0.0, "oklab"),
        Err(GradientError::MismatchedStops)
    );
    assert_eq!(
        Fill::linear_gradient(&colors, &offsets, 0.0, "unknown"),
        Err(GradientError::InvalidInterpolation)
    );
}

#[test]
fn paint_styles_are_explicit_and_theme_free() {
    let border = Border::all(2.0, Color::WHITE);
    let radii = CornerRadii::all(12.0);
    let style = PaintStyle {
        quad: QuadStyle {
            background: Some(Fill::Solid(Color::srgb(0.1, 0.2, 0.3))),
            border: Some(border),
            radii,
            opacity: 0.8,
        },
    };

    assert!(style.is_visible());
    assert_eq!(border.widths, BorderWidths::all(2.0));
    assert_eq!(border.widths.as_array(), [2.0; 4]);
    assert_eq!(radii.as_array(), [12.0; 4]);
    assert!(!PaintStyle::default().is_visible());

    let built = QuadStyle::solid(Color::WHITE)
        .border(border)
        .radius(radii)
        .opacity(0.5);
    assert_eq!(built.border, Some(border));
    assert_eq!(PaintStyle::new(built.clone()).quad, built);
}

#[test]
fn gradient_fill_validation_applies_to_each_family() {
    let colors = [Color::BLACK, Color::WHITE];
    let center = Point::new(0.25, 0.75);
    let radius = Point::new(0.5, 0.25);
    for offsets in [[0.8, 0.2], [-0.1, 1.0], [0.0, f32::NAN]] {
        let expected = if offsets[0] == 0.8 {
            GradientError::UnsortedStops
        } else {
            GradientError::InvalidOffset
        };
        assert_eq!(
            Fill::linear_gradient(&colors, &offsets, 90.0, "srgb"),
            Err(expected)
        );
        assert_eq!(
            Fill::radial_gradient(&colors, &offsets, center, radius, "srgb"),
            Err(expected)
        );
        assert_eq!(
            Fill::conic_gradient(&colors, &offsets, center, 30.0, "srgb"),
            Err(expected)
        );
    }
    for space in ["oklab", "linear-srgb", "srgb"] {
        assert!(Fill::radial_gradient(&colors, &[0.0, 1.0], center, radius, space).is_ok());
        assert!(Fill::conic_gradient(&colors, &[0.0, 1.0], center, 30.0, space).is_ok());
    }
    assert_eq!(
        Fill::radial_gradient(&colors, &[0.0, 1.0], center, radius, "invalid"),
        Err(GradientError::InvalidInterpolation)
    );
    assert_eq!(
        Fill::conic_gradient(&colors, &[0.0, 1.0], center, 30.0, "invalid"),
        Err(GradientError::InvalidInterpolation)
    );
}

#[test]
fn style_value_helpers_keep_edge_order_and_visibility() {
    let corners = CornerRadii {
        top_left: 1.0,
        top_right: 2.0,
        bottom_right: 3.0,
        bottom_left: 4.0,
    };
    assert_eq!(corners.scaled(2.0).as_array(), [2.0, 4.0, 6.0, 8.0]);
    let widths = BorderWidths {
        left: 1.0,
        right: 2.0,
        top: 3.0,
        bottom: 4.0,
    };
    assert_eq!(widths.as_array(), [1.0, 2.0, 3.0, 4.0]);
    assert!(!QuadStyle::default().is_visible());
    assert!(
        QuadStyle::default()
            .border(Border {
                widths,
                color: Color::WHITE
            })
            .is_visible()
    );
    assert!(QuadStyle::solid(Color::TRANSPARENT).is_visible());
}

#[test]
fn runtime_style_builders_preserve_values_across_all_edges() {
    use std::hint::black_box;

    let radius = black_box(7.0);
    let width = black_box(2.5);
    let color = black_box(Color::WHITE);
    let radii = CornerRadii::all(radius);
    assert_eq!(black_box(radii).as_array(), [radius; 4]);
    assert_eq!(
        black_box(radii).scaled(black_box(2.0)).as_array(),
        [14.0; 4]
    );
    let widths = BorderWidths::all(width);
    assert_eq!(black_box(widths).as_array(), [width; 4]);
    let border = Border::all(width, color);
    assert_eq!(border.widths, widths);
    let style = QuadStyle::solid(color)
        .border(black_box(border))
        .radius(black_box(radii))
        .opacity(black_box(0.4));
    assert!(black_box(&style).is_visible());
    assert_eq!(style.opacity, 0.4);
    assert_eq!(style.radii, radii);
    assert!(PaintStyle::new(black_box(style)).is_visible());
}
