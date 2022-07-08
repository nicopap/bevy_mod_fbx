//! Triangulator.

use std::f64;

use anyhow::{anyhow, bail};
// TODO: Switch to bevy_math
use cgmath::{InnerSpace, Point3, Vector2, Vector3};
use fbxcel_dom::v7400::data::mesh::{PolygonVertexIndex, PolygonVertices};

/// Axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Axis {
    X, Y, Z,
}

/// Returns smallest direction.
fn smallest_direction(v: &Vector3<f64>) -> Axis {
    if v.x < v.y {
        if v.z < v.x {
            Axis::Z
        } else {
            Axis::X
        }
    } else if v.z < v.y {
        Axis::Z
    } else {
        Axis::Y
    }
}


/// Triangulator.
pub fn triangulator(
    pvs: &PolygonVertices<'_>,
    poly_pvis: &[PolygonVertexIndex],
    results: &mut Vec<[PolygonVertexIndex; 3]>,
) -> anyhow::Result<()> {
    macro_rules! get_vec {
        ($pvii:expr) => {
            get_vec(pvs, poly_pvis[$pvii])
        };
    }

    match poly_pvis.len() {
        n @ 0..=2 => {
            // Not a polygon.
            // It is impossible to triangulate a point, line, or "nothing".
            bail!("Not enough vertices in the polygon: length={}", n);
        }
        3 => {
            // Got a triangle, no need of triangulation.
            results.push([poly_pvis[0], poly_pvis[1], poly_pvis[2]]);
            Ok(())
        }
        4 => {
            // p0, p1, p2, p3: vertices of the quadrangle (angle{0..3}).
            let p0 = get_vec!(0)?;
            let p1 = get_vec!(1)?;
            let p2 = get_vec!(2)?;
            let p3 = get_vec!(3)?;
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
                results.extend_from_slice(&[
                    [poly_pvis[0], poly_pvis[1], poly_pvis[2]],
                    [poly_pvis[2], poly_pvis[3], poly_pvis[0]],
                ]);
            } else {
                // Either angle1 or angle3 is convex.
                // Cut from p1 to p3.
                results.extend_from_slice(&[
                    [poly_pvis[0], poly_pvis[1], poly_pvis[3]],
                    [poly_pvis[3], poly_pvis[1], poly_pvis[2]],
                ]);
            }
            Ok(())
        }
        n => {
            let points = (0..n).map(|i| get_vec!(i)).collect::<Result<Vec<_>, _>>()?;
            let points_2d: Vec<_> = {
                // Reduce dimensions for faster computation.
                // This helps treat points which are not on a single plane.
                let (min, max) =
                    bounding_box(&points).expect("Should never happen: there are 5 or more points");
                let width = max - min;
                match smallest_direction(&width) {
                    Axis::X => points
                        .into_iter()
                        .map(|v| Vector2::new(v[1], v[2]))
                        .collect(),
                    Axis::Y => points
                        .into_iter()
                        .map(|v| Vector2::new(v[0], v[2]))
                        .collect(),
                    Axis::Z => points
                        .into_iter()
                        .map(|v| Vector2::new(v[0], v[1]))
                        .collect(),
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
                        let prev_cur = prev - cur;
                        let cur_next = cur - next;
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

                let convex_pvi = poly_pvis[convex_index];
                let iter1 = (0..n)
                    .cycle()
                    .skip(convex_index + 1)
                    .take(n - 2)
                    .map(|i| poly_pvis[i]);
                let iter2 = (0..n).cycle().skip(convex_index + 2).map(|i| poly_pvis[i]);
                for (pvi1, pvi2) in iter1.zip(iter2) {
                    results.push([convex_pvi, pvi1, pvi2]);
                }
                Ok(())
            } else {
                bail!(
                    "Unsupported polygon: {}-gon with two or more concave angles",
                    n
                );
            }
        }
    }
}

/// Returns the vector.
fn get_vec(pvs: &PolygonVertices<'_>, pvi: PolygonVertexIndex) -> anyhow::Result<Point3<f64>> {
    pvs.control_point(pvi)
        .map(Into::into)
        .ok_or_else(|| anyhow!("Index out of range: {:?}", pvi))
}

/// Returns bounding box as `(min, max)`.
fn bounding_box<'a>(
    points: impl IntoIterator<Item = &'a Point3<f64>>,
) -> Option<(Point3<f64>, Point3<f64>)> {
    points.into_iter().fold(None, |minmax, point| {
        minmax.map_or_else(
            || Some((*point, *point)),
            |(min, max)| {
                Some((
                    Point3 {
                        x: min.x.min(point.x),
                        y: min.y.min(point.y),
                        z: min.z.min(point.z),
                    },
                    Point3 {
                        x: max.x.max(point.x),
                        y: max.y.max(point.y),
                        z: max.z.max(point.z),
                    },
                ))
            },
        )
    })
}
