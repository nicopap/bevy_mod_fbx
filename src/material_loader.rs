use bevy::{
    pbr::{AlphaMode, StandardMaterial},
    prelude::{Color, Handle, Image},
    utils::HashMap,
};
use fbxcel_dom::v7400::{data::material::ShadingModel, object::material::MaterialHandle};
use rgb::RGB;

/// Load materials from an FBX file.
///
/// Define your own to extend `bevy_fbx`'s material loading capabilities.
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
            .unwrap_or(Color::WHITE);
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

// Note that it's impossible to enable the `maya_pbr` feature right now.
/// Load Maya's PBR material FBX extension.
///
/// This doesn't preserve environment maps or fresnel LUT,
/// since bevy's PBR currently doesn't support environment maps.
///
/// This loader is only available if the `maya_pbr` feature is enabled.
#[cfg(feature = "maya_pbr")]
pub const LOAD_MAYA_PBR: MaterialLoader = MaterialLoader {
    static_load: &[
        "NormalMap",
        "SpecularColor",
        "EmissiveColor",
        "DiffuseColor",
        "TransparentColor",
    ],
    dynamic_load: &[],
    preprocess_textures: |_, _images| {},
    with_textures: |_, _textures| None,
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
        #[cfg(feature = "maya_pbr")]
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
