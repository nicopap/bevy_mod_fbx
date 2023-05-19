use bevy::{
    prelude::{Asset, Handle, Image, Mesh, StandardMaterial, Transform},
    utils::HashMap,
};
use fbxcel_dom::v7400::object::ObjectId;

#[derive(Debug, Asset, Clone, Default)]
pub struct FbxMesh {
    pub name: Option<String>,
    pub bevy_mesh_handles: Vec<Handle<Mesh>>,
    pub materials: Vec<Handle<StandardMaterial>>,
}

/// The data loaded from a FBX scene.
///
/// Note that the loader spawns a [`Scene`], with all the
/// FBX nodes spawned as entities (with their corresponding [`Name`] set)
/// in the ECS,
/// and you should absolutely use the ECS entities over
/// manipulating this data structure.
/// It is provided publicly, because it might be a good store for strong handles.
///
/// [`Scene`]: bevy::scene::Scene
/// [`Name`]: bevy::core::Name
#[derive(Default, Asset, Debug, Clone)]
pub struct FbxScene {
    pub name: Option<String>,
    pub bevy_meshes: HashMap<String, Handle<Mesh>>,
    pub materials: HashMap<String, Handle<StandardMaterial>>,
    pub textures: HashMap<String, Handle<Image>>,
    pub meshes: HashMap<ObjectId, Handle<FbxMesh>>,
    pub hierarchy: HashMap<ObjectId, FbxObject>,
    pub roots: Vec<ObjectId>,
}

/// An FBX object in the scene tree.
///
/// This serves as a node in the transform hierarchy.
#[derive(Default, Debug, Clone)]
pub struct FbxObject {
    pub name: Option<String>,
    pub transform: Transform,
    /// The children of this node.
    ///
    /// # Notes
    /// Not all [`ObjectId`] declared as child of an `FbxObject`
    /// are relevant to Bevy.
    /// Meaning that you won't find the `ObjectId` in `hierarchy` or `meshes`
    /// `HashMap`s of the [`FbxScene`] structure.
    pub children: Vec<ObjectId>,
}
