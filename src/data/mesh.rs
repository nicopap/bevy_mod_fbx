//! Mesh.

use bevy::{
    asset::Handle, pbr::StandardMaterial, reflect::TypeUuid, render::mesh::Mesh as BevyMesh,
};

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "966d55c0-515b-4141-97a1-de30ac8ee44c"]
pub struct FbxMesh {
    pub name: Option<String>,
    pub bevy_mesh_handles: Vec<Handle<BevyMesh>>,
    pub materials: Vec<Handle<StandardMaterial>>,
}
