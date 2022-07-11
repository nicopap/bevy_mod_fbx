//! FBX v7400 support.

use std::{collections::HashMap, path::Path};

use anyhow::{anyhow, bail, Context};
use bevy_asset::{AssetLoader, Handle, LoadContext, LoadedAsset};
use bevy_ecs::prelude::World;
use bevy_hierarchy::BuildWorldChildren;
use bevy_log::{debug, error, trace};
use bevy_math::{DVec2, DVec3};
use bevy_pbr::PbrBundle;
use bevy_render::mesh::{Indices, Mesh as BevyMesh, PrimitiveTopology, VertexAttributeValues};
use bevy_transform::TransformBundle;
// use cgmath::{Point2, Point3, Vector3};
use fbxcel_dom::{
    any::AnyDocument,
    v7400::{
        data::{
            material::ShadingModel, mesh::layer::TypedLayerElementHandle,
            texture::WrapMode as RawWrapMode,
        },
        object::{self, model::TypedModelHandle, ObjectId, TypedObjectHandle},
        Document,
    },
};
use rgb::ComponentMap;

use crate::data::{
    material::{LambertData, Material, ShadingData},
    mesh::Mesh as FbxMesh,
    scene::{MaterialIndex, Scene, TextureIndex},
    texture::{Texture, WrapMode},
};

use crate::triangulator;

// TODO: multiple scenes
pub struct Loader<'b, 'w> {
    scene: Scene,
    material_indices: HashMap<ObjectId, MaterialIndex>,
    texture_indices: HashMap<ObjectId, TextureIndex>,
    load_context: &'b mut LoadContext<'w>,
}

#[derive(Default)]
pub struct FbxLoader;
impl AssetLoader for FbxLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> bevy_asset::BoxedFuture<'a, anyhow::Result<()>> {
        Box::pin(async move {
            let cursor = std::io::Cursor::new(bytes);
            let reader = std::io::BufReader::new(cursor);
            let maybe_doc =
                AnyDocument::from_seekable_reader(reader).expect("Failed to load document");
            if let AnyDocument::V7400(_ver, doc) = maybe_doc {
                let loader = Loader::new(load_context);
                let potential_error = loader
                    .load_scene(*doc)
                    .with_context(|| format!("failed to load {:?}", load_context.path()));
                match potential_error {
                    Err(err) => {
                        error!("{err:?}");
                        Ok(())
                    }
                    Ok(()) => Ok(()),
                }
            } else {
                Err(anyhow!("TODO: better error handling in fbx loader"))
            }
        })
    }
    fn extensions(&self) -> &[&str] {
        &["fbx"]
    }
}

impl<'b, 'w> Loader<'b, 'w> {
    /// Creates a new `Loader`.
    fn new(load_context: &'b mut LoadContext<'w>) -> Self {
        Self {
            scene: Default::default(),
            material_indices: Default::default(),
            texture_indices: Default::default(),
            load_context,
        }
    }

    fn load_scene(self, doc: Document) -> anyhow::Result<()> {
        let scene = self.load(doc)?;
        let mut world = World::default();
        world
            .spawn()
            .insert_bundle(TransformBundle::identity())
            .with_children(|parent| {
                for mesh in &scene.bevy_meshes {
                    parent.spawn_bundle(PbrBundle {
                        mesh: mesh.clone(),
                        ..Default::default()
                    });
                }
            });
        Ok(())
    }

    /// Loads the document.
    fn load(mut self, doc: Document) -> anyhow::Result<Scene> {
        for obj in doc.objects() {
            if let TypedObjectHandle::Model(TypedModelHandle::Mesh(mesh)) = obj.get_typed() {
                self.load_mesh(mesh)?;
            }
        }

        Ok(self.scene)
    }

