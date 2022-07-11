//! FBX v7400 support.

use std::path::Path;

use anyhow::{anyhow, bail, Context};
use bevy::{
    asset::{AssetLoader, Handle, LoadContext, LoadedAsset},
    ecs::world::{FromWorld, World},
    log::{debug, error, trace},
    math::{DVec2, DVec3},
    pbr::StandardMaterial,
    render::{
        mesh::{Indices, Mesh as BevyMesh, PrimitiveTopology, VertexAttributeValues},
        render_resource::{AddressMode, SamplerDescriptor},
        renderer::RenderDevice,
        texture::{CompressedImageFormats, Image, ImageType},
    },
    utils::HashSet,
};
use fbxcel_dom::{
    any::AnyDocument,
    v7400::{
        data::{
            material::ShadingModel,
            mesh::{layer::TypedLayerElementHandle, TriangleVertices},
            texture::WrapMode,
        },
        object::{self, model::TypedModelHandle, TypedObjectHandle},
        Document,
    },
};

use crate::{
    data::{mesh::FbxMesh, scene::Scene},
    utils::triangulate,
};

/// How much to scale down FBX stuff.
const FBX_SCALE: f64 = 100.0;

// TODO: multiple scenes
pub struct Loader<'b, 'w> {
    scene: Scene,
    load_context: &'b mut LoadContext<'w>,
    suported_compressed_formats: CompressedImageFormats,
}

