use argui_core::{Affine2D, Color, Point, Rect, Size};
use argui_paint::{
    ClipChain, ClipRegion, ColorInterpolation, ConicGradient, CornerRadii, GradientError,
    GradientStop, GradientStops, ImageAsset, ImageAssetError, ImageId, LinearGradient,
    RadialGradient,
};

fn stop(offset: f32) -> GradientStop {
    GradientStop::new(offset, Color::srgba(offset, 0.2, 0.4, 1.0))
}

#[test]
fn bilinear_corners_share_storage_without_increasing_the_fill_size() {
    use argui_paint::{BilinearGradient, ColorInterpolation, Fill};
    let corners = [
        Color::WHITE,
        Color::srgb(1.0, 0.0, 0.0),
        Color::BLACK,
        Color::TRANSPARENT,
    ];
    let gradient = BilinearGradient::new(corners, ColorInterpolation::Srgb);
    assert_eq!(gradient.corners(), &corners);
    assert!(std::ptr::eq(gradient.corners(), gradient.clone().corners()));
    assert_eq!(gradient.interpolation, ColorInterpolation::Srgb);
    // Existing gradient storage plus the discriminant, rounded to pointer alignment.
    let alignment = align_of::<Fill>();
    assert_eq!(
        size_of::<Fill>(),
        (size_of::<LinearGradient>() + 1).div_ceil(alignment) * alignment
    );
}

#[test]
fn gradients_accept_runtime_generated_and_many_stops() {
    let stops = GradientStops::from_vec((0..32).map(|i| stop(i as f32 / 31.0)).collect())
        .expect("generated stops are valid");

    assert_eq!(stops.len(), 32);
    assert!(!stops.is_empty());
    let linear = LinearGradient::with_stops(
        Point::default(),
        Point::new(1.0, 0.0),
        argui_paint::ColorInterpolation::Oklab,
        stops.clone(),
    );
    let radial = RadialGradient::with_stops(
        Point::default(),
        Point::new(1.0, 1.0),
        argui_paint::ColorInterpolation::Oklab,
        stops,
    );
    assert_eq!(linear.stops.len(), 32);
    assert_eq!(radial.stops.len(), 32);
}

#[test]
fn gradient_validation_is_precise() {
    assert_eq!(
        GradientStops::from_vec(vec![stop(0.0)]),
        Err(GradientError::TooFewStops)
    );
    assert_eq!(
        GradientStops::from_vec(vec![stop(0.0), stop(1.1)]),
        Err(GradientError::InvalidOffset)
    );
    assert_eq!(
        GradientStops::from_vec(vec![stop(0.8), stop(0.2)]),
        Err(GradientError::UnsortedStops)
    );
}

#[test]
fn raw_images_validate_dimensions_and_byte_count() {
    let image = ImageAsset::rgba8(ImageId(4), 2, 1, vec![255; 8]).unwrap();
    assert_eq!(image.byte_len(), 8);
    assert!(matches!(
        ImageAsset::rgba8(ImageId(4), 2, 1, vec![0; 7]),
        Err(ImageAssetError::InvalidByteLength {
            expected: 8,
            actual: 7
        })
    ));
    assert_eq!(
        ImageAsset::rgba8(ImageId(4), u32::MAX, u32::MAX, Vec::<u8>::new()),
        Err(ImageAssetError::InvalidDimensions)
    );
}

#[test]
fn transformed_clip_chains_require_every_region() {
    let bounds = Rect::new(Point::default(), Size::new(20.0, 20.0));
    let chain = ClipChain::from_regions([
        ClipRegion::new(bounds, Affine2D::IDENTITY),
        ClipRegion::new(bounds, Affine2D::translation(10.0, 0.0)),
    ]);

    assert!(chain.contains(Point::new(15.0, 10.0)));
    assert!(!chain.contains(Point::new(5.0, 10.0)));
    assert!(!chain.is_empty());
    assert_eq!(chain.regions().len(), 2);
    assert_eq!(
        chain
            .appended(ClipRegion::new(bounds, Affine2D::IDENTITY))
            .regions()
            .len(),
        3
    );
    let empty = chain.appended(ClipRegion::new(
        Rect::new(Point::default(), Size::new(20.0, 0.0)),
        Affine2D::IDENTITY,
    ));
    assert!(empty.is_empty());
    assert!(!empty.contains(Point::new(15.0, 10.0)));
}

#[test]
fn rounded_transformed_clips_reject_points_outside_the_corner_curve() {
    let bounds = Rect::new(Point::default(), Size::new(20.0, 20.0));
    let clip = ClipRegion::rounded(
        bounds,
        Affine2D::translation(10.0, 5.0),
        CornerRadii::all(8.0),
    );

    assert!(!clip.contains(Point::new(11.0, 6.0)));
    assert!(clip.contains(Point::new(18.0, 6.0)));
    assert!(clip.contains(Point::new(20.0, 15.0)));
}