    /// Loads the geometry.
    fn load_bevy_mesh(
        &mut self,
        mesh_obj: object::geometry::MeshHandle,
        num_materials: usize,
    ) -> anyhow::Result<Handle<BevyMesh>> {
        debug!("Loading geometry mesh: {:?}", mesh_obj);

        let polygon_vertices = mesh_obj
            .polygon_vertices()
            .context("Failed to get polygon vertices")?;
        let triangle_pvi_indices = polygon_vertices
            .triangulate_each(triangulator::triangulate::triangulate)
            .context("Triangulation failed")?;

        // TODO this seems to duplicate vertices from neighboring triangles. We shouldn't
        // do that and instead set the indice attribute of the BevyMesh properly.
        let get_position = |pos: Option<_>| -> Result<_, anyhow::Error> {
            let cpi = pos.ok_or_else(|| anyhow!("Failed to get control point index"))?;
            let point = polygon_vertices
                .control_point(cpi)
                .ok_or_else(|| anyhow!("Failed to get control point: cpi={:?}", cpi))?;
            // TODO: probably a better conversion method here XD
            Ok(DVec3::from(point).as_vec3().into())
        };
        let positions = triangle_pvi_indices
            .iter_control_point_indices()
            .map(get_position)
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to reconstruct position vertices")?;
        trace!("Expanded positions len: {:?}", positions.len());

        let layer = mesh_obj
            .layers()
            .next()
            .ok_or_else(|| anyhow!("Failed to get layer"))?;

        let normals = {
            let normals = layer
                .layer_element_entries()
                .find_map(|entry| match entry.typed_layer_element() {
                    Ok(TypedLayerElementHandle::Normal(handle)) => Some(handle),
                    _ => None,
                })
                .ok_or_else(|| anyhow!("Failed to get normals"))?
                .normals()
                .context("Failed to get normals")?;
            let get_indices = |tri_vi| -> Result<_, anyhow::Error> {
                let v = normals.normal(&triangle_pvi_indices, tri_vi)?;
                // TODO: probably a better conversion method here XD
                Ok(DVec3::from(v).as_vec3().into())
            };
            triangle_pvi_indices
                .triangle_vertex_indices()
                .map(get_indices)
                .collect::<Result<Vec<_>, _>>()
                .context("Failed to reconstruct normals vertices")?
        };
        let uv = {
            let uv = layer
                .layer_element_entries()
                .find_map(|entry| match entry.typed_layer_element() {
                    Ok(TypedLayerElementHandle::Uv(handle)) => Some(handle),
                    _ => None,
                })
                .ok_or_else(|| anyhow!("Failed to get UV"))?
                .uv()?;
            let get_indices = |tri_vi| -> Result<_, anyhow::Error> {
                let v = uv.uv(&triangle_pvi_indices, tri_vi)?;
                // TODO: probably a better conversion method here XD
                Ok(DVec2::from(v).as_vec2().into())
            };
            triangle_pvi_indices
                .triangle_vertex_indices()
                .map(get_indices)
                .collect::<Result<Vec<_>, _>>()
                .context("Failed to reconstruct UV vertices")?
        };

        let _indices_per_material = {
            let mut indices_per_material = vec![Vec::new(); num_materials];
            let materials = layer
                .layer_element_entries()
                .find_map(|entry| match entry.typed_layer_element() {
                    Ok(TypedLayerElementHandle::Material(handle)) => Some(handle),
                    _ => None,
                })
                .ok_or_else(|| anyhow!("Materials not found for mesh {:?}", mesh_obj))?
                .materials()
                .context("Failed to get materials")?;
            for tri_vi in triangle_pvi_indices.triangle_vertex_indices() {
                let local_material_index = materials
                    .material_index(&triangle_pvi_indices, tri_vi)
                    .context("Failed to get mesh-local material index")?
                    .to_u32();
                indices_per_material
                    .get_mut(local_material_index as usize)
                    .ok_or_else(|| {
                        anyhow!(
                            "FbxMesh-local material index out of range: num_materials={:?}, got={:?}",
                            num_materials,
                            local_material_index
                        )
                    })?
                    .push(tri_vi.to_usize() as u32);
            }
            indices_per_material
        };

        let indices: Vec<_> = triangle_pvi_indices
            .iter_control_point_indices()
            .flatten()
            .map(|t| t.to_u32())
            .collect();
        if uv.len() != positions.len() || uv.len() != normals.len() || uv.len() != indices.len() {
            panic!(
                "mismatched length of buffers: pos{} uv{} normals{} indices{}",
                positions.len(),
                uv.len(),
                normals.len(),
                indices.len()
            );
        }

        let mut mesh = BevyMesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(
            BevyMesh::ATTRIBUTE_POSITION,
            VertexAttributeValues::Float32x3(positions),
        );
        mesh.insert_attribute(
            BevyMesh::ATTRIBUTE_UV_0,
            VertexAttributeValues::Float32x2(uv),
        );
        mesh.insert_attribute(
            BevyMesh::ATTRIBUTE_NORMAL,
            VertexAttributeValues::Float32x3(normals),
        );
        mesh.set_indices(Some(Indices::U32(indices)));
        // TODO: generate tangents in bevy 0.8
        // mesh.generate_tangents()?;

        debug!("Successfully loaded geometry mesh: {:?}", mesh_obj);

        let label = if let Some(name) = mesh_obj.name() {
            format!("FbxMesh@{name}/Primitive")
        } else {
            format!("FbxMesh{}/Primitive", mesh_obj.object_id().raw())
        };
        Ok(self
            .load_context
            .set_labeled_asset(&label, LoadedAsset::new(mesh)))
    }

