use std::cell::RefCell;
use std::fmt;
use std::fmt::Display;
use std::ops::Deref;
use std::rc::Rc;
use std::str::FromStr;

use cgmath::{ElementWise, Vector2, Vector3, Zero};

use crate::sdf::{SDFParam, SDFParamKind, SDFParamValue, SDFSample, SDFSurface};
use crate::sdf::demo::{RcRefCellBool, RcRefCellF32};

#[derive(clap::Parser, Debug, Clone)]
pub struct SDFDemoCube {
    #[clap(short = 't', long, default_value = "brick")]
    cube_material: RcRefCellMaterial,
    #[clap(short, long, default_value = "0.95")]
    cube_half_side: RcRefCellF32,
    #[clap(skip)]
    changed: RcRefCellBool,
}

#[derive(Debug, Copy, Clone)]
pub enum Material {
    Brick,
    Normal,
}

impl FromStr for Material {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "BRICK" => Ok(Material::Brick),
            "NORMAL" => Ok(Material::Normal),
            _ => Err("Invalid cube material".to_string()),
        }
    }
}

impl Display for Material {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Material::Brick => write!(f, "Brick"),
            Material::Normal => write!(f, "Normal"),
        }
    }
}

impl Material {
    pub fn render(&self, dist: f32, p: Vector3<f32>, normal: Vector3<f32>) -> SDFSample {
        match &self {
            // Procedural brick texture
            Material::Brick => sample_brick_texture(p, normal, dist),
            // Simple normal texture
            Material::Normal => SDFSample::new(dist, normal.map(|e| e.abs())),
        }
    }
}

impl SDFDemoCube {
    const ID_MATERIAL: u32 = 0;
    const ID_HALF_SIDE: u32 = 1;
}

impl Default for SDFDemoCube {
    fn default() -> Self {
        use clap::Parser;
        use std::ffi::OsString;
        Self::parse_from::<_, OsString>([])
    }
}

impl SDFSurface for SDFDemoCube {
    fn bounding_box(&self) -> [Vector3<f32>; 2] {
        [Vector3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 1.0)]
    }

    fn sample(&self, p: Vector3<f32>, mut distance_only: bool) -> SDFSample {
        // Compute the distance to the surface.
        let dist_box = p.x.abs().max(p.y.abs()).max(p.z.abs()) - *self.cube_half_side.borrow();
        // Optimization: the air has no texture, so we can skip the texture lookup.
        distance_only = distance_only || dist_box > 0.1;
        if distance_only {
            SDFSample::new(dist_box, Vector3::zero())
        } else {
            self.cube_material.borrow().render(dist_box, p, self.normal(p, None))
        }
    }

    /// Optional: hierarchy.
    fn id(&self) -> u32 {
        1
    }

    /// Optional: hierarchy.
    fn name(&self) -> String {
        "DemoCube".to_string()
    }

    //noinspection DuplicatedCode
    /// Optional: parameters.
    fn parameters(&self) -> Vec<SDFParam> {
        vec![
            SDFParam {
                id: Self::ID_MATERIAL,
                name: "material".to_string(),
                kind: SDFParamKind::String {
                    choices: vec![
                        Material::Brick.to_string(),
                        Material::Normal.to_string(),
                    ],
                },
                value: SDFParamValue::String(self.cube_material.to_string()),
                description: "The material to use for the cube.".to_string(),
            },
            SDFParam {
                id: Self::ID_HALF_SIDE,
                name: "half_side".to_string(),
                kind: SDFParamKind::Int { // Should be float, but testing the int parameter
                    range: 0..=100,
                    step: 1,
                },
                value: SDFParamValue::Int((*self.cube_half_side.borrow() * 100.) as i32),
                description: "Half the length of a side of the cube (mapped from [0-100] to [0.0,1.0]).".to_string(),
            },
        ]
    }

    //noinspection DuplicatedCode
    /// Optional: parameters.
    fn set_parameter(&self, param_id: u32, param_value: &SDFParamValue) -> Result<(), String> {
        if param_id == Self::ID_MATERIAL {
            if let SDFParamValue::String(value) = &param_value {
                *self.cube_material.borrow_mut() = Material::from_str(value.as_str())
                    .expect("predefined choices, should not receive an invalid value");
                *self.changed.borrow_mut() = true;
                return Ok(());
            }
        } else if param_id == Self::ID_HALF_SIDE {
            if let SDFParamValue::Int(value) = &param_value {
                *self.cube_half_side.borrow_mut() = *value as f32 / 100.;
                *self.changed.borrow_mut() = true;
                return Ok(());
            }
        }
        Err(format!("Unknown parameter {} with value {:?}", param_id, param_value))
    }

    //noinspection DuplicatedCode
    /// Optional: parameters.
    fn changed(&self) -> Option<[Vector3<f32>; 2]> {
        super::super::defaults::changed_default_impl(self).or_else(|| {
            // Note: bounding_box() change could be improved.
            let mut changed = self.changed.borrow_mut();
            if *changed {
                *changed = false;
                Some(self.bounding_box())
            } else { None }
        })
    }

    /// Optional: optimized normal computation for the cube.
    fn normal(&self, p: Vector3<f32>, _eps: Option<f32>) -> Vector3<f32> {
        let mut normal = Vector3::zero();
        let cube_side = *self.cube_half_side.borrow();
        if p.x.abs() > cube_side {
            normal.x = p.x.signum();
        }
        if p.y.abs() > cube_side {
            normal.y = p.y.signum();
        }
        if p.z.abs() > cube_side {
            normal.z = p.z.signum();
        }
        normal
    }
}

/// Creates a new SDF sample using an example procedural brick texture.
fn sample_brick_texture(p: Vector3<f32>, normal: Vector3<f32>, distance: f32) -> SDFSample {
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
    SDFSample { distance, color, metallic, roughness, occlusion }
}


#[derive(Debug, Clone)]
pub struct RcRefCellMaterial(Rc<RefCell<Material>>);

impl FromStr for RcRefCellMaterial {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(RcRefCellMaterial(Rc::new(RefCell::new(Material::from_str(s)?))))
    }
}

impl Display for RcRefCellMaterial {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0.borrow())
    }
}

impl Deref for RcRefCellMaterial {
    type Target = RefCell<Material>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}