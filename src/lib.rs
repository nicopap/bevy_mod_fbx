#![allow(unused)] // Remove later.

use bevy_app::{App, Plugin};
use type_uuid::TypeUuid;

mod data;
mod loader;
mod utils;

pub struct FbxPlugin;

impl Plugin for FbxPlugin {
    fn build(&self, app: &mut App) {}
}
