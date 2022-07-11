use bevy_app::prelude::{App, Plugin};
use bevy_asset::AddAsset;

pub use data::mesh::FbxMesh;
pub use data::scene::Scene as FbxScene;
pub use loader::FbxLoader;

pub(crate) mod data;
pub(crate) mod loader;
pub(crate) mod tangents;
pub(crate) mod utils;

/// Adds support for FBX file loading to the app.
#[derive(Default)]
pub struct FbxPlugin;

impl Plugin for FbxPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<FbxLoader>()
            .add_asset::<FbxMesh>()
            .add_asset::<FbxScene>();
    }
}
