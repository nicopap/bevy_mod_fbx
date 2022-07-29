use std::path::Path;

use anyhow::{anyhow, bail, Context};
use bevy::{
    asset::{AssetLoader, BoxedFuture, Handle, LoadContext, LoadedAsset},
    core::Name,
    ecs::world::{FromWorld, World},
    hierarchy::{BuildWorldChildren, WorldChildBuilder},
    log::{debug, error, info, trace},
    math::{DVec2, DVec3, Vec2},
    pbr::{PbrBundle, StandardMaterial},
    prelude::Transform,
    render::{
        mesh::{Indices, Mesh as BevyMesh, PrimitiveTopology, VertexAttributeValues},
        render_resource::{AddressMode, SamplerDescriptor},
        renderer::RenderDevice,
        texture::{CompressedImageFormats, Image, ImageSampler, ImageType},
        view::VisibilityBundle,
    },
    scene::Scene,
    transform::TransformBundle,
    utils::HashMap,
};
use fbxcel_dom::{
    any::AnyDocument,
    fbxcel::tree::v7400::{NodeHandle, NodeId},
    v7400::{
        data::{mesh::layer::TypedLayerElementHandle, texture::WrapMode},
        object::{self, model::TypedModelHandle, texture::TextureHandle, TypedObjectHandle},
        Document,
    },
};

#[cfg(feature = "profile")]
use bevy::log::info_span;

use crate::{
    data::{FbxMesh, FbxObject, FbxScene},
    utils::fbx_extend::{GlobalSettingsExt, NodeHandleTransformExt},
    utils::triangulate,
    MaterialLoader,
};

/// How much to scale down FBX stuff.
const FALLBACK_FBX_SCALE: f64 = 100.0;

pub struct Loader<'b, 'w> {
    scene: FbxScene,
    fbx_scale: f64,
    load_context: &'b mut LoadContext<'w>,
    suported_compressed_formats: CompressedImageFormats,
    material_loaders: Vec<MaterialLoader>,
}

