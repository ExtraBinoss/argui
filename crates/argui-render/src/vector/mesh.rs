use std::collections::HashMap;

use argui_paint::VectorAsset;

use super::GpuVertex;

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct PointKey(i32, i32);

type EdgeKey = (PointKey, PointKey);

pub(super) fn expand(asset: &VectorAsset) -> Vec<GpuVertex> {
    let edge_counts = edge_counts(asset);
    asset
        .indices
        .as_chunks::<3>()
        .0
        .iter()
        .flat_map(|triangle| {
            let boundary = [
                is_boundary(asset, &edge_counts, triangle[1], triangle[2]),
                is_boundary(asset, &edge_counts, triangle[2], triangle[0]),
                is_boundary(asset, &edge_counts, triangle[0], triangle[1]),
            ];
            triangle
                .iter()
                .enumerate()
                .filter_map(|(corner, index)| {
                    let vertex = asset.vertices.get(*index as usize)?;
                    let mut barycentric = [0.0; 3];
                    barycentric[corner] = 1.0;
                    Some(GpuVertex {
                        from: vertex.from,
                        to: vertex.to,
                        color_from: vertex.color_from.as_array(),
                        color_to: vertex.color_to.as_array(),
                        barycentric,
                        boundary,
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn edge_counts(asset: &VectorAsset) -> HashMap<EdgeKey, u32> {
    let mut counts = HashMap::new();
    for triangle in asset.indices.as_chunks::<3>().0 {
        for (left, right) in [
            (triangle[0], triangle[1]),
            (triangle[1], triangle[2]),
            (triangle[2], triangle[0]),
        ] {
            if let Some(edge) = edge(asset, left, right) {
                *counts.entry(edge).or_default() += 1;
            }
        }
    }
    counts
}

fn is_boundary(asset: &VectorAsset, counts: &HashMap<EdgeKey, u32>, left: u32, right: u32) -> f32 {
    f32::from(edge(asset, left, right).is_some_and(|edge| counts.get(&edge) == Some(&1)))
}

fn edge(asset: &VectorAsset, left: u32, right: u32) -> Option<EdgeKey> {
    let left = point(asset.vertices.get(left as usize)?.from);
    let right = point(asset.vertices.get(right as usize)?.from);
    Some(if left <= right {
        (left, right)
    } else {
        (right, left)
    })
}

fn point(value: [f32; 2]) -> PointKey {
    PointKey(
        (value[0] * 10_000.0).round() as i32,
        (value[1] * 10_000.0).round() as i32,
    )
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use argui_core::{Color, Size};
    use argui_paint::{VectorAsset, VectorId, VectorVertex};

    use super::expand;

    #[test]
    fn expansion_marks_only_the_outer_edges() {
        let vertex = |position| VectorVertex {
            from: position,
            to: position,
            color_from: Color::WHITE,
            color_to: Color::WHITE,
        };
        let asset = VectorAsset {
            id: VectorId(1),
            size: Size::new(1.0, 1.0),
            vertices: Arc::from([
                vertex([0.0, 0.0]),
                vertex([1.0, 0.0]),
                vertex([1.0, 1.0]),
                vertex([0.0, 1.0]),
            ]),
            indices: Arc::from([0, 1, 2, 0, 2, 3]),
        };
        let expanded = expand(&asset);
        assert_eq!(expanded.len(), 6);
        assert_eq!(expanded[0].boundary, [1.0, 0.0, 1.0]);
        assert_eq!(expanded[3].boundary, [1.0, 1.0, 0.0]);
        assert_eq!(expanded[0].barycentric, [1.0, 0.0, 0.0]);
        assert_eq!(expanded[2].barycentric, [0.0, 0.0, 1.0]);
    }

    #[test]
    fn invalid_indices_do_not_create_invalid_vertices() {
        let asset = VectorAsset {
            id: VectorId(2),
            size: Size::new(1.0, 1.0),
            vertices: Arc::from([]),
            indices: Arc::from([0, 1, 2]),
        };
        assert!(expand(&asset).is_empty());
    }

    #[test]
    fn coincident_tessellator_vertices_do_not_create_visible_seams() {
        let vertex = |position| VectorVertex {
            from: position,
            to: position,
            color_from: Color::WHITE,
            color_to: Color::WHITE,
        };
        let asset = VectorAsset {
            id: VectorId(3),
            size: Size::new(1.0, 1.0),
            vertices: Arc::from([
                vertex([0.0, 0.0]),
                vertex([1.0, 0.0]),
                vertex([1.0, 1.0]),
                vertex([0.0, 0.0]),
                vertex([1.0, 1.0]),
                vertex([0.0, 1.0]),
            ]),
            indices: Arc::from([0, 1, 2, 3, 4, 5]),
        };
        let expanded = expand(&asset);
        assert_eq!(expanded[0].boundary, [1.0, 0.0, 1.0]);
        assert_eq!(expanded[3].boundary, [1.0, 1.0, 0.0]);
    }
}
