#[cfg(feature = "maya_3dsmax_pbr")]
use crate::utils::fbx_extend::*;

use bevy::{
    pbr::{AlphaMode, StandardMaterial},
    prelude::{Color, Handle, Image},
    utils::HashMap,
};
use fbxcel_dom::v7400::{data::material::ShadingModel, object::material::MaterialHandle};
use rgb::RGB;

/// Load materials from an FBX file.
///
/// Define your own to extend `bevy_mod_fbx`'s material loading capabilities.
#[derive(Clone, Copy)]
pub struct MaterialLoader {
    /// The FBX texture field name used by the material you are loading.
    ///
    /// Textures declared here are directly passed to `with_textures` without modification,
    /// this enables caching and re-using textures without re-reading the files
    /// multiple times over.
    ///
    /// They are loaded by the [`FbxLoader`] and provided to the other functions
    /// defined in the rest of this struct, associated with their names in a `HashMap`.
    ///
    /// [`FbxLoader`]: crate::FbxLoader
    pub static_load: &'static [&'static str],

    /// The FBX texture field name used by textures you wish to transform.
    ///
    /// Textures declared here are passed to `preprocess_textures` for further
    /// processing, enabling preprocessing.
    ///
    /// They are loaded by the [`FbxLoader`] and provided to the other functions
    /// defined in the rest of this struct, associated with their names in a `HashMap`.
    ///
    /// [`FbxLoader`]: crate::FbxLoader
    pub dynamic_load: &'static [&'static str],

    /// Run some math on the loaded textures, handy if you have to convert between texture
    /// formats or swap color channels.
    ///
    /// To update, remove or add textures, return the `HashMap` with the new values.
    ///
    /// The `Image`s are then added to the asset store (`Assets<Image>`) and a handle
    /// to them is passed to `with_textures` in additions to the handles of the textures
    /// declared in the `static_load` field.
    pub preprocess_textures: fn(MaterialHandle, &mut HashMap<&'static str, Image>),

    /// Create and return the bevy [`StandardMaterial`] based on the [`Handle<Image>`] loaded
    /// from the return value of `preprocess_textures`.
    pub with_textures:
        fn(MaterialHandle, HashMap<&'static str, Handle<Image>>) -> Option<StandardMaterial>,
}

const SPECULAR_TO_METALLIC_RATIO: f32 = 0.8;

/// Load Lambert/Phong materials, making minimal effort to convert them
/// into bevy's PBR material.
///
/// Note that the conversion has very poor fidelity, since Phong doesn't map well
/// to PBR.
pub const LOAD_LAMBERT_PHONG: MaterialLoader = MaterialLoader {
    static_load: &[
        "NormalMap",
        "EmissiveColor",
        "DiffuseColor",
        "TransparentColor",
    ],
    dynamic_load: &[],
    preprocess_textures: |_, _| {},
    with_textures: |material_obj, textures| {
        use AlphaMode::{Blend, Opaque};
        use ShadingModel::{Lambert, Phong};
        let properties = material_obj.properties();
        let shading_model = properties
            .shading_model_or_default()
            .unwrap_or(ShadingModel::Unknown);
        if !matches!(shading_model, Lambert | Phong) {
            return None;
        };
        let transparent = textures.get("TransparentColor").cloned();
        let is_transparent = transparent.is_some();
        let diffuse = transparent.or_else(|| textures.get("DiffuseColor").cloned());
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
            emissive_texture: textures.get("EmissiveColor").cloned(),
            base_color_texture: diffuse,
            normal_map_texture: textures.get("NormalMap").cloned(),
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
    static_load: &[],
    dynamic_load: &[],
    preprocess_textures: |_, _| {},
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
    static_load: &[
        "Maya|TEX_normal_map",
        "Maya|TEX_color_map",
        "Maya|TEX_ao_map",
        "Maya|TEX_emissive_map",
    ],
    dynamic_load: &["Maya|TEX_metallic_map", "Maya|TEX_roughness_map"],
    // FIXME: this assumes both metallic map and roughness map
    // are encoded in texture formats that can be stored as
    // a byte array in CPU memory.
    // This is not the case for compressed formats such as KTX or DDS
    // FIXME: this also assumes the texture channels are 8 bit.
    preprocess_textures: |material_handle, images| {
        use bevy::render::render_resource::{TextureDimension::D2, TextureFormat::Rgba8UnormSrgb};
        let mut run = || {
            // return early if we detect this material is not Maya's PBR material
            let mat_maya_type = material_handle.get_i32("Maya|TypeId")?;
            if mat_maya_type != maya_consts::PBR_TYPE_ID {
                return None;
            }
            let combine_colors = |colors: &[u8]| match colors {
                // Only one channel is necessary for the metallic and roughness
                // maps. If we assume the texture is greyscale, we can take any
                // channel (R, G, B) and assume it's approximately the value we want.
                &[bw, ..] => bw,
                _ => unreachable!("A texture must at least have a single channel"),
            };
            // Merge the metallic and roughness map textures into one,
            // following the GlTF standard for PBR textures.
            // The resulting texture should have:
            // - Green channel set to roughness
            // - Blue channel set to metallic
            let metallic = images.remove("Maya|TEX_metallic_map")?;
            let rough = images.remove("Maya|TEX_roughness_map")?;
            let image_size = metallic.texture_descriptor.size;
            let metallic_components =
                metallic.texture_descriptor.format.describe().components as usize;
            let rough_components = rough.texture_descriptor.format.describe().components as usize;
            let metallic_rough: Vec<_> = metallic
                .data
                .chunks(metallic_components)
                .zip(rough.data.chunks(rough_components))
                .flat_map(|(metallic, rough)| {
                    [0, combine_colors(rough), combine_colors(metallic), 255]
                })
                .collect();
            let metallic_rough = Image::new(image_size, D2, metallic_rough, Rgba8UnormSrgb);
            images.insert("Metallic_Roughness", metallic_rough);
            Some(())
        };
        run();
    },
    with_textures: |handle, textures| {
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
        let has_rm_texture = textures.contains_key("Metallic_Roughness");
        let use_texture = if has_rm_texture { 1.0 } else { 0.0 };
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
            base_color_texture: textures.get("Maya|TEX_color_map").cloned(),
            normal_map_texture: textures.get("Maya|TEX_normal_map").cloned(),
            metallic_roughness_texture: textures.get("Metallic_Roughness").cloned(),
            metallic: lerp(metallic, 1.0, use_metallic),
            perceptual_roughness: lerp(roughness, 1.0, use_roughness),
            occlusion_texture: textures.get("Maya|TEX_ao_map").cloned(),
            emissive_texture: textures.get("Maya|TEX_emissive_map").cloned(),
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
