use std::cell::RefCell;
use std::ops::AddAssign;
use std::rc::Rc;

use cgmath::ElementWise;
use three_d::{CpuMesh, CpuTexture3D, Gm, Mesh, Texture3D, Vector3};
use three_d_asset::{Interpolation, Positions, TextureData, Wrapping};

use material::SDFViewerMaterial;

use crate::sdf::SDFSurface;

pub mod material;

/// The SDF viewer controller, that synchronizes the CPU and GPU sides.
pub struct SDFViewer {
    /// The CPU side of the 3D SDF texture.
    pub texture: CpuTexture3D,
    /// The GPU-side mesh and material, including the 3D GPU texture.
    pub volume: Rc<RefCell<Gm<Mesh, SDFViewerMaterial>>>,
    /// Controls the iterative algorithm used to fill the SDF texture (to provide faster previews).
    interlacing_mgr: InterlacingManager,
}

impl SDFViewer {
    /// Creates a new SDF viewer for the given bounding box (tries to keep aspect ratio).
    pub fn from_bb(ctx: &three_d::Context, bb: &[Vector3<f32>; 2], max_voxels_side: Option<usize>) -> Self {
        let max_voxels_side = max_voxels_side.unwrap_or(1024);
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
            data: TextureData::RgbaF32(vec![[1.0/*air*/; 4]; voxels.x * voxels.y * voxels.z]),
            width: voxels.x as u32,
            height: voxels.y as u32,
            depth: voxels.z as u32,
            min_filter: Interpolation::Linear, // Nearest for broken blocky mode
            mag_filter: Interpolation::Linear,
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
            interlacing_mgr: InterlacingManager::new(voxels),
        }
    }

    /// Iteratively requests data from the CPU `Sdf` and stores it in the CPU-side buffer,
    /// maintaining an interactive framerate. Note that this intermediate buffer must be `commit`ed
    /// to the GPU before the SDF is updated on screen.
    ///
    /// Set force to true to force a full update of the SDF.
    ///
    /// Returns the number of updates. It performs at least one update if needed, even if the
    /// time limit is reached.
    pub fn update(&mut self, sdf: impl SDFSurface, max_delta_time: instant::Duration, force: bool) -> usize {
        if force {
            self.interlacing_mgr.reset();
        }
        let mut first = true;
        let mut modified = 0;
        let sdf_bb = sdf.bounding_box();
        let sdf_bb_size = sdf_bb[1] - sdf_bb[0];
        let texture_size_minus_1 = Vector3::new(self.texture.width as f32 - 1., self.texture.height as f32 - 1., self.texture.depth as f32 - 1.);
        let start_time = instant::Instant::now();
        while first || start_time.elapsed() < max_delta_time {
            first = false;
            if let Some(next_index) = self.interlacing_mgr.next() {
                modified += 1;
                let mut next_point = Vector3::new(next_index.x as f32, next_index.y as f32, next_index.z as f32);
                next_point.div_assign_element_wise(texture_size_minus_1); // Normalize to [0, 1]
                next_point.mul_assign_element_wise(sdf_bb_size);
                next_point.add_assign(sdf_bb[0]);
                match &mut self.texture.data {
                    TextureData::RgbaF32(data) => {
                        // 3D texture data is in the row-major order.
                        let flat_index = (next_index.z * self.texture.height as usize + next_index.y) * self.texture.width as usize + next_index.x;
                        let sample = sdf.sample(next_point, false);
                        data[flat_index][0] = sample.distance;
                        // TODO: Provide more voxel data to the shader.
                    }
                    _ => panic!("developer error: expected RgbaF32 texture data"),
                }
            } else {
                break; // No more work to do!
            }
        }
        modified
    }

    /// Commits all previous `update`s to the GPU, updating the GPU-side texture data.
    pub fn commit(&mut self) {
        self.volume.borrow_mut().material.voxels.fill(match &self.texture.data {
            TextureData::RgbaF32(d) => { d.as_slice() }
            _ => panic!("developer error: expected RgbaF32 texture data"),
        }).unwrap();
    }
}

/// The interlacing manager algorithm that fills a 3D texture with the SDF data in a way that the
/// whole surface can be seen quickly at low quality and iteratively improves quality.
struct InterlacingManager {
    /// The 3D limits
    limits: Vector3<usize>,
    /// The current pass.
    pass: usize,
    /// The next index to return.
    next_index: Vector3<usize>,
}

