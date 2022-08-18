use std::fmt::Debug;
use std::fs::File;
use std::io::BufWriter;
use std::io::Read;
use std::path::PathBuf;

use clap::ValueHint;

use mesh::Mesh;

use crate::sdf::SDFSurface;
use crate::sdf::wasm::load_sdf_wasm;

mod mesh;

#[cfg(feature = "isosurface")]
mod isosurface;

#[derive(clap::Parser, Debug, Clone)]
pub struct CliMesher {
    /// Input file: .wasm file representing a SDF.
    #[clap(short, long = "input", parse(from_os_str), value_hint = ValueHint::FilePath)]
    input_file: PathBuf,
    /// Output file: .ply 3D model made of triangles. IT WILL BE OVERWRITTEN.
    #[clap(short, long = "output", parse(from_os_str), value_hint = ValueHint::FilePath)]
    output_file: PathBuf,
    #[clap(flatten)]
    cfg: Config,
    #[clap(subcommand)]
    mesher: Meshers,
}

impl CliMesher {
    pub async fn run(self) {
        // Create/truncate the output file or fail
        let f = File::create(&self.output_file).expect("Could not create output file");
        // Buffer writes for faster performance
        let mut f = BufWriter::new(f);
        // Load the input SDF or fail
        tracing::info!("Loading SDF from {:?}...", self.input_file);
        let mut input_file = File::open(&self.input_file).expect("Could not open input file");
        let mut input_bytes = vec![];
        input_file.read_to_end(&mut input_bytes).expect("Could not read input file");
        drop(input_file);
        let input_sdf = load_sdf_wasm(&input_bytes).await.expect("Could not load input SDF");
        // Apply the meshing algorithm as configured
        tracing::info!("Running the meshing algorithm with {:?} {:?}...", self.cfg, self.mesher);
        let mut mesh = self.mesher.mesh(&input_sdf, self.cfg);
        // Post-process the mesh to get the materials
        tracing::info!("Post-processing the mesh ({} vertices, {} triangles)...", mesh.vertices.len(), mesh.indices.len() / 3);
        mesh.postproc(&input_sdf);
        // Write the mesh to the output file or fail
        tracing::info!("(Over)writing output mesh to {:?}...", self.output_file);
        let written = mesh.serialize_ply(&mut f).expect("Could not write mesh to output file");
        let f = f.into_inner().unwrap();
        f.set_len((written - 1) as u64).unwrap(); // Remove the last \n which breaks some programs like meshlab
    }
}

/// Common config shared by all meshers
#[derive(clap::Parser, Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// The maximum number of voxels or cells used for the largest axis of the volume.
    /// Some algorithms require this to be a power of two.
    #[clap(short = 'v', long, default_value = "64")]
    pub max_voxels_per_axis: usize,
}

pub trait Mesher: Debug {
    /// Mesh reconstructs a mesh from a [`sdf_viewer::sdf::SDFSurface`] trait.
    fn mesh(&self, sdf: &dyn SDFSurface, cfg: Config) -> Mesh;
}

/// Meshers holds the list of currently implemented meshing algorithms.
#[derive(clap::Parser, Debug, Clone)]
#[non_exhaustive]
pub enum Meshers {
    #[cfg(feature = "isosurface")]
    MarchingCubes,
    #[cfg(feature = "isosurface")]
    LinearHashedMarchingCubes,
    // #[cfg(feature = "isosurface")]
    // ExtendedMarchingCubes,
    #[cfg(feature = "isosurface")]
    DualContouringMinimizeQEF,
    #[cfg(feature = "isosurface")]
    DualContouringParticleBasedMinimization,
}

impl Mesher for Meshers {
    fn mesh(&self, s: &dyn SDFSurface, cfg: Config) -> Mesh {
        match self {
            #[cfg(feature = "isosurface")]
            Meshers::MarchingCubes => isosurface::mesh(0, cfg, s),
            #[cfg(feature = "isosurface")]
            Meshers::LinearHashedMarchingCubes => isosurface::mesh(1, cfg, s),
            // #[cfg(feature = "isosurface")]
            // Meshers::ExtendedMarchingCubes => isosurface::mesh(2, cfg, s),
            #[cfg(feature = "isosurface")]
            Meshers::DualContouringMinimizeQEF => isosurface::mesh(3, cfg, s),
            #[cfg(feature = "isosurface")]
            Meshers::DualContouringParticleBasedMinimization => isosurface::mesh(4, cfg, s),
        }
    }
}

