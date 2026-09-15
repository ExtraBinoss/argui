use super::BackdropError;
use argui_core::{Point, Rect, Size};
use argui_paint::ClipChain;

/// Converts intersected rounded/transformed clips into compositor region rectangles.
/// Adjacent equal scanlines coalesce; rectangular sidebars use one rectangle.
/// Converts backdrop clip chains into merged rectangles in logical coordinates.
/// `shapes` are the clip chains describing visible backdrop regions.
///
/// # Errors
/// Returns an error if a shape contains invalid geometry.
pub fn region_rectangles(shapes: &[ClipChain]) -> Result<Vec<Rect>, BackdropError> {
    let mut output = Vec::new();
    for shape in shapes {
        let Some(first) = shape.regions().first() else {
            continue;
        };
        let mut bounds = first.transform.transform_rect(first.bounds);
        for clip in shape.regions() {
            let transformed = clip.transform.transform_rect(clip.bounds);
            if ![
                transformed.origin.x,
                transformed.origin.y,
                transformed.size.width,
                transformed.size.height,
            ]
            .into_iter()
            .all(f32::is_finite)
            {
                return Err(BackdropError::InvalidGeometry);
            }
            bounds = bounds.intersection(transformed).unwrap_or_default();
        }
        let left = bounds.origin.x.ceil() as i32;
        let top = bounds.origin.y.ceil() as i32;
        let right = (bounds.origin.x + bounds.size.width).floor() as i32;
        let bottom = (bounds.origin.y + bounds.size.height).floor() as i32;
        let width = i64::from(right) - i64::from(left);
        let height = i64::from(bottom) - i64::from(top);
        if width <= 0 || height <= 0 {
            continue;
        }
        if width > 32768 || height > 32768 {
            return Err(BackdropError::InvalidGeometry);
        }
        if shape.regions().iter().all(|clip| {
            clip.radii
                .as_array()
                .into_iter()
                .all(|radius| radius <= 0.0)
                && clip.transform.matrix[1] == 0.0
                && clip.transform.matrix[2] == 0.0
        }) {
            output.push(rect(left, top, right, bottom));
            continue;
        }
        let start = output.len();
        for y in top..bottom {
            let contains = |x| shape.contains(Point::new(x as f32 + 0.5, y as f32 + 0.5));
            let Some(x1) = (left..right).find(|&x| contains(x)) else {
                continue;
            };
            let x2 = (x1..right).rfind(|&x| contains(x)).unwrap() + 1;
            let can_merge = output.len() > start;
            if let Some(previous) = output.last_mut().filter(|_| can_merge)
                && previous.origin.x == x1 as f32
                && previous.size.width == (x2 - x1) as f32
                && previous.origin.y + previous.size.height == y as f32
            {
                previous.size.height += 1.0;
            } else {
                output.push(rect(x1, y, x2, y + 1));
            }
        }
    }
    Ok(output)
}

fn rect(left: i32, top: i32, right: i32, bottom: i32) -> Rect {
    Rect::new(
        Point::new(left as f32, top as f32),
        Size::new((right - left) as f32, (bottom - top) as f32),
    )
}