impl InterlacingManager {
    /// Creates a new interlacing manager for the given limits.
    fn new(limits: Vector3<usize>) -> Self {
        let mut slf = Self {
            limits,
            pass: 0,
            next_index: Vector3::new(0, 0, 0),
        };
        slf.reset();
        slf
    }
    /// Resets the interlacing manager to the first pass.
    pub fn reset(&mut self) {
        self.pass = 0;
        let (offset, _) = self.pass_info().unwrap();
        self.next_index = offset;
    }
}

impl Iterator for InterlacingManager {
    type Item = Vector3<usize>;

    /// Requests the next 3D index to be filled, advancing the internal counters.
    fn next(&mut self) -> Option<Self::Item> {
        self.pass_info().map(|(offset, stride)| {
            // Return the next index (copied)
            let res = self.next_index;
            // Move to the next index (or the next pass)
            self.next_index.x += stride.x;
            if self.next_index.x >= self.limits.x {
                self.next_index.x = offset.x;
                self.next_index.y += stride.y;
                if self.next_index.y >= self.limits.y {
                    self.next_index.y = offset.y;
                    self.next_index.z += stride.z;
                    if self.next_index.z >= self.limits.z {
                        self.pass += 1;
                        if let Some((new_offset, _)) = self.pass_info() {
                            self.next_index = new_offset;
                        }
                    }
                }
            }
            res
        })
    }
}

impl InterlacingManager {
    /// Returns the offset and stride of the current pass.
    fn pass_info(&self) -> Option<(Vector3<usize>, Vector3<usize>)> {
        // TODO: Use an actual 3D interlacing algorithm (instead of 8 passes with different offsets)
        match self.pass {
            0 => {
                Some((Vector3::new(0, 0, 0), Vector3::new(2, 2, 2)))
            }
            1 => {
                Some((Vector3::new(0, 0, 1), Vector3::new(2, 2, 2)))
            }
            2 => {
                Some((Vector3::new(0, 1, 0), Vector3::new(2, 2, 2)))
            }
            3 => {
                Some((Vector3::new(0, 1, 1), Vector3::new(2, 2, 2)))
            }
            4 => {
                Some((Vector3::new(1, 0, 0), Vector3::new(2, 2, 2)))
            }
            5 => {
                Some((Vector3::new(1, 0, 1), Vector3::new(2, 2, 2)))
            }
            6 => {
                Some((Vector3::new(1, 1, 0), Vector3::new(2, 2, 2)))
            }
            7 => {
                Some((Vector3::new(1, 1, 1), Vector3::new(2, 2, 2)))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests that all voxels are set once.
    pub fn test_interlacing_impl(limits: Vector3<usize>) {
        let mut voxel_hits = vec![0; limits.x * limits.y * limits.z];
        let mut manager = InterlacingManager::new(limits);
        while let Some(v) = manager.next() {
            let voxel_index = v.x + v.y * limits.x + v.z * limits.x * limits.y;
            voxel_hits[voxel_index] += 1;
            if voxel_hits[voxel_index] > 1 {
                panic!("developer error: voxel was hit for the second time: {:?} on pass {} index {:?}", v, manager.pass, manager.next_index);
            }
        }
        for (voxel_index, voxel_hit) in voxel_hits.into_iter().enumerate() {
            if voxel_hit != 1 {
                let v = Vector3::new(voxel_index % limits.x, voxel_index / limits.x % limits.y, voxel_index / limits.x / limits.y);
                panic!("developer error: voxel was not hit: {:?}", v);
            }
        }
    }

    #[test]
    pub fn test_interlacing_cube_2() {
        test_interlacing_impl(Vector3::new(2, 2, 2));
    }

    #[test]
    pub fn test_interlacing_cube_8() {
        test_interlacing_impl(Vector3::new(8, 8, 8));
    }

    #[test]
    pub fn test_interlacing_cube_64() {
        test_interlacing_impl(Vector3::new(64, 64, 64));
    }

    #[test]
    pub fn test_interlacing_cube_11() {
        test_interlacing_impl(Vector3::new(11, 11, 11));
    }

    #[test]
    pub fn test_interlacing_non_cube() {
        test_interlacing_impl(Vector3::new(8, 11, 17));
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
        Positions::F64(_) => panic!("SDFController: cube_bounds_mesh.positions is F64"),
    }
    cube_bounds_mesh
}