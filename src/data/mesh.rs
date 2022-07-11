//! Mesh.

use super::scene::MaterialIndex;
use bevy::{
    asset::Handle, pbr::StandardMaterial, reflect::TypeUuid, render::mesh::Mesh as BevyMesh,
    utils::HashSet,
};

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "966d55c0-515b-4141-97a1-de30ac8ee44c"]
pub struct FbxMesh {
    pub name: Option<String>,
    pub bevy_mesh_handle: Handle<BevyMesh>,
    pub materials: HashSet<Handle<StandardMaterial>>,
}
