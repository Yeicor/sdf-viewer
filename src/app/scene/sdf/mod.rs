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
use crate::sdf::defaults::merge_bounding_boxes;
use crate::sdf::SDFSurface;

pub mod material;
pub mod loading;

/// The SDF viewer controller, that synchronizes the CPU and GPU sides.
pub struct SDFViewer {
    /// The CPU side of the 3D SDF texture.
    pub tex0: CpuTexture3D,
    /// The CPU side of the 3D SDF texture.
    pub tex1: CpuTexture3D,
    /// The GPU-side mesh and material, including the 3D GPU texture.
    pub volume: Rc<RefCell<Gm<Mesh, SDFViewerMaterial>>>,
    /// Controls the iterative algorithm used to fill the SDF texture (to provide faster previews).
    pub loading_mgr: LoadingManager,
    /// A cache of the bounding box of the SDF, as it is not allowed to change for this instance.
    pub bounding_box: [Vector3<f32>; 2],
    /// Records what part of the SDF has changed (as a bounding box) and has to be rendered
    pub changed_box: Option<[Vector3<f32>; 2]>,
    /// If this is true, another `loading_mgr` pass should be queued after this one
    pub changed_box_while_loading: bool,
    /// The three-d cloned context
    pub ctx: three_d::Context,
}

/// The default value for uncomputed SDF values while loading. Should be small to avoid graphical
/// artifacts, but will slow down rendering if too small. It is also useful for progressive loading.
const AIR_DIST: f32 = 1e-1 + 0.001234;

impl SDFViewer {
    /// Creates a new SDF viewer for the given bounding box (tries to keep aspect ratio).
    pub fn from_bb(ctx: &three_d::Context, bb: &[Vector3<f32>; 2], max_voxels_side: Option<usize>, loading_passes: usize) -> Self {
        let max_voxels_side = max_voxels_side.unwrap_or(256);
        let bb_size = bb[1] - bb[0];
        let mut voxels = Vector3::new(0usize, 0usize, 0usize);
        let max_dim = [bb_size.x, bb_size.y, bb_size.z].iter().enumerate().max_by(
            |el, el2| el.1.partial_cmp(el2.1).unwrap()).unwrap().0;
        match max_dim {
            0 => {
                voxels.x = max_voxels_side;
                voxels.y = (max_voxels_side as f32 * bb_size.y / bb_size.x) as usize;
                voxels.z = (max_voxels_side as f32 * bb_size.z / bb_size.x) as usize;
            }
            1 => {
                voxels.x = (max_voxels_side as f32 * bb_size.x / bb_size.y) as usize;
                voxels.y = max_voxels_side;
                voxels.z = (max_voxels_side as f32 * bb_size.z / bb_size.y) as usize;
            }
            2 => {
                voxels.x = (max_voxels_side as f32 * bb_size.x / bb_size.z) as usize;
                voxels.y = (max_voxels_side as f32 * bb_size.y / bb_size.z) as usize;
                voxels.z = max_voxels_side;
            }
            _ => unreachable!(),
        }
        tracing::info!("Using {}x{}x{} voxels (dimensions: {}x{}x{})", voxels.x, voxels.y, voxels.z,
            bb_size.x, bb_size.y, bb_size.z);
        Self::new_voxels(ctx, voxels, bb, loading_passes)
    }

    /// Creates a new SDF viewer with the given number of voxels in each axis.
    pub fn new_voxels(ctx: &three_d::Context, voxels: Vector3<usize>, bb: &[Vector3<f32>; 2], loading_passes: usize) -> Self {
        let texture = Self::texture_from_data(voxels, vec![[AIR_DIST; 4]; voxels.x * voxels.y * voxels.z]);
        let texture1 = texture.clone();
        let mesh = Mesh::new(ctx, &cube_with_bounds(bb));
        // TODO: Support custom mesh transformations. Alternatively, you can transform the SDF easily.
        // mesh.set_transformation(
        //     three_d::Mat4::from_angle_y(three_d::degrees(45.0)) *
        //         three_d::Mat4::from_angle_x(three_d::degrees(30.0)) *
        //         three_d::Mat4::from_angle_z(three_d::degrees(15.0)) *
        //         three_d::Mat4::from_translation(three_d::vec3(0.25, 0.15, 0.1))
        // );
        let material = SDFViewerMaterial::new(
            Texture3D::new(ctx, &texture),
            Texture3D::new(ctx, &texture1),
            *bb);
        let volume = Gm::new(mesh, material);
        Self {
            tex0: texture,
            tex1: texture1,
            volume: Rc::new(RefCell::new(volume)),
            loading_mgr: LoadingManager::new(voxels, loading_passes),
            bounding_box: *bb,
            changed_box: None,
            changed_box_while_loading: false,
            ctx: ctx.clone(),
        }
    }

