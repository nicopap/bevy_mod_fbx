//! Geometry.

use crate::utils::bbox::OptionalBoundingBox3d;
use bevy::math::{Vec2, Vec3};

/// Geometry mesh.
#[derive(Debug, Clone)]
pub struct GeoMesh {
    /// Name.
    pub name: Option<String>,
    /// Positions.
    pub positions: Vec<Vec3>,
    /// Normals.
    pub normals: Vec<Vec3>,
    /// UV.
    pub uv: Vec<Vec2>,
    /// Indices per materials.
    pub indices_per_material: Vec<Vec<u32>>,
}

impl GeoMesh {
    /// Returns bounding box of the submesh at the given index.
    pub fn bbox_submesh(&self, submesh_i: usize) -> OptionalBoundingBox3d {
        self.indices_per_material.get(submesh_i).map_or_else(
            OptionalBoundingBox3d::new,
            |submesh| {
                submesh
                    .iter()
                    .map(|&pos_i| self.positions[pos_i as usize])
                    .collect()
            },
        )
    }

    /// Returns bounding box of the whole mesh.
    pub fn bbox_mesh(&self) -> OptionalBoundingBox3d {
        self.positions.iter().collect()
    }
}
