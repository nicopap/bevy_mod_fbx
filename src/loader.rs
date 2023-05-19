use std::{any::type_name, mem, ops::Deref, path::Path, rc::Rc};

use anyhow::{anyhow, Context, Error};
use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::{
        debug, error, trace, BuildWorldChildren, FromWorld, Handle, Image, Mesh, Name, PbrBundle,
        Scene, StandardMaterial, Transform, TransformBundle, VisibilityBundle, World,
        WorldChildBuilder,
    },
    render::{
        renderer::RenderDevice,
        texture::{CompressedImageFormats, ImageType},
    },
    utils::{get_short_name, BoxedFuture, HashMap},
};
use fbxcel_dom::{
    any::AnyDocument,
    v7400::object::{
        geometry, material::MaterialHandle, model, model::ModelHandle, model::TypedModelHandle,
        texture::TextureHandle, video::ClipHandle, ObjectId, TypedObjectHandle,
    },
    v7400::{object::ObjectHandle, Document},
};
use serde::{Deserialize, Serialize};

#[cfg(feature = "profile")]
use bevy::log::info_span;
use glam::Vec3;

use crate::{
    data::{FbxMesh, FbxObject, FbxScene},
    fbx_transform::FbxTransform,
    mesh,
    utils::fbx_extend::{GlobalSettingsExt, ModelTreeRootExt},
    MaterialLoader, Textures,
};

type Result<T> = anyhow::Result<T>;
pub(crate) type Ctx<'a, 'b> = &'a mut LoadContext<'b>;

/// Bevy is kinda "meters" based while FBX (or rather: stuff exported by maya) is in "centimeters"
/// Although it doesn't mean much in practice.
const FBX_TO_BEVY_SCALE_FACTOR: f32 = 0.01;

