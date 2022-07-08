//! Geometry.

use cgmath::{Point2, Point3, Vector3};

use crate::utils::bounding_box::OptionalBoundingBox3d;

/// Geometry mesh.
#[derive(Debug, Clone)]
pub struct GeometryMesh {
    /// Name.
    pub name: Option<String>,
    /// Positions.
    pub positions: Vec<Point3<f32>>,
    /// Normals.
    pub normals: Vec<Vector3<f32>>,
    /// UV.
    pub uv: Vec<Point2<f32>>,
    /// Indices per materials.
    pub indices_per_material: Vec<Vec<u32>>,
}

impl GeometryMesh {
    /// Returns bounding box of the submesh at the given index.
    pub fn bbox_submesh(&self, submesh_i: usize) -> OptionalBoundingBox3d<f32> {
        self.indices_per_material.get(submesh_i).map_or_else(
            OptionalBoundingBox3d::new,
            |submesh| {
                submesh
                    .iter()
                    .map(|&pos_i| self.positions[pos_i as usize])
                    .map(Point3::from)
                    .collect()
            },
        )
    }

    /// Returns bounding box of the whole mesh.
    pub fn bbox_mesh(&self) -> OptionalBoundingBox3d<f32> {
        self.positions.iter().cloned().map(Point3::from).collect()
    }
}
