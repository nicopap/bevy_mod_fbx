//! Triangulator.

// TODO: https://github.com/HeavyRain266/bevy_mod_fbx/issues/11

use anyhow::bail;
use bevy::math::{DVec2, DVec3};
use fbxcel_dom::v7400::data::mesh::{PolygonVertexIndex, PolygonVertices};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Axis {
    X,
    Y,
    Z,
}

/// Returns smallest direction.
fn smallest_direction(v: &DVec3) -> Axis {
    match () {
        () if v.x < v.y && v.z < v.x => Axis::Z,
        () if v.x < v.y => Axis::X,
        () if v.z < v.y => Axis::Z,
        () => Axis::Y,
    }
}

/// Triangulate.
pub fn triangulate(
    vertices: &PolygonVertices<'_>,
    indices: &[PolygonVertexIndex],
    triangles: &mut Vec<[PolygonVertexIndex; 3]>,
) -> anyhow::Result<()> {
    let position_at = |i| DVec3::from(vertices.control_point(i).unwrap());

    let n = indices.len();

    match indices {
        // Not a polygon.
        [] | [_] | [_, _] => bail!("A polygon of size {n} cannot be triangulated"),

        // Got a triangle, no need of triangulation.
        &[i0, i1, i2] => triangles.push([i0, i1, i2]),

        &[i0, i1, i2, i3] => {
            // p0, p1, p2, p3: vertices of the quadrangle (angle{0..3}).
            let [p0, p1, p2, p3] = [i0, i1, i2, i3].map(position_at);

            // n1: Normal vector calculated with two edges of the angle1.
            // n3: Normal vector calculated with two edges of the angle3.
            let n1 = (p0 - p1).cross(p1 - p2);
            let n3 = (p2 - p3).cross(p3 - p0);

            // If both angle1 and angle3 are concave, vectors n1 and n3 are
            // oriented in the same direction and `n1.dot(n3)` will be positive.
            // If either angle1 or angle3 is concave, vector n1 and n3 are
            // oriented in the opposite directions and `n1.dot(n3)` will be
            // negative.
            // It does not matter when the vertices of quadrangle is not on the
            // same plane, because whichever diagonal you choose, the cut will
            // be inaccurate.
            if n1.dot(n3) >= 0.0 {
                // Both angle1 and angle3 are concave.
                // This means that either angle0 or angle2 can be convex.
                // Cut from p0 to p2.
                triangles.extend_from_slice(&[[i0, i1, i2], [i2, i3, i0]]);
            } else {
                // Either angle1 or angle3 is convex. Cut from p1 to p3.
                triangles.extend_from_slice(&[[i0, i1, i3], [i3, i1, i2]]);
            }
        }
        indices => {
            let points: Vec<_> = indices.iter().map(|i| position_at(*i)).collect();
            let points_2d: Vec<_> = {
                // Reduce dimensions for faster computation.
                // This helps treat points which are not on a single plane.
                let (min, max) =
                    bounding_box(&points).expect("Should never happen: there are 5 or more points");

                let width = max - min;

                match smallest_direction(&width) {
                    Axis::X => points.into_iter().map(|v| DVec2::new(v[1], v[2])).collect(),
                    Axis::Y => points.into_iter().map(|v| DVec2::new(v[0], v[2])).collect(),
                    Axis::Z => points.into_iter().map(|v| DVec2::new(v[0], v[1])).collect(),
                }
            };
            // Normal directions.
            let normal_directions = {
                // 0 ... n-1
                let iter_cur = points_2d.iter();

                // n-1, 0, ... n-2
                let iter_prev = points_2d.iter().cycle().skip(n - 1);

                // 1, ... n-1, 0
                let iter_next = points_2d.iter().cycle().skip(1);

                iter_cur
                    .zip(iter_prev)
                    .zip(iter_next)
                    .map(|((cur, prev), next)| {
                        let prev_cur = *prev - *cur;
                        let cur_next = *cur - *next;
                        prev_cur.perp_dot(cur_next) > 0.0
                    })
                    .collect::<Vec<_>>()
            };
            assert_eq!(normal_directions.len(), n);

            let dirs_true_count = normal_directions.iter().filter(|&&v| v).count();

            if dirs_true_count <= 1 || dirs_true_count >= n - 1 {
                // Zero or one angles are concave.
                let minor_sign = dirs_true_count <= 1;

                // If there are no concave angles, use 0 as center.
                let convex_index = normal_directions
                    .iter()
                    .position(|&sign| sign == minor_sign)
                    .unwrap_or(0);

                let convex_pvi = indices[convex_index];

                let iter1 = (0..n)
                    .cycle()
                    .skip(convex_index + 1)
                    .take(n - 2)
                    .map(|i| indices[i]);

                let iter2 = (0..n).cycle().skip(convex_index + 2).map(|i| indices[i]);

                for (pvi1, pvi2) in iter1.zip(iter2) {
                    triangles.push([convex_pvi, pvi1, pvi2]);
                }
            } else {
                bail!("Unsupported polygon: {n}-gon with two or more concave angles");
            }
        }
    }
    Ok(())
}

/// Returns bounding box as `(min, max)`.
fn bounding_box<'a>(points: impl IntoIterator<Item = &'a DVec3>) -> Option<(DVec3, DVec3)> {
    points.into_iter().fold(None, |minmax, point| {
        minmax.map_or_else(
            || Some((*point, *point)),
            |(min, max)| {
                Some((
                    DVec3::new(min.x.min(point.x), min.y.min(point.y), min.z.min(point.z)),
                    DVec3::new(max.x.max(point.x), max.y.max(point.y), max.z.max(point.z)),
                ))
            },
        )
    })
}
