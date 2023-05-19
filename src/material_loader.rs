use crate::texture::Textures;
#[cfg(feature = "maya_3dsmax_pbr")]
use crate::utils::fbx_extend::*;

use bevy::{
    pbr::{AlphaMode, StandardMaterial},
    prelude::Color,
};
use fbxcel_dom::v7400::{data::material::ShadingModel, object::material::MaterialHandle};
use rgb::RGB;

/// Load materials from an FBX file.
///
/// Define your own to extend `bevy_mod_fbx`'s material loading capabilities.
#[derive(Clone, Copy)]
pub struct MaterialLoader {
    /// Create and return the bevy [`StandardMaterial`] based on the [`Handle<Image>`] loaded
    /// from the return value of `preprocess_textures`.
    pub with_textures: fn(MaterialHandle, Textures) -> Option<StandardMaterial>,
    pub name: &'static str,
}

const SPECULAR_TO_METALLIC_RATIO: f32 = 0.8;

/// Load Lambert/Phong materials, making minimal effort to convert them
/// into bevy's PBR material.
///
/// Note that the conversion has very poor fidelity, since Phong doesn't map well
/// to PBR.
pub const LOAD_LAMBERT_PHONG: MaterialLoader = MaterialLoader {
    name: "LOAD_LAMBERT_PHONG",
    with_textures: |material_obj, mut textures| {
        use AlphaMode::{Blend, Opaque};
        use ShadingModel::{Lambert, Phong};
        let properties = material_obj.properties();
        let shading_model = properties
            .shading_model_or_default()
            .unwrap_or(ShadingModel::Unknown);
        if !matches!(shading_model, Lambert | Phong) {
            return None;
        };
        let transparent = textures.get("TransparentColor");
        let is_transparent = transparent.is_some();
        let diffuse = transparent.or_else(|| textures.get("DiffuseColor"));
        let base_color = properties
            .diffuse_color_or_default()
            .map_or(Default::default(), ColorAdapter)
            .into();
        let specular = properties.specular_or_default().unwrap_or_default();
        let metallic = (specular.r + specular.g + specular.b) / 3.0;
        let metallic = metallic as f32 * SPECULAR_TO_METALLIC_RATIO;
        let roughness = properties
            .shininess()
            .ok()
            .flatten()
            .map_or(0.8, |s| (2.0 / (2.0 + s)).sqrt());
        Some(StandardMaterial {
            alpha_mode: if is_transparent { Blend } else { Opaque },
            // For bistro only
            // alpha_mode: AlphaMode::Mask(0.8),
            base_color,
            metallic,
            perceptual_roughness: roughness as f32,
            emissive_texture: textures.get("EmissiveColor"),
            base_color_texture: diffuse,
            normal_map_texture: textures.get("NormalMap"),
            flip_normal_map_y: true,
            ..Default::default()
        })
    },
};

/// The default material if all else fails.
///
/// Picks up the non-texture material values if possible,
/// otherwise it will just look like white clay.
pub const LOAD_FALLBACK: MaterialLoader = MaterialLoader {
    name: "LOAD_FALLBACK",
    with_textures: |material_obj, _| {
        let properties = material_obj.properties();
        let base_color = properties
            .diffuse_color()
            .ok()
            .flatten()
            .map(|c| ColorAdapter(c).into())
            .unwrap_or(Color::PINK);
        let metallic = properties
            .specular()
            .ok()
            .flatten()
            .map(|specular| (specular.r + specular.g + specular.b) / 3.0)
            .map(|metallic| metallic as f32 * SPECULAR_TO_METALLIC_RATIO)
            .unwrap_or(0.2);
        let roughness = properties
            .shininess()
            .ok()
            .flatten()
            .map_or(0.8, |s| (2.0 / (2.0 + s)).sqrt());
        Some(StandardMaterial {
            base_color,
            perceptual_roughness: roughness as f32,
            alpha_mode: AlphaMode::Opaque,
            metallic,
            ..Default::default()
        })
    },
};

#[cfg(feature = "maya_3dsmax_pbr")]
mod maya_consts {
    pub const PBR_TYPE_ID: i32 = 1166017;
    pub const DEFAULT_ROUGHNESS: f32 = 0.089;
    pub const DEFAULT_METALIC: f32 = 0.01;
}

// Note that it's impossible to enable the `maya_pbr` feature right now.
/// Load Maya's PBR material FBX extension.
///
/// This doesn't preserve environment maps or fresnel LUT,
/// since bevy's PBR currently doesn't support environment maps.
///
/// This loader is only available if the `maya_pbr` feature is enabled.
#[cfg(feature = "maya_3dsmax_pbr")]
pub const LOAD_MAYA_PBR: MaterialLoader = MaterialLoader {
    name: "LOAD_MAYA_PBR",
    with_textures: |handle, mut textures| {
        // return early if we detect this material is not Maya's PBR material
        let mat_maya_type = handle.get_i32("Maya|TypeId");
        if mat_maya_type != Some(maya_consts::PBR_TYPE_ID) {
            return None;
        }
        let lerp = |from, to, stride| from + (to - from) * stride;
        // Maya has fields that tells how much of the texture should be
        // used in the final computation of the value vs the baseline value.
        // We set the `metallic` and `perceptual_roughness` to
        // lerp(baseline_value, fully_texture_value, use_map)
        // so if `use_map` is 1.0, only the texture pixel counts,
        // while if it is 0.0, only the baseline count, and anything inbetween
        // is a mix of the two.
        let use_texture = 0.0;
        let use_metallic = handle
            .get_f32("Maya|use_metallic_map")
            .unwrap_or(use_texture);
        let use_roughness = handle
            .get_f32("Maya|use_roughness_map")
            .unwrap_or(use_texture);
        let metallic = handle
            .get_f32("Maya|metallic")
            .unwrap_or(maya_consts::DEFAULT_METALIC);
        let roughness = handle
            .get_f32("Maya|roughness")
            .unwrap_or(maya_consts::DEFAULT_ROUGHNESS);
        Some(StandardMaterial {
            flip_normal_map_y: true,
            base_color_texture: textures.get("Maya|TEX_color_map"),
            normal_map_texture: textures.get("Maya|TEX_normal_map"),
            metallic: lerp(metallic, 1.0, use_metallic),
            perceptual_roughness: lerp(roughness, 1.0, use_roughness),
            occlusion_texture: textures.get("Maya|TEX_ao_map"),
            emissive_texture: textures.get("Maya|TEX_emissive_map"),
            alpha_mode: AlphaMode::Opaque,
            ..Default::default()
        })
    },
};

/// The default fbx material loaders.
///
/// If you don't provide your own in the [`FbxMaterialLoaders`] resource,
/// the ones declared in this will be used instead.
///
/// You can also use thise function if you want to add your own loaders
/// and still want to fallback to the default ones.
///
/// [`FbxMaterialLoaders`]: crate::FbxMaterialLoaders
pub const fn default_loader_order() -> &'static [MaterialLoader] {
    &[
        #[cfg(feature = "maya_3dsmax_pbr")]
        LOAD_MAYA_PBR,
        LOAD_LAMBERT_PHONG,
        LOAD_FALLBACK,
    ]
}

#[derive(Default)]
struct ColorAdapter(RGB<f64>);
impl From<ColorAdapter> for Color {
    fn from(ColorAdapter(rgb): ColorAdapter) -> Self {
        Color::rgb(rgb.r as f32, rgb.g as f32, rgb.b as f32)
    }
}
