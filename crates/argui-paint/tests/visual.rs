use argui_core::{Affine2D, Color, Point, Rect, Size};
use argui_paint::{
    ClipChain, ClipRegion, CornerRadii, GradientError, GradientStop, GradientStops, ImageAsset,
    ImageAssetError, ImageId, LinearGradient, RadialGradient,
};

fn stop(offset: f32) -> GradientStop {
    GradientStop::new(offset, Color::srgba(offset, 0.2, 0.4, 1.0))
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
