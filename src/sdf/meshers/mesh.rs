use std::io::Write;

use cgmath::{MetricSpace, vec3, Vector3, Zero};

use crate::metadata::short_version_info;
use crate::sdf::SDFSurface;

/// Mesh stores all data that can be obtained from a [`sdf_viewer::sdf::SDFSurface`] trait.
/// Note that this is a lossy representation of the SDF, you should store the wasm file to be able
/// to recreate a mesh with more quality later.
#[derive(Debug, Clone, Default)]
pub struct Mesh {
    /// The vertices of the mesh.
    pub vertices: Vec<Vertex>,
    /// The indices of the triangle faces of the mesh
    pub indices: Vec<u32>,
}

impl Mesh {
    /// Retrieves the materials for each vertex from the SDF. It also fills the normals if unset.
    /// This is useful for meshers that don't write materials (most of them).
    pub fn postproc<S: SDFSurface>(&mut self, sdf: &S) {
        for v in &mut self.vertices {
            let sample = sdf.sample(vec3(v.position[0], v.position[1], v.position[2]), false);
            if v.normal.distance2(Vector3::zero()) < 0.0001 {
                v.normal = sdf.normal(vec3(v.position[0], v.position[1], v.position[2]), None);
            }
            v.color = sample.color;
            v.metallic = sample.metallic;
            v.roughness = sample.roughness;
            v.occlusion = sample.occlusion;
        }
    }

    /// Serializes the mesh to a PLY model file.
    /// It exports all mesh data, although some values may be in a non-standard format.
    #[cfg(feature = "ply-rs")]
    pub fn serialize_ply<T: Write>(&self, out: &mut T) -> std::io::Result<usize> {
        use ply_rs::ply::{Ply, DefaultElement, Encoding, ElementDef, PropertyDef, PropertyType, ScalarType, Property, Addable};
        use ply_rs::writer::{Writer};

        // create a ply objet
        let mut ply = {
            let mut ply = Ply::<DefaultElement>::new();
            ply.header.encoding = Encoding::Ascii;
            ply.header.comments.push(format!("Created with {}", short_version_info()));

            // Define the elements we want to write.
            let mut vertex_el = ElementDef::new("vertex".to_string());
            let p = PropertyDef::new("x".to_string(),
                                     PropertyType::Scalar(ScalarType::Float));
            vertex_el.properties.add(p);
            let p = PropertyDef::new("y".to_string(),
                                     PropertyType::Scalar(ScalarType::Float));
            vertex_el.properties.add(p);
            let p = PropertyDef::new("z".to_string(),
                                     PropertyType::Scalar(ScalarType::Float));
            vertex_el.properties.add(p);
            let p = PropertyDef::new("nx".to_string(),
                                     PropertyType::Scalar(ScalarType::Float));
            vertex_el.properties.add(p);
            let p = PropertyDef::new("ny".to_string(),
                                     PropertyType::Scalar(ScalarType::Float));
            vertex_el.properties.add(p);
            let p = PropertyDef::new("nz".to_string(),
                                     PropertyType::Scalar(ScalarType::Float));
            vertex_el.properties.add(p);
            let p = PropertyDef::new("red".to_string(),
                                     PropertyType::Scalar(ScalarType::UChar));
            vertex_el.properties.add(p);
            let p = PropertyDef::new("green".to_string(),
                                     PropertyType::Scalar(ScalarType::UChar));
            vertex_el.properties.add(p);
            let p = PropertyDef::new("blue".to_string(),
                                     PropertyType::Scalar(ScalarType::UChar));
            vertex_el.properties.add(p);
            let p = PropertyDef::new("metallic".to_string(),
                                     PropertyType::Scalar(ScalarType::Float));
            vertex_el.properties.add(p);
            let p = PropertyDef::new("roughness".to_string(),
                                     PropertyType::Scalar(ScalarType::Float));
            vertex_el.properties.add(p);
            let p = PropertyDef::new("occlusion".to_string(),
                                     PropertyType::Scalar(ScalarType::Float));
            vertex_el.properties.add(p);
            let vertex_el_properties_len = vertex_el.properties.len();
            ply.header.elements.add(vertex_el);

            let mut face_el = ElementDef::new("face".to_string());
            let p = PropertyDef::new("vertex_index".to_string(),
                                     PropertyType::List(ScalarType::UChar, ScalarType::Int));
            face_el.properties.add(p);
            let face_el_properties_len = face_el.properties.len();
            ply.header.elements.add(face_el);

            // Add data
            let mut vertices = Vec::with_capacity(self.vertices.len());
            for v in &self.vertices {
                let mut vertex = DefaultElement::with_capacity(vertex_el_properties_len);
                vertex.insert("x".to_string(), Property::Float(v.position[0]));
                vertex.insert("y".to_string(), Property::Float(v.position[1]));
                vertex.insert("z".to_string(), Property::Float(v.position[2]));
                vertex.insert("nx".to_string(), Property::Float(v.normal[0]));
                vertex.insert("ny".to_string(), Property::Float(v.normal[1]));
                vertex.insert("nz".to_string(), Property::Float(v.normal[2]));
                vertex.insert("red".to_string(), Property::UChar((v.color[0] * 255.9999) as u8));
                vertex.insert("green".to_string(), Property::UChar((v.color[1] * 255.9999) as u8));
                vertex.insert("blue".to_string(), Property::UChar((v.color[2] * 255.9999) as u8));
                vertex.insert("metallic".to_string(), Property::Float(v.metallic));
                vertex.insert("roughness".to_string(), Property::Float(v.roughness));
                vertex.insert("occlusion".to_string(), Property::Float(v.occlusion));
                vertices.push(vertex);
            }
            ply.payload.insert("vertex".to_string(), vertices);
            let mut faces = Vec::with_capacity(self.indices.len() / 3);
            for v in self.indices.as_slice().chunks_exact(3) {
                let mut face = DefaultElement::with_capacity(face_el_properties_len);
                face.insert("vertex_index".to_string(), Property::ListInt(v.into_iter().map(|e| *e as i32).collect()));
                faces.push(face);
            }
            ply.payload.insert("face".to_string(), faces);

            ply
        };

        // set up a writer
        let w = Writer::new();
        w.write_ply(out, &mut ply)
    }
}

/// A vertex of the mesh.
#[derive(Debug, Clone)]
pub struct Vertex {
    pub position: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub color: Vector3<f32>,
    pub metallic: f32,
    pub roughness: f32,
    pub occlusion: f32,
}

impl Default for Vertex {
    fn default() -> Self {
        Self {
            position: vec3(0.0, 0.0, 0.0),
            normal: vec3(0.0, 0.0, 0.0),
            color: vec3(0.0, 0.0, 0.0),
            metallic: 0.0,
            roughness: 0.0,
            occlusion: 0.0,
        }
    }
}

