//! FBX v7400 support.

use anyhow::{anyhow, bail, Context};
use bevy_asset::{AssetLoader, Handle, LoadContext, LoadedAsset};
use bevy_log::{debug, error, trace};
use bevy_math::{DVec2, DVec3};
use bevy_render::mesh::{Indices, Mesh as BevyMesh, PrimitiveTopology, VertexAttributeValues};
use fbxcel_dom::{
    any::AnyDocument,
    v7400::{
        data::mesh::layer::TypedLayerElementHandle,
        object::{self, model::TypedModelHandle, TypedObjectHandle},
        Document,
    },
};

use crate::data::{mesh::FbxMesh, scene::Scene};

use crate::utils::triangulate;

/// How much to scale down FBX stuff.
const FBX_SCALE: f64 = 100.0;

// TODO: multiple scenes
pub struct Loader<'b, 'w> {
    scene: Scene,
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
                    .load(*doc)
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
            load_context,
        }
    }

    /// Loads the document.
    fn load(mut self, doc: Document) -> anyhow::Result<()> {
        for obj in doc.objects() {
            if let TypedObjectHandle::Model(TypedModelHandle::Mesh(mesh)) = obj.get_typed() {
                self.load_mesh(mesh)?;
            }
        }
        let scene = self.scene;
        let load_context = self.load_context;
        load_context.set_labeled_asset("Scene", LoadedAsset::new(scene));
        debug!("Successfully loaded scene {:?}#Scene", load_context.path());
        Ok(())
    }

    /// Loads the geometry.
    fn load_bevy_mesh(
        &mut self,
        mesh_obj: object::geometry::MeshHandle,
        _num_materials: usize,
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
            // TODO: probably a better conversion method here XD
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

        let mesh_handle = self
            .load_bevy_mesh(bevy_obj, 0)
            .context("Failed to load geometry mesh")?;

        let mesh = FbxMesh {
            name: mesh_obj.name().map(Into::into),
            bevy_mesh_handle: mesh_handle,
            materials: Vec::new(),
        };

        let mesh_handle = self
            .load_context
            .set_labeled_asset(&label, LoadedAsset::new(mesh));
        debug!("Successfully loaded FBX mesh: {label}");

        self.scene.add_mesh(mesh_handle.clone());
        Ok(mesh_handle)
    }
}
