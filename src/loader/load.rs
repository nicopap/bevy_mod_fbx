use anyhow::Result;
use bevy_asset::{AssetLoader, BoxedFuture, LoadContext};
use bevy_ecs::prelude::{FromWorld, World};
use bevy_render::{renderer::RenderDevice, texture::CompressedImageFormats};

// TODO: Error handling
// use thiserror::Error;
//
// #[derive(Debug, Error)]
// pub enum FbxError {}

pub struct FbxLoader {
    supported_compressed_formats: CompressedImageFormats,
}

impl AssetLoader for FbxLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        todo!()
    }

    fn extensions(&self) -> &[&str] {
        &["fbx"]
    }
}

impl FromWorld for FbxLoader {
    fn from_world(world: &mut World) -> Self {
        let supported_compressed_formats = match world.get_resource::<RenderDevice>() {
            Some(render_device) => CompressedImageFormats::from_features(render_device.features()),

            None => CompressedImageFormats::all(),
        };

        Self {
            supported_compressed_formats,
        }
    }
}

// TODO: Implement rest of the loader code.
// TODO: Avoid loading materials until either bevy or lambert2pbr will implement converter.
