use bevy::{
    prelude::{Handle, Image, Mesh, StandardMaterial, Transform},
    reflect::TypeUuid,
    utils::HashMap,
};
use fbxcel_dom::v7400::object::ObjectId;

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "966d55c0-515b-4141-97a1-de30ac8ee44c"]
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
#[derive(Default, Debug, Clone, TypeUuid)]
#[uuid = "e87d49b6-8d6a-43c7-bb33-5315db8516eb"]
pub struct FbxScene {
    pub name: Option<String>,
    pub bevy_meshes: HashMap<Handle<Mesh>, String>,
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