    /// Loads the mesh.
    fn load_mesh(
        &mut self,
        mesh_obj: object::model::MeshHandle,
    ) -> anyhow::Result<Handle<FbxMesh>> {
        let label = if let Some(name) = mesh_obj.name() {
            format!("FbxMesh@{name}")
        } else {
            format!("FbxMesh{}", mesh_obj.object_id().raw())
        };

        debug!("Loading mesh: {label}");

        let bevy_obj = mesh_obj.geometry().context("Failed to get geometry")?;

        let materials = mesh_obj
            .materials()
            .map(|material_obj| self.load_material(material_obj))
            .collect::<anyhow::Result<Vec<_>>>()
            .context("Failed to load materials for mesh")?;

        let mesh_handle = self
            .load_bevy_mesh(bevy_obj, materials.len())
            .context("Failed to load geometry mesh")?;

        let mesh = FbxMesh {
            name: mesh_obj.name().map(Into::into),
            bevy_mesh_handle: mesh_handle,
            materials,
        };

        let mesh_handle = self
            .load_context
            .set_labeled_asset(&label, LoadedAsset::new(mesh));
        debug!("Successfully loaded mesh: {label}");

        self.scene.add_mesh(mesh_handle.clone());
        Ok(mesh_handle)
    }

    // unused code, currently material not supported
    /// Loads the texture.
    fn load_texture(
        &mut self,
        texture_obj: object::texture::TextureHandle,
        transparent: bool,
    ) -> anyhow::Result<TextureIndex> {
        if let Some(index) = self.texture_indices.get(&texture_obj.object_id()) {
            return Ok(*index);
        }

        debug!("Loading texture: {:?}", texture_obj);

        let properties = texture_obj.properties();
        let wrap_mode_u = {
            let val = properties
                .wrap_mode_u_or_default()
                .context("Failed to load wrap mode for U axis")?;
            match val {
                RawWrapMode::Repeat => WrapMode::Repeat,
                RawWrapMode::Clamp => WrapMode::ClampToEdge,
            }
        };
        let wrap_mode_v = {
            let val = properties
                .wrap_mode_v_or_default()
                .context("Failed to load wrap mode for V axis")?;
            match val {
                RawWrapMode::Repeat => WrapMode::Repeat,
                RawWrapMode::Clamp => WrapMode::ClampToEdge,
            }
        };
        let video_clip_obj = texture_obj
            .video_clip()
            .ok_or_else(|| anyhow!("No image data for texture object: {:?}", texture_obj))?;
        let image = self
            .load_video_clip(video_clip_obj)
            .context("Failed to load texture image")?;

        let texture = Texture {
            name: texture_obj.name().map(Into::into),
            image,
            transparent,
            wrap_mode_u,
            wrap_mode_v,
        };

        debug!("Successfully loaded texture: {:?}", texture_obj);

        Ok(self.scene.add_texture(texture))
    }

