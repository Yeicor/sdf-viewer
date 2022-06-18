use cgmath::{ElementWise, Vector2};
use three_d::{MetricSpace, Vector3, Zero};

use crate::sdf::{SdfSample, SDFSurface};

/// An embedded demo `Sdf` implementation
#[derive(clap::Parser, Debug, Clone)]
pub struct SDFDemo {}

impl SDFSurface for SDFDemo {
    fn bounding_box(&self) -> [Vector3<f32>; 2] {
        [Vector3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 1.0)]
    }

    fn sample(&self, p: Vector3<f32>, mut distance_only: bool) -> SdfSample {
        // Compute the distance to the surface.
        let dist_box = p.x.abs().max(p.y.abs()).max(p.z.abs()) - 0.95;
        let dist_sphere = p.distance(Vector3::zero()) - 1.15;
        let dist = dist_box.max(-dist_sphere);
        // Optimization: the air has no texture, so we can skip the texture lookup.
        distance_only = distance_only || dist > 0.1;
        if distance_only {
            SdfSample::new(dist, Vector3::zero())
        } else {
            // Debug normals instead
            // let normal = self.normal(p, None);
            // SdfSample::new(dist, Vector3::new(normal.x.abs() + 1., normal.y.abs() + 1., normal.z.abs() + 1.) * 0.5)
            // Procedural brick texture
            self.sample_brick_texture(p, dist)
        }
    }
}

impl SDFDemo {
    /// Creates a new SDF sample using an example procedural brick texture.
    pub fn sample_brick_texture(&self, p: Vector3<f32>, distance: f32) -> SdfSample {
        const BRICK_COLOR: Vector3<f32> = Vector3::new(1., 15. / 255., 12. / 255.);
        const BRICK_WIDTH: f32 = 0.5;
        const BRICK_HEIGHT: f32 = 0.25;
        const CEMENT_COLOR: Vector3<f32> = Vector3::new(26. / 255., 30. / 255., 16. / 255.);
        const CEMENT_THICKNESS: f32 = 0.2;

        // Use normal for tri-planar mapping (to know in which direction to apply the bricks)
        let normal = self.normal(p, None);
        let compute_tex2d = |tex_coord: Vector2<f32>| {
            let row_num = tex_coord.y / BRICK_HEIGHT;
            let brick_offset = row_num.floor() / 4.;
            let brick_num = (tex_coord.x + brick_offset) / BRICK_WIDTH;
            let brick_coords = Vector2::new((tex_coord.x + brick_offset).abs() % BRICK_WIDTH, tex_coord.y.abs() % BRICK_HEIGHT);
            let mut brick_rand = Vector3::new(brick_num, row_num, brick_num.floor() + row_num.floor());
            brick_rand = brick_rand.map(|x| (x.floor() + 1000.).powf(1.432).fract() * 0.1 + 0.95);
            let max_cement_displacement = CEMENT_THICKNESS / 2.0 * BRICK_HEIGHT;
            if brick_coords.x < max_cement_displacement || brick_coords.x > BRICK_WIDTH - max_cement_displacement ||
                brick_coords.y < max_cement_displacement || brick_coords.y > BRICK_HEIGHT - max_cement_displacement {
                // Cement
                (CEMENT_COLOR, 0.4, 0.5, 1.0)
            } else {
                // Brick
                (BRICK_COLOR.mul_element_wise(brick_rand), 0.2, 0.8, 0.0)
            }
        };
        let (color, metallic, roughness, occlusion) =
            if normal.x.abs() > normal.y.abs() { // Use abs because opposite sides look the same
                if normal.x.abs() > normal.z.abs() {
                    let uv = Vector2::new(p.z, p.y);
                    compute_tex2d(uv)
                } else {
                    let uv = Vector2::new(p.x, p.y);
                    compute_tex2d(uv)
                }
            } else if normal.y.abs() > normal.z.abs() {
                let uv = Vector2::new(p.x, p.z);
                compute_tex2d(uv)
            } else { // normal.z.abs() > normal.y.abs()
                let uv = Vector2::new(p.x, p.y);
                compute_tex2d(uv)
            };
        SdfSample { distance, color, metallic, roughness, occlusion }
    }
}