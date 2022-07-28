//! Collection of temporary extensions to the fbxcell_dom types
//! until they are merged upstream.

use fbxcel_dom::{
    fbxcel::low::v7400::AttributeValue,
    v7400::{
        object::{
            material::MaterialHandle, property::loaders::PrimitiveLoader, texture::TextureHandle,
            TypedObjectHandle,
        },
        GlobalSettings,
    },
};

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

pub trait MaterialHandleQuickPropsExt<'a> {
    fn get_f32(&self, field: &str) -> Option<f32>;
    fn get_u32(&self, field: &str) -> Option<u32>;
    fn get_i32(&self, field: &str) -> Option<i32>;
    fn get_bool(&self, field: &str) -> Option<bool>;
}
impl<'a> MaterialHandleQuickPropsExt<'a> for MaterialHandle<'a> {
    fn get_f32(&self, field: &str) -> Option<f32> {
        let props = self.properties();
        let prop = props.get_property(field)?;
        prop.load_value(PrimitiveLoader::<f32>::new()).ok()
    }
    fn get_u32(&self, field: &str) -> Option<u32> {
        let props = self.properties();
        let prop = props.get_property(field)?;
        prop.load_value(PrimitiveLoader::<u32>::new()).ok()
    }
    fn get_i32(&self, field: &str) -> Option<i32> {
        let props = self.properties();
        let prop = props.get_property(field)?;
        prop.load_value(PrimitiveLoader::<i32>::new()).ok()
    }
    fn get_bool(&self, field: &str) -> Option<bool> {
        let props = self.properties();
        let prop = props.get_property(field)?;
        prop.load_value(PrimitiveLoader::<bool>::new()).ok()
    }
}

pub trait GlobalSettingsExt<'a> {
    fn fbx_scale(&self) -> Option<f64>;
}
impl<'a> GlobalSettingsExt<'a> for GlobalSettings<'a> {
    fn fbx_scale(&self) -> Option<f64> {
        let prop = self.raw_properties().get_property("UnitScaleFactor")?;
        let attribute = prop.value_part().get(0)?;
        match attribute {
            AttributeValue::F64(scale) => Some(*scale),
            _ => None,
        }
    }
}
