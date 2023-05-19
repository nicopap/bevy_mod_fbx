//! Collection of temporary extensions to the fbxcell_dom types
//! until they are merged upstream.

use bevy::math::{DVec2, DVec3, DVec4, EulerRot, Vec2, Vec3, Vec4};
use mint::{Vector2, Vector3, Vector4};

use fbxcel_dom::{
    fbxcel::low::v7400::AttributeValue,
    v7400::{
        object::{
            material::MaterialHandle,
            model::ModelHandle,
            property::{
                loaders::{MintLoader, PrimitiveLoader, RgbLoader},
                LoadProperty, ObjectProperties, PropertyHandle,
            },
            texture::TextureHandle,
            ObjectHandle, TypedObjectHandle,
        },
        Document, GlobalSettings,
    },
};
use rgb::{RGB, RGBA};

pub trait MaterialHandleExt<'a> {
    fn load_texture(&self, name: &str) -> Option<TextureHandle>;
}
impl<'a> MaterialHandleExt<'a> for MaterialHandle<'a> {
    fn load_texture(&self, name: &str) -> Option<TextureHandle> {
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

pub trait Loadable: Sized {
    fn get_property(properties: ObjectProperties, attribute: &str) -> anyhow::Result<Self>;
}
struct EnumLoader<T> {
    enum_name: &'static str,
    try_into: fn(i32) -> anyhow::Result<T>,
}
impl<T> EnumLoader<T> {
    fn new(enum_name: &'static str) -> Self
    where
        T: TryFrom<i32, Error = anyhow::Error>,
    {
        Self {
            enum_name,
            try_into: T::try_from,
        }
    }
}
impl<'a, T> LoadProperty<'a> for EnumLoader<T> {
    type Error = anyhow::Error;
    type Value = T;
    fn expecting(&self) -> String {
        self.enum_name.to_string()
    }
    fn load(self, node: &PropertyHandle<'a>) -> Result<Self::Value, Self::Error> {
        use anyhow::anyhow;
        let attributes = node.value_part();
        match attributes {
            [attribute, ..] => match attribute {
                AttributeValue::I32(value) => (self.try_into)(*value),
                attribute => Err(anyhow!(
                    "Was expecting a I32 attribute value when decoding {}, got {attribute:?}",
                    self.enum_name,
                )),
            },
            _ => Err(anyhow!(
                "There were no elements in the FBX attributes when decoding {}",
                self.enum_name,
            )),
        }
    }
}

// Part of the "FbxNode" native_typename
/// The `InheritType` property, equivalent to `EInheritType` in the FBX SDK.
///
/// This controls the order in which parent to child transformation is applied.
/// Note that "child" here is the node with the `InheritType` attribute.
///
/// - See also: http://docs.autodesk.com/FBX/2014/ENU/FBX-SDK-Documentation/cpp_ref/class_fbx_transform.html#a0affdd70d8df512d82fdb6a30112bf0c
#[derive(Copy, Clone, Default, Debug)]
pub enum InheritType {
    /// Parent Rotation → child rotation → Parent Scale → child scale (default)
    #[default]
    RrSs,
    /// Parent Rotation → Parent Scale → child rotation → child scale
    RSrs,
    /// Parent Rotation → child rotation → child scale
    Rrs,
}
impl TryFrom<i32> for InheritType {
    type Error = anyhow::Error;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        use InheritType::*;
        match value {
            0 => Ok(RrSs),
            1 => Ok(RSrs),
            2 => Ok(Rrs),
            i => Err(anyhow::anyhow!("{i} not in range of InheritType enum")),
        }
    }
}
// Part of the "FbxNode" native_typename
/// The order of rotation of the `Rotation` attributes.
///
/// Rotation in FBX is defined as a Vec3 of **degrees**
/// (not radians) of Tait-Bryan angles (commonly called Euler angles).
///
/// Note that for reasons unbeknownst, to translate this into the bevy equivalent,
/// the euler angles must be negated and the resulting matrix inverted.
#[allow(clippy::upper_case_acronyms)]
#[derive(Copy, Clone, Default, Debug)]
pub enum RotationOrder {
    #[default]
    XYZ,
    XZY,
    YZX,
    YXZ,
    ZXY,
    ZYX,
    // Not sure what that means.
    SphericXYZ,
}
impl From<RotationOrder> for EulerRot {
    fn from(ord: RotationOrder) -> Self {
        use EulerRot as Er;
        use RotationOrder as Ro;
        match ord {
            Ro::XYZ => Er::XYZ,
            Ro::XZY => Er::XZY,
            Ro::YZX => Er::YZX,
            Ro::YXZ => Er::YXZ,
            Ro::ZXY => Er::ZXY,
            Ro::ZYX => Er::ZYX,
            Ro::SphericXYZ => Er::XYZ,
        }
    }
}
impl TryFrom<i32> for RotationOrder {
    type Error = anyhow::Error;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        use RotationOrder::*;
        match value {
            0 => Ok(XYZ),
            1 => Ok(XZY),
            2 => Ok(YZX),
            3 => Ok(YXZ),
            4 => Ok(ZXY),
            5 => Ok(ZYX),
            6 => Ok(SphericXYZ),
            i => Err(anyhow::anyhow!("{i} not in range of RotationOrder enum")),
        }
    }
}

