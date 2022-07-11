//! Mesh.

use bevy_asset::Handle;
use bevy_reflect::TypeUuid;
use bevy_render::mesh::Mesh as BevyMesh;

use super::scene::MaterialIndex;

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "966d55c0-515b-4141-97a1-de30ac8ee44c"]
pub struct Mesh {
    pub name: Option<String>,
    pub bevy_mesh_handle: Handle<BevyMesh>,
    pub materials: Vec<MaterialIndex>,
}
