use bevy::prelude::{Handle, Image};
use fbxcel_dom::v7400::object::material::MaterialHandle;

use crate::{
    loader::{Ctx, Loader},
    utils::fbx_extend::MaterialHandleExt,
};

pub struct Textures<'a, 'b> {
    obj: MaterialHandle<'a>,
    ctx: Ctx<'a, 'b>,
    loader: &'a mut Loader,
}
impl<'a, 'b> Textures<'a, 'b> {
    pub(crate) fn new(ctx: Ctx<'a, 'b>, obj: MaterialHandle<'a>, loader: &'a mut Loader) -> Self {
        Self { ctx, obj, loader }
    }

    pub fn get(&mut self, fbx_texture_field: &str) -> Option<Handle<Image>> {
        let fbx_handle = self.obj.load_texture(fbx_texture_field)?;
        Some(self.loader.load_texture(self.ctx, fbx_handle))
    }
}