macro_rules! impl_loadable {
    ( $( $loader:expr => $target:ty ),* $(,)? ) => {
        $(
            impl_loadable!(@single $target, $loader );
        )*
    };
    (@single $target:ty, $loader:expr) => {
        impl Loadable for $target {
            fn get_property(properties: ObjectProperties, attribute: &str) -> anyhow::Result<Self> {
                let loader = $loader;
                let property= properties.get_property(attribute).ok_or_else(||
                    anyhow::anyhow!("no {attribute} in properties when decoding {}", stringify!($target))
                )?;
                Ok(loader.load(&property)?.into())
            }
        }
    };
}
impl_loadable!(
    RgbLoader::<RGB<f64>>::default() => RGB<f64>,
    RgbLoader::<RGB<f32>>::default() => RGB<f32>,
    RgbLoader::<RGBA<f64>>::default() => RGBA<f64>,
    RgbLoader::<RGBA<f32>>::default() => RGBA<f32>,
    PrimitiveLoader::<bool>::default() => bool,
    PrimitiveLoader::<f32>::default() => f32,
    PrimitiveLoader::<f64>::default() => f64,
    PrimitiveLoader::<i16>::default() => i16,
    PrimitiveLoader::<i32>::default() => i32,
    PrimitiveLoader::<i64>::default() => i64,
    PrimitiveLoader::<u16>::default() => u16,
    PrimitiveLoader::<u32>::default() => u32,
    PrimitiveLoader::<u64>::default() => u64,
    MintLoader::<Vector2<f32>>::default() => Vec2,
    MintLoader::<Vector2<f64>>::default() => DVec2,
    MintLoader::<Vector3<f32>>::default() => Vec3,
    MintLoader::<Vector3<f64>>::default() => DVec3,
    MintLoader::<Vector4<f32>>::default() => Vec4,
    MintLoader::<Vector4<f64>>::default() => DVec4,
    EnumLoader::<InheritType>::new("InheritType") => InheritType,
    EnumLoader::<RotationOrder>::new("RotationOrder") => EulerRot,
);

// TODO: additional useful fields in the Model node:
// - "Primary Visibility"
// - "Casts Shadows"
// - "Receive Shadows"
// - "Culling"
fn is_object_root(object: &ObjectHandle) -> bool {
    object
        .destination_objects()
        .any(|obj| obj.label().is_none() && obj.object_id().raw() == 0)
}

pub trait ModelTreeRootExt {
    fn model_roots(&self) -> Vec<ModelHandle<'_>>;
}
impl ModelTreeRootExt for Document {
    fn model_roots(&self) -> Vec<ModelHandle<'_>> {
        self.objects()
            .filter(is_object_root)
            .filter_map(|obj| match obj.get_typed() {
                TypedObjectHandle::Model(o) => Some(*o),
                _ => None,
            })
            .collect()
    }
}
