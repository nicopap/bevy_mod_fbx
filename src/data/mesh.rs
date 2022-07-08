//! Mesh.

use super::scene::{GeometryMeshIndex, MaterialIndex};

/// Mesh.
#[derive(Debug, Clone)]
pub struct Mesh {
    /// Name.
    pub name: Option<String>,
    /// Geometry mesh index.
    pub geometry_mesh_index: GeometryMeshIndex,
    /// Materials.
    pub materials: Vec<MaterialIndex>,
}

impl Mesh {
    /// Returns geometry mesh index.
    pub fn geometry_mesh_index(&self) -> GeometryMeshIndex {
        self.geometry_mesh_index
    }
}
