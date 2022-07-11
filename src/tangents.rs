use bevy_render::{
    mesh::{Indices, Mesh, PrimitiveTopology, VertexAttributeValues},
    render_resource::VertexFormat,
};

struct MikktspaceGeometryHelper<'a> {
    indices: &'a Indices,
    positions: &'a Vec<[f32; 3]>,
    normals: &'a Vec<[f32; 3]>,
    uvs: &'a Vec<[f32; 2]>,
    tangents: Vec<[f32; 4]>,
}

impl MikktspaceGeometryHelper<'_> {
    fn index(&self, face: usize, vert: usize) -> usize {
        let index_index = face * 3 + vert;

        match self.indices {
            Indices::U16(indices) => indices[index_index] as usize,
            Indices::U32(indices) => indices[index_index] as usize,
        }
    }
}

impl bevy_mikktspace::Geometry for MikktspaceGeometryHelper<'_> {
    fn num_faces(&self) -> usize {
        self.indices.len() / 3
    }

    fn num_vertices_of_face(&self, _: usize) -> usize {
        3
    }

    fn position(&self, face: usize, vert: usize) -> [f32; 3] {
        self.positions[self.index(face, vert)]
    }

    fn normal(&self, face: usize, vert: usize) -> [f32; 3] {
        self.normals[self.index(face, vert)]
    }

    fn tex_coord(&self, face: usize, vert: usize) -> [f32; 2] {
        self.uvs[self.index(face, vert)]
    }

    fn set_tangent_encoded(&mut self, tangent: [f32; 4], face: usize, vert: usize) {
        let idx = self.index(face, vert);
        self.tangents[idx] = tangent;
    }
}
#[derive(thiserror::Error, Debug)]
/// Failed to generate tangents for the mesh.
pub(crate) enum GenerateTangentsError {
    #[error("cannot generate tangents for {0:?}")]
    UnsupportedTopology(PrimitiveTopology),
    #[error("missing indices")]
    MissingIndices,
    #[error("missing vertex attributes '{0}'")]
    MissingVertexAttribute(&'static str),
    #[error("the '{0}' vertex attribute should have {1:?} format")]
    InvalidVertexAttributeFormat(&'static str, VertexFormat),
    #[error("mesh not suitable for tangent generation")]
    MikktspaceError,
}

pub(crate) fn generate_tangents_for_mesh(
    mesh: &Mesh,
) -> Result<Vec<[f32; 4]>, GenerateTangentsError> {
    match mesh.primitive_topology() {
        PrimitiveTopology::TriangleList => {}
        other => return Err(GenerateTangentsError::UnsupportedTopology(other)),
    };

    let positions = match mesh.attribute(Mesh::ATTRIBUTE_POSITION).ok_or(
        GenerateTangentsError::MissingVertexAttribute(Mesh::ATTRIBUTE_POSITION.name),
    )? {
        VertexAttributeValues::Float32x3(vertices) => vertices,
        _ => {
            return Err(GenerateTangentsError::InvalidVertexAttributeFormat(
                Mesh::ATTRIBUTE_POSITION.name,
                VertexFormat::Float32x3,
            ))
        }
    };
    let normals = match mesh.attribute(Mesh::ATTRIBUTE_NORMAL).ok_or(
        GenerateTangentsError::MissingVertexAttribute(Mesh::ATTRIBUTE_NORMAL.name),
    )? {
        VertexAttributeValues::Float32x3(vertices) => vertices,
        _ => {
            return Err(GenerateTangentsError::InvalidVertexAttributeFormat(
                Mesh::ATTRIBUTE_NORMAL.name,
                VertexFormat::Float32x3,
            ))
        }
    };
    let uvs = match mesh.attribute(Mesh::ATTRIBUTE_UV_0).ok_or(
        GenerateTangentsError::MissingVertexAttribute(Mesh::ATTRIBUTE_UV_0.name),
    )? {
        VertexAttributeValues::Float32x2(vertices) => vertices,
        _ => {
            return Err(GenerateTangentsError::InvalidVertexAttributeFormat(
                Mesh::ATTRIBUTE_UV_0.name,
                VertexFormat::Float32x2,
            ))
        }
    };
    let indices = mesh
        .indices()
        .ok_or(GenerateTangentsError::MissingIndices)?;

    let len = positions.len();
    let tangents = vec![[0., 0., 0., 0.]; len];
    let mut mikktspace_mesh = MikktspaceGeometryHelper {
        indices,
        positions,
        normals,
        uvs,
        tangents,
    };
    let success = bevy_mikktspace::generate_tangents(&mut mikktspace_mesh);
    if !success {
        return Err(GenerateTangentsError::MikktspaceError);
    }

    // mikktspace seems to assume left-handedness so we can flip the sign to correct for this
    for tangent in &mut mikktspace_mesh.tangents {
        tangent[3] = -tangent[3];
    }

    Ok(mikktspace_mesh.tangents)
}
