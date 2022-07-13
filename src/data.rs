use bevy::{
    asset::Handle,
    pbr::StandardMaterial,
    reflect::TypeUuid,
    render::{mesh::Mesh as BevyMesh, texture::Image},
    utils::{HashMap, HashSet},
};

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "966d55c0-515b-4141-97a1-de30ac8ee44c"]
pub struct FbxMesh {
    pub name: Option<String>,
    pub bevy_mesh_handles: Vec<Handle<BevyMesh>>,
    pub materials: Vec<Handle<StandardMaterial>>,
}
#[derive(Default, Debug, Clone, TypeUuid)]
#[uuid = "e87d49b6-8d6a-43c7-bb33-5315db8516eb"]
pub struct FbxScene {
    pub name: Option<String>,
    pub bevy_meshes: HashMap<Handle<BevyMesh>, String>,
    pub materials: HashMap<String, Handle<StandardMaterial>>,
    pub textures: HashMap<String, Handle<Image>>,
    pub meshes: HashSet<Handle<FbxMesh>>,
}
