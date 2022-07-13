//! Mesh.

use bevy::asset::Handle;
use bevy::reflect::TypeUuid;
use bevy::render::mesh::Mesh as BevyMesh;

use super::scene::MaterialIndex;

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "966d55c0-515b-4141-97a1-de30ac8ee44c"]
pub struct FbxMesh {
    pub name: Option<String>,
    pub bevy_mesh_handle: Handle<BevyMesh>,
    pub materials: Vec<MaterialIndex>,
}
