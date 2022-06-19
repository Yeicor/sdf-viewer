use std::cell::RefCell;
use std::ops::AddAssign;
use std::rc::Rc;

use cgmath::ElementWise;
use cgmath::num_traits::Pow;
use eframe::glow::HasContext;
use three_d::{context, CpuMesh, CpuTexture3D, Gm, Mesh, Texture3D, Vector3};
use three_d::{Interpolation, Positions, TextureData, Wrapping};

use material::SDFViewerMaterial;

use crate::app::scene::sdf::loading::LoadingManager;
use crate::sdf::SDFSurface;

pub mod material;
pub mod loading;

/// The SDF viewer controller, that synchronizes the CPU and GPU sides.
pub struct SDFViewer {
    /// The CPU side of the 3D SDF texture.
    pub texture: CpuTexture3D,
    /// The GPU-side mesh and material, including the 3D GPU texture.
    pub volume: Rc<RefCell<Gm<Mesh, SDFViewerMaterial>>>,
    /// Controls the iterative algorithm used to fill the SDF texture (to provide faster previews).
    pub loading_mgr: LoadingManager,
    /// The three-d cloned context
    pub ctx: three_d::Context,
}

/// The default value for uncomputed SDF values while loading. Should be small to avoid graphical
/// artifacts, but will slow down rendering if too small. It is also useful for progressive loading.
const AIR_DIST: f32 = 0.001234;

impl SDFViewer {
    /// Creates a new SDF viewer for the given bounding box (tries to keep aspect ratio).
    pub fn from_bb(ctx: &three_d::Context, bb: &[Vector3<f32>; 2], max_voxels_side: Option<usize>) -> Self {
        let max_voxels_side = max_voxels_side.unwrap_or(256);
        let bb_size = bb[1] - bb[0];
        let mut voxels = Vector3::new(0usize, 0usize, 0usize);
        if bb_size.x > bb_size.y {
            if bb_size.x > bb_size.z {
                voxels.x = max_voxels_side;
                voxels.y = (max_voxels_side as f32 * bb_size.y / bb_size.x) as usize;
                voxels.z = (max_voxels_side as f32 * bb_size.z / bb_size.x) as usize;
            } else {
                voxels.x = (max_voxels_side as f32 * bb_size.x / bb_size.y) as usize;
                voxels.y = max_voxels_side;
                voxels.z = (max_voxels_side as f32 * bb_size.z / bb_size.y) as usize;
            }
        } else {
            voxels.x = (max_voxels_side as f32 * bb_size.x / bb_size.z) as usize;
            voxels.y = (max_voxels_side as f32 * bb_size.y / bb_size.z) as usize;
            voxels.z = max_voxels_side;
        }
        Self::new_voxels(ctx, voxels, bb)
    }

    /// Creates a new SDF viewer with the given number of voxels in each axis.
    pub fn new_voxels(ctx: &three_d::Context, voxels: Vector3<usize>, bb: &[Vector3<f32>; 2]) -> Self {
        let texture = CpuTexture3D {
            data: TextureData::RgbaF32(vec![[AIR_DIST; 4]; voxels.x * voxels.y * voxels.z]),
            width: voxels.x as u32,
            height: voxels.y as u32,
            depth: voxels.z as u32,
            min_filter: Interpolation::Nearest, // Nearest for broken blocky mode
            mag_filter: Interpolation::Nearest,
            mip_map_filter: None,
            wrap_s: Wrapping::MirroredRepeat, // <- Should be safe, even out of bounds
            wrap_t: Wrapping::MirroredRepeat,
            wrap_r: Wrapping::MirroredRepeat,
        };
        let mesh = Mesh::new(ctx, &cube_with_bounds(bb)).unwrap();
        let material = SDFViewerMaterial::new(
            Texture3D::new(ctx, &texture).unwrap(), *bb);
        let volume = Gm::new(mesh, material);
        Self {
            texture,
            volume: Rc::new(RefCell::new(volume)),
            loading_mgr: LoadingManager::new(voxels, 3),
            ctx: ctx.clone(),
        }
    }

