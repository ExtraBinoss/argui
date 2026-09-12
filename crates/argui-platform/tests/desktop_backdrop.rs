#![cfg(all(feature = "desktop-backdrop", not(target_arch = "wasm32")))]
use argui_core::{Affine2D, Point, Rect, Size};
use argui_paint::{ClipChain, ClipRegion, CornerRadii};
use argui_platform::desktop_backdrop::{BackdropError, region_rectangles};

fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect::new(Point::new(x, y), Size::new(width, height))
}
fn shape(bounds: Rect, radius: f32) -> ClipChain {
    ClipChain::from_regions([ClipRegion::rounded(
        bounds,
        Affine2D::IDENTITY,
        CornerRadii::all(radius),
    )])
}

#[test]
fn native_masks_preserve_rounding_clipping_and_disjoint_panels() {
    let rounded = shape(rect(0.0, 0.0, 100.0, 80.0), 14.0).appended(ClipRegion::new(
        rect(0.0, 0.0, 74.0, 65.0),
        Affine2D::IDENTITY,
    ));
    let rectangles = region_rectangles(std::slice::from_ref(&rounded)).unwrap();
    assert!(rectangles.len() < 25);
    for y in 0..80 {
        for x in 0..100 {
            let point = Point::new(x as f32 + 0.5, y as f32 + 0.5);
            assert_eq!(
                rectangles.iter().any(|rect| rect.contains(point)),
                rounded.contains(point)
            );
        }
    }
    let other = rect(200.0, 100.0, 200.0, 120.0);
    let all = region_rectangles(&[rounded, shape(other, 0.0)]).unwrap();
    assert_eq!(all.last(), Some(&other));
}

#[test]
fn rectangular_masks_coalesce_and_invalid_or_empty_shapes_fail_safely() {
    let sidebar = rect(0.0, 64.0, 260.0, 716.0);
    assert_eq!(
        region_rectangles(&[shape(sidebar, 0.0)]).unwrap(),
        vec![sidebar]
    );
    for empty in [
        ClipChain::default(),
        shape(Rect::default(), 4.0),
        shape(rect(0.0, 0.0, 0.5, 0.5), 0.0),
    ] {
        assert!(region_rectangles(&[empty]).unwrap().is_empty());
    }
    for bad in [
        rect(f32::NAN, 0.0, 10.0, 10.0),
        rect(0.0, 0.0, 40_000.0, 10.0),
        rect(0.0, 0.0, 10.0, 40_000.0),
    ] {
        assert_eq!(
            region_rectangles(&[shape(bad, 0.0)]),
            Err(BackdropError::InvalidGeometry)
        );
    }
    let hidden = shape(rect(0.0, 0.0, 10.0, 10.0), 2.0).appended(ClipRegion::new(
        rect(30.0, 30.0, 10.0, 10.0),
        Affine2D::IDENTITY,
    ));
    assert!(region_rectangles(&[hidden]).unwrap().is_empty());
    assert!(
        BackdropError::Unsupported
            .to_string()
            .contains("unavailable")
    );
    assert!(
        BackdropError::InvalidGeometry
            .to_string()
            .contains("geometry")
    );
    assert_eq!(
        BackdropError::Platform("request failed".into()).to_string(),
        "request failed"
    );
}

#[test]
fn rotated_and_singular_shapes_match_the_rendered_clip() {
    for matrix in [[0.8, 0.6, -0.6, 0.8], [0.0; 4]] {
        let shape = ClipChain::from_regions([ClipRegion::rounded(
            rect(0.0, 0.0, 50.0, 30.0),
            Affine2D {
                matrix,
                translation: Point::new(30.0, 10.0),
            },
            CornerRadii::all(3.0),
        )]);
        let regions = region_rectangles(std::slice::from_ref(&shape)).unwrap();
        for y in 10..60 {
            for x in 15..60 {
                let point = Point::new(x as f32 + 0.5, y as f32 + 0.5);
                assert_eq!(
                    regions.iter().any(|region| region.contains(point)),
                    shape.contains(point)
                );
            }
        }
    }
}
