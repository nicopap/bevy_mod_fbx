//! Collection of temporary extensions to the fbxcell_dom types
//! until they are merged upstream.

use fbxcel_dom::v7400::object::material::MaterialHandle;
use fbxcel_dom::v7400::object::texture::TextureHandle;
use fbxcel_dom::v7400::object::TypedObjectHandle;

pub trait MaterialHandleExt<'a> {
    fn load_texture(&self, name: &'static str) -> Option<TextureHandle>;
}
impl<'a> MaterialHandleExt<'a> for MaterialHandle<'a> {
    fn load_texture(&self, name: &'static str) -> Option<TextureHandle> {
        self.source_objects()
            .filter(|obj| obj.label() == Some(name))
            .filter_map(|obj| obj.object_handle())
            .find_map(|obj| match obj.get_typed() {
                TypedObjectHandle::Texture(o) => Some(o),
                _ => None,
            })
    }
}
