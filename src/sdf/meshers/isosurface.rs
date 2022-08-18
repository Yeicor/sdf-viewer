use std::ops::{Add, Sub};

use cgmath::{ElementWise, vec3, Vector3};
use isosurface::{DualContouring, LinearHashedMarchingCubes, MarchingCubes};
use isosurface::distance::Signed;
use isosurface::extractor::IndexedInterleavedNormals;
use isosurface::feature::{MinimiseQEF, ParticleBasedMinimisation};
use isosurface::math::Vec3;
use isosurface::sampler::Sample;
use isosurface::source::{HermiteSource, ScalarSource};

use crate::sdf::meshers::Config;
use crate::sdf::meshers::mesh::{Mesh, Vertex};
use crate::sdf::SDFSurface;

pub(crate) fn mesh(algorithm: u8, cfg: Config, sdf: &dyn SDFSurface) -> Mesh {
    // Set up algorithm outputs
    let mut vertices = vec![];
    let mut indices = vec![];

    // Run the algorithm
    let surface_wrapper = SDFSurfaceWrapper { sdf };
    let mut extractor = IndexedInterleavedNormals::new(&mut vertices, &mut indices, &surface_wrapper);
    match algorithm {
        0 => { // Marching Cubes
            let mut alg = MarchingCubes::<Signed>::new(cfg.max_voxels_per_axis);
            alg.extract(&surface_wrapper, &mut extractor)
        }
        1 => { // Linear Hashed Marching Cubes
            let depth = (cfg.max_voxels_per_axis as f32).log2() as usize;
            let mut alg = LinearHashedMarchingCubes::new(depth);
            alg.extract(&surface_wrapper, &mut extractor)
        }
        // 2 => { // Extended Marching Cubes // TODO: Provide VectorSource
        //     let mut alg = ExtendedMarchingCubes::new(cfg.max_voxels_per_axis);
        //     alg.extract(&surface_wrapper, &mut extractor)
        // }
        3 => { // Dual Contouring (MinimizeQEF)
            let mut alg = DualContouring::new(cfg.max_voxels_per_axis, MinimiseQEF {});
            alg.extract(&surface_wrapper, &mut extractor)
        }
        4 => { // Dual Contouring (ParticleBasedMinimization)
            let mut alg = DualContouring::new(cfg.max_voxels_per_axis, ParticleBasedMinimisation {});
            alg.extract(&surface_wrapper, &mut extractor)
        }
        // TODO: Dual contouring with support for simplification
        // TODO: More algorithms
        _ => panic!("Unsupported algorithm"),
    };

    // Convert outputs to our mesh
    let vertices = vertices
        .chunks_exact(6 /* vertex + normal */)
        .map(|v| {
            let vp = surface_wrapper.vert_pos_to(Vec3::new(v[0], v[1], v[2]));
            Vertex { // NOTE: FLIP vertices AND normals
                position: vec3(vp.y, vp.x, vp.z),
                normal: vec3(v[4], v[3], v[5]),
                ..Vertex::default() // NOTE: Unsupported by this mesher (use post-processing shared tool)
            }
        }).collect();
    Mesh {
        vertices,
        indices,
        ..Mesh::default()
    }
}

struct SDFSurfaceWrapper<S: SDFSurface> {
    sdf: S,
}

impl<S: SDFSurface> Sample<Signed> for SDFSurfaceWrapper<S> {
    fn sample(&self, p: Vec3) -> Signed {
        self.sample_scalar(p)
    }
}

impl<S: SDFSurface> ScalarSource for SDFSurfaceWrapper<S> {
    fn sample_scalar(&self, p: Vec3) -> Signed {
        // Perform the sample
        let sample = self.sdf.sample(self.vert_pos_to(p), true);
        Signed(sample.distance)
    }
}

impl<S: SDFSurface> HermiteSource for SDFSurfaceWrapper<S> {
    fn sample_normal(&self, p: Vec3) -> Vec3 {
        // Perform the sample
        let sample = self.sdf.normal(self.vert_pos_to(p), None);
        Vec3::new(sample.x, sample.y, sample.z) // NOTE: Flip normals for this mesher
    }
}

impl<S: SDFSurface> SDFSurfaceWrapper<S> {
    fn vert_pos_to(&self, p: Vec3) -> Vector3<f32> {
        // Convert (0,0,0)-(1,1,1) to the bounding box
        let bb = self.sdf.bounding_box();
        vec3(p.x, p.y, p.z).mul_element_wise(bb[1].sub(bb[0])).add(bb[0])
    }
}