pub struct FbxLoader {
    supported: CompressedImageFormats,
    material_loaders: Vec<MaterialLoader>,
}
impl FromWorld for FbxLoader {
    fn from_world(world: &mut World) -> Self {
        let supported = match world.get_resource::<RenderDevice>() {
            Some(render_device) => CompressedImageFormats::from_features(render_device.features()),

            None => CompressedImageFormats::all(),
        };
        let loaders: crate::FbxMaterialLoaders = world.get_resource().cloned().unwrap_or_default();
        Self {
            supported,
            material_loaders: loaders.0,
        }
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
                let loader =
                    Loader::new(self.supported, self.material_loaders.clone(), load_context);
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

fn generate_scene(
    root: NodeId,
    hierarchy: &HashMap<NodeId, FbxObject>,
    models: &HashMap<NodeId, FbxMesh>,
) -> Scene {
    #[cfg(feature = "profile")]
    let _generate_scene_span = info_span!("generate_scene").entered();

    let mut scene_world = World::default();
    scene_world
        .spawn()
        .insert_bundle(VisibilityBundle::default())
        .insert_bundle(TransformBundle::identity())
        .with_children(|commands| {
            generate_scene_helper(root, commands, hierarchy, models);
        });
    Scene::new(scene_world)
}
fn generate_scene_helper(
    current: NodeId,
    commands: &mut WorldChildBuilder,
    hierarchy: &HashMap<NodeId, FbxObject>,
    models: &HashMap<NodeId, FbxMesh>,
) {
    let current_node = match hierarchy.get(&current) {
        Some(node) => node,
        None => return,
    };
    let mut entity = commands.spawn_bundle(VisibilityBundle::default());
    entity.insert_bundle(TransformBundle::from_transform(current_node.transform));
    // entity.insert_bundle(TransformBundle::default());
    if let Some(name) = &current_node.name {
        entity.insert(Name::new(name.clone()));
    }
    entity.with_children(|commands| {
        if let Some(mesh) = models.get(&current) {
            for (mat, bevy_mesh) in mesh.materials.iter().zip(&mesh.bevy_mesh_handles) {
                let mut entity = commands.spawn_bundle(PbrBundle {
                    mesh: bevy_mesh.clone(),
                    material: mat.clone(),
                    ..Default::default()
                });
                if let Some(name) = mesh.name.as_ref() {
                    entity.insert(Name::new(name.clone()));
                }
            }
        }
        for node_id in &current_node.children {
            generate_scene_helper(*node_id, commands, hierarchy, models);
        }
    });
}

impl<'b, 'w> Loader<'b, 'w> {
    fn new(
        formats: CompressedImageFormats,
        loaders: Vec<MaterialLoader>,
        load_context: &'b mut LoadContext<'w>,
    ) -> Self {
        Self {
            scene: FbxScene::default(),
            fbx_scale: FALLBACK_FBX_SCALE,
            load_context,
            material_loaders: loaders,
            suported_compressed_formats: formats,
        }
    }

    async fn load(mut self, doc: Document) -> anyhow::Result<()> {
        info!(
            "Started loading scene {}#FbxScene",
            self.load_context.path().to_string_lossy(),
        );
        let mut meshes = HashMap::new();
        if let Some(fbx_scale) = doc.global_settings().and_then(|g| g.fbx_scale()) {
            self.fbx_scale = fbx_scale;
        }
        let mut hierarchy = HashMap::default();
        let transform_root = traverse_hierarchy(doc.tree().root(), &mut hierarchy);

        for obj in doc.objects() {
            if let TypedObjectHandle::Model(TypedModelHandle::Mesh(mesh)) = obj.get_typed() {
                let node_id: NodeId = *obj.object_node_id();
                meshes.insert(node_id, self.load_mesh(mesh).await?);
            }
        }
        let mut scene = self.scene;
        scene.hierarchy = hierarchy.clone();
        scene.root = Some(transform_root);
        let load_context = self.load_context;
        load_context.set_labeled_asset("FbxScene", LoadedAsset::new(scene));

        let scene = generate_scene(transform_root, &hierarchy, &meshes);
        load_context.set_labeled_asset("Scene", LoadedAsset::new(scene));
        info!(
            "Successfully loaded scene {}#FbxScene",
            load_context.path().to_string_lossy(),
        );
        Ok(())
    }

    fn load_bevy_mesh(
        &mut self,
        mesh_obj: object::geometry::MeshHandle,
        num_materials: usize,
    ) -> anyhow::Result<Vec<Handle<BevyMesh>>> {
        let label = match mesh_obj.name() {
            Some(name) if !name.is_empty() => format!("FbxMesh@{name}/Primitive"),
            _ => format!("FbxMesh{}/Primitive", mesh_obj.object_id().raw()),
        };
        trace!(
            "loading geometry mesh for node_id: {:?}",
            mesh_obj.object_node_id()
        );

        #[cfg(feature = "profile")]
        let _load_geometry_mesh = info_span!("load_geometry_mesh", label = &label).entered();

        #[cfg(feature = "profile")]
        let triangulate_mesh = info_span!("traingulate_mesh", label = &label).entered();

        let polygon_vertices = mesh_obj
            .polygon_vertices()
            .context("Failed to get polygon vertices")?;
        let triangle_pvi_indices = polygon_vertices
            .triangulate_each(triangulate::triangulate)
            .context("Triangulation failed")?;

        #[cfg(feature = "profile")]
        drop(triangulate_mesh);

        // TODO this seems to duplicate vertices from neighboring triangles. We shouldn't
        // do that and instead set the indice attribute of the BevyMesh properly.
        let get_position = |pos: Option<_>| -> Result<_, anyhow::Error> {
            let cpi = pos.ok_or_else(|| anyhow!("Failed to get control point index"))?;
            let point = polygon_vertices
                .control_point(cpi)
                .ok_or_else(|| anyhow!("Failed to get control point: cpi={:?}", cpi))?;
            Ok((DVec3::from(point) / self.fbx_scale).as_vec3().into())
        };
        let positions = triangle_pvi_indices
            .iter_control_point_indices()
            .map(get_position)
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to reconstruct position vertices")?;

        debug!("Expand position lenght to {}", positions.len());

        let layer = mesh_obj
            .layers()
            .next()
            .ok_or_else(|| anyhow!("Failed to get layer"))?;

        let indices_per_material = || -> Result<_, anyhow::Error> {
            if num_materials == 0 {
                return Ok(None);
            };
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
            Ok(Some(indices_per_material))
        };
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
                let fbx_uv_space = DVec2::from(v).as_vec2();
                let bevy_uv_space = fbx_uv_space * Vec2::new(1.0, -1.0) + Vec2::new(0.0, 1.0);
                Ok(bevy_uv_space.into())
            };
            triangle_pvi_indices
                .triangle_vertex_indices()
                .map(get_indices)
                .collect::<Result<Vec<_>, _>>()
                .context("Failed to reconstruct UV vertices")?
        };

        if uv.len() != positions.len() || uv.len() != normals.len() {
            bail!(
                "mismatched length of buffers: pos{} uv{} normals{}",
                positions.len(),
                uv.len(),
                normals.len(),
            );
        }

        // TODO: remove unused vertices from partial models
        // this is complicated, as it also requires updating the indices.

        // A single mesh may have multiple materials applied to a different subset of
        // its vertices. In the following code, we create a unique mesh per material
        // we found.
        let full_mesh_indices: Vec<_> = triangle_pvi_indices
            .triangle_vertex_indices()
            .map(|t| t.to_usize() as u32)
            .collect();
        let all_indices = if let Some(per_materials) = indices_per_material()? {
            per_materials
        } else {
            vec![full_mesh_indices.clone()]
        };

        debug!("Material count for {label}: {}", all_indices.len());

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
        mesh.set_indices(Some(Indices::U32(full_mesh_indices)));
        mesh.generate_tangents()
            .context("Failed to generate tangents")?;

        let all_handles = all_indices
            .into_iter()
            .enumerate()
            .map(|(i, material_indices)| {
                debug!("Material {i} has {} vertices", material_indices.len());

                let mut material_mesh = mesh.clone();
                material_mesh.set_indices(Some(Indices::U32(material_indices)));

                let label = format!("{label}{i}");

                let handle = self
                    .load_context
                    .set_labeled_asset(&label, LoadedAsset::new(material_mesh));
                self.scene.bevy_meshes.insert(handle.clone(), label);
                handle
            })
            .collect();
        Ok(all_handles)
    }

