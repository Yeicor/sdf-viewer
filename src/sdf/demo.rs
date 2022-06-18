use std::ffi::OsString;

use cgmath::{ElementWise, InnerSpace, Vector2};
use three_d::{MetricSpace, Vector3, Zero};

use crate::sdf::{SdfSample, SDFSurface};

/// An embedded demo `Sdf` implementation to showcase/test most features. Subtracts a cube and a sphere.
#[derive(clap::Parser, Debug, Clone)]
pub struct SDFDemo {
    #[clap(flatten)]
    cube: SDFDemoCube,
    #[clap(flatten)]
    sphere: SDFDemoSphere,
}

impl Default for SDFDemo {
    fn default() -> Self {
        use clap::Parser;
        Self::parse_from::<_, OsString>([])
    }
}

impl SDFSurface for SDFDemo {
    fn bounding_box(&self) -> [Vector3<f32>; 2] {
        [Vector3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 1.0)]
    }

    fn sample(&self, p: Vector3<f32>, distance_only: bool) -> SdfSample {
        // Compute the distance to the surface by subtracting a sphere to a cube.
        let sample_box = self.cube.sample(p, distance_only);
        let sample_sphere = self.sphere.sample(p, distance_only);
        let dist = sample_box.distance.max(-sample_sphere.distance);
        // Choose the material based on which object's surface is closer.
        let mut sample = if sample_box.distance.abs() < sample_sphere.distance.abs()
        { sample_box } else { sample_sphere };
        // Overwrite the sample with the combined distance
        sample.distance = dist;
        sample
    }

    /// Optional: hierarchy.
    fn children(&self) -> Vec<Box<dyn SDFSurface>> {
        vec![Box::new(self.cube.clone()), Box::new(self.sphere.clone())]
    }

    /// Optional: hierarchy.
    fn id(&self) -> usize {
        0
    }

    /// Optional: hierarchy.
    fn name(&self) -> String {
        "Demo".to_string()
    }

    /// Optional: optimized normal computation for the difference.
    fn normal(&self, p: Vector3<f32>, eps: Option<f32>) -> Vector3<f32> {
        // Return the normal of the closest surface.
        let sample_box = self.cube.sample(p, true);
        let sample_sphere = self.sphere.sample(p, true);
        if sample_box.distance.abs() < sample_sphere.distance.abs() {
            self.cube.normal(p, eps)
        } else {
            -self.sphere.normal(p, eps) // Negated!
        }
    }
}

#[derive(clap::Parser, Debug, Clone)]
pub struct SDFDemoCube {
    #[clap(short, long, default_value = "0.95")]
    cube_side: f32,
}

impl Default for SDFDemoCube {
    fn default() -> Self {
        use clap::Parser;
        Self::parse_from::<_, OsString>([])
    }
}

impl SDFSurface for SDFDemoCube {
    fn bounding_box(&self) -> [Vector3<f32>; 2] {
        [Vector3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 1.0)]
    }

    fn sample(&self, p: Vector3<f32>, mut distance_only: bool) -> SdfSample {
        // Compute the distance to the surface.
        let dist_box = p.x.abs().max(p.y.abs()).max(p.z.abs()) - self.cube_side;
        // Optimization: the air has no texture, so we can skip the texture lookup.
        distance_only = distance_only || dist_box > 0.1;
        if distance_only {
            SdfSample::new(dist_box, Vector3::zero())
        } else {
            // Procedural brick texture
            sample_brick_texture(p, self.normal(p, None), dist_box)
        }
    }

    /// Optional: hierarchy.
    fn id(&self) -> usize {
        1
    }

    /// Optional: hierarchy.
    fn name(&self) -> String {
        "DemoCube".to_string()
    }

    /// Optional: optimized normal computation for the cube.
    fn normal(&self, p: Vector3<f32>, _eps: Option<f32>) -> Vector3<f32> {
        let mut normal = Vector3::zero();
        if p.x.abs() > self.cube_side {
            normal.x = p.x.signum();
        }
        if p.y.abs() > self.cube_side {
            normal.y = p.y.signum();
        }
        if p.z.abs() > self.cube_side {
            normal.z = p.z.signum();
        }
        normal
    }
}

#[derive(clap::Parser, Debug, Clone)]
pub struct SDFDemoSphere {
    #[clap(short, long, default_value = "1.05")]
    sphere_radius: f32,
}

impl Default for SDFDemoSphere {
    fn default() -> Self {
        use clap::Parser;
        Self::parse_from::<_, OsString>([])
    }
}

impl SDFSurface for SDFDemoSphere {
    fn bounding_box(&self) -> [Vector3<f32>; 2] {
        [Vector3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 1.0)]
    }

    fn sample(&self, p: Vector3<f32>, mut distance_only: bool) -> SdfSample {
        // Compute the distance to the surface.
        let dist_sphere = p.distance(Vector3::zero()) - self.sphere_radius;
        // Optimization: the air has no texture, so we can skip the texture lookup.
        distance_only = distance_only || dist_sphere > 0.1;
        if distance_only {
            SdfSample::new(dist_sphere, Vector3::zero())
        } else {
            // Simple color gradient texture for the sphere based on normals.
            let color = self.normal(p, None).map(|n| n.abs());
            // info!("Sphere normal color: {:?}", color);
            SdfSample::new(dist_sphere, color)
        }
    }

    /// Optional: hierarchy.
    fn id(&self) -> usize {
        2
    }

    /// Optional: hierarchy.
    fn name(&self) -> String {
        "DemoSphere".to_string()
    }

    /// Optional: optimized normal computation for the sphere.
    fn normal(&self, p: Vector3<f32>, _eps: Option<f32>) -> Vector3<f32> {
        p.normalize()
    }
}

/// Creates a new SDF sample using an example procedural brick texture.
fn sample_brick_texture(p: Vector3<f32>, normal: Vector3<f32>, distance: f32) -> SdfSample {
    const BRICK_COLOR: Vector3<f32> = Vector3::new(1., 15. / 255., 12. / 255.);
    const BRICK_WIDTH: f32 = 0.5;
    const BRICK_HEIGHT: f32 = 0.25;
    const CEMENT_COLOR: Vector3<f32> = Vector3::new(126. / 255., 130. / 255., 116. / 255.);
    const CEMENT_THICKNESS: f32 = 0.2;

    // The procedural 2D brick texture is a simple grid of bricks and cement.
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

    // Use the normal for tri-planar mapping (to know in which plane to apply the bricks)
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
            let uv = Vector2::new(p.z, p.x);
            compute_tex2d(uv)
        } else { // normal.z.abs() > normal.y.abs()
            let uv = Vector2::new(p.x, p.y);
            compute_tex2d(uv)
        };
    SdfSample { distance, color, metallic, roughness, occlusion }
}