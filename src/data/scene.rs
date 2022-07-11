#![allow(dead_code)]
//! Scene.

use super::{material::Material, mesh::FbxMesh, texture::Texture};
use bevy_asset::Handle;
use bevy_reflect::TypeUuid;
use bevy_render::mesh::Mesh as BevyMesh;
use bevy_utils::{HashMap, HashSet};

#[derive(Default, Debug, Clone, TypeUuid)]
#[uuid = "e87d49b6-8d6a-43c7-bb33-5315db8516eb"]
pub struct Scene {
    pub name: Option<String>,
    pub bevy_meshes: HashMap<Handle<BevyMesh>, String>,
    pub materials: Vec<Material>,
    pub meshes: HashSet<Handle<FbxMesh>>,
    pub textures: Vec<Texture>,
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

    /// Add a material.
    pub(crate) fn add_material(&mut self, material: Material) -> MaterialIndex {
        let index = MaterialIndex::new(self.materials.len());

        self.materials.push(material);

        index
    }

    /// Returns an iterator of materials.
    pub fn materials(&self) -> impl Iterator<Item = &Material> {
        self.materials.iter()
    }

    /// Returns a reference to the material.
    pub fn material(&self, i: MaterialIndex) -> Option<&Material> {
        self.materials.get(i.to_usize())
    }

    /// Add a mesh.
    pub(crate) fn add_mesh(&mut self, mesh: Handle<FbxMesh>) {
        self.meshes.insert(mesh);
    }

    /// Add a texture.
    pub(crate) fn add_texture(&mut self, texture: Texture) -> TextureIndex {
        let index = TextureIndex::new(self.textures.len());

        self.textures.push(texture);

        index
    }

    /// Returns an iterator of textures.
    pub fn textures(&self) -> impl Iterator<Item = &Texture> {
        self.textures.iter()
    }

    /// Returns a reference to the texture.
    pub fn texture(&self, i: TextureIndex) -> Option<&Texture> {
        self.textures.get(i.to_usize())
    }
}

macro_rules! define_index_type {
    ($(
        $(#[$meta:meta])*
        $ty:ident;
    )*) => {
        $(
            $(#[$meta])*
            #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub struct $ty(u32);

            impl $ty {
                /// Creates a new index.
                ///
                /// # Panics
                ///
                /// Panics if the given index is larger than `std::u32::MAX`.
                pub(crate) fn new(i: usize) -> Self {
                    assert!(i <= std::u32::MAX as usize);
                    Self(i as u32)
                }

                /// Retuns `usize` value.
                pub fn to_usize(self) -> usize {
                    self.0 as usize
                }
            }
        )*
    };
}

define_index_type! {
    /// Material index.
    MaterialIndex;
    /// Texture index.
    TextureIndex;
}
