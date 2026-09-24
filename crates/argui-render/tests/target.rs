use argui_core::{Color, Point, Rect, Size};
use argui_paint::{Filter, LayerStyle, Shadow};

#[allow(dead_code)]
#[path = "../src/target.rs"]
mod target;
use target::{PixelRegion, backdrop_sample_region};

/// Backdrop sampling extends beyond a filtered panel even without a shadow.
#[test]
fn blur_only_samples_around_the_output() {
    let viewport = PixelRegion::viewport(256, 256);
    let style = LayerStyle::new(Rect::new(Point::new(100.0, 80.0), Size::new(60.0, 40.0)))
        .backdrop(Filter::Blur(8.0));
    let output = PixelRegion::from_rect(style.transformed_bounds(), viewport).unwrap();
    let sample = backdrop_sample_region(&style, viewport, output);
    assert_eq!(sample.origin, [64, 32]);
    assert_eq!(sample.size, [128, 128]);
    assert_eq!(sample.intersection(output), Some(output));
}

/// A shadow may enlarge the output by a whole tile beyond the blur's sampling halo.
#[test]
fn blur_and_shadow_keep_the_entire_output_in_the_snapshot() {
    let viewport = PixelRegion::viewport(256, 256);
    let style = LayerStyle::new(Rect::new(Point::new(96.0, 80.0), Size::new(64.0, 48.0)))
        .backdrop(Filter::Blur(10.0))
        .shadow(Shadow::drop([0.0, 5.0], 12.0, Color::BLACK));
    let output = PixelRegion::from_rect(style.transformed_bounds(), viewport).unwrap();
    assert_eq!(output.origin, [32, 32]);
    assert_eq!(output.size, [192, 160]);
    let sample = backdrop_sample_region(&style, viewport, output);
    assert_eq!(sample.intersection(output), Some(output));
    assert_eq!(sample, output);
}

/// Exact composition clips do not expand to the 32-pixel damage tile boundary.
#[test]
fn effect_clip_keeps_exact_scroll_viewport_edges() {
    let viewport = PixelRegion::viewport(256, 256);
    let clip = Rect::new(Point::new(13.0, 47.0), Size::new(80.0, 90.0));
    let region = PixelRegion::from_clip_rect(clip, viewport).unwrap();
    assert_eq!(region.origin, [13, 47]);
    assert_eq!(region.size, [80, 90]);
    assert_eq!(
        region.intersection(PixelRegion::from_rect(clip, viewport).unwrap()),
        Some(region)
    );
    assert!(
        PixelRegion::from_clip_rect(
            Rect::new(Point::new(300.0, 300.0), Size::new(10.0, 10.0)),
            viewport,
        )
        .is_none()
    );
}
