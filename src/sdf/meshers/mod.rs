use std::fmt::Debug;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::mpsc;

use clap::ValueHint;

use mesh::Mesh;

use crate::sdf::SDFSurface;
use crate::sdf::wasm::load;

mod mesh;

#[cfg(feature = "isosurface")]
mod isosurface;

#[derive(clap::Parser, Debug, Clone)]
pub struct CliMesher {
    /// Input file or URL: .wasm file representing a SDF.
    #[clap(short, long = "input")]
    input: String,
    /// Output file: .ply 3D model made of triangles. Set to "-" to write to stdout.
    #[clap(short, long = "output", parse(from_os_str), value_hint = ValueHint::FilePath)]
    output_file: PathBuf,
    #[clap(flatten)]
    cfg: Config,
    #[clap(subcommand)]
    mesher: Meshers,
}

impl CliMesher {
    /// Runs the CLI for the mesher, using all the configured parameters.
    pub async fn run_cli(mut self) -> anyhow::Result<()> {
        // Check that the output file does not exist yet or fail
        if self.output_file.to_str().eq(&Some("-")) {
            self.output_file = "/dev/stdout".into();
        }
        if self.output_file.to_str().eq(&Some("/dev/stdout")) ||
            File::open(&self.output_file).is_ok() {
            anyhow::bail!("Output file already exists");
        }
        // Create/truncate the output file or fail
        let f = File::create(&self.output_file)?;
        // Buffer writes for faster performance
        let mut f = BufWriter::new(f);
        // Run as usual
        let written = self.run_custom_out(&mut f).await?;
        // Remove the last \n which breaks some programs like meshlab
        let f = f.into_inner()?;
        f.set_len((written - 1) as u64)?;
        Ok(())
    }

    /// Runs the mesher and writes the output to the given writer instead of the configured file.
    pub async fn run_custom_out<W: Write>(self, w: &mut W) -> anyhow::Result<usize> {
        // Start loading input SDF (using common code with the app)
        tracing::info!("Loading SDF from {:?}...", self.input);
        let (sender_of_updates, receiver_of_updates) = mpsc::channel();
        let load_fn = move || {
            load::load_sdf_from_path_or_url(sender_of_updates, self.input.clone());
        };
        #[cfg(target_arch = "wasm32")]
        {
            load_fn().await; // Will run asynchronously
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            std::thread::spawn(load_fn); // Will run synchronously in a new thread
        }
        // Wait for the loaded SDF to be ready
        let input_sdf = receiver_of_updates.recv()?.recv()?;
        drop(receiver_of_updates);
        // Apply the meshing algorithm as configured
        tracing::info!("Running the meshing algorithm with {:?} {:?}...", self.cfg, self.mesher);
        // TODO: Progress reporting + ETA?
        let mut mesh = self.mesher.mesh(&input_sdf, self.cfg);
        // Post-process the mesh to get the materials
        tracing::info!("Post-processing the mesh ({} vertices, {} triangles)...", mesh.vertices.len(), mesh.indices.len() / 3);
        mesh.postproc(&input_sdf);
        // Write the mesh to the output file or fail
        tracing::info!("Serializing output mesh...");
        Ok(mesh.serialize_ply(w)?)
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

