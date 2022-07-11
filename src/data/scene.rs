#![allow(dead_code)]
//! Scene.

use super::mesh::FbxMesh;
use bevy::{
    asset::Handle,
    pbr::StandardMaterial,
    reflect::TypeUuid,
    render::{mesh::Mesh as BevyMesh, texture::Image},
    utils::{HashMap, HashSet},
};

#[derive(Default, Debug, Clone, TypeUuid)]
#[uuid = "e87d49b6-8d6a-43c7-bb33-5315db8516eb"]
pub struct Scene {
    pub name: Option<String>,
    pub bevy_meshes: HashMap<Handle<BevyMesh>, String>,
    pub materials: HashMap<String, Handle<StandardMaterial>>,
    pub textures: HashMap<String, Handle<Image>>,
    pub meshes: HashSet<Handle<FbxMesh>>,
}

impl Scene {
    /// Creates a new `Scene`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the scene name.
    pub fn set_name(&mut self, name: impl Into<Option<String>>) {
        self.name = name.into();
    }

    /// Add a mesh.
    pub(crate) fn add_mesh(&mut self, mesh: Handle<FbxMesh>) {
        self.meshes.insert(mesh);
    }
}
