//! Collection of temporary extensions to the fbxcell_dom types
//! until they are merged upstream.
use bevy::math::{Quat, Vec3};

use fbxcel_dom::{
    fbxcel::{low::v7400::AttributeValue, tree::v7400::NodeHandle},
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

// a bit of trivia on how transform is encoded in FBX:
// rotation: EulerXYZ in degrees, three steps: PreRotation, Lcl Rotation, PostRotation
// the other: probably mirrors rotation and all have a Pre, Lcl and Post version
// https://forums.autodesk.com/t5/fbx-forum/maya-quot-rotate-axis-quot-vs-fbx-quot-postrotation-quot/td-p/4168814
// https://help.autodesk.com/cloudhelp/2017/ENU/FBX-Developer-Help/files/GUID-C35D98CB-5148-4B46-82D1-51077D8970EE.htm
// http://docs.autodesk.com/FBX/2014/ENU/FBX-SDK-Documentation/cpp_ref/class_fbx_node.html
// (see "Pivot Management" section for last link)
pub trait NodeHandleTransformExt<'a> {
    fn get_vec3(&self, name: &str) -> Option<Vec3>;
    fn rotation(&self) -> Quat;
    fn scale(&self) -> Vec3;
    fn translation(&self) -> Vec3;
}

// Add `unwrap_or{_default}` to all elements of
// an expression of the form foo $op bar $op baz ...
// where `$op` is a math operator such as +, -, *
macro_rules! op_or {
    ( default $op:tt ($head:expr $( , $tail:expr )+) ) => {
        $head.unwrap_or_default() $op op_or!(default $op ( $($tail),* ))
    };
    ( default $op:tt ($head:expr) ) => {
        $head.unwrap_or_default()
    };
    ( ($default:expr) $op:tt ($head:expr $( , $tail:expr )+) ) => {
        $head.unwrap_or($default) $op op_or!(($default) $op ( $($tail),* ))
    };
    ( ($default:expr) $op:tt ($head:expr) ) => {
        $head.unwrap_or($default)
    };
}
// TODO: additional useful fields in the Model node:
// - "Primary Visibility"
// - "Casts Shadows"
// - "Receive Shadows"
// - "Culling"
// - "Shading" (seems to always be false though?)
// TODO: probably need to impl that on ObjectHandle
// so that it's possible to use `properties_by_native_typename`
// so that defaults are used when value absent.
//
// Also note that "Lcl" stands for "Local"
impl<'a> NodeHandleTransformExt<'a> for NodeHandle<'a> {
    fn get_vec3(&self, requested: &str) -> Option<Vec3> {
        use AttributeValue::{String as Str, F64};
        let prop = self.first_child_by_name("Properties70")?;
        let attributes = prop.children().find_map(|c| {
            let attributes = c.attributes();
            let is_requested = attributes.first() == Some(&Str(requested.to_owned()));
            is_requested.then(|| attributes)
        })?;
        match attributes {
            &[.., F64(x), F64(y), F64(z)] => Some(Vec3::new(x as f32, y as f32, z as f32)),
            _ => None,
        }
    }
    // The formula is: Lcl * Pre * Post^-1
    fn rotation(&self) -> Quat {
        use bevy::math::EulerRot::XYZ;
        let to_quat = |Vec3 { x, y, z }| {
            Quat::from_euler(XYZ, x.to_radians(), y.to_radians(), z.to_radians())
        };

        let pre = self.get_vec3("PreRotation").map(to_quat);
        let value = self.get_vec3("Lcl Rotation").map(to_quat);
        let post = self
            .get_vec3("PostRotation")
            .map(to_quat)
            .map(Quat::inverse);

        op_or!(default * (value, pre, post))
    }
    // The formula is: Lcl * Pre * Post^-1
    fn scale(&self) -> Vec3 {
        let pre = self.get_vec3("PreScaling");
        let value = self.get_vec3("Lcl Scaling");
        let post = self.get_vec3("PostScaling").map(|v| 1.0 / v);

        op_or!((Vec3::ONE) * (value, pre, post))
    }
    // The formula is: Lcl + Pre + Post
    // (actually not sure "Pre" and "Post" fields exist for translation)
    fn translation(&self) -> Vec3 {
        let pre = self.get_vec3("PreTranslation");
        let value = self.get_vec3("Lcl Translation");
        let post = self.get_vec3("PostTranslation");

        op_or!(default + (value, pre, post))
    }
}