    /// Iteratively requests data from the CPU `Sdf` and stores it in the CPU-side buffer,
    /// maintaining an interactive framerate. Note that this intermediate buffer must be `commit`ed
    /// to the GPU before the SDF is updated on screen.
    ///
    /// Set force to > 0 to force a full update of the SDF, with the given number of lower to higher
    /// resolution passes.
    ///
    /// Returns the number of updates. It performs at least one update if needed, even if the
    /// time limit is reached.
    pub fn update(&mut self, sdf: impl SDFSurface, max_delta_time: instant::Duration) -> usize {
        let mut first = true;
        let start_iter = self.loading_mgr.iterations();
        let sdf_bb = sdf.bounding_box();
        let sdf_bb_size = sdf_bb[1] - sdf_bb[0];
        let texture_size_minus_1 = Vector3::new(self.texture.width as f32 - 1., self.texture.height as f32 - 1., self.texture.depth as f32 - 1.);
        let start_time = instant::Instant::now();

        while first || start_time.elapsed() < max_delta_time {
            first = false; // TODO: Cross-platform parallel iteration?
            if let Some(next_index) = self.loading_mgr.next() {
                match &mut self.texture.data {
                    TextureData::RgbaF32(data) => {
                        // Compute the flat index: 3D texture data is in the row-major order.
                        let flat_index = (next_index.z * self.texture.height as usize + next_index.y) * self.texture.width as usize + next_index.x;
                        if data[flat_index][0] == AIR_DIST { // Only update if not already computed
                            // Compute the position in the SDF surface.
                            let mut next_point = Vector3::new(next_index.x as f32, next_index.y as f32, next_index.z as f32);
                            next_point.div_assign_element_wise(texture_size_minus_1); // Normalize to [0, 1]
                            next_point.mul_assign_element_wise(sdf_bb_size);
                            next_point.add_assign(sdf_bb[0]);
                            // Actually sample the SDF.
                            let sample = sdf.sample(next_point, false);
                            data[flat_index][0] = sample.distance;
                            data[flat_index][1] = material::pack_color(sample.color);
                            data[flat_index][2] = material::pack_color(Vector3::new(sample.metallic, sample.roughness, sample.occlusion))
                            // info!("Updated voxel color {:?}", data[flat_index][1]);
                            // TODO: Provide more voxel data to the shader, like a material kind index for using custom GLSL code.
                        }
                    }
                    _ => panic!("developer error: expected RgbaF32 texture data"),
                }
            } else {
                break; // No more work to do!
            }
        }
        self.loading_mgr.iterations() - start_iter
    }

    /// Commits all previous `update`s to the GPU, updating the GPU-side texture data.
    pub fn commit(&mut self) {
        let mut vol_mut = self.volume.borrow_mut();
        vol_mut.material.voxels.fill(match &self.texture.data {
            TextureData::RgbaF32(d) => { d.as_slice() }
            _ => panic!("developer error: expected RgbaF32 texture data"),
        }).unwrap();
        vol_mut.material.lod_dist_between_samples = 2f32.pow(self.loading_mgr.passes_left() as u8);
        if vol_mut.material.lod_dist_between_samples == 1. {
            unsafe { // OpenGL calls are always unsafe
                // The texture is bound by previous fill call
                self.ctx.tex_parameter_i32(context::TEXTURE_3D,
                                           context::TEXTURE_MIN_FILTER,
                                           context::LINEAR as i32);
                self.ctx.tex_parameter_i32(context::TEXTURE_3D,
                                           context::TEXTURE_MAG_FILTER,
                                           context::LINEAR as i32);
            }
        }
    }
}

/// Creates a cube mesh with the given bounds.
fn cube_with_bounds(bb: &[Vector3<f32>; 2]) -> CpuMesh {
    let mut cube_bounds_mesh = CpuMesh::cube();
    match cube_bounds_mesh.positions {
        Positions::F32(ref mut d) => {
            for p in d {
                if p.x < 0.0 {
                    p.x = bb[0].x;
                }
                if p.y < 0.0 {
                    p.y = bb[0].y;
                }
                if p.z < 0.0 {
                    p.z = bb[0].z;
                }
                if p.x > 0.0 {
                    p.x = bb[1].x;
                }
                if p.y > 0.0 {
                    p.y = bb[1].y;
                }
                if p.z > 0.0 {
                    p.z = bb[1].z;
                }
            }
        }
        _ => panic!("SDFController: cube_bounds_mesh.positions is not F32"),
    }
    cube_bounds_mesh
}