#[derive(Serialize, Deserialize, Clone, Copy, Default)]
pub struct FbxLoaderSettings {
    override_scale: Option<f32>,
}
pub struct Loader {
    errors: Vec<Error>,
    scene: FbxScene,
    meshes: HashMap<ObjectId, FbxMesh>,
    suported_compressed_formats: CompressedImageFormats,
    material_loaders: Rc<[MaterialLoader]>,
    override_scale: Option<f32>,
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
    type Asset = FbxScene;
    type Settings = FbxLoaderSettings;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        settings: &'a FbxLoaderSettings,
        ctx: Ctx<'a, '_>,
    ) -> BoxedFuture<'a, Result<FbxScene>> {
        Box::pin(async move {
            let mut buffered = Vec::new();
            reader.read_to_end(&mut buffered).await?;
            let maybe_doc = AnyDocument::from_reader(&*buffered).expect("Failed to load document");
            if let AnyDocument::V7400(_ver, doc) = maybe_doc {
                let mut loader =
                    Loader::new(self.supported, self.material_loaders.clone(), *settings);
                let context = format!("failed to load {:?}", ctx.path());
                let potential_error = loader.load(ctx, *doc).context(context);
                if let Err(err) = potential_error {
                    Err(anyhow!("{err:?}"))
                } else {
                    Ok(loader.scene)
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

fn spawn_scene(
    fbx_file_scale: f32,
    roots: &[ObjectId],
    hierarchy: &HashMap<ObjectId, FbxObject>,
    models: &HashMap<ObjectId, FbxMesh>,
) -> Scene {
    trace!("Spawning scene");
    let mut scene_world = World::default();
    scene_world
        .spawn((
            VisibilityBundle::default(),
            TransformBundle::from_transform(Transform::from_scale(
                Vec3::ONE * FBX_TO_BEVY_SCALE_FACTOR * fbx_file_scale,
            )),
            Name::new("Fbx scene root"),
        ))
        .with_children(|commands| {
            for root in roots {
                spawn_scene_rec(*root, commands, hierarchy, models);
            }
        });
    Scene::new(scene_world)
}
fn spawn_scene_rec(
    current: ObjectId,
    commands: &mut WorldChildBuilder,
    hierarchy: &HashMap<ObjectId, FbxObject>,
    models: &HashMap<ObjectId, FbxMesh>,
) {
    let current_node = match hierarchy.get(&current) {
        Some(node) => node,
        None => return,
    };
    trace!("Spawning child");
    let mut entity = commands.spawn((
        VisibilityBundle::default(),
        TransformBundle::from_transform(current_node.transform),
    ));
    if let Some(name) = &current_node.name {
        entity.insert(Name::new(name.clone()));
    }
    entity.with_children(|commands| {
        if let Some(mesh) = models.get(&current) {
            for (mat, bevy_mesh) in mesh.materials.iter().zip(&mesh.bevy_mesh_handles) {
                trace!("With materials: {mat:?} {bevy_mesh:?}");
                let mut entity = commands.spawn(PbrBundle {
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
            spawn_scene_rec(*node_id, commands, hierarchy, models);
        }
    });
}
fn object_label<'a, T: 'a + Deref<Target = ObjectHandle<'a>>>(object: T) -> String {
    let label = get_short_name(type_name::<T>());
    let label = match object.name() {
        Some(name) if !name.is_empty() => format!("{label}@{name}"),
        None | Some(_) => format!("{label}{}", object.object_id().raw()),
    };
    debug!("{label}");
    label
}

impl Loader {
    fn new(
        formats: CompressedImageFormats,
        loaders: Vec<MaterialLoader>,
        settings: FbxLoaderSettings,
    ) -> Self {
        Loader {
            errors: Vec::new(),
            scene: FbxScene::default(),
            material_loaders: loaders.into(),
            suported_compressed_formats: formats,
            meshes: HashMap::default(),
            override_scale: settings.override_scale,
        }
    }

    fn load(&mut self, ctx: Ctx, doc: Document) -> Result<()> {
        let mut meshes = HashMap::new();
        let mut hierarchy = HashMap::new();

        let fbx_scale = || {
            let scale = doc.global_settings()?;
            let scale = scale.fbx_scale()?;
            Some(scale as f32)
        };
        let fbx_scale = self.override_scale.or_else(fbx_scale).unwrap_or(1.0);

        let roots = doc.model_roots();
        for root in &roots {
            traverse_hierarchy(*root, &mut hierarchy);
        }

        for obj in doc.objects() {
            if let TypedObjectHandle::Model(TypedModelHandle::Mesh(mesh)) = obj.get_typed() {
                let label = object_label(*mesh);
                let mesh = ctx.labeled_asset_scope(label, |ctx| self.load_mesh(ctx, mesh));
                meshes.insert(obj.object_id(), mesh);
            }
        }
        if !self.errors.is_empty() {
            error!("Failed to load scene, got some erorrs:");
            for error in self.errors.drain(..) {
                error!("{error}");
            }
            return Err(anyhow!("Scene incomplete"));
        }
        let roots: Vec<_> = roots.into_iter().map(|obj| obj.object_id()).collect();
        let scene = spawn_scene(fbx_scale, &roots, &hierarchy, &self.meshes);
        trace!("Scene: {scene:?}");
        ctx.add_labeled_asset("Scene".to_string(), scene);

        let mut fbx_scene = mem::take(&mut self.scene);
        fbx_scene.hierarchy = hierarchy;
        fbx_scene.roots = roots;
        fbx_scene.meshes = meshes;
        trace!("FbxScene: {fbx_scene:#?}");
        ctx.add_labeled_asset("FbxScene".to_string(), fbx_scene);
        Ok(())
    }

    fn load_primitives(
        &mut self,
        ctx: Ctx,
        obj: geometry::MeshHandle,
    ) -> Result<Vec<Handle<Mesh>>> {
        let label = object_label(*obj);
        Ok(mesh::load(obj)?
            .enumerate()
            .map(|(i, mesh)| {
                let label = format!("{label}__{i}");
                let handle = ctx.add_labeled_asset(label.clone(), mesh);
                self.scene.bevy_meshes.insert(label, handle.clone());
                handle
            })
            .collect())
    }

    fn load_mesh(&mut self, ctx: Ctx, obj: model::MeshHandle<'_>) -> FbxMesh {
        match self.load_mesh_inner(ctx, obj) {
            Ok(value) => {
                self.meshes.insert(obj.object_id(), value.clone());
                value
            }
            Err(err) => {
                self.errors.push(err);
                FbxMesh::default()
            }
        }
    }
    // Similarly to glTF, FBX meshes can have multiple different materials, it's not just a mesh.
    fn load_mesh_inner(&mut self, ctx: Ctx, obj: model::MeshHandle<'_>) -> Result<FbxMesh> {
        let err = "Failed to get geometry";
        let geometry = obj.geometry().context(err)?;

        let materials = obj
            .materials()
            .map(|m| self.load_material(ctx, m))
            .collect::<Vec<_>>();

        let meshes = self.load_primitives(ctx, geometry).context(err)?;
        trace!(
            "Mesh {:?} with {} materials & {} meshes",
            object_label(*obj),
            materials.len(),
            meshes.len()
        );

        Ok(FbxMesh {
            name: obj.name().map(Into::into),
            bevy_mesh_handles: meshes,
            materials,
        })
    }

    fn image(&self, file_ext: &str, buffer: &[u8]) -> Result<Image> {
        let is_srgb = false; // TODO
        Ok(Image::from_buffer(
            buffer,
            ImageType::Extension(file_ext),
            self.suported_compressed_formats,
            is_srgb,
        )?)
    }
    fn load_video_clip(&mut self, ctx: Ctx, video_clip_obj: ClipHandle) -> Handle<Image> {
        // TODO: unwrap
        let relative_file = video_clip_obj.relative_filename().unwrap();

        let file_ext = Path::new(&relative_file)
            .extension()
            .unwrap()
            .to_str()
            .unwrap()
            .to_ascii_lowercase();

        let mut image = || {
            let (name, image) = if let Some(content) = video_clip_obj.content() {
                let image = self.image(&file_ext, content)?;
                let file = relative_file.to_string();
                trace!("embedded texture: {file}");
                (file.clone(), ctx.add_labeled_asset(file, image))
            } else {
                let parent = ctx.path().parent().unwrap();
                let clean_relative_filename = relative_file.replace('\\', "/");
                let image_path = parent.join(clean_relative_filename);
                trace!("File texture: {image_path:?}");
                (
                    image_path.to_string_lossy().to_string(),
                    ctx.load(image_path),
                )
            };
            self.scene.textures.insert(name.to_string(), image.clone());
            Ok(image)
        };

        image().unwrap_or_else(|err| {
            self.errors.push(err);
            Handle::default()
        })
    }
    pub(crate) fn load_texture(&mut self, ctx: Ctx, obj: TextureHandle<'_>) -> Handle<Image> {
        // TODO(feat): set the address mode correctly.
        match obj.video_clip() {
            Some(video_clip) => self.load_video_clip(ctx, video_clip),
            None => {
                let error = anyhow!("No image data for texture {:?}", obj.name());
                self.errors.push(error);
                Handle::default()
            }
        }
    }
    fn load_material(&mut self, ctx: Ctx, obj: MaterialHandle) -> Handle<StandardMaterial> {
        let mut material = None;
        let loaders = self.material_loaders.clone();
        for &loader in loaders.iter() {
            material = (loader.with_textures)(obj, Textures::new(ctx, obj, self));
            if material.is_some() {
                trace!("Created material using loader '{}'", loader.name);
                break;
            }
        }
        let err = "None of the material loaders could load this material";
        let material = material.map(|m| {
            let label = object_label(obj);
            let handle = ctx.add_labeled_asset(label.clone(), m);
            self.scene.materials.insert(label, handle.clone());
            handle
        });
        material.context(err).unwrap_or_else(|err| {
            self.errors.push(err);
            Handle::default()
        })
    }
}

fn traverse_hierarchy(node: ModelHandle, hierarchy: &mut HashMap<ObjectId, FbxObject>) {
    #[cfg(feature = "profile")]
    let _hierarchy_span = info_span!("traverse_fbx_hierarchy").entered();

    traverse_hierarchy_rec(node, None, hierarchy);
    debug!("Tree has {} nodes", hierarchy.len());
    trace!("root: {:?}", node.object_node_id());
}
fn traverse_hierarchy_rec(
    node: ModelHandle,
    parent: Option<FbxTransform>,
    hierarchy: &mut HashMap<ObjectId, FbxObject>,
) -> bool {
    let name = node.name().map(|s| s.to_owned());
    let data = FbxTransform::from_node(node, parent);

    let mut mesh_leaf = false;
    node.child_models().for_each(|child| {
        mesh_leaf |= traverse_hierarchy_rec(*child, Some(data), hierarchy);
    });
    if node.subclass() == "Mesh" {
        mesh_leaf = true;
    }
    // Only keep nodes that have Mesh children
    // (ie defines something visible in the scene)
    // I've found some very unwindy FBX files with several thousand
    // nodes that served no practical purposes,
    // This also trims deformers and limb nodes, which we currently
    // do not support
    if mesh_leaf {
        let fbx_object = FbxObject {
            name,
            transform: data.as_local_transform(parent.as_ref().map(|p| p.global)),
            children: node.child_models().map(|c| c.object_id()).collect(),
        };
        hierarchy.insert(node.object_id(), fbx_object);
    }
    mesh_leaf
}
