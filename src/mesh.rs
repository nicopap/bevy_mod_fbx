//! Create meshes from FBX data

use std::{iter, vec};

use anyhow::{anyhow, bail, Context};
use bevy::{
    prelude::{debug, error, trace, Mesh},
    render::{
        mesh::Indices, mesh::VertexAttributeValues as Attribs, render_resource::PrimitiveTopology,
    },
};
use fbxcel_dom::v7400::{
    data::mesh::layer::{
        color::Colors, material::MaterialIndex, material::Materials, normal::Normals, uv::Uv,
        LayerHandle, TypedLayerElementHandle as LayerElem,
    },
    data::mesh::{TriangleVertexIndex, TriangleVertices},
    object::geometry::MeshHandle,
};
use glam::{DVec2, DVec3, Vec2};

use crate::utils::triangulate;

macro_rules! extract_type {
    ($name:ident, $method:ident) => {
        |e| match e {
            Ok(LayerElem::$name(handle)) => handle.$method().ok(),
            _ => None,
        }
    };
}
struct Layer<'a>(LayerHandle<'a>);
impl<'a> Layer<'a> {
    fn new(obj: MeshHandle<'a>) -> anyhow::Result<Self> {
        // TODO(feat): we want N layers here, not just the first one
        // note that glTF does support N layers, but bevy doesn't either.
        let layer = obj.layers().next().context("Failed to get layer")?;
        Ok(Layer(layer))
    }
    fn get_type<T: 'a>(
        &self,
        kind: &str,
        mut f: impl FnMut(anyhow::Result<LayerElem<'a>>) -> Option<T>,
    ) -> anyhow::Result<T> {
        self.0
            .layer_element_entries()
            .find_map(|entry| f(entry.typed_layer_element()))
            .ok_or_else(|| anyhow!("{kind} not found for mesh"))
    }
    fn primitives(&self) -> anyhow::Result<Materials<'a>> {
        self.get_type("primitives", extract_type!(Material, materials))
    }
    fn uvs(&self) -> anyhow::Result<Uv<'a>> {
        self.get_type("uvs", extract_type!(Uv, uv))
    }
    fn normals(&self) -> anyhow::Result<Normals<'a>> {
        self.get_type("normals", extract_type!(Normal, normals))
    }
    fn colors(&self) -> anyhow::Result<Colors<'a>> {
        self.get_type("colors", extract_type!(Color, color))
    }
}
struct Triangles<'a>(TriangleVertices<'a>);
impl<'a> Triangles<'a> {
    fn pick<A>(
        &self,
        mut per_vertex: impl FnMut(&TriangleVertices, TriangleVertexIndex) -> anyhow::Result<A>,
    ) -> anyhow::Result<Vec<A>> {
        self.0
            .triangle_vertex_indices()
            .map(|i| per_vertex(&self.0, i))
            .collect::<Result<Vec<_>, _>>()
    }
}