    // unused code, currently material not supported
    /// Loads the texture image.
    fn load_video_clip(
        &mut self,
        video_clip_obj: object::video::ClipHandle,
    ) -> anyhow::Result<image::DynamicImage> {
        debug!("Loading texture image: {:?}", video_clip_obj);

        let relative_filename = video_clip_obj
            .relative_filename()
            .context("Failed to get relative filename of texture image")?;
        trace!("Relative filename: {:?}", relative_filename);
        let file_ext = Path::new(&relative_filename)
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .map(str::to_ascii_lowercase);
        trace!("File extension: {:?}", file_ext);
        let content = video_clip_obj
            .content()
            .ok_or_else(|| anyhow!("Currently, only embedded texture is supported"))?;
        let image = match file_ext.as_ref().map(AsRef::as_ref) {
            Some("tga") => image::load_from_memory_with_format(content, image::ImageFormat::Tga)
                .context("Failed to load TGA image")?,
            _ => image::load_from_memory(content).context("Failed to load image")?,
        };

        debug!("Successfully loaded texture image: {:?}", video_clip_obj);

        Ok(image)
    }

    // unused code, currently material not supported
    /// Loads the material.
    fn load_material(
        &mut self,
        material_obj: object::material::MaterialHandle,
    ) -> anyhow::Result<MaterialIndex> {
        if let Some(index) = self.material_indices.get(&material_obj.object_id()) {
            return Ok(*index);
        }

        debug!("Loading material: {:?}", material_obj);

        let diffuse_texture = material_obj
            .transparent_texture()
            .map(|v| (true, v))
            .or_else(|| material_obj.diffuse_texture().map(|v| (false, v)))
            .map(|(transparent, texture_obj)| {
                self.load_texture(texture_obj, transparent)
                    .context("Failed to load diffuse texture")
            })
            .transpose()?;

        let properties = material_obj.properties();
        let shading_data = match properties
            .shading_model_or_default()
            .context("Failed to get shading model")?
        {
            ShadingModel::Lambert | ShadingModel::Phong => {
                let ambient_color = properties
                    .ambient_color_or_default()
                    .context("Failed to get ambient color")?;
                let ambient_factor = properties
                    .ambient_factor_or_default()
                    .context("Failed to get ambient factor")?;
                let ambient = (ambient_color * ambient_factor).map(|v| v as f32);
                let diffuse_color = properties
                    .diffuse_color_or_default()
                    .context("Failed to get diffuse color")?;
                let diffuse_factor = properties
                    .diffuse_factor_or_default()
                    .context("Failed to get diffuse factor")?;
                let diffuse = (diffuse_color * diffuse_factor).map(|v| v as f32);
                let emissive_color = properties
                    .emissive_color_or_default()
                    .context("Failed to get emissive color")?;
                let emissive_factor = properties
                    .emissive_factor_or_default()
                    .context("Failed to get emissive factor")?;
                let emissive = (emissive_color * emissive_factor).map(|v| v as f32);
                ShadingData::Lambert(LambertData {
                    ambient,
                    diffuse,
                    emissive,
                })
            }
            v => bail!("Unknown shading model: {:?}", v),
        };

        let material = Material {
            name: material_obj.name().map(Into::into),
            diffuse_texture,
            data: shading_data,
        };

        debug!("Successfully loaded material: {:?}", material_obj);

        Ok(self.scene.add_material(material))
    }
}