#[test]
fn gradient_constructors_preserve_parameters_and_report_bad_stops() {
    let start = Point::new(0.1, 0.2);
    let end = Point::new(0.9, 0.8);
    let valid = [stop(0.0), stop(1.0)];
    let linear = LinearGradient::new(start, end, ColorInterpolation::Srgb, valid).unwrap();
    assert_eq!((linear.start, linear.end), (start, end));
    assert_eq!(linear.stops.as_slice(), &valid);
    let radial = RadialGradient::new(start, end, ColorInterpolation::Oklab, valid).unwrap();
    assert_eq!((radial.center, radial.radius), (start, end));
    let conic = ConicGradient::with_stops(start, 45.0, ColorInterpolation::Srgb, radial.stops);
    assert_eq!(conic.start_angle, 45.0);
    assert_eq!(conic.stops.as_slice(), &valid);
    assert_eq!(
        LinearGradient::new(start, end, ColorInterpolation::Srgb, [stop(0.0)]),
        Err(GradientError::TooFewStops)
    );
    assert_eq!(
        RadialGradient::new(start, end, ColorInterpolation::Srgb, [stop(0.8), stop(0.2)]),
        Err(GradientError::UnsortedStops)
    );
    assert_eq!(GradientStop::default().color, Color::TRANSPARENT);
}

#[test]
fn clip_regions_test_each_corner_and_noninvertible_transforms() {
    let bounds = Rect::new(Point::new(2.0, 3.0), Size::new(20.0, 20.0));
    let radii = CornerRadii {
        top_left: 2.0,
        top_right: 4.0,
        bottom_right: 6.0,
        bottom_left: 8.0,
    };
    let clip = ClipRegion::rounded(bounds, Affine2D::IDENTITY, radii);
    for corner in [
        Point::new(2.0, 3.0),
        Point::new(22.0, 3.0),
        Point::new(22.0, 23.0),
        Point::new(2.0, 23.0),
    ] {
        assert!(!clip.contains(corner));
    }
    assert!(clip.contains(Point::new(12.0, 13.0)));
    assert!(!clip.contains(Point::new(25.0, 13.0)));
    let singular = ClipRegion::new(
        bounds,
        Affine2D {
            matrix: [0.0, 0.0, 0.0, 1.0],
            translation: Point::default(),
        },
    );
    assert!(!singular.contains(Point::new(12.0, 13.0)));
    assert!(ClipChain::default().contains(Point::new(100.0, 100.0)));
}

#[test]
fn clip_region_retains_compositor_identity_without_changing_geometry() {
    let bounds = Rect::new(Point::new(4.0, 6.0), Size::new(12.0, 18.0));
    let id = argui_paint::CompositorId::new(27);
    let plain = ClipRegion::new(bounds, Affine2D::IDENTITY);
    let retained = plain.compositor(id);
    assert_eq!(retained.compositor, Some(id));
    assert_eq!(
        retained.contains(Point::new(10.0, 12.0)),
        plain.contains(Point::new(10.0, 12.0))
    );
    assert_eq!(
        retained.contains(Point::new(3.0, 12.0)),
        plain.contains(Point::new(3.0, 12.0))
    );
}

#[test]
fn image_ids_are_unique_and_errors_explain_the_failed_contract() {
    let first = ImageId::fresh();
    let second = ImageId::fresh();
    assert_ne!(first, second);
    let bad = ImageAsset::rgba8(first, 1, 1, vec![0; 3]).unwrap_err();
    assert!(bad.to_string().contains("InvalidByteLength"));
    assert!(
        GradientError::InvalidOffset
            .to_string()
            .contains("InvalidOffset")
    );
}

#[test]
fn runtime_visual_builders_retain_dynamic_inputs() {
    use std::hint::black_box;

    let offset = black_box(0.35);
    let color = black_box(Color::WHITE);
    let dynamic_stop = GradientStop::new(offset, color);
    assert_eq!(dynamic_stop.offset, offset);
    let stops = GradientStops::new([stop(0.0), dynamic_stop, stop(1.0)]).unwrap();
    assert_eq!(stops.as_slice()[1], dynamic_stop);
    let center = black_box(Point::new(0.3, 0.7));
    let angle = black_box(65.0);
    let gradient = ConicGradient::with_stops(center, angle, ColorInterpolation::Oklab, stops);
    assert_eq!(gradient.center, center);
    assert_eq!(gradient.start_angle, angle);

    let bounds = black_box(Rect::new(Point::new(1.0, 2.0), Size::new(10.0, 12.0)));
    let transform = black_box(Affine2D::translation(3.0, 4.0));
    let rounded = ClipRegion::rounded(bounds, transform, CornerRadii::all(black_box(2.0)));
    assert!(rounded.contains(Point::new(9.0, 12.0)));
    let plain = ClipRegion::new(bounds, transform);
    let chain = ClipChain::from_regions(black_box(vec![plain, rounded]));
    assert_eq!(chain.regions().len(), 2);
    assert!(chain.contains(Point::new(9.0, 12.0)));
    assert!(
        !chain
            .appended(black_box(ClipRegion::new(
                Rect::new(Point::new(0.0, 0.0), Size::new(0.0, 1.0)),
                Affine2D::IDENTITY,
            )))
            .contains(Point::new(9.0, 12.0))
    );
}

#[test]
fn shared_image_bytes_and_clip_regions_keep_their_backing_storage() {
    use std::sync::Arc;

    let pixels: Arc<[u8]> = vec![10, 20, 30, 255].into();
    let image = ImageAsset::rgba8(ImageId::fresh(), 1, 1, pixels.clone()).unwrap();
    assert!(Arc::ptr_eq(&image.rgba8, &pixels));
    assert_eq!(image.byte_len(), 4);

    let bounds = Rect::new(Point::default(), Size::new(4.0, 4.0));
    let regions: Arc<[ClipRegion]> = vec![ClipRegion::new(bounds, Affine2D::IDENTITY)].into();
    let chain = ClipChain::from_regions(regions.clone());
    assert!(std::ptr::eq(chain.regions().as_ptr(), regions.as_ptr()));
    assert!(chain.contains(Point::new(2.0, 2.0)));
}