    fn texture_from_data(size: Vector3<usize>, data: Vec<[f32; 4]>) -> CpuTexture3D {
        CpuTexture3D {
            name: "SDFViewerTexture".to_string(),
            data: TextureData::RgbaF32(data),
            width: size.x as u32,
            height: size.y as u32,
            depth: size.z as u32,
            min_filter: Interpolation::Nearest, // Nearest for "broken" blocky mode
            mag_filter: Interpolation::Nearest,
            mip_map_filter: None,
            wrap_s: Wrapping::MirroredRepeat, // <- MirroredRepeat should be safe, even out of bounds
            wrap_t: Wrapping::MirroredRepeat,
            wrap_r: Wrapping::MirroredRepeat,
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
        // Check whether the SDF self-reports updates.
        let mut just_changed_box = false;
        if let Some(new_box) = sdf.changed() {
            self.changed_box = Some(match &self.changed_box {
                // TODO: List of bounding boxes instead of performing the union to increase performance?
                Some(prev_box) => merge_bounding_boxes(prev_box, &new_box),
                None => new_box,
            });
            self.changed_box_while_loading = self.loading_mgr.len() > 0 || self.changed_box_while_loading;
            just_changed_box = true;
        }

        // If we still have changes to process and the loading manager is not busy, perform another
        // full pass. Only the part that changed will be re-sampled. It will reuse previous samples
        // while sampling the new part to avoid flickering.
        if let Some(_changed_box) = &self.changed_box {
            if self.loading_mgr.len() == 0 {
                self.loading_mgr = LoadingManager::new(self.loading_mgr.limits, 3 /* TODO: Configure */);
                if !just_changed_box {
                    if !self.changed_box_while_loading { // Stop doing more loading_mgr passes
                        self.changed_box = None;
                    }
                    self.changed_box_while_loading = false;
                }
            }
        }

        // Declare some variables to control the iterations and cache some values.
        let mut first = true;
        let start_iter = self.loading_mgr.iterations();
        let sdf_bb = self.bounding_box;
        let sdf_bb_size = sdf_bb[1] - sdf_bb[0];
        let texture_size_minus_1 = Vector3::new(self.tex0.width as f32 - 1., self.tex0.height as f32 - 1., self.tex0.depth as f32 - 1.);
        let tex0_data_ref = match &mut self.tex0.data {
            TextureData::RgbaF32(data) => data,
            _ => panic!("developer error: expected RgbaF32 texture data"),
        };
        let tex1_data_ref = match &mut self.tex1.data {
            TextureData::RgbaF32(data) => data,
            _ => panic!("developer error: expected RgbaF32 texture data"),
        };
        let start_time = instant::Instant::now();

        // Start sampling the SDF on the CPU to prepare the data for the GPU, as long as there is time.
        while first || start_time.elapsed() < max_delta_time {
            first = false; // TODO: Cross-platform parallel iteration?
            if let Some(index) = self.loading_mgr.next() {
                // Compute the flat index: 3D texture data is in the row-major order.
                let flat_index = (index.z * self.tex0.height as usize + index.y) * self.tex0.width as usize + index.x;
                // Compute the position in the SDF surface.
                let mut pos = Vector3::new(index.x as f32, index.y as f32, index.z as f32);
                pos.div_assign_element_wise(texture_size_minus_1); // Normalize to [0, 1]
                pos.mul_assign_element_wise(sdf_bb_size);
                pos.add_assign(sdf_bb[0]);
                // Check if the update is required: was AIR on initial load, or has changed since.
                let mut update_required = tex0_data_ref[flat_index][0] == AIR_DIST;
                if let Some(changed_box) = self.changed_box {
                    update_required = update_required ||
                        pos.x >= changed_box[0].x && pos.x <= changed_box[1].x &&
                            pos.y >= changed_box[0].y && pos.y <= changed_box[1].y &&
                            pos.z >= changed_box[0].z && pos.z <= changed_box[1].z;
                }
                if update_required { // Only update if not already computed
                    // Actually sample the SDF.
                    let mut sample = sdf.sample(pos, false);
                    // Apply a function to store the distance (-inf, inf) in the texture [0, 1].
                    // KEEP IN SYNC WITH GPU CODE!
                    tex0_data_ref[flat_index][0] = (1e-1 + sample.distance).max(0.0).min(1.0);
                    if sample.color.x == 0. && sample.color.y == 0. && sample.color.z == 0. {
                        // Avoid invisible objects if left as default with dark environment
                        sample.color = Vector3::new(0.5, 0.5, 0.5);
                    }
                    tex0_data_ref[flat_index][1] = sample.color.x;
                    tex0_data_ref[flat_index][2] = sample.color.y;
                    tex0_data_ref[flat_index][3] = sample.color.z;
                    #[cfg(target_arch = "wasm32")]
                    { // Gamma correction for the same colors on web and desktop
                        tex0_data_ref[flat_index][1] = tex0_data_ref[flat_index][1].powf(2.2);
                        tex0_data_ref[flat_index][2] = tex0_data_ref[flat_index][2].powf(2.2);
                        tex0_data_ref[flat_index][3] = tex0_data_ref[flat_index][3].powf(2.2);
                    }
                    tex1_data_ref[flat_index][0] = sample.metallic;
                    tex1_data_ref[flat_index][1] = sample.roughness;
                    // NOTE: Default occlusion is 1, to use the ambient light by default
                    tex1_data_ref[flat_index][2] = if sample.occlusion <= 0.0 { 1.0 } else { sample.occlusion };
                    // info!("Updated voxel color {:?}", data[flat_index][1]);
                    // TODO: Provide more voxel data to the shader, like a material kind index for using custom GLSL code.
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
        vol_mut.material.tex0.fill(match &self.tex0.data {
            TextureData::RgbaF32(d) => { d.as_slice() }
            _ => panic!("developer error: expected RgbaF32 texture data"),
        });
        vol_mut.material.lod_dist_between_samples = 2f32.pow(self.loading_mgr.passes_left() as u8);
        if vol_mut.material.lod_dist_between_samples == 1. {
            // NOTE: The texture is bound by previous fill call
            self.set_texture_3d_filter_linear()
        }
        vol_mut.material.tex1.fill(match &self.tex1.data {
            TextureData::RgbaF32(d) => { d.as_slice() }
            _ => panic!("developer error: expected RgbaF32 texture data"),
        });
        if vol_mut.material.lod_dist_between_samples == 1. {
            // NOTE: The texture is bound by previous fill call
            self.set_texture_3d_filter_linear()
        }
    }

    fn set_texture_3d_filter_linear(&self) {
        unsafe { // OpenGL calls are always unsafe
            self.ctx.tex_parameter_i32(context::TEXTURE_3D,
                                       context::TEXTURE_MIN_FILTER,
                                       context::LINEAR as i32);
            self.ctx.tex_parameter_i32(context::TEXTURE_3D,
                                       context::TEXTURE_MAG_FILTER,
                                       context::LINEAR as i32);
        }
    }
}

/// Creates a cube mesh with the given bounds.
fn cube_with_bounds(bb: &[Vector3<f32>; 2]) -> CpuMesh {
    let mut cube_bounds_mesh = CpuMesh::cube();
    match cube_bounds_mesh.positions {
        Positions::F32(ref mut d) => {
            for p in d {
                if p.x <= 0.0 {
                    p.x = bb[0].x;
                }
                if p.y <= 0.0 {
                    p.y = bb[0].y;
                }
                if p.z <= 0.0 {
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