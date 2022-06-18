use cgmath::Vector3;

/// The algorithm that fills a 3D texture with the SDF data in a way that the
/// whole surface can be seen quickly at low quality and iteratively improves quality.
pub struct LoadingManager {
    /// The 3D limits
    limits: Vector3<usize>,
    /// The current number of indices skipped in each dimension.
    /// It explores powers of two, in descending order, stopping at 1.
    step_size: usize,
    /// The next index to return.
    next_index: Vector3<usize>,
    /// The number of iterations performed in the current pass.
    iterations: usize,
    /// The number of iterations performed in total (since the last reset).
    total_iterations: usize,
}

impl LoadingManager {
    /// Creates a new interlacing manager for the given limits.
    pub fn new(limits: Vector3<usize>, passes: usize) -> Self {
        let mut slf = Self {
            limits,
            step_size: 0,
            next_index: Vector3::new(0, 0, 0),
            iterations: 0,
            total_iterations: 0,
        };
        slf.reset(passes);
        slf
    }
    /// Resets the interlacing manager to the first pass.
    pub fn reset(&mut self, passes: usize) {
        self.step_size = 2usize.pow(passes as u32 - 1);
        self.next_index = Vector3::new(0, 0, 0);
        self.iterations = 0;
        self.total_iterations = 0;
    }
}

impl Iterator for LoadingManager {
    type Item = Vector3<usize>;

    /// Requests the next 3D index to be filled, advancing the internal counters.
    fn next(&mut self) -> Option<Self::Item> {
        if self.step_size == 0 {
            return None;
        }
        self.iterations += 1;
        self.total_iterations += 1;
        // Return the next index (copied)
        let res = self.next_index;
        // println!("{:?} (skipping {}, iterations (done + remaining): {}+{})", res, self.step_size, self.iterations, self.len());
        // Move to the next index (or the next pass)
        self.next_index.x += self.step_size;
        if self.next_index.x >= self.limits.x {
            self.next_index.x = 0;
            self.next_index.y += self.step_size;
            if self.next_index.y >= self.limits.y {
                self.next_index.y = 0;
                self.next_index.z += self.step_size;
                if self.next_index.z >= self.limits.z {
                    self.step_size = prev_power_of_2((self.step_size - 1) as u32) as usize;
                    self.next_index = Vector3::new(0, 0, 0);
                    self.iterations = 0;
                }
            }
        }
        // std::thread::sleep(std::time::Duration::from_micros(2)); // Slow loading debugging
        Some(res)
    }
}

impl ExactSizeIterator for LoadingManager {
    fn len(&self) -> usize {
        let mut step_size = self.step_size;
        let mut iterations = 0;
        while step_size > 0 {
            let steps_per_dim = (self.limits + Vector3::new(step_size - 1, step_size - 1, step_size - 1)) / step_size;
            iterations += steps_per_dim.x * steps_per_dim.y * steps_per_dim.z;
            step_size = prev_power_of_2((step_size - 1) as u32) as usize;
        }
        iterations - self.iterations
    }
}

impl LoadingManager {
    /// Returns the number of iterations performed so far.
    pub fn iterations(&self) -> usize {
        self.total_iterations
    }

    /// Returns the current passes left (goes down 1 by 1 while loading, ending at 0 when finished).
    pub fn passes_left(&self) -> usize {
        if self.step_size == 0 {
            0
        } else {
            (self.step_size as f32).log2() as usize + 1
        }
    }
}

fn prev_power_of_2(mut x: u32) -> u32 {
    x = x | (x >> 1);
    x = x | (x >> 2);
    x = x | (x >> 4);
    x = x | (x >> 8);
    x = x | (x >> 16);
    x - (x >> 1)
}

#[cfg(test)]
mod tests {
    use crate::app::scene::sdf::*;

    /// Tests that all voxels are set once.
    pub fn test_loading_impl(limits: Vector3<usize>) {
        let mut voxel_hits = vec![0; limits.x * limits.y * limits.z];
        let num_passes = 3;
        let mut manager = LoadingManager::new(limits, num_passes);
        let mut remaining_iterations = manager.len();
        let mut iterations = 0;
        let total_iterations = iterations + remaining_iterations;
        while let Some(v) = manager.next() {
            let voxel_flat_index = v.x + v.y * limits.x + v.z * limits.x * limits.y;
            voxel_hits[voxel_flat_index] += 1;
            // Multiple voxel hits are OK, up to the number of passes (conflicts are checked from calling function).
            assert!(voxel_hits[voxel_flat_index] <= num_passes);
            // Check that the number of iterations is OK!
            iterations += 1;
            remaining_iterations = manager.len();
            assert_eq!(total_iterations, iterations + remaining_iterations);
        }
        for (voxel_index, voxel_hit) in voxel_hits.into_iter().enumerate() {
            if voxel_hit < 1 {
                let v = Vector3::new(voxel_index % limits.x, voxel_index / limits.x % limits.y, voxel_index / limits.x / limits.y);
                panic!("developer error: voxel was not hit: {:?}", v);
            }
        }
    }

    #[test]
    pub fn test_interlacing_cube_2() {
        test_loading_impl(Vector3::new(2, 2, 2));
    }

    #[test]
    pub fn test_interlacing_cube_8() {
        test_loading_impl(Vector3::new(8, 8, 8));
    }

    #[test]
    pub fn test_interlacing_cube_64() {
        test_loading_impl(Vector3::new(64, 64, 64));
    }

    #[test]
    pub fn test_interlacing_cube_11() {
        test_loading_impl(Vector3::new(11, 11, 11));
    }

    #[test]
    pub fn test_interlacing_non_cube() {
        test_loading_impl(Vector3::new(8, 11, 17));
    }
}