fn load_single_primitive(
    indices: Vec<u32>,
    positions: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    normals: Vec<[f32; 3]>,
) -> Mesh {
    trace!(
        "Mesh with {} vertices & {} indices",
        positions.len(),
        indices.len()
    );
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, Attribs::Float32x3(positions));
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, Attribs::Float32x2(uvs));
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, Attribs::Float32x3(normals));
    mesh.set_indices(Some(Indices::U32(indices)));
    // TODO(err): better handling
    if let Err(err) = mesh.generate_tangents() {
        error!("Could't generate tangents: {err}");
    }
    mesh
}
fn load_subset_primitive(
    indices: &[u32],
    positions: &[[f32; 3]],
    uvs: &[[f32; 2]],
    normals: &[[f32; 3]],
) -> Mesh {
    // TODO(perf): shouldn't dumbly duplicate data here.
    load_single_primitive(
        indices.to_vec(),
        positions.to_vec(),
        uvs.to_vec(),
        normals.to_vec(),
    )
    // let all_handles = all_indices
    //     .into_iter()
    //     .enumerate()
    //     .map(|(i, material_indices)| {
    //         debug!("Material {i} has {} vertices", material_indices.len());

    //         let mut material_mesh = mesh.clone();
    //         material_mesh.set_indices(Some(Indices::U32(material_indices)));

    //         let label = format!("{label}{i}");

    //         let handle = self
    //             .load_context
    //             .set_labeled_asset(&label, LoadedAsset::new(material_mesh));
    //         self.scene.bevy_meshes.insert(handle.clone(), label);
    //         handle
    //     })
    //     .collect();
    // Ok(all_handles)
}
pub(crate) fn load(obj: MeshHandle) -> anyhow::Result<IterMesh> {
    let mesh_vertices = obj.polygon_vertices()?;

    let mesh_triangles = mesh_vertices.triangulate_each(triangulate::triangulate)?;

    // TODO this seems to duplicate vertices from neighboring triangles. We shouldn't
    // do that and instead set the indice attribute of the Mesh properly.
    let get_position = |mesh_index: Option<_>| -> Result<_, anyhow::Error> {
        let mesh_index = mesh_index.context("Failed to get mesh index")?;
        let point = mesh_vertices
            .control_point(mesh_index)
            .ok_or_else(|| anyhow!("Failed to get mesh index {mesh_index:?}"))?;
        Ok(DVec3::from(point).as_vec3().into())
    };
    let positions = mesh_triangles
        .iter_control_point_indices()
        .map(get_position)
        .collect::<Result<Vec<_>, _>>()
        .context("Failed to reconstruct position vertices")?;

    let triangles = Triangles(mesh_triangles);
    let layer = Layer::new(obj)?;

    debug!("Expand position lenght to {}", positions.len());
    let normals = layer.normals()?;
    let normals = triangles.pick(|t, i| {
        let v = normals.normal(t, i)?;
        Ok(DVec3::from(v).as_vec3().into())
    })?;

    let uvs = layer.uvs()?;
    let uvs = triangles.pick(|t, i| {
        let uv = uvs.uv(t, i)?;
        let fbx_uv_space = DVec2::from(uv).as_vec2();
        let bevy_uv_space = fbx_uv_space * Vec2::new(1.0, -1.0) + Vec2::new(0.0, 1.0);
        Ok(bevy_uv_space.into())
    })?;

    if uvs.len() != positions.len() || uvs.len() != normals.len() || positions.is_empty() {
        bail!(
            "mismatched length of buffers: pos:{} uv:{} normals:{}",
            positions.len(),
            uvs.len(),
            normals.len(),
        );
    }

    let primitives = layer.primitives()?;
    let mut primitives = triangles.pick(|t, i| {
        let prim_index = primitives.material_index(t, i)?;
        Ok((prim_index, i.to_usize() as u32))
    })?;
    primitives.sort_by_key(|(prim, _)| *prim);

    if primitives.is_empty() {
        let indices = triangles.pick(|_, i| Ok(i.to_usize() as u32)).unwrap();
        let mesh = load_single_primitive(indices, positions, uvs, normals);
        Ok(IterMesh::Single(Some(mesh)))
    } else {
        let many = CreateMeshes {
            pos: positions.into_boxed_slice(),
            uvs: uvs.into_boxed_slice(),
            normals: normals.into_boxed_slice(),
            indices: primitives.into_iter().peekable(),
            current_indices: Vec::new(),
            last_prim: None,
        };
        Ok(IterMesh::Many(many))
    }
}
pub(crate) enum IterMesh {
    Single(Option<Mesh>),
    Many(CreateMeshes),
}
pub(crate) struct CreateMeshes {
    pos: Box<[[f32; 3]]>,
    uvs: Box<[[f32; 2]]>,
    normals: Box<[[f32; 3]]>,
    indices: iter::Peekable<vec::IntoIter<(MaterialIndex, u32)>>,
    current_indices: Vec<u32>,
    last_prim: Option<MaterialIndex>,
}
impl Iterator for IterMesh {
    type Item = Mesh;
    fn next(&mut self) -> Option<Self::Item> {
        let CreateMeshes {
            pos,
            uvs,
            normals,
            indices,
            current_indices,
            last_prim,
        } = match self {
            IterMesh::Single(single) => return single.take(),
            IterMesh::Many(many) => many,
        };
        loop {
            match (&mut *last_prim, indices.peek()) {
                (Some(old_prim), Some((new_prim, _))) if *old_prim != *new_prim => {
                    let ret = load_subset_primitive(current_indices, pos, uvs, normals);
                    current_indices.clear();
                    *last_prim = Some(*new_prim);
                    return Some(ret);
                }
                (Some(_), Some(..)) => current_indices.push(indices.next().unwrap().1),
                // TODO(bug): broken if empty iterator
                (Some(_), None) => {
                    let ret = load_subset_primitive(current_indices, pos, uvs, normals);
                    current_indices.clear();
                    *last_prim = None;
                    return Some(ret);
                }
                (None, None) => return None,
                (None, Some((new_prim, _))) => *last_prim = Some(*new_prim),
            }
        }
    }
}