pub struct FbxLoader {
    supported: CompressedImageFormats,
}
impl FromWorld for FbxLoader {
    fn from_world(world: &mut World) -> Self {
        let supported = match world.get_resource::<RenderDevice>() {
            Some(render_device) => CompressedImageFormats::from_features(render_device.features()),

            None => CompressedImageFormats::all(),
        };
        Self { supported }
    }
}
impl AssetLoader for FbxLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, anyhow::Result<()>> {
        Box::pin(async move {
            let cursor = std::io::Cursor::new(bytes);
            let reader = std::io::BufReader::new(cursor);
            let maybe_doc =
                AnyDocument::from_seekable_reader(reader).expect("Failed to load document");
            if let AnyDocument::V7400(_ver, doc) = maybe_doc {
                let loader = Loader::new(self.supported, load_context);
                let potential_error = loader
                    .load(*doc)
                    .await
                    .with_context(|| format!("failed to load {:?}", load_context.path()));
                if let Err(err) = potential_error {
                    error!("{err:?}");
                }
                Ok(())
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
    fn new(formats: CompressedImageFormats, load_context: &'b mut LoadContext<'w>) -> Self {
        Self {
            scene: Default::default(),
            load_context,
            suported_compressed_formats: formats,
        }
    }

    async fn load(mut self, doc: Document) -> anyhow::Result<()> {
        for obj in doc.objects() {
            if let TypedObjectHandle::Model(TypedModelHandle::Mesh(mesh)) = obj.get_typed() {
                self.load_mesh(mesh).await?;
            }
        }
        let scene = self.scene;
        let load_context = self.load_context;
        load_context.set_labeled_asset("Scene", LoadedAsset::new(scene));
        debug!("Successfully loaded scene {:?}#Scene", load_context.path());
        Ok(())
    }

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
            .triangulate_each(triangulate::triangulate)
            .context("Triangulation failed")?;
        let indices: Vec<_> = triangle_pvi_indices
            .triangle_vertex_indices()
            .map(|t| t.to_usize() as u32)
            .collect();

        // TODO this seems to duplicate vertices from neighboring triangles. We shouldn't
        // do that and instead set the indice attribute of the BevyMesh properly.
        let get_position = |pos: Option<_>| -> Result<_, anyhow::Error> {
            let cpi = pos.ok_or_else(|| anyhow!("Failed to get control point index"))?;
            let point = polygon_vertices
                .control_point(cpi)
                .ok_or_else(|| anyhow!("Failed to get control point: cpi={:?}", cpi))?;
            Ok((DVec3::from(point) / FBX_SCALE).as_vec3().into())
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
                Ok(DVec2::from(v).as_vec2().into())
            };
            triangle_pvi_indices
                .triangle_vertex_indices()
                .map(get_indices)
                .collect::<Result<Vec<_>, _>>()
                .context("Failed to reconstruct UV vertices")?
        };

        trace!("{:?}", indices);
        if uv.len() != positions.len() || uv.len() != normals.len() || uv.len() != indices.len() {
            bail!(
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
        // let tangents = generate_tangents_for_mesh(&mesh)?;
        // mesh.insert_attribute(BevyMesh::ATTRIBUTE_TANGENT, tangents);

        let label = match mesh_obj.name() {
            Some(name) if !name.is_empty() => format!("FbxMesh@{name}/Primitive"),
            _ => format!("FbxMesh{}/Primitive", mesh_obj.object_id().raw()),
        };
        debug!("Successfully loaded geometry mesh: {label}");

        let handle = self
            .load_context
            .set_labeled_asset(&label, LoadedAsset::new(mesh));
        self.scene.bevy_meshes.insert(handle.clone(), label);
        Ok(handle)
    }

    async fn load_mesh(
        &mut self,
        mesh_obj: object::model::MeshHandle<'_>,
    ) -> anyhow::Result<Handle<FbxMesh>> {
        let label = if let Some(name) = mesh_obj.name() {
            format!("FbxMesh@{name}")
        } else {
            format!("FbxMesh{}", mesh_obj.object_id().raw())
        };

        debug!("Loading mesh: {label}");

        let bevy_obj = mesh_obj.geometry().context("Failed to get geometry")?;

        // TODO: this sucks, but necessary because of `async` `read_asset_bytes` call in d
        // `load_video_clip`  that virally infect everything.
        // This can't even be ran in parallel due to deduplication
        let mut materials = HashSet::new();
        for mat in mesh_obj.materials() {
            let mat = self.load_material(mat).await;
            let mat = mat.context("Failed to load materials for mesh")?;
            materials.insert(mat);
        }
        // TODO
        // How does FBX materials work? See `convert_vertex` in fyrox/resource/fbx/mod.rs
        // It seems they assign multiple textures to the same mesh, and provide a list
        // of vertices in the mesh that it accounts for

        let mesh_handle = self
            .load_bevy_mesh(bevy_obj, 0)
            .context("Failed to load geometry mesh")?;

        let mesh = FbxMesh {
            name: mesh_obj.name().map(Into::into),
            bevy_mesh_handle: mesh_handle,
            materials,
        };

        let mesh_handle = self
            .load_context
            .set_labeled_asset(&label, LoadedAsset::new(mesh));
        debug!("Successfully loaded FBX mesh: {label}");

        self.scene.add_mesh(mesh_handle.clone());
        Ok(mesh_handle)
    }

    async fn load_video_clip(
        &mut self,
        video_clip_obj: object::video::ClipHandle<'_>,
    ) -> anyhow::Result<Image> {
        debug!("Loading texture image: {:?}", video_clip_obj);

        let relative_filename = video_clip_obj
            .relative_filename()
            .context("Failed to get relative filename of texture image")?;
        trace!("Relative filename: {:?}", relative_filename);
        let file_ext = Path::new(&relative_filename)
            .extension()
            .unwrap()
            .to_str()
            .unwrap()
            .to_ascii_lowercase();
        let image: Vec<u8> = if let Some(content) = video_clip_obj.content() {
            // TODO: the clone here is absolutely unnecessary, but there
            // is no way to reconciliate its lifetime with the other branch of
            // this if/else
            content.to_vec()
        } else {
            let parent = self.load_context.path().parent().unwrap();
            let image_path = parent.join(relative_filename);
            self.load_context.read_asset_bytes(image_path).await?
        };
        let is_srgb = false; // TODO
        let image = Image::from_buffer(
            &image,
            ImageType::Extension(&file_ext),
            self.suported_compressed_formats,
            is_srgb,
        );
        let image = image.context("Failed to read image buffer data")?;
        debug!("Successfully loaded texture image: {:?}", video_clip_obj);
        Ok(image)
    }

    async fn load_texture(
        &mut self,
        texture_obj: object::texture::TextureHandle<'_>,
    ) -> anyhow::Result<Handle<Image>> {
        let label = match texture_obj.name() {
            Some(name) if !name.is_empty() => format!("FbxTexture@{name}"),
            _ => format!("FbxTexture{}", texture_obj.object_id().raw()),
        };
        if let Some(handle) = self.scene.textures.get(&label) {
            trace!("already encountered texture: {label}, skipping");
            return Ok(handle.clone_weak());
        }

        debug!("Loading texture: {:?}", label);

        let properties = texture_obj.properties();
        let address_mode_u = {
            let val = properties
                .wrap_mode_u_or_default()
                .context("Failed to load wrap mode for U axis")?;
            match val {
                WrapMode::Repeat => AddressMode::Repeat,
                WrapMode::Clamp => AddressMode::ClampToEdge,
            }
        };
        let address_mode_v = {
            let val = properties
                .wrap_mode_v_or_default()
                .context("Failed to load wrap mode for V axis")?;
            match val {
                WrapMode::Repeat => AddressMode::Repeat,
                WrapMode::Clamp => AddressMode::ClampToEdge,
            }
        };
        let video_clip_obj = texture_obj
            .video_clip()
            .ok_or_else(|| anyhow!("No image data for texture object: {:?}", label))?;
        let image: Result<Image, anyhow::Error> = self.load_video_clip(video_clip_obj).await;
        let mut image = image.context("Failed to load texture image")?;

        image.sampler_descriptor = SamplerDescriptor {
            address_mode_u,
            address_mode_v,
            ..Default::default()
        };

        let handle = self
            .load_context
            .set_labeled_asset(&label, LoadedAsset::new(image));
        debug!("Successfully loaded texture: {:?}", label);
        self.scene.textures.insert(label, handle.clone());
        Ok(handle)
    }

    async fn load_material(
        &mut self,
        material_obj: object::material::MaterialHandle<'_>,
    ) -> anyhow::Result<Handle<StandardMaterial>> {
        let label = match material_obj.name() {
            Some(name) if !name.is_empty() => format!("FbxMaterial@{name}"),
            _ => format!("FbxMaterial{}", material_obj.object_id().raw()),
        };
        if let Some(handle) = self.scene.materials.get(&label) {
            trace!("already encountered material: {label}, skipping");
            return Ok(handle.clone_weak());
        }

        debug!("Loading material: {:?}", material_obj);

        let texture = material_obj
            .transparent_texture()
            .or_else(|| material_obj.diffuse_texture());
        let texture = match texture {
            Some(texture) => Some({
                let texture: Result<_, anyhow::Error> = self.load_texture(texture).await;
                texture.context("Failed to load diffuse texture")?
            }),
            None => None,
        };

        let properties = material_obj.properties();
        let shading_model = properties
            .shading_model_or_default()
            .context("Failed to get shading model")?;
        let mut material = match shading_model {
            ShadingModel::Lambert | ShadingModel::Phong => {
                // TODO: convert shading model to PBR, see
                // https://github.com/Sagoia/FBX2glTF/blob/dc300136c080c2f206b447ed15fb73e942653120/src/gltf/Raw2Gltf.cpp#L255
                // and following code
                StandardMaterial::default()
            }
            v => bail!("Unknown shading model: {:?}", v),
        };
        material.base_color_texture = texture;
        let handle = self
            .load_context
            .set_labeled_asset(&label, LoadedAsset::new(material));
        debug!("Successfully loaded material: {:?}", label);
        self.scene.materials.insert(label, handle.clone());
        Ok(handle)
    }
}