    // Note: FBX meshes can have multiple different materials, it's not just a mesh.
    // the FBX equivalent of a bevy Mesh is a geometry mesh
    async fn load_mesh(
        &mut self,
        mesh_obj: object::model::MeshHandle<'_>,
    ) -> anyhow::Result<FbxMesh> {
        let label = if let Some(name) = mesh_obj.name() {
            format!("FbxMesh@{name}")
        } else {
            format!("FbxMesh{}", mesh_obj.object_id().raw())
        };
        debug!("Loading FBX mesh: {label}");

        let bevy_obj = mesh_obj.geometry().context("Failed to get geometry")?;

        // async and iterators into for are necessary because of `async` `read_asset_bytes`
        // call in `load_video_clip`  that virally infect everything.
        // This can't even be ran in parallel, because we store already-encountered materials.
        let mut materials = Vec::new();
        for mat in mesh_obj.materials() {
            let mat = self.load_material(mat).await;
            let mat = mat.context("Failed to load materials for mesh")?;
            materials.push(mat);
        }
        let material_count = materials.len();
        if material_count == 0 {
            materials.push(Handle::default());
        }

        let bevy_mesh_handles = self
            .load_bevy_mesh(bevy_obj, material_count)
            .context("Failed to load geometry mesh")?;

        let mesh = FbxMesh {
            name: mesh_obj.name().map(Into::into),
            bevy_mesh_handles,
            materials,
        };

        let mesh_handle = self
            .load_context
            .set_labeled_asset(&label, LoadedAsset::new(mesh.clone()));

        let node_id = *mesh_obj.object_node_id();
        self.scene.meshes.insert(node_id, mesh_handle);

        Ok(mesh)
    }

    async fn load_video_clip(
        &mut self,
        video_clip_obj: object::video::ClipHandle<'_>,
    ) -> anyhow::Result<Image> {
        debug!("Loading texture image: {:?}", video_clip_obj.name());

        let relative_filename = video_clip_obj
            .relative_filename()
            .context("Failed to get relative filename of texture image")?;
        debug!("Relative filename: {:?}", relative_filename);

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
            let clean_relative_filename = relative_filename.replace('\\', "/");
            let image_path = parent.join(clean_relative_filename);
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
        debug!(
            "Successfully loaded texture image: {:?}",
            video_clip_obj.name()
        );

        Ok(image)
    }

