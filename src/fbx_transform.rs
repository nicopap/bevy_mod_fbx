//! Handles FBX transform propagation, and translation
//! into bevy Transform and GlobalTransform.

// a bit of trivia on how transform is encoded in FBX:
// rotation: EulerXYZ in degrees, three steps: PreRotation, Lcl Rotation, PostRotation
// https://forums.autodesk.com/t5/fbx-forum/maya-quot-rotate-axis-quot-vs-fbx-quot-postrotation-quot/td-p/4168814
// https://help.autodesk.com/cloudhelp/2017/ENU/FBX-Developer-Help/files/GUID-C35D98CB-5148-4B46-82D1-51077D8970EE.htm
// http://docs.autodesk.com/FBX/2014/ENU/FBX-SDK-Documentation/cpp_ref/class_fbx_node.html
// https://help.autodesk.com/cloudhelp/2016/ENU/FBX-Developer-Help/cpp_ref/_view_scene_2_draw_scene_8cxx-example.html#a78
// (see "Pivot Management" section for last link)
//
// The following code is somewhat taken from:
// https://stackoverflow.com/questions/34452946/how-can-i-get-the-correct-position-of-fbx-mesh
// which itself is taken from:
// https://help.autodesk.com/cloudhelp/2016/ENU/FBX-Developer-Help/cpp_ref/_transformations_2main_8cxx-example.html#a30
//
// FbxNode also has {Rotation,Scaling,Translation}Active propreties,
// what is their meaning?
// - https://forums.autodesk.com/t5/fbx-forum/rotationactive/td-p/4267206
// - https://help.autodesk.com/cloudhelp/2016/ENU/FBX-Developer-Help/cpp_ref/class_fbx_node.html
use std::f32::consts::TAU;

use anyhow::Result;
use bevy::math::{DVec3, EulerRot};
use bevy::prelude::{Mat4, Transform, Vec3};
use fbxcel_dom::v7400::object::{model::ModelHandle, property::ObjectProperties, ObjectHandle};

use crate::utils::fbx_extend::{InheritType, Loadable};

#[derive(Copy, Clone, Debug)]
struct Translation(Vec3);
impl Translation {
    fn mat(&self) -> Mat4 {
        Mat4::from_translation(self.0)
    }
    fn from_double(p: DVec3) -> Translation {
        Self(p.as_vec3())
    }
}

// FBX encodes rotations in Eulers (customizable order) in degrees,
// and for some reasons, it needs to be negated and then inverted.
#[derive(Copy, Clone, Debug)]
struct Rotation(Vec3, EulerRot);
impl Rotation {
    fn from_euler(euler: EulerRot, angles: DVec3) -> Self {
        Rotation(angles.as_vec3() * -(TAU / 360.0), euler)
    }
    fn mat(&self) -> Mat4 {
        let Vec3 { x, y, z } = self.0;
        Mat4::from_euler(self.1, x, y, z).inverse()
    }
}

#[derive(Copy, Clone, Debug)]
struct Scale(Vec3);
impl Scale {
    fn mat(&self) -> Mat4 {
        Mat4::from_scale(self.0)
    }
    fn from_double(p: DVec3) -> Scale {
        Self(p.as_vec3())
    }
    const IDENTITY: Self = Self(Vec3::ONE);
}