    async fn run_loader(
        &mut self,
        material_obj: object::material::MaterialHandle<'_>,
        MaterialLoader {
            static_load,
            dynamic_load,
            preprocess_textures,
            with_textures,
        }: MaterialLoader,
    ) -> anyhow::Result<Option<StandardMaterial>> {
        use crate::utils::fbx_extend::*;
        enum TextureSource<'a> {
            Processed(Image),
            Handle(TextureHandle<'a>),
        }
        let mut textures = HashMap::default();
        // code is a bit tricky so here is a rundown:
        // 1. Load all textures that are meant to be preprocessed by the
        //    MaterialLoader
        for &label in dynamic_load {
            if let Some(texture) = material_obj.load_texture(label) {
                let texture = self.get_texture(texture).await?;
                textures.insert(label, texture);
            }
        }
        preprocess_textures(material_obj, &mut textures);
        // 2. Put the loaded images and the non-preprocessed texture labels into an iterator
        let mut texture_handles = HashMap::with_capacity(textures.len() + static_load.len());
        let texture_handles_iter = textures
            .drain()
            .map(|(label, image)| (label, TextureSource::Processed(image)))
            .chain(static_load.iter().filter_map(|l| {
                material_obj
                    .load_texture(l)
                    .map(|te| (*l, TextureSource::Handle(te)))
            }));
        // 3. For each of those, create an image handle (with potential caching based on the texture name)
        for (label, texture) in texture_handles_iter {
            let handle_label = match texture {
                TextureSource::Handle(texture_handle) => match texture_handle.name() {
                    Some(name) if !name.is_empty() => format!("FbxTexture@{name}"),
                    _ => format!("FbxTexture{}", texture_handle.object_id().raw()),
                },
                TextureSource::Processed(_) => match material_obj.name() {
                    Some(name) if !name.is_empty() => format!("FbxTextureMat@{name}/{label}"),
                    _ => format!("FbxTextureMat{}/{label}", material_obj.object_id().raw()),
                },
            };

            // Either copy the already-created handle or create a new asset
            // for each image or texture to load.
            let handle = if let Some(handle) = self.scene.textures.get(&handle_label) {
                debug!("Already encountered texture: {label}, skipping");

                handle.clone()
            } else {
                let texture = match texture {
                    TextureSource::Processed(texture) => texture,
                    TextureSource::Handle(texture) => self.get_texture(texture).await?,
                };
                let handle = self
                    .load_context
                    .set_labeled_asset(&handle_label, LoadedAsset::new(texture));
                self.scene.textures.insert(handle_label, handle.clone());
                handle
            };
            texture_handles.insert(label, handle);
        }
        // 4. Call with all the texture handles
        Ok(with_textures(material_obj, texture_handles))
    }

    async fn get_texture(
        &mut self,
        texture_obj: object::texture::TextureHandle<'_>,
    ) -> anyhow::Result<Image> {
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
            .context("No image data for texture object")?;

        let image: Result<Image, anyhow::Error> = self.load_video_clip(video_clip_obj).await;
        let mut image = image.context("Failed to load texture image")?;

        image.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor {
            address_mode_u,
            address_mode_v,
            ..Default::default()
        });
        Ok(image)
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
            debug!("Already encountered material: {label}, skipping");

            return Ok(handle.clone_weak());
        }
        debug!("Loading FBX material: {label}");

        let mut material = None;
        let loaders = self.material_loaders.clone();
        for &loader in &loaders {
            if let Some(loader_material) = self.run_loader(material_obj, loader).await? {
                material = Some(loader_material);
                break;
            }
        }
        let material = material.context("None of the material loaders could load this material")?;
        let handle = self
            .load_context
            .set_labeled_asset(&label, LoadedAsset::new(material));
        debug!("Successfully loaded material: {label}");

        self.scene.materials.insert(label, handle.clone());
        Ok(handle)
    }
}

fn traverse_hierarchy(node: NodeHandle, hierarchy: &mut HashMap<NodeId, FbxObject>) -> NodeId {
    #[cfg(feature = "profile")]
    let _hierarchy_span = info_span!("traverse_fbx_hierarchy").entered();

    if let Some(node) = node.first_child_by_name("Objects") {
        traverse_hierarchy_helper(node, hierarchy);
        info!("Tree has {} nodes", hierarchy.len());
        trace!("root: {:?}", node.node_id());
        // trace!("{hierarchy:#?}");
        node.node_id()
    } else {
        error!("Couldn't find the FBX file's node transform hierarchy (`Objects` node). Cannot load a model without it");
        node.node_id()
    }
}
fn traverse_hierarchy_helper(node: NodeHandle, hierarchy: &mut HashMap<NodeId, FbxObject>) {
    let name = node
        .attributes()
        .get(1)
        .and_then(|s| s.get_string())
        .map(|s| s.to_string());

    let fbx_object = FbxObject {
        name,
        // TODO: the transform stuff needs to use the `ObjectHandle::properties`
        // method over raw access to Properties70 attributes of `NodeHandle`.
        transform: Transform {
            translation: node.translation(),
            scale: node.scale(),
            rotation: node.rotation(),
        },
        children: node.children().map(|c| c.node_id()).collect(),
    };
    hierarchy.insert(node.node_id(), fbx_object);
    node.children()
        .filter(|c| c.name() == "Model")
        .for_each(|child| {
            traverse_hierarchy_helper(child, hierarchy);
        });
}