#[derive(Clone, Debug)]
struct NodeScale {
    pivot: Translation,
    offset: Translation,
    local: Scale,
}
#[derive(Clone, Debug)]
struct NodeRotation {
    pivot: Translation,
    offset: Translation,
    local: Rotation,
    pre: Rotation,
    post: Rotation,
}
/// Handle the awkward translation from FBX to Bevy transform.
///
/// The transform propagation in FBX is _way too flexible_,
/// and doesn't translate well to bevy's simple global/matrix+local/TRS.
/// So it's necessary to compute the actual global position
/// based on the FBX formula and do a second pass
/// where we set the local transform infered
/// from the computed FBX global position.
#[derive(Clone, Debug)]
struct FbxNodeTransformInfo {
    rotation: NodeRotation,
    translation: Translation,
    scale: NodeScale,
    inherit_type: InheritType,
}
impl FbxNodeTransformInfo {
    // if you were wondering: "Lcl" stands for "Local"
    // FIXME: Non-zero {Rotation,Scaling}{Pivot,Offset} is untested.
    // TODO: Geometric{Translation,Scaling,Rotation}
    // (see docs.autodesk.com and stackoverflow.com links at top of this file)
    fn from_object(object: ObjectHandle) -> Result<Self> {
        fn load<T: Loadable>(p: ObjectProperties, attribute: &str) -> Result<T> {
            T::get_property(p, attribute)
        }
        let p = object.properties_by_native_typename("FbxNode");
        let e = load(p, "RotationOrder")?;
        Ok(FbxNodeTransformInfo {
            rotation: NodeRotation {
                pivot: Translation::from_double(load(p, "RotationPivot")?),
                offset: Translation::from_double(load(p, "RotationOffset")?),
                local: Rotation::from_euler(e, load(p, "Lcl Rotation")?),
                pre: Rotation::from_euler(e, load(p, "PreRotation")?),
                post: Rotation::from_euler(e, load(p, "PostRotation")?),
            },
            translation: Translation::from_double(load(p, "Lcl Translation")?),
            scale: NodeScale {
                pivot: Translation::from_double(load(p, "ScalingPivot")?),
                offset: Translation::from_double(load(p, "ScalingOffset")?),
                local: Scale::from_double(load(p, "Lcl Scaling")?),
            },
            inherit_type: load(p, "InheritType")?,
        })
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct LocalScale(Scale);

// This is similar to mat.to_scale_rotation_translation()
// but takes into account shear operations (meaning: rotation followed by non-uniform scale)
// The implementation is the one used in the Autodesk scene translation example file.
fn get_reverse_transform(mat: Mat4) -> (Mat4, Mat4, Mat4) {
    let mat_q = Mat4::from_quat;
    let mat_t = Mat4::from_translation;
    let (_, rotation, translation) = mat.to_scale_rotation_translation();
    let rotation = mat_q(rotation);
    let translation = mat_t(translation);
    let shear_scale = mat * rotation.inverse() * translation.inverse();
    (shear_scale, rotation, translation)
}

// NOTE: there were no thought about performance put into this,
// the goal of this method is to get something working ASAP,
// performance can wait.
// I particularly dislike the amount of matrix inversion and multiplication this incures.
fn global_transform(node: FbxNodeTransformInfo, parent: Option<FbxTransform>) -> Mat4 {
    let mat_t = Mat4::from_translation;
    let rot = node.rotation;
    let scale = node.scale;
    let rotation = rot.pre.mat() * rot.local.mat() * rot.post.mat();

    let FbxTransform {
        global: parent_transform,
        local_scale: local_parent_scale,
    } = parent.unwrap_or_default();

    let (parent_shear_scale, parent_rotation, parent_translation) =
        get_reverse_transform(parent_transform);
    let parent_nonlocal_scale = parent_shear_scale * local_parent_scale.mat().inverse();

    let inherited_rot_scale = match node.inherit_type {
        InheritType::RrSs => parent_rotation * rotation * parent_shear_scale * scale.local.mat(),
        InheritType::RSrs => parent_rotation * parent_shear_scale * rotation * scale.local.mat(),
        InheritType::Rrs => parent_rotation * rotation * parent_nonlocal_scale * scale.local.mat(),
    };
    let with_off_piv = |offset: Translation, pivot: Translation, transform| {
        offset.mat() * pivot.mat() * transform * pivot.mat().inverse()
    };
    let translation = node.translation.mat()
        * with_off_piv(rot.offset, rot.pivot, rotation)
        * with_off_piv(scale.offset, scale.pivot, scale.local.mat());
    let translation = translation.to_scale_rotation_translation().2;
    // NOTE: this is unlike the Autodesk resource provided on top, it seems
    // we need to remove the scale component from the parent's global matrix
    // we multiply the translation with. Absolutely no idea why, but it works.
    let parent_non_scale_transform = parent_translation * parent_rotation;
    let global_translation = parent_non_scale_transform.transform_vector3(translation);
    mat_t(global_translation) * inherited_rot_scale
}

/// Fbx global transform, including parent local scale to compute
/// the children's transform.
#[derive(Debug, Clone, Copy)]
pub(crate) struct FbxTransform {
    local_scale: Scale,
    pub(crate) global: Mat4,
}
impl Default for FbxTransform {
    fn default() -> Self {
        FbxTransform {
            local_scale: Scale::IDENTITY,
            global: Mat4::IDENTITY,
        }
    }
}
impl FbxTransform {
    pub(crate) fn from_node(node: ModelHandle, parent: Option<FbxTransform>) -> Self {
        let transform = FbxNodeTransformInfo::from_object(*node).unwrap();
        FbxTransform::from_fbxtrans(transform, parent)
    }
    fn from_fbxtrans(trans: FbxNodeTransformInfo, parent: Option<FbxTransform>) -> Self {
        FbxTransform {
            local_scale: trans.scale.local,
            global: global_transform(trans, parent),
        }
    }
    // Problem: `Self` is the _global_ position of fbx node, not local.
    // An FBX local transform can't be translated directly into a bevy Transform,
    // so we need to compute the global transform and work backward from there.
    // 1. we have parent(FbxGlobalTransform)
    // 2. we just computed child(FbxGlobalTransform)
    // 3. GlobalTransform = FbxGlobalTransform
    // 4. We need to find the child(Transform):
    //    - from bevy's transform mat: child(GlobalTransform) = parent(GlobalTransform) * child(Transform)
    //    - We have: child(GlobalTransform) and parent(GlobalTransform)
    //    - child(Transform) = child(GlobalTransform) * parent(GlobalTransform)¯¹
    pub(crate) fn as_local_transform(&self, parent: Option<Mat4>) -> Transform {
        let mat = if let Some(parent) = parent {
            self.global * parent.inverse()
        } else {
            self.global
        };
        Transform::from_matrix(mat)
    }
}
